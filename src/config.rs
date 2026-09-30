//! Where everything lives, and the only place that decides it.
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
    /// Where console.html lives. Unused until the console is served from Rust (PORT.md step 7).
    #[allow(dead_code)]
    pub public: PathBuf,
    pub port: u16,
    pub bind: String,
}

impl Config {
    /// Read from the environment, falling back to paths relative to `base` (the directory holding
    /// the project, not the binary — a compiled-in path is how the Crystal ended up unable to be
    /// moved without being rebuilt).
    pub fn from_env(base: &Path) -> Self {
        Config {
            data_root: env_path("CURIO_DATA").unwrap_or_else(|| base.join("data")),
            public: env_path("CURIO_PUBLIC").unwrap_or_else(|| base.join("public")),
            port: std::env::var("CURIO_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_PORT),
            bind: std::env::var("CURIO_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string()),
        }
    }

    pub fn for_root(data_root: PathBuf) -> Self {
        Config { public: data_root.join("public"), data_root, port: DEFAULT_PORT, bind: DEFAULT_BIND.to_string() }
    }

    pub fn assets(&self) -> PathBuf { self.data_root.join("assets") }
    pub fn watch(&self) -> PathBuf { self.data_root.join("watch") }
    pub fn intake(&self) -> PathBuf { self.data_root.join("intake") }
    pub fn trash(&self) -> PathBuf { self.data_root.join("trash") }
    pub fn cache(&self) -> PathBuf { self.data_root.join("cache") }
    pub fn backup(&self) -> PathBuf { self.data_root.join("backup") }
    /// A read-only reflink mirror of assets/ as of the last sync. This is what makes undo possible:
    /// by the time sync notices an edit the old bytes are already overwritten, and they survive
    /// only because this captured them at the PREVIOUS sync.
    pub fn backup_current(&self) -> PathBuf { self.backup().join("current") }
    pub fn backup_history(&self) -> PathBuf { self.backup().join("history") }
    /// What curio knows about assets/ without re-hashing the whole store on every boot.
    pub fn manifest_path(&self) -> PathBuf { self.data_root.join("manifest.tsv") }

    pub fn dirs(&self) -> Vec<PathBuf> {
        vec![
            self.data_root.clone(), self.assets(), self.watch(), self.intake(),
            self.trash(), self.cache(), self.backup(), self.backup_current(), self.backup_history(),
        ]
    }

    /// Make every directory, and collect abandoned render scratch on the way up.
    pub fn ensure_dirs(&self) -> Result<()> {
        for d in self.dirs() {
            std::fs::create_dir_all(&d)?;
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
        let cfg = Config::for_root(root.clone());
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
        let cfg = Config::for_root(root.clone());
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
        let cfg = Config::for_root(PathBuf::from("/nowhere"));
        assert_eq!(cfg.port, 19463);
        assert_eq!(cfg.bind, "127.0.0.1");
    }
}

