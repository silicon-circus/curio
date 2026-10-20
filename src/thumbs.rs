//! THUMBNAILS, BECAUSE A GRID OF MASTERS IS TENS OF MEGABYTES.
//!
//! DataDungeon streams an asset's bytes at full size and nothing else, and that is right for it — it
//! stays lean, and making smaller copies is a viewer's concern. A page of fifty 1.7 MB PNGs is not a
//! page anybody wants to wait for, so curio makes a small webp of each picture once and keeps it.
//!
//! KEYED BY THE ASSET'S HASH. DataDungeon's `file_hash` is the SHA-256 of the object's bytes, so a
//! hash always means the same picture, in any store, forever. Nothing here is ever invalidated: an
//! asset whose bytes change has a new hash and so a new thumbnail, and the old one ages out of the
//! cache. That is also why a thumbnail can be cached by the browser as immutable — the hash is in its
//! URL.
//!
//! THE HASH IS CHECKED, NOT TRUSTED. It arrives in the URL, so a page could pair one asset's id with
//! another's hash and, if the thumbnail were filed under the hash it was told, every later request
//! for the second picture would show the first. The bytes are hashed when they arrive, and a
//! thumbnail is cached only under the hash it actually has. One that does not match is still made
//! and sent, but not kept.
//!
//! Decoding, scaling and encoding happen in process, bounded by a semaphore, with predictable memory.

