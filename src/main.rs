//! curio — the Silicon Circus asset server.
//!
//! Tom: "An asset server. The archive can become a project to serve the assets, and keep the object
//! and tagged hardlinks in sync." And: "Served by name. The object store is for dedup mainly."
//!
//! The point is not to automate the chore, it is to delete it. A repo that references
//! /a/boardwalk.cattacula.night.real.webp holds no asset, converts nothing, copies nothing, and
//! cannot drift from the master. There is no step between having a picture and using it.
//!
//! This is the Rust port. See PORT.md for what changed and why — briefly: no subprocess, no
//! objects/, bounded render surface, backup built in, and derivation that goes downward only in
//! size as well as format.

mod config;
mod derive;
mod manifest;
mod media;
mod paths;
mod serve;

use anyhow::Result;
use axum::{routing::get, Json, Router};
use config::Config;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub cfg: Config,
    pub manifest: manifest::Cache,
    pub renderer: derive::Renderer,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::from_env(&project_base());
    cfg.ensure_dirs()?;

    // Bounded by default. Rendering is CPU-bound and every permit is a decode plus a scale held
    // in memory, so the ceiling is deliberate rather than however many requests arrive at once.
    let jobs = std::env::var("CURIO_RENDER_JOBS").ok().and_then(|s| s.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| (n.get() / 2).max(1)).unwrap_or(2).min(4));
    let cache_mb: u64 = std::env::var("CURIO_CACHE_MAX_MB").ok().and_then(|s| s.parse().ok()).unwrap_or(2048);
    let state = Arc::new(AppState {
        manifest: manifest::Cache::new(cfg.manifest_path()),
        renderer: derive::Renderer::new(cfg.cache(), jobs, cache_mb * (1 << 20)),
        cfg,
    });
    eprintln!("  render jobs {jobs}, cache ceiling {cache_mb} MB");
    let app = Router::new()
        .route("/a/{*name}", get(serve::asset))
        .route("/health", get(health))
        .with_state(state.clone());

    let addr = format!("{}:{}", state.cfg.bind, state.cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("curio {} listening on http://{}", env!("CARGO_PKG_VERSION"), addr);
    eprintln!("  assets {}", state.cfg.assets().display());
    axum::serve(listener, app).await?;
    Ok(())
}

/// The directory the project lives in. Deliberately derived from the running binary rather than
/// compiled in: the Crystal version resolved its data path from __DIR__ at COMPILE time, so a
/// binary built in one place and copied to another went looking for the build machine's directory,
/// found nothing, and served an empty store while reporting "ok": true.
fn project_base() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().and_then(|p| p.parent()).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}

async fn health(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let cfg = &state.cfg;
    let m = manifest::load(&cfg.manifest_path()).unwrap_or_default();
    Json(json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "serving": m.len(),
        "intake": count_files(&cfg.intake()),
        "watch": count_files(&cfg.watch()),
    }))
}

fn count_files(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir).map(|rd| rd.flatten().count()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_base_is_the_parent_of_the_bin_directory() {
        // target/debug/curio -> target ; the shape that matters is "not compiled in"
        let base = project_base();
        assert!(base.is_absolute() || base == PathBuf::from("."));
    }

    #[tokio::test]
    async fn health_reports_the_manifest_size() {
        let root = std::env::temp_dir().join(format!("curio-health-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let cfg = Config::for_root(root.clone());
        cfg.ensure_dirs().unwrap();

        let mut m = manifest::Manifest::new();
        for n in ["a.png", "b.png", "c.webp"] {
            m.insert(n.into(), manifest::Entry {
                name: n.into(), hash: "x".into(), size: 1, mtime: 1, inode: 1,
            });
        }
        manifest::save(&cfg.manifest_path(), &m).unwrap();
        std::fs::write(cfg.intake().join("waiting.png"), b"x").unwrap();

        let state = Arc::new(AppState {
            manifest: manifest::Cache::new(cfg.manifest_path()),
            renderer: derive::Renderer::new(cfg.cache(), 1, 1 << 20),
            cfg,
        });
        let Json(v) = health(axum::extract::State(state)).await;
        assert_eq!(v["ok"], true);
        assert_eq!(v["serving"], 3);
        assert_eq!(v["intake"], 1);
        assert_eq!(v["watch"], 0);
        assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));

        std::fs::remove_dir_all(&root).unwrap();
    }
}
