//! THE CONSOLE'S API: EVERY STORE'S, UNDER `/api/stores/<name>/`, PLUS THE SWITCH.
//!
//! Reads go straight through to the store with its read token. Writes need three things, checked in
//! this order, and each refusal says which one was missing:
//!
//!   1. the store has a write token in curio's config        otherwise 403
//!   2. update mode is on, and this request carries its key otherwise 423
//!   3. DataDungeon agrees                                   its own answer, passed through
//!
//! The routes are spelled out rather than forwarded wholesale, so curio decides which of a store's
//! endpoints exist here and which token each one gets — a catch-all would send the write token
//! wherever a path happened to match.

// A refusal here IS the answer sent to the browser, so it is a Response, large or not.
#![allow(clippy::result_large_err)]

use crate::dd::{self, Access};
use crate::thumbs::{self, Thumb};
use crate::update;
use crate::AppState;
use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, RawQuery, Request, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

type St = State<Arc<AppState>>;

// ── the gate every request passes ──────────────────────────────────────────────────────────────

const LOOPBACK: [&str; 4] = ["127.0.0.1", "localhost", "[::1]", "::1"];

fn host_is_loopback(host: &str) -> bool {
    let name = if let Some(end) = host.strip_prefix('[').and_then(|h| h.find(']').map(|i| i + 2)) {
        &host[..end]
    } else {
        host.split(':').next().unwrap_or("")
    };
    LOOPBACK.contains(&name)
}

/// EVERY REQUEST MUST PROVE IT CAME FROM THIS MACHINE'S OWN CONSOLE.
///
/// Binding to loopback stops the network; it does not stop a web page. A page with a zero-TTL name
/// that rebinds to 127.0.0.1 is genuinely same-origin with curio, and CORS never stopped a form POST
/// in the first place. So every request needs a loopback `Host`, and one that carries an `Origin`
/// needs it to be the very origin it was sent to. Comparing against the request's own `Host` rather
/// than curio's port is what lets the console work through an SSH tunnel on another local port,
/// while a page served by some other local port is still refused.
///
/// This used to guard only writes, when reads were public assets. Now curio holds every store's read
/// token, so a read is privileged too, and the gate covers everything.
///
/// It is also where using update mode's key counts as activity, so browsing while unlocked keeps the
/// switch on.
pub async fn local_only(State(state): St, req: Request, next: Next) -> Response {
    let host = req.headers().get(header::HOST).and_then(|v| v.to_str().ok()).unwrap_or("");
    if !host_is_loopback(host) {
        return dd::error(StatusCode::FORBIDDEN, "curio answers this machine only".into());
    }
    if let Some(origin) = req.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        if origin != format!("http://{host}") {
            return dd::error(StatusCode::FORBIDDEN, "cross-origin request refused".into());
        }
    }
    // A GET embedded by another page — an <img>, an <audio> — carries no Origin, so the check above
    // never sees it. It could not read what came back, but it would still query the store, and its
    // success or failure says whether an asset exists. Browsers label every request with where it
    // came from; only curio's own page, or an address typed in, is let through.
    if let Some(site) = req.headers().get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        if site != "same-origin" && site != "none" {
            return dd::error(StatusCode::FORBIDDEN, "embedded from another site".into());
        }
    }
    if let Some(key) = update_key(req.headers()) {
        state.update.hold(key);
    }
    next.run(req).await
}

fn update_key(h: &HeaderMap) -> Option<&str> {
    h.get(update::HEADER).and_then(|v| v.to_str().ok()).filter(|k| !k.is_empty())
}

// ── the console and its configuration ─────────────────────────────────────────────────────────

const CONSOLE: &str = include_str!("../public/console.html");

/// The console is compiled in, so the binary needs nothing beside it.
pub async fn console() -> Response {
    let mut res = ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], CONSOLE).into_response();
    let h = res.headers_mut();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    h.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(
        "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; \
         img-src 'self' data:; media-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'"));
    h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    res
}

pub async fn health(State(state): St) -> Response {
    no_store(Json(json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "stores": state.cfg.stores.len(),
    })).into_response())
}

/// The stores and whether each can be curated. Tokens are never part of any answer.
pub async fn stores(State(state): St) -> Response {
    no_store(Json(json!({
        "stores": state.cfg.stores.iter().map(|s| json!({
            "name": s.name, "url": s.url, "writable": s.writable(),
        })).collect::<Vec<_>>(),
        "idle_seconds": state.update.idle().as_secs(),
    })).into_response())
}

