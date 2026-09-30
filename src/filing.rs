//! watch/ — DROP IT IN NAMED, AND IT FILES ITSELF AFTER SHOWING YOU THE PLAN.
//!
//! A top-level file is one asset. A top-level directory is a collection and keeps its shape, because
//! a kit whose models reference `Textures/colormap.png` relative to themselves stops working the
//! moment it is flattened.
//!
//! IT PRINTS WHAT IT INTENDS TO DO AND THEN ASKS. A folder you are still arranging looks exactly
//! like a folder you have finished arranging, and only you know which.
//!
//! AND IT NEVER OVERWRITES. A name already being served is left exactly where it is and reported —
//! not moved, not renamed, not merged, and not silently dropped even when the bytes are identical.
//! "I moved it to watch/ and it vanished" is indistinguishable from "it worked", and those are very
//! different things to have happened.
//!
//! Every destination is re-tested at the moment of writing, not only when the plan was printed. The
//! whole reason this is a command rather than a daemon is to put a human pause in the middle, and a
//! guarantee that only holds while nobody is looking is not the guarantee that was made.

use crate::config::Config;
use crate::paths;
use crate::store::safe_name;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    File,
    Collection,
    Held,
}

#[derive(Debug)]
pub struct Item {
    pub kind: Kind,
    pub name: String,
    pub source: PathBuf,
    pub files: usize,
    pub bytes: u64,
    pub why: String,
}

pub fn plan(cfg: &Config) -> Vec<Item> {
    let watch = cfg.watch();
    let mut items = Vec::new();
    let entries = match std::fs::read_dir(&watch) {
        Ok(e) => e,
        Err(_) => return items,
    };
    let mut names: Vec<_> = entries.flatten()
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .collect();
    names.sort_by_key(|e| e.file_name());

    for entry in names {
        let raw = entry.file_name().to_string_lossy().into_owned();
        let source = entry.path();
        let name = safe_name(&raw);
        let is_dir = source.is_dir();
        let (files, bytes) = measure(&source);

        if name.is_empty() {
            items.push(Item { kind: Kind::Held, name: raw, source, files, bytes,
                why: "not a usable name".into() });
            continue;
        }
        let taken = paths::within(&cfg.assets(), &name).map(|p| p.exists()).unwrap_or(true);
        if taken {
            items.push(Item { kind: Kind::Held, name, source, files, bytes,
                why: "already being served".into() });
            continue;
        }
        items.push(Item {
            kind: if is_dir { Kind::Collection } else { Kind::File },
            name, source, files, bytes, why: String::new(),
        });
    }
    items
}

fn measure(path: &std::path::Path) -> (usize, u64) {
    if path.is_file() {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        return (1, size);
    }
    let found = crate::sync::walk(path).unwrap_or_default();
    let bytes = found.values().map(|(s, _, _)| *s).sum();
    (found.len(), bytes)
}

#[derive(Debug, Default)]
pub struct Filed {
    pub filed: usize,
    pub files: usize,
    pub skipped: usize,
}

