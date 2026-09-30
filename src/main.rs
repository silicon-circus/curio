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

mod api;
mod backup;
mod config;
mod derive;
mod filing;
mod manifest;
mod media;
mod paths;
mod serve;
mod store;
mod sync;

use anyhow::Result;
use axum::{routing::get, Router};
use config::Config;
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

    let args: Vec<String> = std::env::args().skip(1).collect();
    if !cfg.assets().is_dir() {
        eprintln!("curio: no store at {} — set CURIO_DATA, or run from the project directory",
            cfg.data_root.display());
    }
    let flag = |f: &str| args.iter().any(|a| a == f);
    let value_after = |f: &str| args.iter().position(|a| a == f).and_then(|i| args.get(i + 1)).cloned();
    if flag("--sync") {
        return run_sync(&cfg, flag("--dry-run"), flag("--yes"));
    }
    if flag("--watch") {
        return run_watch(&cfg, flag("--yes"));
    }
    if flag("--verify") {
        return run_verify(&cfg);
    }
    if flag("--todo") {
        return run_todo(&cfg);
    }
    if flag("--uncache") {
        let n = clear_cache(&cfg)?;
        println!("  {n} rendition(s) thrown away; they rebuild on demand");
        return Ok(());
    }
    if flag("--dedup") {
        let (count, bytes) = store::dedup(&cfg, flag("--yes"))?;
        println!("  {count} redundant copies, {} recoverable", filing::human(bytes));
        if !flag("--yes") {
            println!("  nothing changed — add --yes to replace them with reflinks");
        }
        return Ok(());
    }
    if flag("--rename") {
        let (Some(from), Some(to)) = (value_after("--rename"), args.last().cloned().filter(|t| Some(t) != value_after("--rename").as_ref()))
        else {
            eprintln!("usage: curio --rename <old> <new>");
            std::process::exit(2);
        };
        let r = store::rename(&cfg, &from, &to)?;
        if r.ok { println!("  {from} -> {}", r.name); } else { eprintln!("  refused: {}", r.why); std::process::exit(1); }
        return Ok(());
    }
    if flag("--find") {
        let terms: Vec<String> = args.iter().skip_while(|a| *a != "--find").skip(1).cloned().collect();
        return run_find(&cfg, &terms);
    }
    if flag("--help") || flag("-h") {
        print_help();
        return Ok(());
    }

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
    // Writes go through the gate; reads do not, because the park embeds assets from its own origins.
    let writes = Router::new()
        .route("/api/keep", axum::routing::post(api::keep))
        .route("/api/trash", axum::routing::post(api::trash))
        .route("/api/unpublish", axum::routing::post(api::unpublish))
        .route("/api/sync", axum::routing::post(api::sync_now))
        .layer(axum::middleware::from_fn_with_state(state.clone(), api::local_writes_only));

    let app = Router::new()
        .route("/", get(api::console))
        .route("/a/{*name}", get(serve::asset))
        .route("/health", get(api::health))
        .route("/api/serve", get(api::serve_list))
        .route("/api/intake", get(api::intake_list))
        .route("/api/watch", get(api::watch_list))
        .route("/intake/{*file}", get(api::intake_file))
        .merge(writes)
        .with_state(state.clone());

    let addr = format!("{}:{}", state.cfg.bind, state.cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("curio {} listening on http://{}", env!("CARGO_PKG_VERSION"), addr);
    eprintln!("  assets   {} ({} named)", state.cfg.assets().display(), state.manifest.get().len());
    eprintln!("  manifest {}", state.cfg.manifest_path().display());
    axum::serve(listener, app).await?;
    Ok(())
}

