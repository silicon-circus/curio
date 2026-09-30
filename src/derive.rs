//! ANSWERING A REQUEST THE STORE CANNOT ANSWER LITERALLY.
//!
//! A rendition is not a new asset, it is the same asset answered differently. It is made once and
//! kept in cache/, which is disposable: delete it and the next request rebuilds it.
//!
//! DOWNWARD ONLY, IN FORMAT AND IN SIZE.
//!
//! The format rule was already here: png/jpg -> webp, never the reverse, because a .png conjured
//! out of .webp bytes would be a lossless-looking file that is nothing of the sort. The size rule is
//! the same principle on a second axis — NEVER FABRICATE PIXELS THAT WERE NEVER THERE. A width
//! larger than the master is clamped to the master, so `?w=8192` on a 1024px picture is not a 257 MB
//! file invented out of nothing, it is the master.
//!
//! That is not a limit imposed to stop a denial of service, although it does. It is what curio is
//! for. Enlarging is an authoring decision: the art here is generated in the first place, so if a
//! bigger master is wanted, render a bigger master. Interpolation cannot add detail and AI
//! super-resolution belongs upstream, not in a serving path.
//!
//! NO SUBPROCESS. The Crystal version shelled out to ImageMagick, which meant an absent binary was
//! an unhandled exception, 40 concurrent requests were 41 processes at ~10 GB RSS, and the whole
//! delegate/coder surface came along for the ride. Decoding, scaling and encoding all happen in
//! process here, bounded by a semaphore, with predictable memory.

