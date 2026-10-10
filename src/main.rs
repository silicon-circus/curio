//! curio — a web UI for DataDungeon stores.
//!
//! Tom, 2026-10-07: DataDungeon is the asset server from now on, and it stays free of UI code; curio
//! becomes the UI. Filter a store's assets by tag, see what is there, and curate it — read-only by
//! default, with a big red switch for when you mean to change something.
//!
//! curio holds no assets and no database. Everything it shows comes from a store's JSON API, and
//! everything it changes is a call to that API. What it adds is what a browser cannot safely do on
//! its own: hold the tokens, keep the write switch honest, and make thumbnails so a page of pictures
//! is not a page of masters.
//!
//! Until 0.3.2 curio was the asset server itself, serving the park's files by name. That program is
//! tagged `v0.3.2-server`.

mod config;
mod dd;
mod routes;
mod thumbs;
mod update;

use anyhow::{Context, Result};
use axum::routing::{get, post};
use axum::Router;
use config::Config;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub struct AppState {
    pub cfg: Config,
    pub dd: dd::Dd,
    pub update: update::UpdateMode,
    pub thumbs: thumbs::Thumbs,
}

pub fn app(state: Arc<AppState>) -> Router {
    let s = "/api/stores/{store}";
    Router::new()
        .route("/", get(routes::console))
        .route("/health", get(routes::health))
        .route("/api/stores", get(routes::stores))
        .route("/api/update", get(routes::update_status).post(routes::update_on).delete(routes::update_off))
        .route(&format!("{s}/types"), get(routes::types))
        .route(&format!("{s}/types/{{ty}}/tags"), get(routes::tag_keys))
        .route(&format!("{s}/types/{{ty}}/tags/{{key}}"), get(routes::tag_values))
        .route(&format!("{s}/types/{{ty}}/promote"), post(routes::promote))
        .route(&format!("{s}/types/{{ty}}/views"), get(routes::views))
        .route(&format!("{s}/assets"), get(routes::assets))
        .route(&format!("{s}/assets/{{id}}"),
            get(routes::asset).patch(routes::update_asset).delete(routes::delete_asset))
        .route(&format!("{s}/assets/{{id}}/links"), get(routes::links).post(routes::link).delete(routes::unlink))
        .route(&format!("{s}/assets/{{id}}/retire"), post(routes::retire))
        .route(&format!("{s}/assets/{{id}}/reinstate"), post(routes::reinstate))
        .route(&format!("{s}/assets/{{id}}/object"), get(routes::object))
        .route(&format!("{s}/assets/{{id}}/thumb/{{hash}}"), get(routes::thumb))
        .route(&format!("{s}/check"), post(routes::check))
        .route(&format!("{s}/vacuum"), post(routes::vacuum))
        .route(&format!("{s}/prune"), post(routes::prune))
        .route(&format!("{s}/measure"), post(routes::measure))
        .layer(axum::middleware::from_fn_with_state(state.clone(), routes::local_only))
        .with_state(state)
}

pub fn state(cfg: Config) -> Result<Arc<AppState>> {
    // Bounded, because every permit is a decoded picture held in memory.
    let jobs = std::thread::available_parallelism().map(|n| (n.get() / 2).max(1)).unwrap_or(2).min(4);
    Ok(Arc::new(AppState {
        dd: dd::Dd::new(),
        update: update::UpdateMode::new(Duration::from_secs(cfg.idle_minutes * 60)),
        thumbs: thumbs::Thumbs::new(&cfg.thumbs, jobs, cfg.thumbs_max_bytes)?,
        cfg,
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |f: &str| args.iter().any(|a| a == f);
    if flag("--help") || flag("-h") {
        print_help();
        return Ok(());
    }
    if flag("--version") {
        println!("curio {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let path = args.iter().position(|a| a == "--config")
        .and_then(|i| args.get(i + 1)).map(PathBuf::from)
        .unwrap_or_else(Config::default_path);
    if !path.is_file() {
        eprintln!("curio: no config at {}\n\n{}", path.display(), SAMPLE);
        std::process::exit(2);
    }
    let cfg = Config::load(&path)?;
    if flag("--stores") {
        return check_stores(&cfg).await;
    }

    let addr = format!("{}:{}", cfg.bind, cfg.port);
    let state = state(cfg)?;
    let listener = tokio::net::TcpListener::bind(&addr).await.with_context(|| format!("binding {addr}"))?;
    eprintln!("curio {} on http://{addr}", env!("CARGO_PKG_VERSION"));
    eprintln!("  config  {}", state.cfg.source.display());
    for s in &state.cfg.stores {
        eprintln!("  store   {:<14} {}{}", s.name, s.url, if s.writable() { "  (writable)" } else { "" });
    }
    if state.cfg.stores.is_empty() {
        eprintln!("  no stores configured — add a [[store]] to the config");
    }
    eprintln!("  update mode lapses after {} idle minute(s)", state.cfg.idle_minutes);
    axum::serve(listener, app(state)).await?;
    Ok(())
}

/// Ask every store for its types, and say plainly which ones answer. The first thing to run after
/// editing the config, because a wrong token or a store that is not running otherwise shows up as an
/// empty console.
async fn check_stores(cfg: &Config) -> Result<()> {
    let dd = dd::Dd::new();
    let mut failed = 0;
    for s in &cfg.stores {
        let rw = if s.writable() { "read-write" } else { "read-only" };
        match dd.send(s, dd::Access::Read, axum::http::Method::GET, &["types"], None, None).await {
            Ok(res) if res.status().is_success() => {
                let types: serde_json::Value = res.json().await.unwrap_or_default();
                let list = types.as_array().map(|ts| ts.iter()
                    .map(|t| format!("{} {}", t["name"].as_str().unwrap_or("?"), t["count"]))
                    .collect::<Vec<_>>().join(", ")).unwrap_or_default();
                println!("  ok    {:<14} {rw:<10} {}", s.name, if list.is_empty() { "no types".into() } else { list });
            }
            Ok(res) => {
                failed += 1;
                println!("  FAIL  {:<14} {} answered {}", s.name, s.url, res.status());
            }
            Err(_) => {
                failed += 1;
                println!("  FAIL  {:<14} {} did not answer", s.name, s.url);
            }
        }
    }
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

const SAMPLE: &str = "\
A config lists the DataDungeon stores to show. Make one like this, then chmod 600 it:

    # ~/.config/curio/curio.toml
    [[store]]
    name = \"pirateship\"
    url = \"http://127.0.0.1:7077\"      # dd-serve, or an app that mounts DataDungeon::API
    # read_token = \"…\"                 # if the store asks for one
    # write_token = \"…\"                # only for a store you mean to curate";

fn print_help() {
    println!("curio {} — a web UI for DataDungeon stores\n", env!("CARGO_PKG_VERSION"));
    println!("    curio                    serve the console (http://127.0.0.1:19463)");
    println!("    curio --stores           check that every configured store answers");
    println!("    curio --config PATH      use this config instead of the default\n");
    println!("    CURIO_CONFIG             the config (default ~/.config/curio/curio.toml)");
    println!("    CURIO_PORT, CURIO_BIND   override the config's port and bind address");
    println!("    CURIO_CACHE              thumbnails (default ~/.cache/curio/thumbs)");
    println!("    CURIO_CACHE_MAX_MB       thumbnail cache ceiling (default 1024)\n");
    println!("{SAMPLE}");
}

#[cfg(test)]
mod tests;
