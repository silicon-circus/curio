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

use crate::media;
use crate::paths;
use crate::AppState;
use axum::body::Body;
use axum::extract::{Path as AxumPath, Request, State};
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
    req: Request,
) -> Response {
    let Some(path) = paths::within(&state.cfg.assets(), &name) else {
        return (StatusCode::BAD_REQUEST, "bad name").into_response();
    };
    if !path.is_file() {
        return (StatusCode::NOT_FOUND, "no such asset").into_response();
    }

    // Prefer the manifest's hash: it is a strong validator and it is already computed. Fall back to
    // size and mtime for an asset that exists but has not been synced yet, which is a weak
    // validator but still a validator.
    let etag = match state.manifest.get().get(&name) {
        Some(e) => format!("\"{}\"", e.hash),
        None => match crate::manifest::stat_of(&path) {
            Ok((size, mtime, _)) => format!("W/\"{size:x}-{mtime:x}\""),
            Err(_) => return (StatusCode::NOT_FOUND, "no such asset").into_response(),
        },
    };

    let served = media::for_path(&path);

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