use anyhow::{anyhow, bail, Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Formats that have already thrown information away. Re-encoding one of these is not compressing a
/// picture, it is compressing somebody else's compression artifacts, and the encoder cannot tell
/// those from detail.
pub const LOSSY: &[&str] = &["jpg", "jpeg", "webp", "avif", "gif"];

/// What we are willing to decode at all.
pub const RASTER: &[&str] = &["png", "jpg", "jpeg", "webp", "gif"];

/// The whole rule, as a table: target -> the masters allowed to answer for it.
pub const DERIVABLE: &[(&str, &[&str])] = &[("webp", &["png", "jpg", "jpeg"])];

/// Requested widths are rounded UP to a multiple of this rather than allowlisted.
///
/// An allowlist would 404 a caller asking for 641, which breaks `srcset` and device-pixel-ratio
/// widths; quantising gives it 704 pixels and a correct image. The key space drops from 8192
/// distinct widths per name to about thirty, without anybody outside having to know the list.
pub const WIDTH_STEP: u32 = 64;

/// Quality is quantised too, for the same reason and to the same effect.
pub const QUALITY_STEP: u8 = 5;

pub fn is_lossy(ext: &str) -> bool { LOSSY.contains(&ext) }
pub fn is_raster(ext: &str) -> bool { RASTER.contains(&ext) }

pub fn ext_of(path: &Path) -> String {
    path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

/// The name asked for is not a file. Is it one we are allowed to make, and out of what?
///
/// Lossless sources first: given both foo.png and foo.jpg, the png is the better parent for a webp
/// because it is the one that has not already lost something.
pub fn source_for(asset_path: &Path) -> Option<PathBuf> {
    let target = ext_of(asset_path);
    let sources = DERIVABLE.iter().find(|(t, _)| *t == target).map(|(_, s)| *s)?;
    for src_ext in sources {
        let candidate = asset_path.with_extension(src_ext);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// DERIVABLE read the other way round: given a master's extension, what else can it answer as?
///
/// `source_for` answers "this name is missing, may I make it?", which is what a request asks. This
/// answers "what can this master become?", which is what a LISTING asks — and without it every
/// consumer has to reimplement the table to tell a name curio will serve from one it will not.
pub fn targets_for(ext: &str) -> Vec<&'static str> {
    let e = ext.to_lowercase();
    DERIVABLE.iter()
        .filter(|(target, sources)| sources.contains(&e.as_str()) && *target != e.as_str())
        .map(|(target, _)| *target)
        .collect()
}

/// QUALITY FOLLOWS THE SOURCE, because the same number does not mean the same thing twice.
///
/// webp from a png is compressing a picture and the artifacts are the first the file has ever
/// carried: 82 is plenty. webp from a jpg is a SECOND lossy pass over bytes that are already
/// dented, so the encoder needs more room to avoid compounding what it inherited: 90.
pub fn default_quality(src_ext: &str, target_ext: &str) -> u8 {
    if src_ext == target_ext {
        88
    } else if is_lossy(src_ext) {
        90
    } else {
        82
    }
}

/// Quality is meaningless on a lossless target — on PNG output the number is a zlib level, so a low
/// "quality" makes the file BIGGER. Measured on a 1024px master: q=1 gave 415 kB, q=95 gave 257 kB.
/// So the dial only exists where it means what it says.
pub fn quality_applies(target_ext: &str) -> bool {
    is_lossy(target_ext)
}

/// Nothing in the park is wider than this, and clamping here means the arithmetic below cannot
/// overflow whatever a caller types.
pub const MAX_WIDTH: u32 = 16384;

pub fn quantise_width(w: u32) -> u32 {
    let w = w.clamp(1, MAX_WIDTH);
    w.div_ceil(WIDTH_STEP) * WIDTH_STEP
}

pub fn quantise_quality(q: u8) -> u8 {
    let q = q.clamp(1, 100);
    (q.div_ceil(QUALITY_STEP) * QUALITY_STEP).min(100)
}

/// THE CACHE KEY CARRIES THE SOURCE'S STATE, NOT JUST ITS NAME.
///
/// Keyed on the name alone, a rendition outlives the master it was made from: touch up a picture and
/// the site keeps being handed the 320-wide version of the one you replaced, for ever. Size and
/// mtime mean an edit changes the key, the old one is never asked for again, and the next request
/// renders fresh. A stat, not a hash: this happens per request and hashing a 3 MB master to serve a
/// 14 kB thumbnail is not a trade.
///
/// The digest is of the name RELATIVE TO assets/, not the basename. The Crystal version used the
/// basename and so two collection members called Textures/colormap.png collided on one key and
/// served each other's picture — reproducible, and only latent because assets/ happens to be flat.
pub fn cache_name(
    rel_name: &str,
    size: u64,
    mtime: i64,
    target_ext: &str,
    width: Option<u32>,
    quality: Option<u8>,
) -> String {
    use sha2::{Digest, Sha256};
    let digest: String = Sha256::digest(rel_name.as_bytes())
        .iter().take(4).map(|b| format!("{b:02x}")).collect();
    let stem = Path::new(rel_name).file_stem().map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "asset".into());
    let mut parts = vec![stem, digest, format!("s{size}"), format!("m{mtime}")];
    if let Some(w) = width {
        parts.push(format!("w{w}"));
    }
    if let Some(q) = quality {
        parts.push(format!("q{q}"));
    }
    format!("{}.{}", parts.join("."), target_ext)
}

/// Bounded rendering. One semaphore, so a burst of requests queues instead of forking a process per
/// request, and a cache with a size cap, so the key space cannot turn into unbounded disk.
pub struct Renderer {
    cache_dir: PathBuf,
    permits: Arc<Semaphore>,
    max_cache_bytes: u64,
}

pub struct Rendition {
    pub path: PathBuf,
    /// False when the answer is the master itself — nothing to make.
    pub derived: bool,
}

impl Rendition {
    /// The cache filename, which encodes everything that determines the bytes — the master's size
    /// and mtime, the width, the quality — and is therefore a strong validator for this URL.
    pub fn path_key(&self) -> Option<String> {
        self.derived.then(|| self.path.file_name().map(|n| n.to_string_lossy().into_owned()))?
    }
}

impl Renderer {
    pub fn new(cache_dir: PathBuf, jobs: usize, max_cache_bytes: u64) -> Self {
        Renderer { cache_dir, permits: Arc::new(Semaphore::new(jobs.max(1))), max_cache_bytes }
    }

    /// Make `src` answer as `target_ext`, optionally narrowed to `want_width`.
    ///
    /// Returns the master itself when there is nothing to do: same format, and a width at or above
    /// the master's own. That is the no-upscale rule paying for itself — a request for more pixels
    /// than exist is not an error and not an invention, it is the picture.
    pub async fn render(
        &self,
        src: &Path,
        rel_name: &str,
        target_ext: &str,
        want_width: Option<u32>,
        want_quality: Option<u8>,
    ) -> Result<Rendition> {
        let src_ext = ext_of(src);
        if !is_raster(&src_ext) || !is_raster(target_ext) {
            bail!("{src_ext} -> {target_ext} is not a raster conversion");
        }
        let (size, mtime, _) = crate::manifest::stat_of(src)?;

        // Quantise only what a CALLER supplies. The defaults are already a canonical set of three
        // values, so rounding them changes the output for no benefit — 82 became 85 and every
        // rendition came out about 30% larger than the Crystal's for no visible gain. The key space
        // is bounded by quantising arbitrary input, not by nudging our own numbers.
        let quality = if quality_applies(target_ext) {
            Some(match want_quality {
                Some(q) => quantise_quality(q),
                None => default_quality(&src_ext, target_ext),
            })
        } else {
            None
        };
        let width = want_width.map(quantise_width).filter(|w| *w > 0);

        let src = src.to_path_buf();
        let cache_dir = self.cache_dir.clone();
        let rel = rel_name.to_string();
        let target = target_ext.to_string();
        let max_bytes = self.max_cache_bytes;

        let _permit = self.permits.acquire().await.map_err(|e| anyhow!("render queue closed: {e}"))?;
        tokio::task::spawn_blocking(move || {
            render_blocking(&src, &rel, &target, width, quality, size, mtime, &cache_dir, max_bytes)
        })
        .await
        .context("render task panicked")?
    }
}

#[allow(clippy::too_many_arguments)]
fn render_blocking(
    src: &Path,
    rel_name: &str,
    target_ext: &str,
    width: Option<u32>,
    quality: Option<u8>,
    size: u64,
    mtime: i64,
    cache_dir: &Path,
    max_cache_bytes: u64,
) -> Result<Rendition> {
    let src_ext = ext_of(src);

    // Decode first, because the master's own width decides whether there is any scaling to do.
    let img = image::open(src).with_context(|| format!("decoding {}", src.display()))?;
    let img = to_eight_bit(img);
    let master_w = img.width();

    // NEVER UPSCALE. A width at or beyond the master collapses to no resize at all.
    let effective = width.map(|w| w.min(master_w)).filter(|w| *w < master_w);

    if effective.is_none() && src_ext == target_ext {
        return Ok(Rendition { path: src.to_path_buf(), derived: false });
    }

    let cached = cache_dir.join(cache_name(rel_name, size, mtime, target_ext, effective, quality));
    if cached.is_file() {
        return Ok(Rendition { path: cached, derived: true });
    }

    let scaled = match effective {
        Some(w) => scale_to_width(&img, w)?,
        None => img,
    };
    let bytes = encode(&scaled, target_ext, quality)?;

    // STAGE, THEN RENAME. `cp` is not atomic, and the Crystal version's "does the cached file
    // exist" check was therefore answering "has a copy STARTED" — a second request mid-copy was
    // handed a truncated image, served 200, and cached by the client for a year. A rename inside one
    // directory is atomic, so the final name either does not exist or is whole.
    std::fs::create_dir_all(cache_dir)?;
    let stage = cache_dir.join(format!("{}{}-{}", crate::config::SCRATCH_PREFIX,
        std::process::id(), cached.file_name().unwrap().to_string_lossy()));
    std::fs::write(&stage, &bytes).with_context(|| format!("writing {}", stage.display()))?;
    if let Err(e) = std::fs::rename(&stage, &cached) {
        let _ = std::fs::remove_file(&stage);
        return Err(e).context("publishing rendition into cache");
    }

    evict_to_fit(cache_dir, max_cache_bytes, &cached)?;
    Ok(Rendition { path: cached, derived: true })
}

/// SIXTEEN-BIT MASTERS ARE REAL AND THIS STORE IS FULL OF THEM — 163 of its 522 PNGs, which is
/// nearly a third. They decode to Rgba16, and the first version of this scaler only handled the
/// eight-bit pixel layouts, so every one of them returned 500 while ImageMagick had been converting
/// them all along. Found by comparing against the live store rather than against fixtures.
///
/// Everything is normalised to eight bits before scaling. webp has no sixteen-bit form, so for the
/// only lossy target this is not a loss at all; for a same-format png resize it is a reduction, and
/// a deliberate one — a thumbnail does not need sixteen bits, the master keeps them, and "downward
/// only" is the rule this whole module is built on.
fn to_eight_bit(img: image::DynamicImage) -> image::DynamicImage {
    use image::ColorType::{L8, La8, Rgb8, Rgba8};
    match img.color() {
        L8 | La8 | Rgb8 | Rgba8 => img,
        c if c.has_alpha() => image::DynamicImage::ImageRgba8(img.to_rgba8()),
        _ => image::DynamicImage::ImageRgb8(img.to_rgb8()),
    }
}

/// PREMULTIPLIED ALPHA, WHICH IS THE WHOLE REASON THIS IS NOT `image::imageops::resize`.
///
/// Most of this store is RGBA — 41 of 60 sampled PNGs — and the die-cut props are fully transparent
/// at the edges by design. Scaling those channel by channel averages the colour of transparent
/// pixels into the visible edge and haloes exactly the silhouettes that were carefully cut out.
/// fast_image_resize multiplies by alpha before scaling and divides after, which is the correct
/// operation; it is also SIMD-accelerated, so it is faster than the naive version as well as right.
fn scale_to_width(img: &image::DynamicImage, width: u32) -> Result<image::DynamicImage> {
    use fast_image_resize::images::Image;
    use fast_image_resize::{FilterType, IntoImageView, ResizeAlg, ResizeOptions, Resizer};

    let height = ((img.height() as u64 * width as u64) / img.width().max(1) as u64).max(1) as u32;
    let pixel_type = img.pixel_type().ok_or_else(|| anyhow!("unsupported pixel layout"))?;
    let mut dst = Image::new(width, height, pixel_type);

    // MITCHELL, AND THE REASON IS THE ENCODER, NOT THE PIXELS.
    //
    // Lanczos3 is the sharper filter and the obvious first choice, but sharpening a downscale is
    // ringing, and ringing is high-frequency detail that a lossy codec has to spend bytes on.
    // Measured against ImageMagick's output for the same asset at the same quality, over sixteen
    // real masters resized to 320px:
    //
    //     lanczos3    median +19.2%   mean +15.2%   worst +41.1%
    //     catmullrom  median  -1.5%   mean  -0.1%   worst +13.8%
    //     mitchell    median  -2.6%   mean  -4.8%   worst  +8.6%
    //
    // So Mitchell is both the closest to what the park has been served all along and slightly
    // cheaper, with the least spread. Full-size conversion with no resize is byte-identical to
    // ImageMagick either way, because at that point both are just libwebp at the same quality.
    Resizer::new()
        .resize(img, &mut dst, &ResizeOptions::new()
            .resize_alg(ResizeAlg::Convolution(FilterType::Mitchell)))
        .context("scaling")?;

    let out = match pixel_type {
        fast_image_resize::PixelType::U8x4 => image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(width, height, dst.into_vec())
                .ok_or_else(|| anyhow!("rgba buffer size mismatch"))?),
        fast_image_resize::PixelType::U8x3 => image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(width, height, dst.into_vec())
                .ok_or_else(|| anyhow!("rgb buffer size mismatch"))?),
        fast_image_resize::PixelType::U8 => image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(width, height, dst.into_vec())
                .ok_or_else(|| anyhow!("gray buffer size mismatch"))?),
        fast_image_resize::PixelType::U8x2 => image::DynamicImage::ImageLumaA8(
            image::GrayAlphaImage::from_raw(width, height, dst.into_vec())
                .ok_or_else(|| anyhow!("gray+alpha buffer size mismatch"))?),
        other => bail!("unsupported pixel type {other:?}"),
    };
    Ok(out)
}