// ── update mode ────────────────────────────────────────────────────────────────────────────────

pub async fn update_status(State(state): St, headers: HeaderMap) -> Response {
    let on = update_key(&headers).is_some_and(|k| state.update.hold(k));
    no_store(Json(json!({ "on": on, "idle_seconds": state.update.idle().as_secs() })).into_response())
}

pub async fn update_on(State(state): St) -> Response {
    if !state.cfg.stores.iter().any(|s| s.writable()) {
        return dd::error(StatusCode::CONFLICT,
            "no store has a write token, so there is nothing update mode could change".into());
    }
    let key = state.update.unlock();
    no_store(Json(json!({ "on": true, "key": key, "idle_seconds": state.update.idle().as_secs() })).into_response())
}

pub async fn update_off(State(state): St) -> Response {
    state.update.lock();
    no_store(Json(json!({ "on": false })).into_response())
}

// ── reads ──────────────────────────────────────────────────────────────────────────────────────

fn store<'a>(state: &'a AppState, name: &str) -> Result<&'a crate::config::Store, Response> {
    state.cfg.store(name).ok_or_else(|| dd::error(StatusCode::NOT_FOUND, format!("no store called {name:?}")))
}

async fn read(state: &AppState, name: &str, segments: &[&str], query: Option<&str>) -> Response {
    match store(state, name) {
        Ok(s) => state.dd.relay(s, Access::Read, Method::GET, segments, query, None).await,
        Err(e) => e,
    }
}

pub async fn types(State(state): St, Path(name): Path<String>) -> Response {
    read(&state, &name, &["types"], None).await
}

pub async fn tag_keys(State(state): St, Path((name, ty)): Path<(String, String)>) -> Response {
    read(&state, &name, &["types", &ty, "tags"], None).await
}

pub async fn tag_values(
    State(state): St,
    Path((name, ty, key)): Path<(String, String, String)>,
    RawQuery(q): RawQuery,
) -> Response {
    read(&state, &name, &["types", &ty, "tags", &key], q.as_deref()).await
}

/// How many of a type's pictures are seen from each named direction.
pub async fn views(State(state): St, Path((name, ty)): Path<(String, String)>, RawQuery(q): RawQuery) -> Response {
    read(&state, &name, &["types", &ty, "views"], q.as_deref()).await
}

pub async fn assets(State(state): St, Path(name): Path<String>, RawQuery(q): RawQuery) -> Response {
    read(&state, &name, &["assets"], q.as_deref()).await
}

pub async fn asset(State(state): St, Path((name, id)): Path<(String, i64)>) -> Response {
    read(&state, &name, &["assets", &id.to_string()], None).await
}

/// What an asset links to — `used-by` boardwalk, `derived-from` another asset — and what links to it.
pub async fn links(State(state): St, Path((name, id)): Path<(String, i64)>) -> Response {
    read(&state, &name, &["assets", &id.to_string(), "links"], None).await
}

pub async fn check(State(state): St, Path(name): Path<String>) -> Response {
    match store(&state, &name) {
        Ok(s) => state.dd.relay(s, Access::Read, Method::POST, &["maintenance", "check"], None, None).await,
        Err(e) => e,
    }
}

#[derive(Deserialize)]
pub struct ObjectQuery {
    download: Option<String>,
}

/// AN ASSET'S BYTES, SERVED FROM CURIO'S OWN ORIGIN — WHICH IS WHY THEY ARE SANDBOXED.
///
/// A store can hold HTML or SVG, and either one opened as a document would run its script with
/// curio's origin, same-origin with an API that holds every store's tokens. `sandbox` puts any such
/// document in a unique origin with no script at all, while leaving the same bytes working in an
/// `<img>`, `<audio>` or `<video>`, which is how the console shows them.
pub async fn object(
    State(state): St,
    Path((name, id)): Path<(String, i64)>,
    Query(q): Query<ObjectQuery>,
) -> Response {
    let s = match store(&state, &name) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let mut res = state.dd.relay(s, Access::Read, Method::GET, &["assets", &id.to_string(), "object"], None, None).await;
    let h = res.headers_mut();
    h.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    h.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(
        "sandbox; default-src 'none'; img-src 'self'; media-src 'self'; style-src 'unsafe-inline'"));
    if q.download.is_some() {
        let ext = h.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).and_then(extension).unwrap_or("bin");
        if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{name}-{id}.{ext}\"")) {
            h.insert(header::CONTENT_DISPOSITION, v);
        }
    }
    res
}

