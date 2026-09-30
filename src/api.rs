//! THE CONSOLE'S DATA, AND THE FOUR ROUTES THAT CHANGE THINGS.
//!
//! WRITES MUST PROVE THEY CAME FROM THIS MACHINE. None of these routes asks who is calling, which is
//! right for a tool serving one browser and wrong the moment the port is reachable from anywhere
//! else. Binding to loopback stops the network; it does not stop a web page.
//!
//! CORS is not the control. Dropping the wildcard from a POST stops an attacker READING the reply,
//! never making the request: a form POST needs no preflight, and nothing validated `Host`, which is
//! the whole basis of DNS rebinding — a page with a zero-TTL name resolves to 127.0.0.1, is then
//! genuinely same-origin, and every content type is free again. So a mutating request needs a
//! loopback `Host` and, when the browser sends one, an `Origin` that is this server.
//!
//! The console is served from here, so it passes without knowing any of this exists.

use crate::config::Config;
use crate::{derive, filing, manifest, store, sync, AppState};
use axum::extract::{Path as AxumPath, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use std::sync::Arc;

const LOOPBACK: [&str; 4] = ["127.0.0.1", "localhost", "[::1]", "::1"];

fn host_is_loopback(host: &str) -> bool {
    let name = if let Some(end) = host.strip_prefix('[').and_then(|h| h.find(']').map(|i| i + 2)) {
        &host[..end]
    } else {
        host.split(':').next().unwrap_or("")
    };
    LOOPBACK.contains(&name)
}

/// The gate. Applied to the mutating routes only; reads are deliberately open, because the whole
/// park embeds assets from its own origins.
pub async fn local_writes_only(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let host = req.headers().get(header::HOST).and_then(|v| v.to_str().ok()).unwrap_or("");
    if !host_is_loopback(host) {
        return (StatusCode::FORBIDDEN, "curio takes writes from this machine only").into_response();
    }
    if let Some(origin) = req.headers().get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        let port = state.cfg.port;
        let allowed = LOOPBACK.iter().any(|h| origin == format!("http://{h}:{port}"));
        if !allowed {
            return (StatusCode::FORBIDDEN, "cross-origin write refused").into_response();
        }
    }
    next.run(req).await
}

fn no_store(mut res: Response) -> Response {
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// WHAT WILL BE SERVED, NOT ONLY WHAT IS STORED.
///
/// `items` is the manifest: one row per file. That was the whole answer until formats started being
/// derived, and then it became a lie by omission — a rendition is never a manifest entry, because
/// only the master is stored, so `/a/halloween.tent.top.webp` answers 200 while that name appears
/// nowhere in this list. Every consumer then has to rebuild the derivation table to tell "curio will
/// not serve this" from "curio will make this". The rules are ours; they belong in one place.
pub async fn serve_list(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let q = params.get("q").map(|s| s.to_lowercase()).unwrap_or_default();
    let limit = params.get("limit").and_then(|v| v.parse::<usize>().ok()).unwrap_or(500).min(100_000);
    let m = state.manifest.get();

    let names: Vec<&manifest::Entry> = m.values()
        .filter(|e| q.is_empty() || e.name.to_lowercase().contains(&q))
        .collect();

    // A derived name that is ALSO published in its own right is not derivable, it is a name.
    let mut derivable = Vec::new();
    for e in m.values() {
        let ext = derive::ext_of(std::path::Path::new(&e.name));
        for target in derive::targets_for(&ext) {
            let d = format!("{}.{}", e.name.trim_end_matches(&format!(".{ext}")), target);
            if m.contains_key(&d) {
                continue;
            }
            if !q.is_empty() && !d.to_lowercase().contains(&q) {
                continue;
            }
            derivable.push(json!({"name": d, "from": e.name, "url": format!("/a/{d}")}));
        }
    }
    derivable.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));

    no_store(Json(json!({
        "total": names.len(),
        "items": names.iter().take(limit).map(|e| json!({
            "name": e.name, "hash": e.hash, "size": e.size, "url": format!("/a/{}", e.name),
        })).collect::<Vec<_>>(),
        "derivable_total": derivable.len(),
        "derivable": derivable.into_iter().take(limit).collect::<Vec<_>>(),
    })).into_response())
}

pub async fn intake_list(State(state): State<Arc<AppState>>) -> Response {
    let items: Vec<_> = store::intake_list(&state.cfg).iter().map(|w| json!({
        "file": w.file, "size": w.size, "mtime": w.mtime, "url": format!("/intake/{}", w.file),
    })).collect();
    no_store(Json(json!({"items": items})).into_response())
}

/// The console SHOWS the plan; it does not run it. Filing is a command you type.
pub async fn watch_list(State(state): State<Arc<AppState>>) -> Response {
    let items: Vec<_> = filing::plan(&state.cfg).iter().map(|i| json!({
        "kind": match i.kind { filing::Kind::File => "file",
                               filing::Kind::Collection => "collection",
                               filing::Kind::Held => "held" },
        "name": i.name, "files": i.files, "bytes": i.bytes, "why": i.why,
    })).collect();
    no_store(Json(json!({"items": items})).into_response())
}