/// libwebp for lossy webp, because `image`'s own webp encoder is lossless-oriented and
/// quality-follows-source needs a real quality dial.
fn encode(img: &image::DynamicImage, target_ext: &str, quality: Option<u8>) -> Result<Vec<u8>> {
    match target_ext {
        "webp" => {
            let q = quality.unwrap_or(82) as f32;
            // Only carry an alpha channel if there is one. Forcing to_rgba8() on an opaque picture
            // made libwebp encode an all-255 alpha plane and every rendition came out about a
            // quarter larger than ImageMagick's for no reason at all.
            if img.color().has_alpha() {
                let rgba = img.to_rgba8();
                Ok(webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height()).encode(q).to_vec())
            } else {
                let rgb = img.to_rgb8();
                Ok(webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height()).encode(q).to_vec())
            }
        }
        "png" => {
            let mut out = std::io::Cursor::new(Vec::new());
            img.write_to(&mut out, image::ImageFormat::Png).context("encoding png")?;
            Ok(out.into_inner())
        }
        "jpg" | "jpeg" => {
            let mut out = std::io::Cursor::new(Vec::new());
            let q = quality.unwrap_or(88);
            let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, q);
            enc.encode_image(&img.to_rgb8()).context("encoding jpeg")?;
            Ok(out.into_inner())
        }
        other => bail!("no encoder for {other}"),
    }
}