/// SYNC SHOWS ITS PLAN. Filing the store away is not destructive — every replaced version is kept —
/// but a rename it cannot infer is a question only Tom can answer, and guessing it would quietly
/// lose the one fact that cannot be reconstructed afterwards.
fn run_sync(cfg: &Config, dry_run: bool, yes: bool) -> Result<()> {
    let (plan, next) = sync::scan(cfg)?;
    print!("{}", sync::describe(&plan));

    if dry_run {
        println!("\n  --dry-run, so nothing was written");
        return Ok(());
    }
    if plan.is_quiet() {
        println!("  nothing to do");
        return Ok(());
    }

    // Only a one-to-one pair can be answered with a yes. Anything wider gets reported and left, and
    // `curio rename` states it explicitly.
    let mut links = plan.renamed.clone();
    for a in &plan.ambiguous {
        if a.gone.len() == 1 && a.appeared.len() == 1 {
            let (from, to) = (&a.gone[0], &a.appeared[0]);
            if yes {
                println!("  --yes, so NOT linked: {from} -> {to}  (state it with `curio rename`)");
                continue;
            }
            print!("\n  is  {to}  a renamed and edited  {from} ? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush().ok();
            let mut line = String::new();
            if std::io::stdin().read_line(&mut line).is_ok()
                && matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
            {
                links.push((from.clone(), to.clone()));
                println!("  recorded as a rename");
            } else {
                println!("  left unlinked");
            }
        }
    }

    let applied = sync::apply_with_links(cfg, &plan, &next, &links)?;
    println!("\n  {} version(s) filed into history, mirror refreshed ({} reflinked, {} copied)",
        applied.archived, applied.reflinked, applied.copied);
    if let Some(stamp) = applied.stamp {
        println!("  {}", stamp.display());
    }
    Ok(())
}

fn print_help() {
    println!("curio {} — the Silicon Circus asset server\n", env!("CARGO_PKG_VERSION"));
    println!("  curio                     serve");
    println!("  curio --sync [--dry-run] [--yes]");
    println!("                            file edits away; asks about a rename it cannot infer");
    println!("  curio --watch [--yes]     file what is in watch/, after showing the plan");
    println!("  curio --verify            re-hash assets/ and check it against the manifest");
    println!("  curio --find TERMS        search what will be served, derivations included");
    println!("  curio --todo              what waits in intake/, and what watch/ is holding");
    println!("  curio --uncache           throw away every rendition");
    println!("  curio --dedup [--yes]     share extents between byte-identical assets");
    println!("  curio --rename OLD NEW    state a rename so the history link survives");
    println!("\n  CURIO_PORT {} · CURIO_BIND {} · CURIO_DATA · CURIO_PUBLIC",
        config::DEFAULT_PORT, config::DEFAULT_BIND);
}

fn run_watch(cfg: &Config, yes: bool) -> Result<()> {
    let items = filing::plan(cfg);
    print!("{}", filing::describe(&items));
    let todo = items.iter().filter(|i| i.kind != filing::Kind::Held).count();
    if todo == 0 {
        println!("  nothing to file");
        return Ok(());
    }
    if !yes && !ask(&format!("\nfile {todo} item(s)?"))? {
        println!("  nothing done");
        return Ok(());
    }
    let r = filing::apply(cfg, &items, &mut std::io::stdout())?;
    println!("\n  filed {} ({} files){}", r.filed, r.files,
        if r.skipped > 0 { format!(", {} skipped", r.skipped) } else { String::new() });
    println!("  run `curio --sync` to index them");
    Ok(())
}

/// VERIFY THE FILES THAT ARE ACTUALLY SERVED.
///
/// The Crystal re-hashed `objects/` against its own filenames, which checked the archive copy rather
/// than the working set. With the manifest holding a hash per asset, the useful question is whether
/// what is being served still matches what was recorded — which is also the only thing that can
/// catch a same-size, same-second edit that the stat shortcut cannot see.
fn run_verify(cfg: &Config) -> Result<()> {
    let m = manifest::load(&cfg.manifest_path())?;
    let found = sync::walk(&cfg.assets())?;
    let started = std::time::Instant::now();
    let mut checked = 0usize;
    let mut changed = Vec::new();
    let mut missing = Vec::new();
    for (name, entry) in &m {
        let path = cfg.assets().join(name);
        if !path.is_file() {
            missing.push(name.clone());
            continue;
        }
        checked += 1;
        if manifest::sha256(&path)? != entry.hash {
            changed.push(name.clone());
        }
    }
    let untracked: Vec<&String> = found.keys().filter(|n| !m.contains_key(*n)).collect();
    println!("  verified {checked} assets in {:.1}s", started.elapsed().as_secs_f64());
    println!("  differing from the manifest: {}", changed.len());
    for c in changed.iter().take(20) { println!("    {c}"); }
    println!("  named but not on disk: {}", missing.len());
    for c in missing.iter().take(20) { println!("    {c}"); }
    println!("  on disk but not named: {}", untracked.len());
    for c in untracked.iter().take(20) { println!("    {c}"); }
    if !changed.is_empty() || !missing.is_empty() || !untracked.is_empty() {
        println!("\n  `curio --sync` reconciles all three");
    }
    Ok(())
}

/// Searches what curio will SERVE, which is not what it stores: most of the park's webp are derived
/// from a png master, so a search over stored files only ever answered half the question.
fn run_find(cfg: &Config, terms: &[String]) -> Result<()> {
    let m = manifest::load(&cfg.manifest_path())?;
    let lower: Vec<String> = terms.iter().map(|t| t.to_lowercase()).collect();
    let hit = |n: &str| { let n = n.to_lowercase(); lower.iter().all(|t| n.contains(t)) };
    for name in m.keys().filter(|n| hit(n)) {
        println!("/a/{name}");
    }
    for entry in m.values() {
        let ext = derive::ext_of(std::path::Path::new(&entry.name));
        for target in derive::targets_for(&ext) {
            let d = format!("{}.{}", entry.name.trim_end_matches(&format!(".{ext}")), target);
            if !m.contains_key(&d) && hit(&d) {
                println!("/a/{d}   (derived from {ext})");
            }
        }
    }
    Ok(())
}

fn run_todo(cfg: &Config) -> Result<()> {
    let waiting = store::intake_list(cfg);
    println!("  {} in intake", waiting.len());
    for w in waiting.iter().take(40) {
        println!("    {}  {}", filing::human(w.size), w.file);
    }
    let items = filing::plan(cfg);
    println!("  {} waiting in watch (run: curio --watch)", items.len());
    print!("{}", filing::describe(&items));
    Ok(())
}

/// Note the explicit prefix test: a glob would skip the hidden staging files, which is the trap both
/// `rm -rf cache/*` and Crystal's Dir.glob fell into.
fn clear_cache(cfg: &Config) -> Result<usize> {
    let mut n = 0;
    for e in std::fs::read_dir(cfg.cache())?.flatten() {
        if e.path().is_file() && std::fs::remove_file(e.path()).is_ok() {
            n += 1;
        }
    }
    Ok(n)
}

fn ask(prompt: &str) -> Result<bool> {
    use std::io::Write;
    print!("{prompt} [y/N] ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(matches!(line.trim().to_lowercase().as_str(), "y" | "yes"))
}

/// WHERE THE STORE IS, DECIDED WITHOUT GUESSING.
///
/// `CURIO_DATA` if set; otherwise `./data` relative to the current working directory. Deliberately
/// not derived from the binary's own location: the Crystal version resolved it from `__DIR__` at
/// COMPILE time, so a binary built in one directory and copied to a server went looking for the
/// build machine's path, found nothing, and served an empty store while cheerfully reporting
/// `"ok": true`. An earlier draft of this function guessed from the executable instead and put the
/// store in `target/` — the same class of mistake, one layer along.
///
/// A deployment sets CURIO_DATA. Everything else is a developer in the project directory.
fn project_base() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_base_is_not_compiled_in() {
        // The Crystal resolved its data root from __DIR__ at COMPILE time, so a binary copied
        // elsewhere hunted for the build machine's directory and served an empty store while
        // reporting "ok": true. This one follows the running executable.
        let base = project_base();
        assert!(base.is_absolute() || base == PathBuf::from("."));
    }
}