fn extension(content_type: &str) -> Option<&'static str> {
    Some(match content_type.split(';').next()?.trim() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        "audio/mpeg" => "mp3",
        "audio/ogg" | "audio/opus" => "opus",
        "audio/wav" | "audio/x-wav" => "wav",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "application/pdf" => "pdf",
        "application/json" => "json",
        "text/plain" => "txt",
        "text/markdown" => "md",
        _ => return None,
    })
}

#[derive(Deserialize)]
pub struct ThumbQuery {
    w: Option<u32>,
}

/// A small webp of a picture, made once per hash and width. See thumbs.rs for why the hash is in the
/// URL and why it is checked rather than believed.
pub async fn thumb(
    State(state): St,
    Path((name, id, hash)): Path<(String, i64, String)>,
    Query(q): Query<ThumbQuery>,
) -> Response {
    if !thumbs::valid_hash(&hash) {
        return dd::error(StatusCode::BAD_REQUEST, "a thumbnail is asked for by the asset's file_hash".into());
    }
    let width = thumbs::quantise_width(q.w.unwrap_or(thumbs::DEFAULT_WIDTH));
    if let Some(path) = state.thumbs.cached(&hash, width) {
        return send_thumb(&path).await;
    }
    let s = match store(&state, &name) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let mut res = match state.dd.send(s, Access::Read, Method::GET, &["assets", &id.to_string(), "object"], None, None).await {
        Ok(r) => r,
        Err(e) => return e,
    };
    if res.status() != StatusCode::OK {
        return dd::relay(res, s);
    }
    let ct = res.headers().get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    if !thumbs::is_raster(&ct) {
        return dd::error(StatusCode::UNSUPPORTED_MEDIA_TYPE, format!("no thumbnail for {ct}"));
    }
    let mut bytes = Vec::new();
    loop {
        match res.chunk().await {
            Ok(Some(c)) => {
                if bytes.len() + c.len() > thumbs::MAX_SOURCE_BYTES {
                    return dd::error(StatusCode::PAYLOAD_TOO_LARGE, "too large to make a thumbnail of".into());
                }
                bytes.extend_from_slice(&c);
            }
            Ok(None) => break,
            Err(e) => return dd::error(StatusCode::BAD_GATEWAY, format!("{name}: object cut off: {e}")),
        }
    }
    match state.thumbs.make(&hash, width, bytes).await {
        Ok(Thumb::Cached(path)) => send_thumb(&path).await,
        Ok(Thumb::Unkept(webp)) => {
            eprintln!("curio: {name} asset {id}: its bytes do not hash to {hash}; thumbnail sent but not kept");
            no_store(([(header::CONTENT_TYPE, "image/webp")], webp).into_response())
        }
        Err(e) => {
            eprintln!("curio: {name} asset {id}: no thumbnail: {e:#}");
            dd::error(StatusCode::UNPROCESSABLE_ENTITY, format!("could not make a thumbnail: {e:#}"))
        }
    }
}

async fn send_thumb(path: &std::path::Path) -> Response {
    match tokio::fs::read(path).await {
        Ok(bytes) => ([
            (header::CONTENT_TYPE, "image/webp"),
            // The hash is in the URL, so this URL's answer can never change.
            (header::CACHE_CONTROL, "private, max-age=31536000, immutable"),
        ], Body::from(bytes)).into_response(),
        // Evicted between the check and the read. Rare, and the console retries a broken image.
        Err(_) => dd::error(StatusCode::SERVICE_UNAVAILABLE, "thumbnail went missing; ask again".into()),
    }
}

// ── writes ─────────────────────────────────────────────────────────────────────────────────────

/// The two checks that are curio's to make. DataDungeon makes the third.
fn may_write<'a>(state: &'a AppState, name: &str, headers: &HeaderMap) -> Result<&'a crate::config::Store, Response> {
    let s = store(state, name)?;
    if !s.writable() {
        return Err(dd::error(StatusCode::FORBIDDEN,
            format!("{name} is read-only here: curio's config gives it no write token")));
    }
    if !update_key(headers).is_some_and(|k| state.update.hold(k)) {
        return Err(dd::error(StatusCode::LOCKED, "Update mode is off. Turn it on to make changes.".into()));
    }
    Ok(s)
}