/// Preview something that has not been kept yet. Same containment as `/a/`, and never cached.
pub async fn intake_file(
    State(state): State<Arc<AppState>>,
    AxumPath(file): AxumPath<String>,
    req: Request,
) -> Response {
    let Some(path) = crate::paths::within(&state.cfg.intake(), &file) else {
        return (StatusCode::BAD_REQUEST, "bad name").into_response();
    };
    if !path.is_file() {
        return (StatusCode::NOT_FOUND, "no such file").into_response();
    }
    let served = crate::media::for_path(&path);
    match tower::ServiceExt::oneshot(tower_http::services::ServeFile::new(&path), req).await {
        Ok(res) => {
            let (mut parts, body) = res.into_parts();
            parts.headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(served.content_type));
            parts.headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            parts.headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
            if let Some(csp) = served.csp {
                parts.headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(csp));
            }
            Response::from_parts(parts, axum::body::Body::new(body))
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "could not read file").into_response(),
    }
}

pub async fn keep(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let file = body["file"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    match store::keep(&state.cfg, file, name) {
        Ok(r) => {
            let code = if r.ok { StatusCode::OK } else { StatusCode::BAD_REQUEST };
            (code, Json(json!({"ok": r.ok, "why": r.why, "url": format!("/a/{}", r.name)}))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false, "why": e.to_string()}))).into_response(),
    }
}

pub async fn trash(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let file = body["file"].as_str().unwrap_or("");
    let intake = state.cfg.intake();
    let ok = store::discard(&state.cfg, &intake, file).unwrap_or(false);
    let code = if ok { StatusCode::OK } else { StatusCode::BAD_REQUEST };
    (code, Json(json!({"ok": ok}))).into_response()
}

pub async fn unpublish(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let name = body["name"].as_str().unwrap_or("");
    let ok = store::unpublish(&state.cfg, name).unwrap_or(false);
    let code = if ok { StatusCode::OK } else { StatusCode::BAD_REQUEST };
    (code, Json(json!({"ok": ok}))).into_response()
}

pub async fn sync_now(State(state): State<Arc<AppState>>) -> Response {
    let cfg: Config = state.cfg.clone();
    let out = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let (plan, next) = sync::scan(&cfg)?;
        let applied = sync::apply(&cfg, &plan, &next)?;
        Ok((plan.unchanged, plan.edited.len(), plan.added.len(), plan.removed.len(),
            plan.renamed.len(), plan.ambiguous.len(), applied.archived))
    }).await;
    match out {
        Ok(Ok((unchanged, edited, added, removed, renamed, ambiguous, archived))) => Json(json!({
            "unchanged": unchanged, "edited": edited, "added": added, "removed": removed,
            "renamed": renamed, "ambiguous": ambiguous, "archived": archived,
        })).into_response(),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ok": false}))).into_response(),
    }
}

pub async fn health(State(state): State<Arc<AppState>>) -> Response {
    let cfg = &state.cfg;
    let mut res = Json(json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "serving": state.manifest.get().len(),
        "intake": store::count_files(&cfg.intake()),
        "watch": store::count_files(&cfg.watch()),
        "held": filing::plan(cfg).iter().filter(|i| i.kind == filing::Kind::Held).count(),
    })).into_response();
    res.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

pub async fn console(State(state): State<Arc<AppState>>) -> Response {
    match std::fs::read(state.cfg.public.join("console.html")) {
        Ok(bytes) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8"),
                       (header::CACHE_CONTROL, "no-cache")], bytes).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "no console.html in CURIO_PUBLIC").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_reports_real_counts() {
        let root = std::env::temp_dir().join(format!("curio-api-health-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let cfg = Config::isolated(root.clone());
        cfg.ensure_dirs().unwrap();
        let mut m = manifest::Manifest::new();
        for n in ["a.png", "b.png", "c.webp"] {
            m.insert(n.into(), manifest::Entry {
                name: n.into(), hash: "x".into(), size: 1, mtime: 1, inode: 1 });
        }
        manifest::save(&cfg.manifest_path(), &m).unwrap();
        std::fs::write(cfg.intake().join("waiting.png"), b"x").unwrap();

        let state = Arc::new(AppState {
            manifest: manifest::Cache::new(cfg.manifest_path()),
            renderer: derive::Renderer::new(cfg.cache(), 1, 1 << 20),
            cfg,
        });
        let res = health(State(state)).await;
        assert_eq!(res.status(), StatusCode::OK);
        let body = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["serving"], 3);
        assert_eq!(v["intake"], 1);
        assert_eq!(v["watch"], 0);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn loopback_hosts_are_recognised_with_and_without_a_port() {
        for h in ["127.0.0.1", "127.0.0.1:19463", "localhost", "localhost:19463", "[::1]:19463"] {
            assert!(host_is_loopback(h), "{h} should be loopback");
        }
        for h in ["attacker.example", "attacker.example:19463", "192.168.1.14:19463", "", "0.0.0.0"] {
            assert!(!host_is_loopback(h), "{h} must not pass");
        }
    }
}
