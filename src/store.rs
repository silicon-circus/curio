//! THE ONE CHORE, BOTH ANSWERS — and taking a name away again.
//!
//! Without `objects/` these are all just careful moves. Keeping an intake file is a rename into
//! `assets/`; binning it is a rename into `trash/`; unpublishing is the same rename from the other
//! direction. Nothing is hashed here, because `sync` hashes and it is the only thing that needs to.
//!
//! Every path that came out of a request goes through `paths::within` first. That is not belt and
//! braces: before it existed, `{"file":"../../.ssh/id_ed25519"}` published any file this process
//! could read under a name the caller chose and then deleted the original.

use crate::config::Config;
use crate::paths;
use anyhow::Result;
use std::path::Path;

/// A name is a URL, so it has to survive being one.
///
/// Note the order: strip what a URL cannot carry FIRST, then test the result. The Crystal checked
/// for ".." before stripping, so `a.%.b.png` became `a..b.png` after the check had already passed —
/// harmless there by luck, and the wrong order regardless.
pub fn safe_name(name: &str) -> String {
    let collapsed: String = name.trim().split_whitespace().collect::<Vec<_>>().join("-");
    let kept: String = collapsed.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_' || *c == '-' || *c == '/')
        .collect();
    if kept.is_empty() || kept.split('/').any(|part| part == ".." || part == ".") {
        return String::new();
    }
    kept.trim_start_matches('/').to_string()
}

pub struct Kept {
    pub ok: bool,
    pub why: &'static str,
    pub name: String,
}

/// Answered yes: move it out of intake and into the working set under the name you chose.
pub fn keep(cfg: &Config, file: &str, name: &str) -> Result<Kept> {
    let name = safe_name(name);
    if name.is_empty() {
        return Ok(Kept { ok: false, why: "bad name", name });
    }
    let Some(src) = paths::within(&cfg.intake(), file) else {
        return Ok(Kept { ok: false, why: "no such file in intake", name });
    };
    if !src.is_file() {
        return Ok(Kept { ok: false, why: "no such file in intake", name });
    }
    let Some(dest) = paths::within(&cfg.assets(), &name) else {
        return Ok(Kept { ok: false, why: "bad name", name });
    };
    if dest.exists() {
        return Ok(Kept { ok: false, why: "that name is already being served", name });
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    move_file(&src, &dest)?;
    Ok(Kept { ok: true, why: "", name })
}

/// Answered no: into `trash/`, not into oblivion. A second file of the same name gets a suffix,
/// because silently overwriting the first would be exactly the loss `trash/` exists to prevent.
pub fn discard(cfg: &Config, from: &Path, rel: &str) -> Result<bool> {
    let Some(src) = paths::within(from, rel) else {
        return Ok(false);
    };
    if !src.is_file() {
        return Ok(false);
    }
    let base = Path::new(rel).file_name().map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| rel.to_string());
    let mut dest = cfg.trash().join(&base);
    let stem = Path::new(&base).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = Path::new(&base).extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let mut n = 1;
    while dest.exists() {
        dest = cfg.trash().join(format!("{stem}~{n}{ext}"));
        n += 1;
    }
    std::fs::create_dir_all(cfg.trash())?;
    move_file(&src, &dest)?;
    Ok(true)
}

/// Taking a name away. The bytes are not destroyed: they go to `trash/`, which is yours, and the
/// copy in `backup/current/` is filed into history by the next sync, which is the system's.
pub fn unpublish(cfg: &Config, name: &str) -> Result<bool> {
    let assets = cfg.assets();
    discard(cfg, &assets, name)
}

/// Rename within the working set, stating a rename that sync could not have inferred.
///
/// Doing this instead of `mv` is what lets an edit and a rename happen together without the history
/// link being lost — which is the one fact about a change that cannot be recovered afterwards.
pub fn rename(cfg: &Config, from: &str, to: &str) -> Result<Kept> {
    let to_name = safe_name(to);
    if to_name.is_empty() {
        return Ok(Kept { ok: false, why: "bad name", name: to_name });
    }
    let (Some(src), Some(dest)) = (paths::within(&cfg.assets(), from), paths::within(&cfg.assets(), &to_name))
    else {
        return Ok(Kept { ok: false, why: "bad name", name: to_name });
    };
    if !src.is_file() {
        return Ok(Kept { ok: false, why: "no such asset", name: to_name });
    }
    if dest.exists() {
        return Ok(Kept { ok: false, why: "that name is already being served", name: to_name });
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&src, &dest)?;
    Ok(Kept { ok: true, why: "", name: to_name })
}

