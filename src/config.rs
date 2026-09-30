//! Where everything lives, and the only place that decides it.
//!
//! A STORE IS A PROJECT, LIKE A GIT REPOSITORY — and its directories are split by how much you
//! would mind losing them, not by what is tidiest to put together.
//!
//! You point curio at a project root and that is where your files live. One global store for
//! everything works, but per-project isolates one asset set from another, makes a project portable
//! and backed up as a unit, and stops mattering less as more consumers than boardwalk appear.
//!
//!   VALUABLE — you touch these, and they live in the project root, visible and draggable:
//!     assets/ intake/ trash/ watch/ and manifest.tsv
//!   REGENERABLE — out of the working tree entirely, because it rebuilds:
//!     the cache, under XDG (~/.cache/curio/<project>)
//!   SAFETY COPY — never inside the thing it is a copy of:
//!     the backup, under XDG (~/.local/share/curio/<project>/backup)
//!
//! Both of the last two are independently configurable, which is what lets one binary be a
//! repo-shaped tool on a workstation and a minimal headless server in production. On a park droplet:
//! CURIO_DATA=/var/lib/curio holds assets/ and nothing else, CURIO_CACHE=/var/cache/curio because
//! FHS says /var/cache is for what can be regenerated, and CURIO_BACKUP=none because the
//! authoritative store and its history live on the workstation — server recovery is another rsync.
//!
//! SIX DIRECTORIES, AND EACH ONE ANSWERS A DIFFERENT QUESTION.
//!
//!   watch/    "file this for me" — drop something in with the name you want and it is filed and
//!             published without being asked twice. `curio --watch` prints what it intends to do
//!             and then asks, because a folder you are still arranging looks exactly like a folder
//!             you have finished arranging, and only you know which.
//!   intake/   "is this a keeper?" — the only chore. Things land here; you look at them and say
//!             keep (with a name) or bin. Anything ambiguous comes here rather than being guessed
//!             at, which is what keeps the automatic path safe.
//!   assets/   "what can be asked for by name?" — the working set, and the filename IS the URL.
//!             Open one in GIMP and touch it up; that is the point.
//!   backup/   "what was this file yesterday?" — current/ mirrors assets/, history/<stamp>/ holds
//!             the versions that were replaced. Read-only, reflinked, and the system's doing, not
//!             yours: see the note on trash/ below.
//!   cache/    "have I already answered this exact request?" — renditions. Disposable.
//!   trash/    "what did I throw away?" — YOURS. "I probably don't need this any more, but I am not
//!             deleting it yet, just in case." Deliberate, and never pruned on a schedule, which is
//!             precisely why backup/history/ CAN be: nothing in there was put there on purpose.
//!
//! There is no objects/. It saved 35 files of deduplication out of 1051, held 749 objects that no
//! name pointed at, and kept no record of what any of them used to be called — so "version history,
//! and it's free" was retention without recall. backup/ keeps the same bytes under the name they had
//! and the date they were replaced, which is the part that was missing.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Default port. 19463 is curio's allocated slot; 26037 was a number somebody picked.
pub const DEFAULT_PORT: u16 = 19463;

/// LOOPBACK BY DEFAULT, because there is no password on any of this.
///
/// The routes that mutate the store do not ask who is calling. That is right for a tool serving one
/// machine's browser and wrong the moment the port is reachable from anywhere else, so the open
/// state has to be typed. `CURIO_BIND=0.0.0.0` belongs only behind something that terminates the
/// public side and forwards nothing but reads.
pub const DEFAULT_BIND: &str = "127.0.0.1";

#[derive(Debug, Clone)]
pub struct Config {
    pub data_root: PathBuf,
    pub public: PathBuf,
    pub port: u16,
    pub bind: String,
    /// `None` means the XDG default for this project. Held as a field rather than read from the
    /// environment on each call, so a store can be described without touching process-wide state —
    /// which is also what makes the tests hermetic instead of writing into the real ~/.cache.
    cache_at: Option<PathBuf>,
    backup_at: Option<PathBuf>,
    backup_off: bool,
}