/// cache/ is disposable, so it gets a ceiling rather than a promise. Oldest first, because a
/// rendition nobody has asked for recently is the cheapest one to lose.
///
/// WITH A GRACE PERIOD, AND THAT IS NOT A DETAIL. Eviction runs at the end of every render, so
/// without one it deletes renditions that OTHER requests published moments ago and are still about
/// to send — measured, 14 of 60 concurrent requests came back 404 with a 1 MB ceiling. A 404 is the
/// worst possible answer to that, because it is indistinguishable from "no such asset": the caller
/// cannot tell it from a name that does not exist and will not retry. So anything younger than the
/// grace period is off limits, as is the rendition this render just made.
///
/// A cache ceiling smaller than the working set therefore gets exceeded rather than enforced. That
/// is the right way round: a soft cap briefly overshot costs disk, and a hard one costs correctness.
const EVICT_GRACE: std::time::Duration = std::time::Duration::from_secs(60);

fn evict_to_fit(cache_dir: &Path, max_bytes: u64, just_written: &Path) -> Result<()> {
    if max_bytes == 0 {
        return Ok(());
    }
    let now = std::time::SystemTime::now();
    let mut eligible: Vec<(std::time::SystemTime, u64, PathBuf)> = Vec::new();
    let mut total: u64 = 0;
    for e in std::fs::read_dir(cache_dir)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with(crate::config::SCRATCH_PREFIX) {
            continue;
        }
        let Ok(m) = e.metadata() else { continue };
        if !m.is_file() {
            continue;
        }
        total += m.len();
        if e.path() == just_written {
            continue;
        }
        let modified = m.modified().unwrap_or(std::time::UNIX_EPOCH);
        if now.duration_since(modified).map(|age| age < EVICT_GRACE).unwrap_or(true) {
            continue; // young enough that something may still be sending it
        }
        eligible.push((modified, m.len(), e.path()));
    }
    if total <= max_bytes {
        return Ok(());
    }
    eligible.sort_by_key(|(modified, _, _)| *modified);
    for (_, len, path) in eligible {
        if total <= max_bytes {
            return Ok(());
        }
        if std::fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(len);
        }
    }
    eprintln!("curio: cache is {} MB over its {} MB ceiling and everything else is too new to \
        evict — raise CURIO_CACHE_MAX_MB if this persists",
        (total.saturating_sub(max_bytes)) / (1 << 20), max_bytes / (1 << 20));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivation_goes_downward_only() {
        // a webp can be made from a lossless or a lossy master
        assert_eq!(targets_for("png"), vec!["webp"]);
        assert_eq!(targets_for("jpg"), vec!["webp"]);
        assert_eq!(targets_for("jpeg"), vec!["webp"]);
        // and nothing can be made from a webp — that is the whole rule
        assert!(targets_for("webp").is_empty(), "a webp must never father a png");
        // a format never derives itself
        assert!(!targets_for("png").contains(&"png"));
        // and things that are not pictures derive nothing
        for e in ["", "glb", "xcf", "mp4", "svg"] {
            assert!(targets_for(e).is_empty(), "{e}");
        }
        assert_eq!(targets_for("PNG"), vec!["webp"], "case must not matter");
    }

    #[test]
    fn quality_follows_the_source_and_only_where_it_means_something() {
        assert_eq!(default_quality("png", "webp"), 82, "lossless source");
        assert_eq!(default_quality("jpg", "webp"), 90, "a second lossy pass needs more room");
        assert_eq!(default_quality("webp", "webp"), 88, "same-format resize keeps the old default");
        assert!(quality_applies("webp"));
        assert!(!quality_applies("png"), "on a png the number is a zlib level, not a quality");
    }

    #[test]
    fn widths_quantise_up_and_cannot_overflow() {
        assert_eq!(quantise_width(1), 64);
        assert_eq!(quantise_width(64), 64);
        assert_eq!(quantise_width(65), 128);
        assert_eq!(quantise_width(641), 704, "a srcset width gets a real image, not a 404");
        assert_eq!(quantise_width(u32::MAX), MAX_WIDTH);
        assert_eq!(quantise_width(0), 64);
    }

    #[test]
    fn qualities_quantise_and_clamp() {
        assert_eq!(quantise_quality(1), 5);
        assert_eq!(quantise_quality(82), 85);
        assert_eq!(quantise_quality(100), 100);
        assert_eq!(quantise_quality(200), 100);
        assert_eq!(quantise_quality(0), 5);
    }

    #[test]
    fn the_cache_key_separates_collection_members_with_the_same_basename() {
        // The Crystal version keyed on the basename, so these two collided and served each other's
        // picture. Reproducible then; impossible now.
        let a = cache_name("coll_a/colormap.png", 87852, 1790745336, "webp", Some(320), Some(82));
        let b = cache_name("coll_b/colormap.png", 87852, 1790745336, "webp", Some(320), Some(82));
        assert_ne!(a, b, "identical size and mtime must still not collide");
        assert!(a.starts_with("colormap."), "still readable: {a}");
        assert!(a.ends_with(".webp"));
    }

    #[test]
    fn the_cache_key_moves_when_the_master_does() {
        let base = cache_name("a.png", 100, 5, "webp", Some(320), Some(82));
        assert_ne!(base, cache_name("a.png", 101, 5, "webp", Some(320), Some(82)), "size");
        assert_ne!(base, cache_name("a.png", 100, 6, "webp", Some(320), Some(82)), "mtime");
        assert_ne!(base, cache_name("a.png", 100, 5, "webp", Some(640), Some(82)), "width");
        assert_ne!(base, cache_name("a.png", 100, 5, "webp", Some(320), Some(90)), "quality");
        assert_ne!(base, cache_name("a.png", 100, 5, "png", Some(320), Some(82)), "target");
    }

    fn fixture(dir: &Path, name: &str, w: u32, h: u32, alpha: bool) -> PathBuf {
        let p = dir.join(name);
        let img = if alpha {
            let mut i = image::RgbaImage::new(w, h);
            for (x, y, px) in i.enumerate_pixels_mut() {
                // an opaque disc on a fully transparent ground, like the die-cut props
                let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
                let d = (((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)) as f32).sqrt();
                *px = if d < w as f32 * 0.4 {
                    image::Rgba([220, 40, 40, 255])
                } else {
                    image::Rgba([0, 255, 0, 0]) // a colour that must never bleed into the edge
                };
            }
            image::DynamicImage::ImageRgba8(i)
        } else {
            image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |x, _| {
                image::Rgb([(x % 256) as u8, 128, 64])
            }))
        };
        img.save(&p).unwrap();
        p
    }

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("curio-derive-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(p.join("cache")).unwrap();
        p
    }

    #[test]
    fn source_for_prefers_the_lossless_master() {
        let d = tmp("source");
        fixture(&d, "both.png", 32, 32, false);
        fixture(&d, "both.jpg", 32, 32, false);
        assert_eq!(source_for(&d.join("both.webp")), Some(d.join("both.png")));
        // and never upward
        assert_eq!(source_for(&d.join("both.png")), None, "a png must not be derived from anything");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn never_upscales_and_says_so_by_returning_the_master() {
        let d = tmp("upscale");
        let src = fixture(&d, "m.png", 200, 100, false);
        let r = Renderer::new(d.join("cache"), 2, 64 << 20);

        // a width beyond the master is the master, not an invention
        let got = r.render(&src, "m.png", "png", Some(4000), None).await.unwrap();
        assert!(!got.derived, "an upscale request must not produce a file");
        assert_eq!(got.path, src);

        // and below it, a real rendition
        let got = r.render(&src, "m.png", "png", Some(64), None).await.unwrap();
        assert!(got.derived);
        let out = image::open(&got.path).unwrap();
        assert_eq!(out.width(), 64);
        assert_eq!(out.height(), 32, "aspect ratio preserved");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn transparent_edges_do_not_bleed() {
        // THE REASON fast_image_resize IS HERE. A naive per-channel scale averages the colour of
        // fully transparent pixels into the visible edge; this fixture makes that loud by using
        // pure green at alpha 0 around a red disc. If premultiplication is missing, the edge
        // pixels go green.
        let d = tmp("alpha");
        let src = fixture(&d, "disc.png", 256, 256, true);
        let r = Renderer::new(d.join("cache"), 2, 64 << 20);
        let got = r.render(&src, "disc.png", "png", Some(64), None).await.unwrap();
        let out = image::open(&got.path).unwrap().to_rgba8();

        let mut worst_green_on_visible = 0u8;
        for px in out.pixels() {
            if px[3] > 32 {
                // on any pixel you can actually see, green must not have leaked in
                worst_green_on_visible = worst_green_on_visible.max(px[1]);
            }
        }
        assert!(worst_green_on_visible < 90,
            "transparent green bled into the visible edge (max g={worst_green_on_visible}); \
             premultiplication is not happening");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn converts_png_to_webp_and_caches_it_once() {
        let d = tmp("webp");
        let src = fixture(&d, "c.png", 128, 128, true);
        let r = Renderer::new(d.join("cache"), 2, 64 << 20);
        let first = r.render(&src, "c.png", "webp", None, None).await.unwrap();
        assert!(first.derived);
        assert_eq!(image::open(&first.path).unwrap().width(), 128);
        let bytes = std::fs::read(&first.path).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF", "not a webp");

        let second = r.render(&src, "c.png", "webp", None, None).await.unwrap();
        assert_eq!(first.path, second.path);
        let files: Vec<_> = std::fs::read_dir(d.join("cache")).unwrap().flatten().collect();
        assert_eq!(files.len(), 1, "rendered twice");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn sixteen_bit_masters_render() {
        // 163 of the 522 pngs in the live store are 16-bit, and the first version of the scaler
        // returned "unsupported pixel type U16x4" for every one of them.
        let d = tmp("sixteen");
        let p = d.join("deep.png");
        let mut img = image::ImageBuffer::<image::Rgba<u16>, Vec<u16>>::new(200, 100);
        for (x, _, px) in img.enumerate_pixels_mut() {
            *px = image::Rgba([(x * 300) as u16, 20000, 40000, 65535]);
        }
        image::DynamicImage::ImageRgba16(img).save(&p).unwrap();

        let r = Renderer::new(d.join("cache"), 2, 64 << 20);
        let got = r.render(&p, "deep.png", "webp", Some(64), None).await.unwrap();
        assert!(got.derived);
        let out = image::open(&got.path).unwrap();
        assert_eq!(out.width(), 64);
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn an_opaque_picture_gets_no_alpha_plane() {
        let d = tmp("opaque");
        let src = fixture(&d, "o.png", 256, 256, false);
        let r = Renderer::new(d.join("cache"), 2, 64 << 20);
        let got = r.render(&src, "o.png", "webp", Some(128), None).await.unwrap();
        let decoded = image::open(&got.path).unwrap();
        assert!(!decoded.color().has_alpha(), "an alpha plane was invented for an opaque image");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn leaves_no_scratch_behind() {
        let d = tmp("scratch");
        let src = fixture(&d, "s.png", 128, 128, false);
        let r = Renderer::new(d.join("cache"), 2, 64 << 20);
        r.render(&src, "s.png", "webp", Some(64), None).await.unwrap();
        let scratch: Vec<String> = std::fs::read_dir(d.join("cache")).unwrap().flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(crate::config::SCRATCH_PREFIX)).collect();
        assert!(scratch.is_empty(), "left staging files: {scratch:?}");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn concurrent_requests_for_one_rendition_all_get_whole_files() {
        let d = tmp("concurrent");
        let src = fixture(&d, "big.png", 900, 900, true);
        let r = Arc::new(Renderer::new(d.join("cache"), 4, 256 << 20));
        let mut set = Vec::new();
        for _ in 0..12 {
            let r = r.clone();
            let src = src.clone();
            set.push(tokio::spawn(async move {
                let got = r.render(&src, "big.png", "webp", Some(640), None).await.unwrap();
                std::fs::read(&got.path).unwrap()
            }));
        }
        let mut sizes = std::collections::HashSet::new();
        for h in set {
            let bytes = h.await.unwrap();
            assert!(image::load_from_memory(&bytes).is_ok(), "a response was not a decodable image");
            sizes.insert(bytes.len());
        }
        assert_eq!(sizes.len(), 1, "requests saw different byte counts: {sizes:?}");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn the_cache_has_a_ceiling() {
        let d = tmp("evict");
        let src = fixture(&d, "e.png", 400, 400, false);
        // a cap small enough that a handful of renditions must not all fit
        let r = Renderer::new(d.join("cache"), 1, 20_000);
        for w in [64u32, 128, 192, 256, 320, 384] {
            r.render(&src, "e.png", "webp", Some(w), None).await.unwrap();
        }
        let total: u64 = std::fs::read_dir(d.join("cache")).unwrap().flatten()
            .filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum();
        assert!(total <= 20_000 + 8_000, "cache grew past its ceiling: {total} bytes");
        std::fs::remove_dir_all(&d).unwrap();
    }
}

#[cfg(test)]
mod eviction_tests {
    use super::*;

    #[tokio::test]
    async fn eviction_never_takes_a_rendition_another_request_just_made() {
        // Measured before this guard existed: 14 of 60 concurrent requests came back 404, because
        // eviction runs at the end of every render and deleted what its neighbours had just
        // published. A 404 is the worst possible answer, being indistinguishable from a name that
        // does not exist — so the caller cannot tell it apart and will not retry.
        let d = std::env::temp_dir().join(format!("curio-evictrace-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("cache")).unwrap();

        // twelve distinct masters, and a ceiling far too small for all of them
        let mut srcs = Vec::new();
        for i in 0..12u32 {
            let p = d.join(format!("m{i}.png"));
            let img = image::RgbImage::from_fn(300, 300, |x, y| {
                image::Rgb([(x % 256) as u8, (y % 256) as u8, (i * 20) as u8])
            });
            image::DynamicImage::ImageRgb8(img).save(&p).unwrap();
            srcs.push((format!("m{i}.png"), p));
        }
        let r = Arc::new(Renderer::new(d.join("cache"), 4, 16 * 1024));

        let mut handles = Vec::new();
        for _round in 0..3 {
            for (name, path) in &srcs {
                let r = r.clone();
                let (name, path) = (name.clone(), path.clone());
                handles.push(tokio::spawn(async move {
                    let got = r.render(&path, &name, "webp", Some(128), None).await.unwrap();
                    // the file must still be there when we come to send it
                    got.path.is_file()
                }));
            }
        }
        let mut vanished = 0;
        for h in handles {
            if !h.await.unwrap() {
                vanished += 1;
            }
        }
        assert_eq!(vanished, 0, "{vanished} renditions were evicted before they could be served");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[tokio::test]
    async fn eviction_still_works_once_the_grace_period_has_passed() {
        let d = std::env::temp_dir().join(format!("curio-evictold-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let cache = d.join("cache");
        std::fs::create_dir_all(&cache).unwrap();

        // pre-existing renditions, backdated past the grace period
        let old_time = std::time::SystemTime::now() - (EVICT_GRACE * 2);
        for i in 0..6 {
            let p = cache.join(format!("old{i}.s1.m1.q82.webp"));
            std::fs::write(&p, vec![0u8; 8 * 1024]).unwrap();
            let f = std::fs::File::options().write(true).open(&p).unwrap();
            f.set_modified(old_time).unwrap();
        }
        let before: u64 = std::fs::read_dir(&cache).unwrap().flatten()
            .filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum();

        let src = d.join("m.png");
        image::DynamicImage::ImageRgb8(image::RgbImage::new(200, 200)).save(&src).unwrap();
        Renderer::new(cache.clone(), 1, 16 * 1024)
            .render(&src, "m.png", "webp", Some(64), None).await.unwrap();

        let after: u64 = std::fs::read_dir(&cache).unwrap().flatten()
            .filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum();
        assert!(after < before, "nothing was evicted: {before} -> {after}");
        assert!(after <= 24 * 1024, "still far over the ceiling: {after}");
        std::fs::remove_dir_all(&d).unwrap();
    }
}
