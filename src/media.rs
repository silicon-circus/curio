//! WHAT A NAME IS ALLOWED TO BE SERVED AS.
//!
//! An asset server that reflects the filename's extension straight into Content-Type will serve
//! whatever somebody publishes. The Crystal version did, and `/a/x.html` came back as `text/html`
//! with a year-long cache header — script execution on curio's own origin, same-origin to every
//! route that mutates the store, and `nosniff` is no help when the declared type IS html.
//!
//! So the type is chosen from a table, not inferred, and the two types that can carry script get
//! a policy with them:
//!
//!   svg   renders in an <img> either way; script inside one only runs when the SVG is the
//!         document. A restrictive CSP stops that while leaving the eleven favicons and icons in
//!         the store working exactly as before.
//!   html  gets `sandbox allow-scripts`, which puts the page in a UNIQUE origin. It still renders
//!         and its own inline script still runs — the one HTML asset in the store is a
//!         self-contained page with no fetch calls — but it is same-origin with nothing, so it
//!         cannot drive curio's API.
//!
//! Anything not in the table is served as a download rather than guessed at.

pub struct Served {
    pub content_type: &'static str,
    pub csp: Option<&'static str>,
    pub attachment: bool,
}

const OCTET: &str = "application/octet-stream";

pub fn for_path(path: &std::path::Path) -> Served {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let (content_type, csp) = match ext.as_str() {
        "png" => ("image/png", None),
        "webp" => ("image/webp", None),
        "jpg" | "jpeg" => ("image/jpeg", None),
        "gif" => ("image/gif", None),
        "avif" => ("image/avif", None),
        "tif" | "tiff" => ("image/tiff", None),
        "bmp" => ("image/bmp", None),
        "ico" => ("image/x-icon", None),

        "svg" => ("image/svg+xml", Some("default-src 'none'; style-src 'unsafe-inline'")),
        "html" | "htm" => ("text/html; charset=utf-8", Some("sandbox allow-scripts")),

        "glb" => ("model/gltf-binary", None),
        "gltf" => ("model/gltf+json", None),
        "bin" => (OCTET, None),

        "mp4" => ("video/mp4", None),
        "webm" => ("video/webm", None),
        "mp3" => ("audio/mpeg", None),
        "wav" => ("audio/wav", None),
        "ogg" => ("audio/ogg", None),

        "json" => ("application/json", None),
        "md" | "txt" | "csv" | "tsv" | "yml" | "yaml" | "url" => ("text/plain; charset=utf-8", None),

        // GIMP sources and anything else: a download, not a guess.
        _ => return Served { content_type: OCTET, csp: None, attachment: true },
    };
    Served { content_type, csp, attachment: false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn images_get_their_type_and_no_policy() {
        for (n, t) in [("a.png", "image/png"), ("a.webp", "image/webp"), ("a.JPG", "image/jpeg")] {
            let s = for_path(Path::new(n));
            assert_eq!(s.content_type, t, "{n}");
            assert!(s.csp.is_none());
            assert!(!s.attachment);
        }
    }

    #[test]
    fn svg_still_renders_but_cannot_run_script() {
        let s = for_path(Path::new("boardwalk.favicon.svg"));
        assert_eq!(s.content_type, "image/svg+xml", "the favicons must keep working");
        assert_eq!(s.csp, Some("default-src 'none'; style-src 'unsafe-inline'"));
        assert!(!s.attachment);
    }

    #[test]
    fn html_is_sandboxed_into_a_unique_origin() {
        let s = for_path(Path::new("pirateship.tunnel-balance-crossing.html"));
        assert_eq!(s.csp, Some("sandbox allow-scripts"),
            "html must not be same-origin with curio's own API");
    }

    #[test]
    fn models_and_media_are_typed_for_the_loaders_that_want_them() {
        assert_eq!(for_path(Path::new("x.glb")).content_type, "model/gltf-binary");
        assert_eq!(for_path(Path::new("scene.bin")).content_type, OCTET);
        assert_eq!(for_path(Path::new("x.mp4")).content_type, "video/mp4");
    }

    #[test]
    fn unknown_extensions_are_downloads_not_guesses() {
        for n in ["master.xcf", "weird.zzz", "noextension"] {
            let s = for_path(Path::new(n));
            assert_eq!(s.content_type, OCTET, "{n}");
            assert!(s.attachment, "{n} should be offered as a download");
        }
    }
}