use anyhow::{anyhow, bail, Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// The formats worth decoding. Anything else gets an icon in the console, not a thumbnail.
///
/// TIFF IS HERE FOR THE MASTERS. The park keeps its originals as eight-bit TIFFs in 64×64 tiles,
/// deflated, as DataDungeon 0.15.4 writes them (Thomas, 2026-10-20). Tiles let transfs, where
/// DataDungeon's files are to live, find what two saves share by chunking.
/// Only Safari can show a TIFF, so curio's webp is how anyone else sees one.
pub const RASTER: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif", "image/tiff"];

/// The largest object curio will pull in to make a thumbnail of.
pub const MAX_SOURCE_BYTES: usize = 96 << 20;

pub const DEFAULT_WIDTH: u32 = 320;
const MAX_WIDTH: u32 = 1024;
const QUALITY: f32 = 80.0;

/// Bumped when the rendering changes, so old thumbnails are not served as new ones.
const VERSION: &str = "v1";
const STAGE_PREFIX: &str = ".stage-";

pub fn is_raster(content_type: &str) -> bool {
    let base = content_type.split(';').next().unwrap_or("").trim();
    RASTER.contains(&base)
}

/// Widths are quantised up to a multiple of 64 rather than allowlisted, so the console can ask for
/// what its layout needs at the screen's pixel density without the cache growing a copy per pixel.
pub fn quantise_width(w: u32) -> u32 {
    (w.clamp(1, MAX_WIDTH).div_ceil(64) * 64).min(MAX_WIDTH)
}

pub fn valid_hash(h: &str) -> bool {
    h.len() == 64 && h.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub struct Thumbs {
    dir: PathBuf,
    permits: Arc<Semaphore>,
    max_bytes: u64,
    /// The cache's size, kept as a running total so that making a thumbnail does not mean listing
    /// the whole cache. Recounted whenever eviction runs.
    total: AtomicU64,
    seq: AtomicU64,
}

pub enum Thumb {
    /// Made and kept, or already kept.
    Cached(PathBuf),
    /// Made from bytes whose hash was not the one asked for: correct to send, wrong to keep.
    Unkept(Vec<u8>),
}

impl Thumbs {
    pub fn new(root: &Path, jobs: usize, max_bytes: u64) -> Result<Self> {
        let dir = root.join(VERSION);
        std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        let mut total = 0;
        for e in std::fs::read_dir(&dir)? {
            let e = e?;
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with(STAGE_PREFIX) {
                // Left by a process that died mid-write. Never a whole thumbnail.
                let _ = std::fs::remove_file(e.path());
            } else if let Ok(m) = e.metadata() {
                total += m.len();
            }
        }
        Ok(Thumbs {
            dir,
            permits: Arc::new(Semaphore::new(jobs.max(1))),
            max_bytes,
            total: AtomicU64::new(total),
            seq: AtomicU64::new(0),
        })
    }

    pub fn path_for(&self, hash: &str, width: u32) -> PathBuf {
        self.dir.join(format!("{hash}.w{width}.webp"))
    }

    /// The cached thumbnail, if there is one.
    pub fn cached(&self, hash: &str, width: u32) -> Option<PathBuf> {
        let p = self.path_for(hash, width);
        p.is_file().then_some(p)
    }

    /// Make a thumbnail of `bytes`, which were fetched as the asset whose hash is `hash`.
    pub async fn make(&self, hash: &str, width: u32, bytes: Vec<u8>) -> Result<Thumb> {
        let _permit = self.permits.acquire().await.map_err(|e| anyhow!("thumbnail queue closed: {e}"))?;
        let hash = hash.to_string();
        let dest = self.path_for(&hash, width);
        let stage = self.dir.join(format!("{STAGE_PREFIX}{}-{}-{hash}.w{width}",
            std::process::id(), self.seq.fetch_add(1, Ordering::Relaxed)));
        let made = tokio::task::spawn_blocking(move || -> Result<(bool, Vec<u8>)> {
            let genuine = hex(&Sha256::digest(&bytes)) == hash;
            Ok((genuine, render(&bytes, width)?))
        })
        .await
        .context("thumbnail task panicked")??;

        let (genuine, webp) = made;
        if !genuine {
            return Ok(Thumb::Unkept(webp));
        }
        // STAGE, THEN RENAME. A rename inside one directory is atomic, so the final name either does
        // not exist or is whole — a concurrent request never reads half a file. The stage name is
        // unique per call, because two requests for the same thumbnail at once are ordinary.
        std::fs::write(&stage, &webp).with_context(|| format!("writing {}", stage.display()))?;
        if let Err(e) = std::fs::rename(&stage, &dest) {
            let _ = std::fs::remove_file(&stage);
            return Err(e).context("filing the thumbnail");
        }
        let total = self.total.fetch_add(webp.len() as u64, Ordering::Relaxed) + webp.len() as u64;
        if self.max_bytes > 0 && total > self.max_bytes {
            match evict_to_fit(&self.dir, self.max_bytes, &dest) {
                Ok(left) => self.total.store(left, Ordering::Relaxed),
                Err(e) => eprintln!("curio: could not trim the thumbnail cache: {e:#}"),
            }
        }
        Ok(Thumb::Cached(dest))
    }
}

/// Decode, shrink to `width` if the picture is wider, and encode as webp. Never wider than the
/// picture itself: a thumbnail is not where pixels get invented.
fn render(bytes: &[u8], width: u32) -> Result<Vec<u8>> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .context("reading the picture's format")?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(1 << 30);
    reader.limits(limits);
    let img = to_eight_bit(reader.decode().context("decoding")?);
    let img = if img.width() > width { scale_to_width(&img, width)? } else { img };
    encode_webp(&img)
}

/// SIXTEEN-BIT MASTERS ARE REAL — nearly a third of the PNGs in boardwalk's art. They decode to
/// Rgba16, and the scaler handles only eight-bit layouts, so everything is normalised to eight bits
/// first. webp has no sixteen-bit form, so for a thumbnail this loses nothing.
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
/// Much of the art is RGBA with fully transparent edges by design. Scaling those channel by channel
/// averages the colour of transparent pixels into the visible edge and haloes exactly the silhouettes
/// that were carefully cut out. fast_image_resize multiplies by alpha before scaling and divides
/// after, which is the correct operation, and is SIMD-accelerated as well.
///
/// MITCHELL rather than Lanczos3, because sharpening a downscale is ringing, and ringing is detail
/// a lossy encoder has to spend bytes on. Measured over sixteen real masters at 320px: Lanczos3 came
/// out 19% larger at the median, Mitchell 2.6% smaller.
fn scale_to_width(img: &image::DynamicImage, width: u32) -> Result<image::DynamicImage> {
    use fast_image_resize::images::Image;
    use fast_image_resize::{FilterType, IntoImageView, ResizeAlg, ResizeOptions, Resizer};

    let height = ((img.height() as u64 * width as u64) / img.width().max(1) as u64).max(1) as u32;
    let pixel_type = img.pixel_type().ok_or_else(|| anyhow!("unsupported pixel layout"))?;
    let mut dst = Image::new(width, height, pixel_type);
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

/// libwebp, because `image`'s own webp encoder is lossless only. An alpha channel is carried only
/// when there is one: forcing RGBA on an opaque picture makes libwebp encode an all-255 alpha plane
/// and the file comes out about a quarter larger for nothing.
fn encode_webp(img: &image::DynamicImage) -> Result<Vec<u8>> {
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        Ok(webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height()).encode(QUALITY).to_vec())
    } else {
        let rgb = img.to_rgb8();
        Ok(webp::Encoder::from_rgb(rgb.as_raw(), rgb.width(), rgb.height()).encode(QUALITY).to_vec())
    }
}

