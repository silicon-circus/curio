//! SERVING, BY NAME. The name is the whole address.
//!
//! Three things this route does that the Crystal version did not:
//!
//!   1. The requested name goes through `paths::within`, so a symlink planted in assets/ cannot
//!      reach out of it. (Traversal spelled with ".." was already blocked, by the router's own
//!      normalisation and a string check — but only on the read routes.)
//!   2. Content-Type comes from an allowlist, not from the extension. See media.rs.
//!   3. There is a validator. The old route carried `max-age=31536000` and a comment claiming "a
//!      weak validator for the name itself" that was never implemented, so a client that cached a
//!      stale — or truncated — response had no way to ever ask again. Now every response has an
//!      ETag, and the cache window is short enough that a revalidation costs a 304.
//!
//! Access-Control-Allow-Origin is set HERE rather than globally. The park embeds assets from its own
//! origins, so reads need the wildcard; the store's listings and its mutating routes do not, and a
//! global filter is how they got it.

use crate::derive;
use crate::media;
use crate::paths;
use crate::AppState;
use axum::body::Body;
use axum::extract::{Path as AxumPath, Query, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::services::ServeFile;

/// Short, because an asset in assets/ is a file Tom edits in place. Revalidation is a 304 against
/// the ETag, so the cost of being wrong is one conditional request rather than a year of staleness.
const CACHE_CONTROL: &str = "public, max-age=300, must-revalidate";

pub async fn asset(
    State(state): State<Arc<AppState>>,
    AxumPath(name): AxumPath<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    req: Request,
) -> Response {
    let Some(asked) = paths::within(&state.cfg.assets(), &name) else {
        return (StatusCode::BAD_REQUEST, "bad name").into_response();
    };

    // A name that is not a file may still be one curio is allowed to MAKE. A real file always wins:
    // names published in both .png and .webp are common here, and a webp somebody made by hand is
    // not something to second-guess with an encoder.
    let converting = !asked.is_file();
    let src = if converting {
        match derive::source_for(&asked) {
            Some(s) => s,
            None => return (StatusCode::NOT_FOUND, "no such asset").into_response(),
        }
    } else {
        asked.clone()
    };

    // Garbage in a query string is ignored rather than refused: ?w=banana serves the asset, which is
    // what the Crystal did and is the kinder answer for something embedded in a page.
    let width = params.get("w").and_then(|v| v.parse::<u32>().ok()).filter(|w| *w > 0);
    let quality = params.get("q").and_then(|v| v.parse::<u8>().ok());

    let mut path = asked.clone();
    let mut rendition_key: Option<String> = None;

    if converting || width.is_some() {
        let target_ext = derive::ext_of(&asked);
        match state.renderer.render(&src, &name, &target_ext, width, quality).await {
            Ok(r) => {
                if r.derived {
                    rendition_key = r.path.file_name().map(|n| n.to_string_lossy().into_owned());
                }
                path = r.path;
            }
            Err(e) => {
                if converting {
                    // Nothing to fall back ON: the bytes on disk are not the format that was asked
                    // for, and serving a png under a .webp name is a lie the browser believes.
                    eprintln!("curio: cannot render {name}: {e:#}");
                    return (StatusCode::INTERNAL_SERVER_ERROR, "could not render").into_response();
                }
                // A failed RESIZE is different — the master is still a correct answer to the name,
                // just larger than asked for. Serve it whole rather than fail a page over a
                // thumbnail. The Crystal claimed this and did not do it: a missing `magick` was an
                // unhandled exception, so every ?w= URL on the site became a 500.
                eprintln!("curio: serving {name} whole, resize failed: {e:#}");
                path = src.clone();
            }
        }
    }

    if !path.is_file() {
        return (StatusCode::NOT_FOUND, "no such asset").into_response();
    }

    // A rendition's cache filename already encodes everything that determines its bytes — the
    // master's size and mtime, the width, the quality — so it IS a strong validator. For a master,
    // prefer the manifest's hash, and fall back to size and mtime for one not yet synced.
    let etag = match rendition_key {
        Some(key) => format!("\"{key}\""),
        None => match state.manifest.get().get(&name) {
            Some(e) => format!("\"{}\"", e.hash),
            None => match crate::manifest::stat_of(&path) {
                Ok((size, mtime, _)) => format!("W/\"{size:x}-{mtime:x}\""),
                Err(_) => return (StatusCode::NOT_FOUND, "no such asset").into_response(),
            },
        },
    };

    // The type comes from the name that was ASKED for, not the file that answered it: /a/foo.webp
    // served out of foo.png is a webp.
    let served = media::for_path(&asked);

    if let Some(inm) = req.headers().get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()) {
        if inm.split(',').any(|c| c.trim() == etag || c.trim() == "*") {
            let mut res = StatusCode::NOT_MODIFIED.into_response();
            decorate(res.headers_mut(), &etag, &served);
            return res;
        }
    }

    // ServeFile for the body, because it handles Range — the park has video and audio in here and
    // seeking needs it. Its guessed Content-Type is then replaced by ours.
    match ServeFile::new(&path).oneshot(req).await {
        Ok(res) => {
            let (mut parts, body) = res.into_parts();
            decorate(&mut parts.headers, &etag, &served);
            Response::from_parts(parts, Body::new(body))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "could not read asset").into_response(),
    }
}

fn decorate(h: &mut header::HeaderMap, etag: &str, served: &media::Served) {
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(served.content_type));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL));
    h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    if let Ok(v) = HeaderValue::from_str(etag) {
        h.insert(header::ETAG, v);
    }
    if let Some(csp) = served.csp {
        h.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(csp));
    }
    if served.attachment {
        h.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("attachment"));
    }
}