async fn write(
    state: &AppState,
    name: &str,
    headers: &HeaderMap,
    method: Method,
    segments: &[&str],
    query: Option<&str>,
    body: Option<Bytes>,
) -> Response {
    match may_write(state, name, headers) {
        Ok(s) => no_store(state.dd.relay(s, Access::Write, method, segments, query, body).await),
        Err(e) => e,
    }
}

pub async fn update_asset(
    State(state): St,
    Path((name, id)): Path<(String, i64)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    write(&state, &name, &headers, Method::PATCH, &["assets", &id.to_string()], None, Some(body)).await
}

pub async fn delete_asset(State(state): St, Path((name, id)): Path<(String, i64)>, headers: HeaderMap) -> Response {
    write(&state, &name, &headers, Method::DELETE, &["assets", &id.to_string()], None, None).await
}

pub async fn promote(
    State(state): St,
    Path((name, ty)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    write(&state, &name, &headers, Method::POST, &["types", &ty, "promote"], None, Some(body)).await
}

pub async fn vacuum(State(state): St, Path(name): Path<String>, headers: HeaderMap, body: Bytes) -> Response {
    write(&state, &name, &headers, Method::POST, &["maintenance", "vacuum"], None, Some(body)).await
}

/// Retiring hides an asset and keeps it — curation's everyday soft delete — and reinstating undoes it.
/// Delete stays the final step.
pub async fn retire(State(state): St, Path((name, id)): Path<(String, i64)>, headers: HeaderMap) -> Response {
    write(&state, &name, &headers, Method::POST, &["assets", &id.to_string(), "retire"], None, None).await
}

pub async fn reinstate(State(state): St, Path((name, id)): Path<(String, i64)>, headers: HeaderMap) -> Response {
    write(&state, &name, &headers, Method::POST, &["assets", &id.to_string(), "reinstate"], None, None).await
}

/// `{type, keep_last, retired_before}`: delete a type's retired assets for good.
pub async fn prune(State(state): St, Path(name): Path<String>, headers: HeaderMap, body: Bytes) -> Response {
    write(&state, &name, &headers, Method::POST, &["maintenance", "prune"], None, Some(body)).await
}

/// `{type, all}`: have DataDungeon measure its pictures — size, transparency, dominant colour and the
/// box around the visible pixels. A write, because it fills fields in the store.
pub async fn measure(State(state): St, Path(name): Path<String>, headers: HeaderMap, body: Bytes) -> Response {
    write(&state, &name, &headers, Method::POST, &["maintenance", "measure"], None, Some(body)).await
}

/// `{rel, name}` or `{rel, to}`: link this asset to a name (`used-by` boardwalk) or to another asset.
pub async fn link(
    State(state): St,
    Path((name, id)): Path<(String, i64)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    write(&state, &name, &headers, Method::POST, &["assets", &id.to_string(), "links"], None, Some(body)).await
}

/// `?rel=&name=` or `?rel=&to=`, passed through as given.
pub async fn unlink(
    State(state): St,
    Path((name, id)): Path<(String, i64)>,
    RawQuery(q): RawQuery,
    headers: HeaderMap,
) -> Response {
    write(&state, &name, &headers, Method::DELETE, &["assets", &id.to_string(), "links"], q.as_deref(), None).await
}

fn no_store(mut res: Response) -> Response {
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_hosts_are_recognised_with_and_without_a_port() {
        for h in ["127.0.0.1", "127.0.0.1:19463", "localhost", "localhost:19463", "[::1]:19463"] {
            assert!(host_is_loopback(h), "{h} should be loopback");
        }
        for h in ["attacker.example", "attacker.example:19463", "192.168.1.14:19463", "", "0.0.0.0"] {
            assert!(!host_is_loopback(h), "{h} must not pass");
        }
    }

    #[test]
    fn downloads_get_a_sensible_extension() {
        assert_eq!(extension("audio/mpeg"), Some("mp3"));
        assert_eq!(extension("image/jpeg; charset=binary"), Some("jpg"));
        assert_eq!(extension("application/x-unknown"), None);
    }
}