impl Config {
    /// Read from the environment, falling back to paths relative to `base` (the directory holding
    /// the project, not the binary — a compiled-in path is how the Crystal ended up unable to be
    /// moved without being rebuilt).
    pub fn from_env(base: &Path) -> Self {
        let raw_backup = std::env::var("CURIO_BACKUP").unwrap_or_default();
        let backup_off = matches!(raw_backup.as_str(), "none" | "off");
        Config {
            data_root: env_path("CURIO_DATA").unwrap_or_else(|| base.join("data")),
            public: env_path("CURIO_PUBLIC").unwrap_or_else(|| base.join("public")),
            port: std::env::var("CURIO_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_PORT),
            bind: std::env::var("CURIO_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string()),
            cache_at: env_path("CURIO_CACHE"),
            backup_at: if backup_off { None } else { env_path("CURIO_BACKUP") },
            backup_off,
        }
    }

    /// A project at `data_root`, with the cache and backup in their XDG places.
    pub fn for_root(data_root: PathBuf) -> Self {
        Config {
            public: data_root.join("public"),
            data_root,
            port: DEFAULT_PORT,
            bind: DEFAULT_BIND.to_string(),
            cache_at: None,
            backup_at: None,
            backup_off: false,
        }
    }

    /// A store that keeps everything, cache and backup included, inside one directory.
    ///
    /// Not the default — a copy inside the thing it copies is the arrangement this layout exists to
    /// avoid — but it is the right shape for a throwaway store, and it is what the tests use so that
    /// running them cannot litter a real home directory.
    pub fn isolated(data_root: PathBuf) -> Self {
        Config {
            cache_at: Some(data_root.join("cache")),
            backup_at: Some(data_root.join("backup")),
            ..Self::for_root(data_root)
        }
    }

    pub fn assets(&self) -> PathBuf { self.data_root.join("assets") }
    pub fn watch(&self) -> PathBuf { self.data_root.join("watch") }
    pub fn intake(&self) -> PathBuf { self.data_root.join("intake") }
    pub fn trash(&self) -> PathBuf { self.data_root.join("trash") }
    /// A stable, legible key for this store: the project directory's own name, plus enough of a
    /// digest of its absolute path that two projects called `art` cannot collide.
    pub fn project_key(&self) -> String {
        use sha2::{Digest, Sha256};
        let canonical = self.data_root.canonicalize().unwrap_or_else(|_| self.data_root.clone());
        let digest: String = Sha256::digest(canonical.to_string_lossy().as_bytes())
            .iter().take(4).map(|b| format!("{b:02x}")).collect();
        let name = canonical.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "store".into());
        format!("{name}-{digest}")
    }

    /// Renditions. Regenerable, so it belongs out of the project entirely.
    pub fn cache(&self) -> PathBuf {
        self.cache_at.clone().unwrap_or_else(||
            xdg_dir("XDG_CACHE_HOME", ".cache").join("curio").join(self.project_key()))
    }

    /// The safety copy, which is never inside the thing it is a copy of.
    ///
    /// XDG *data*, not XDG cache: this is the one directory here whose loss cannot be undone by
    /// regenerating it. `CURIO_BACKUP=none` turns it off, which is what a server wants — its masters
    /// arrived from somewhere that already has the history.
    pub fn backup(&self) -> PathBuf {
        self.backup_at.clone().unwrap_or_else(||
            xdg_dir("XDG_DATA_HOME", ".local/share").join("curio").join(self.project_key()).join("backup"))
    }

    pub fn backup_enabled(&self) -> bool {
        !self.backup_off
    }
    /// A read-only reflink mirror of assets/ as of the last sync. This is what makes undo possible:
    /// by the time sync notices an edit the old bytes are already overwritten, and they survive
    /// only because this captured them at the PREVIOUS sync.
    pub fn backup_current(&self) -> PathBuf { self.backup().join("current") }
    pub fn backup_history(&self) -> PathBuf { self.backup().join("history") }
    /// What curio knows about assets/ without re-hashing the whole store on every boot.
    pub fn manifest_path(&self) -> PathBuf { self.data_root.join("manifest.tsv") }

    /// Everything this store uses, wherever it has been pointed.
    pub fn dirs(&self) -> Vec<PathBuf> {
        let mut v = vec![self.data_root.clone(), self.assets(), self.cache()];
        if self.backup_enabled() {
            v.push(self.backup());
            v.push(self.backup_current());
            v.push(self.backup_history());
        }
        v
    }

    /// The three a person drags files into. Made when a store is first created and never
    /// re-created, so a headless server that only ever receives `assets/` does not accumulate empty
    /// directories for chores it will never do.
    fn authoring_dirs(&self) -> Vec<PathBuf> {
        vec![self.intake(), self.watch(), self.trash()]
    }

    /// Make every directory, and collect abandoned render scratch on the way up.
    pub fn ensure_dirs(&self) -> Result<()> {
        let fresh = !self.assets().is_dir();
        for d in self.dirs() {
            std::fs::create_dir_all(&d)?;
        }
        if fresh {
            for d in self.authoring_dirs() {
                std::fs::create_dir_all(&d)?;
            }
        }
        self.sweep_cache_scratch()
    }

    /// A render killed partway — Ctrl-C is enough — leaves its staging file behind, and nothing ever
    /// collected them. Note the explicit prefix test rather than a glob: shell globs and Crystal's
    /// Dir.glob both skip hidden entries by default, and every one of these is hidden by design.
    /// That trap has now been walked into twice, in two languages.
    pub fn sweep_cache_scratch(&self) -> Result<()> {
        let cache = self.cache();
        if !cache.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(&cache)? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().starts_with(SCRATCH_PREFIX) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
        Ok(())
    }
}

/// Staging files are hidden so that nothing serving or listing the cache ever sees a partial write.
pub const SCRATCH_PREFIX: &str = ".tmp-";

/// `$XDG_*_HOME` when set and absolute, else `$HOME/<fallback>`, else a relative path so that a
/// process with no HOME still runs rather than panicking.
fn xdg_dir(var: &str, fallback: &str) -> PathBuf {
    if let Some(p) = env_path(var).filter(|p| p.is_absolute()) {
        return p;
    }
    match std::env::var("HOME").ok().filter(|h| !h.is_empty()) {
        Some(home) => PathBuf::from(home).join(fallback),
        None => PathBuf::from(fallback),
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var(key).ok().filter(|s| !s.is_empty()).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("curio-cfg-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn ensure_dirs_creates_every_directory() {
        let root = tmp("dirs");
        let cfg = Config::isolated(root.clone());
        cfg.ensure_dirs().unwrap();
        for d in cfg.dirs() {
            assert!(d.is_dir(), "{} was not created", d.display());
        }
        // the ones that did not exist before this port
        assert!(cfg.backup_current().is_dir());
        assert!(cfg.backup_history().is_dir());
        // and the one that must NOT come back
        assert!(!root.join("objects").exists(), "objects/ is gone and should stay gone");
        assert!(!root.join("names").exists(), "names/ is called assets/ now");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn ensure_dirs_sweeps_scratch_but_keeps_renditions() {
        let root = tmp("sweep");
        let cfg = Config::isolated(root.clone());
        cfg.ensure_dirs().unwrap();
        std::fs::write(cfg.cache().join(".tmp-123-x.webp"), b"partial").unwrap();
        std::fs::write(cfg.cache().join(".tmp-9.webp"), b"partial").unwrap();
        std::fs::write(cfg.cache().join("real.s1.m1.q82.webp"), b"whole").unwrap();
        cfg.ensure_dirs().unwrap();
        let left: Vec<String> = std::fs::read_dir(cfg.cache()).unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(left, vec!["real.s1.m1.q82.webp".to_string()]);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn defaults_are_loopback_and_the_allocated_port() {
        let cfg = Config::isolated(PathBuf::from("/nowhere"));
        assert_eq!(cfg.port, 19463);
        assert_eq!(cfg.bind, "127.0.0.1");
    }
}


#[cfg(test)]
mod layout_tests {
    use super::*;

    fn isolated(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("curio-layout-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn the_project_key_is_legible_and_path_specific() {
        let a = isolated("keya");
        let b = isolated("keyb");
        let ka = Config::for_root(a.clone()).project_key();
        let kb = Config::for_root(b.clone()).project_key();
        assert_ne!(ka, kb, "two projects must not share a key");
        assert!(ka.starts_with(a.file_name().unwrap().to_string_lossy().as_ref()),
            "the key should be readable: {ka}");
        // and stable across calls
        assert_eq!(ka, Config::for_root(a.clone()).project_key());
        std::fs::remove_dir_all(&a).unwrap();
        std::fs::remove_dir_all(&b).unwrap();
    }

    #[test]
    fn cache_and_backup_are_not_inside_the_project() {
        // The point of the split: a copy is never kept inside the thing it is a copy of, and
        // something regenerable never sits in the working tree.
        let root = isolated("outside");
        let cfg = Config::for_root(root.clone());
        assert!(!cfg.cache().starts_with(&root), "cache is inside the project: {:?}", cfg.cache());
        assert!(!cfg.backup().starts_with(&root), "backup is inside the project: {:?}", cfg.backup());
        assert!(cfg.assets().starts_with(&root), "assets must be IN the project");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_fresh_store_gets_the_authoring_directories() {
        let root = isolated("fresh");
        let cfg = Config::isolated(root.clone());
        cfg.ensure_dirs().unwrap();
        for d in [cfg.assets(), cfg.intake(), cfg.watch(), cfg.trash()] {
            assert!(d.is_dir(), "{} missing", d.display());
        }
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_server_store_does_not_regrow_chore_directories() {
        // A droplet rsyncs assets/ up and never keeps or bins anything. Empty intake/, watch/ and
        // trash/ would just be clutter suggesting work that happens elsewhere.
        let root = isolated("server");
        std::fs::create_dir_all(root.join("assets")).unwrap();
        let cfg = Config::isolated(root.clone());
        cfg.ensure_dirs().unwrap();
        assert!(cfg.assets().is_dir());
        for d in [cfg.intake(), cfg.watch(), cfg.trash()] {
            assert!(!d.exists(), "{} should not have been created", d.display());
        }
        std::fs::remove_dir_all(&root).unwrap();
    }
}