/// Oldest first, with a grace period. Without one, eviction deletes thumbnails that other requests
/// filed moments ago and are about to send, and the browser gets a 404 it cannot tell from "no such
/// asset". So anything younger than the grace period is off limits, as is the thumbnail just made,
/// and a ceiling smaller than the working set is overshot rather than enforced: a soft cap briefly
/// exceeded costs disk, a hard one costs correctness. Returns the size left.
const EVICT_GRACE: std::time::Duration = std::time::Duration::from_secs(60);

fn evict_to_fit(dir: &Path, max_bytes: u64, just_written: &Path) -> Result<u64> {
    let now = std::time::SystemTime::now();
    let mut eligible: Vec<(std::time::SystemTime, u64, PathBuf)> = Vec::new();
    let mut total: u64 = 0;
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        if e.file_name().to_string_lossy().starts_with(STAGE_PREFIX) {
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
            continue;
        }
        eligible.push((modified, m.len(), e.path()));
    }
    eligible.sort_by_key(|(modified, _, _)| *modified);
    for (_, len, path) in eligible {
        if total <= max_bytes {
            break;
        }
        if std::fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(len);
        }
    }
    Ok(total)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(w: u32, h: u32, alpha: bool) -> Vec<u8> {
        let img = if alpha {
            image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(w, h, |x, _| image::Rgba([x as u8, 80, 160, 128])))
        } else {
            image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |x, _| image::Rgb([x as u8, 80, 160])))
        };
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    fn decoded(webp: &[u8]) -> image::DynamicImage {
        image::load_from_memory_with_format(webp, image::ImageFormat::WebP).unwrap()
    }

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("curio-thumbs-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn widths_round_up_to_a_multiple_of_64_and_stop_at_1024() {
        assert_eq!(quantise_width(1), 64);
        assert_eq!(quantise_width(320), 320);
        assert_eq!(quantise_width(321), 384);
        assert_eq!(quantise_width(5000), 1024);
    }

    #[test]
    fn only_lowercase_sha256_hex_is_a_hash() {
        assert!(valid_hash(&"a".repeat(64)));
        assert!(!valid_hash(&"A".repeat(64)));
        assert!(!valid_hash(&"a".repeat(63)));
        assert!(!valid_hash(&format!("../{}", "a".repeat(61))), "it becomes a filename");
    }

    #[test]
    fn content_types_with_parameters_still_count() {
        assert!(is_raster("image/png"));
        assert!(is_raster("image/jpeg; charset=binary"));
        assert!(!is_raster("image/svg+xml"));
        assert!(!is_raster("audio/mpeg"));
    }

    #[test]
    fn a_wide_picture_shrinks_and_keeps_its_shape() {
        let out = decoded(&render(&png(800, 400, false), 320).unwrap());
        assert_eq!((out.width(), out.height()), (320, 160));
        assert!(!out.color().has_alpha(), "an opaque picture gets no alpha plane");
    }

    /// Pictures from `png` saved by libvips as a master is kept: eight bits in 64×64 tiles,
    /// uncompressed (70×10) or deflated with the horizontal predictor (200×100). Neither size fills
    /// its tiles, so the part-filled ones on the right and bottom are checked too.
    #[test]
    fn a_tiled_tiff_master_renders_exactly_as_its_png_does() {
        let none = include_bytes!("testdata/tiled-rgba-none.tif");
        assert_eq!(render(none, 320).unwrap(), render(&png(70, 10, true), 320).unwrap());
        assert_eq!(render(none, 64).unwrap(), render(&png(70, 10, true), 64).unwrap());
        let deflated = include_bytes!("testdata/tiled-rgba.tif");
        assert_eq!(render(deflated, 320).unwrap(), render(&png(200, 100, true), 320).unwrap());
        assert_eq!(render(deflated, 128).unwrap(), render(&png(200, 100, true), 128).unwrap());
        assert!(is_raster("image/tiff"));
    }

    #[test]
    fn a_narrow_picture_is_never_enlarged() {
        let out = decoded(&render(&png(100, 50, true), 320).unwrap());
        assert_eq!((out.width(), out.height()), (100, 50));
        assert!(out.color().has_alpha());
    }

    #[tokio::test]
    async fn a_thumbnail_is_kept_only_under_the_hash_its_bytes_have() {
        let root = scratch("hash");
        let t = Thumbs::new(&root, 1, 0).unwrap();
        let bytes = png(640, 320, false);
        let real = hex(&Sha256::digest(&bytes));
        let other = "0".repeat(64);

        match t.make(&other, 320, bytes.clone()).await.unwrap() {
            Thumb::Unkept(webp) => assert_eq!(decoded(&webp).width(), 320, "still made and sent"),
            Thumb::Cached(_) => panic!("filed under a hash the bytes do not have"),
        }
        assert!(t.cached(&other, 320).is_none());

        match t.make(&real, 320, bytes).await.unwrap() {
            Thumb::Cached(p) => assert_eq!(p, t.path_for(&real, 320)),
            Thumb::Unkept(_) => panic!("genuine bytes should be kept"),
        }
        assert!(t.cached(&real, 320).is_some());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[tokio::test]
    async fn the_same_thumbnail_made_twice_at_once_is_not_an_error() {
        let root = scratch("race");
        let t = Arc::new(Thumbs::new(&root, 4, 0).unwrap());
        let bytes = png(640, 320, false);
        let hash = hex(&Sha256::digest(&bytes));
        let jobs: Vec<_> = (0..4).map(|_| {
            let (t, b, h) = (t.clone(), bytes.clone(), hash.clone());
            tokio::spawn(async move { t.make(&h, 320, b).await.map(|_| ()) })
        }).collect();
        for j in jobs {
            j.await.unwrap().unwrap();
        }
        let left: Vec<_> = std::fs::read_dir(root.join(VERSION)).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(left.len(), 1, "one thumbnail and no stray stage files: {left:?}");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_dead_process_stage_files_are_swept_on_start() {
        let root = scratch("sweep");
        std::fs::create_dir_all(root.join(VERSION)).unwrap();
        std::fs::write(root.join(VERSION).join(".stage-1-0-x.w320"), b"half").unwrap();
        let t = Thumbs::new(&root, 1, 0).unwrap();
        assert_eq!(t.total.load(Ordering::Relaxed), 0);
        assert_eq!(std::fs::read_dir(root.join(VERSION)).unwrap().count(), 0);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