/// `rename` across filesystems is `EXDEV`, and `data/` being one mount is a rule this store already
/// keeps — but a store assembled by hand might not, so fall back rather than fail.
fn move_file(src: &Path, dest: &Path) -> Result<()> {
    match std::fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(_) => {
            std::fs::copy(src, dest)?;
            std::fs::remove_file(src)?;
            Ok(())
        }
    }
}

pub struct Waiting {
    pub file: String,
    pub size: u64,
    pub mtime: i64,
}

/// What is sitting in `intake/`, newest first — the queue for the only chore.
pub fn intake_list(cfg: &Config) -> Vec<Waiting> {
    let root = cfg.intake();
    let mut out: Vec<Waiting> = crate::sync::walk(&root)
        .unwrap_or_default()
        .into_iter()
        .map(|(file, (size, mtime, _))| Waiting { file, size, mtime })
        .collect();
    out.sort_by_key(|w| -w.mtime);
    out
}

pub fn count_files(dir: &Path) -> usize {
    crate::sync::walk(dir).map(|m| m.len()).unwrap_or(0)
}

/// Replace byte-identical assets with reflinks of one another.
///
/// Aliasing one picture under several names is legitimate — the park does it for a plank texture
/// shared between two venues, and for three favicons. REFLINK, NEVER HARDLINK: a hardlink is one
/// inode under two names, so editing either rewrites both, which is the corruption this store is
/// built to avoid. A reflink shares extents and diverges the moment something writes.
pub fn dedup(cfg: &Config, apply: bool) -> Result<(usize, u64)> {
    use std::collections::BTreeMap;
    let manifest = crate::manifest::load(&cfg.manifest_path())?;
    let mut by_hash: BTreeMap<&str, Vec<&String>> = BTreeMap::new();
    for e in manifest.values() {
        by_hash.entry(e.hash.as_str()).or_default().push(&e.name);
    }
    let mut count = 0usize;
    let mut bytes = 0u64;
    for (_, names) in by_hash.iter().filter(|(_, v)| v.len() > 1) {
        let keeper = cfg.assets().join(names[0]);
        for other in &names[1..] {
            let path = cfg.assets().join(other);
            let (Ok((s1, _, i1)), Ok((s2, _, i2))) =
                (crate::manifest::stat_of(&keeper), crate::manifest::stat_of(&path)) else { continue };
            if s1 != s2 || i1 == i2 {
                continue;
            }
            count += 1;
            bytes += s2;
            if apply {
                // Stage and rename, so a failure cannot leave the asset missing.
                let stage = path.with_extension("curio-dedup");
                if crate::backup::set_aside(&keeper, &stage).is_ok() {
                    crate::backup::make_writable(&stage)?;
                    std::fs::rename(&stage, &path)?;
                } else {
                    let _ = std::fs::remove_file(&stage);
                }
            }
        }
    }
    Ok((count, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(name: &str) -> (std::path::PathBuf, Config) {
        let root = std::env::temp_dir().join(format!("curio-store-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let cfg = Config::isolated(root.clone());
        cfg.ensure_dirs().unwrap();
        (root, cfg)
    }

    #[test]
    fn safe_name_strips_first_then_tests() {
        assert_eq!(safe_name("boardwalk.deck.webp"), "boardwalk.deck.webp");
        assert_eq!(safe_name("  two words.png "), "two-words.png");
        // the Crystal tested for ".." BEFORE stripping, so this slipped through as "a..b.png"
        assert_eq!(safe_name("a.%.b.png"), "a..b.png", "stripping may create dots, which is fine");
        assert_eq!(safe_name(".%."), "", "...but a bare traversal component is not a name");
        assert_eq!(safe_name(".."), "");
        assert_eq!(safe_name("../../etc/passwd"), "", "a traversal component anywhere is refused");
        assert_eq!(safe_name("/leading"), "leading");
        assert_eq!(safe_name("kit/Textures/colormap.png"), "kit/Textures/colormap.png");
    }

    #[test]
    fn keep_moves_an_intake_file_under_the_chosen_name() {
        let (root, cfg) = store("keep");
        std::fs::write(cfg.intake().join("candidate.png"), b"pixels").unwrap();
        let r = keep(&cfg, "candidate.png", "boardwalk.thing.png").unwrap();
        assert!(r.ok, "{}", r.why);
        assert_eq!(std::fs::read(cfg.assets().join("boardwalk.thing.png")).unwrap(), b"pixels");
        assert!(!cfg.intake().join("candidate.png").exists(), "intake copy should be gone");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn keep_refuses_to_reach_outside_intake() {
        let (root, cfg) = store("keepescape");
        std::fs::write(root.join("secret.txt"), b"MY-SSH-KEY").unwrap();
        let r = keep(&cfg, "../secret.txt", "leaked.txt").unwrap();
        assert!(!r.ok);
        assert!(root.join("secret.txt").is_file(), "the file must still be there");
        assert!(!cfg.assets().join("leaked.txt").exists());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn keep_never_overwrites_a_published_name() {
        let (root, cfg) = store("keepclash");
        std::fs::write(cfg.assets().join("taken.png"), b"original").unwrap();
        std::fs::write(cfg.intake().join("new.png"), b"different").unwrap();
        let r = keep(&cfg, "new.png", "taken.png").unwrap();
        assert!(!r.ok);
        assert_eq!(std::fs::read(cfg.assets().join("taken.png")).unwrap(), b"original");
        assert!(cfg.intake().join("new.png").is_file(), "and the candidate is left where it was");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn discard_suffixes_rather_than_overwriting() {
        let (root, cfg) = store("trash");
        for body in [b"first", b"secnd"] {
            std::fs::write(cfg.intake().join("dup.png"), body).unwrap();
            assert!(discard(&cfg, &cfg.intake(), "dup.png").unwrap());
        }
        let mut names: Vec<String> = std::fs::read_dir(cfg.trash()).unwrap().flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        names.sort();
        assert_eq!(names, vec!["dup.png".to_string(), "dup~1.png".to_string()]);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn discard_and_unpublish_refuse_traversal() {
        let (root, cfg) = store("discardescape");
        std::fs::write(root.join("victim.txt"), b"keep me").unwrap();
        assert!(!discard(&cfg, &cfg.intake(), "../victim.txt").unwrap());
        assert!(!unpublish(&cfg, "../victim.txt").unwrap());
        assert!(root.join("victim.txt").is_file());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn unpublish_takes_the_name_and_keeps_the_bytes() {
        let (root, cfg) = store("unpublish");
        std::fs::write(cfg.assets().join("gone.png"), b"still here").unwrap();
        assert!(unpublish(&cfg, "gone.png").unwrap());
        assert!(!cfg.assets().join("gone.png").exists());
        assert_eq!(std::fs::read(cfg.trash().join("gone.png")).unwrap(), b"still here");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rename_states_what_sync_could_not_infer() {
        let (root, cfg) = store("rename");
        std::fs::write(cfg.assets().join("old.png"), b"bytes").unwrap();
        let r = rename(&cfg, "old.png", "new.png").unwrap();
        assert!(r.ok, "{}", r.why);
        assert!(cfg.assets().join("new.png").is_file());
        assert!(!cfg.assets().join("old.png").exists());
        // and it will not clobber
        std::fs::write(cfg.assets().join("other.png"), b"other").unwrap();
        assert!(!rename(&cfg, "other.png", "new.png").unwrap().ok);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn dedup_reports_before_it_acts_and_shares_extents_not_inodes() {
        let (root, cfg) = store("dedup");
        for n in ["a.png", "b.png", "c.png"] {
            std::fs::write(cfg.assets().join(n), b"identical bytes here").unwrap();
        }
        std::fs::write(cfg.assets().join("d.png"), b"different").unwrap();
        let (plan, next) = crate::sync::scan(&cfg).unwrap();
        crate::sync::apply(&cfg, &plan, &next).unwrap();

        let (count, bytes) = dedup(&cfg, false).unwrap();
        assert_eq!(count, 2, "three identical files means two redundant copies");
        assert!(bytes > 0);

        let before: Vec<u64> = ["a.png", "b.png", "c.png"].iter()
            .map(|n| crate::manifest::stat_of(&cfg.assets().join(n)).unwrap().2).collect();
        dedup(&cfg, true).unwrap();
        for n in ["a.png", "b.png", "c.png"] {
            assert_eq!(std::fs::read(cfg.assets().join(n)).unwrap(), b"identical bytes here");
        }
        let after: Vec<u64> = ["a.png", "b.png", "c.png"].iter()
            .map(|n| crate::manifest::stat_of(&cfg.assets().join(n)).unwrap().2).collect();
        // NOT hardlinked: every alias must keep its own inode, or editing one would rewrite the rest
        assert_eq!(after.iter().collect::<std::collections::BTreeSet<_>>().len(), 3,
            "aliases were hardlinked together: {before:?} -> {after:?}");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