/// Do exactly what the plan said, and nothing that was not in it — including checking again.
pub fn apply(cfg: &Config, items: &[Item], log: &mut dyn std::io::Write) -> Result<Filed> {
    let mut out = Filed::default();
    for item in items {
        if item.kind == Kind::Held {
            continue;
        }
        let Some(dest) = paths::within(&cfg.assets(), &item.name) else {
            writeln!(log, "  SKIPPED {} — not a usable name", item.name)?;
            out.skipped += 1;
            continue;
        };
        if dest.exists() {
            writeln!(log, "  SKIPPED {} — appeared in assets/ since the plan was made", item.name)?;
            out.skipped += 1;
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match item.kind {
            Kind::Collection => {
                let mut moved = 0usize;
                for (rel, _) in crate::sync::walk(&item.source)? {
                    let target = dest.join(&rel);
                    if target.exists() {
                        continue;
                    }
                    if let Some(parent) = target.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::rename(item.source.join(&rel), &target)?;
                    moved += 1;
                }
                let _ = remove_empty_tree(&item.source);
                out.files += moved;
            }
            Kind::File => {
                std::fs::rename(&item.source, &dest)?;
                out.files += 1;
            }
            Kind::Held => unreachable!(),
        }
        out.filed += 1;
        writeln!(log, "  filed {}{}", item.name,
            if item.kind == Kind::Collection { "/" } else { "" })?;
    }
    Ok(out)
}

fn remove_empty_tree(path: &std::path::Path) -> std::io::Result<()> {
    if path.is_dir() {
        for e in std::fs::read_dir(path)?.flatten() {
            let _ = remove_empty_tree(&e.path());
        }
        std::fs::remove_dir(path)?;
    }
    Ok(())
}

pub fn describe(items: &[Item]) -> String {
    let mut s = String::new();
    for i in items {
        match i.kind {
            Kind::Held => s.push_str(&format!("  HELD        {} — {}\n", i.name, i.why)),
            Kind::Collection => s.push_str(&format!(
                "  COLLECTION  {}\n              -> assets/{}/  {} files, {}\n",
                i.name, i.name, i.files, human(i.bytes))),
            Kind::File => s.push_str(&format!(
                "  FILE        {}\n              -> assets/{}  {}\n", i.name, i.name, human(i.bytes))),
        }
    }
    s
}

pub fn human(b: u64) -> String {
    if b >= 1_000_000 {
        format!("{:.1} MB", b as f64 / 1e6)
    } else {
        format!("{} kB", (b as f64 / 1024.0).round() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(name: &str) -> (PathBuf, Config) {
        let root = std::env::temp_dir().join(format!("curio-filing-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let cfg = Config::for_root(root.clone());
        cfg.ensure_dirs().unwrap();
        (root, cfg)
    }

    #[test]
    fn a_loose_file_is_one_asset() {
        let (root, cfg) = store("file");
        std::fs::write(cfg.watch().join("boardwalk.thing.png"), b"pixels").unwrap();
        let items = plan(&cfg);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, Kind::File);
        let mut log = Vec::new();
        let r = apply(&cfg, &items, &mut log).unwrap();
        assert_eq!((r.filed, r.files, r.skipped), (1, 1, 0));
        assert_eq!(std::fs::read(cfg.assets().join("boardwalk.thing.png")).unwrap(), b"pixels");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_directory_keeps_its_shape() {
        // A kit whose models reference Textures/colormap.png relative to themselves breaks if this
        // is flattened.
        let (root, cfg) = store("collection");
        let kit = cfg.watch().join("pirateship.kit");
        std::fs::create_dir_all(kit.join("Textures")).unwrap();
        std::fs::write(kit.join("barrel.glb"), b"glb").unwrap();
        std::fs::write(kit.join("Textures/colormap.png"), b"atlas").unwrap();
        let items = plan(&cfg);
        assert_eq!(items[0].kind, Kind::Collection);
        assert_eq!(items[0].files, 2);
        let mut log = Vec::new();
        apply(&cfg, &items, &mut log).unwrap();
        assert!(cfg.assets().join("pirateship.kit/Textures/colormap.png").is_file());
        assert!(!kit.exists(), "watch/ should be left clean");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_name_already_served_is_held_not_overwritten() {
        let (root, cfg) = store("held");
        std::fs::write(cfg.assets().join("taken.png"), b"original").unwrap();
        std::fs::write(cfg.watch().join("taken.png"), b"incoming").unwrap();
        let items = plan(&cfg);
        assert_eq!(items[0].kind, Kind::Held);
        let mut log = Vec::new();
        let r = apply(&cfg, &items, &mut log).unwrap();
        assert_eq!(r.filed, 0);
        assert_eq!(std::fs::read(cfg.assets().join("taken.png")).unwrap(), b"original");
        assert!(cfg.watch().join("taken.png").is_file(),
            "the incoming file must still be in watch/, not vanished");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_name_taken_between_plan_and_apply_is_skipped_not_clobbered() {
        // The plan checked and apply did not, once. cp overwrites by default, and the whole reason
        // this is a command is the human pause in the middle.
        let (root, cfg) = store("race");
        std::fs::write(cfg.watch().join("late.png"), b"incoming").unwrap();
        let items = plan(&cfg);
        assert_eq!(items[0].kind, Kind::File, "clear at plan time");
        std::fs::write(cfg.assets().join("late.png"), b"appeared meanwhile").unwrap();
        let mut log = Vec::new();
        let r = apply(&cfg, &items, &mut log).unwrap();
        assert_eq!((r.filed, r.skipped), (0, 1));
        assert_eq!(std::fs::read(cfg.assets().join("late.png")).unwrap(), b"appeared meanwhile");
        assert!(String::from_utf8_lossy(&log).contains("SKIPPED"));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_unusable_name_is_held() {
        let (root, cfg) = store("badname");
        std::fs::write(cfg.watch().join(".."), b"x").ok();
        std::fs::write(cfg.watch().join("%%%"), b"x").unwrap();
        let items = plan(&cfg);
        assert!(items.iter().all(|i| i.kind == Kind::Held), "{items:?}");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
