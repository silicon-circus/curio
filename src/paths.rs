//! A PATH THAT CAME FROM A REQUEST IS NOT A PATH UNTIL IT HAS BEEN PROVED TO STAY PUT.
//!
//! This exists because the Crystal version did not have it. `keep` and `discard` took a filename out
//! of an HTTP body and joined it to a directory, and joining does not care about "..", so
//! `{"file":"../../.ssh/id_ed25519"}` named any file the process could read — `keep` then published
//! it under a name the caller chose AND DELETED THE ORIGINAL. The two GET routes had a ".." guard
//! all along; the four routes that actually mutated the filesystem had nothing. The guard was on the
//! door nobody came through.
//!
//! So every request-supplied path goes through here, reads and writes alike.

use std::path::{Component, Path, PathBuf};

/// Resolve `rel` inside `root`, or `None` if it would escape.
///
/// Two checks, because they catch different things. Rejecting `..` components catches traversal
/// spelled out in the path, including any percent-encoded form, since the router decodes before we
/// see it. Comparing canonical paths catches a *symlink* placed inside the directory, which no
/// amount of string inspection can see.
pub fn within(root: &Path, rel: &str) -> Option<PathBuf> {
    if rel.is_empty() {
        return None;
    }
    let rel_path = Path::new(rel);
    for c in rel_path.components() {
        match c {
            Component::Normal(part) => {
                // An empty or dot component is harmless but a NUL is not a filename at all.
                if part.to_string_lossy().contains('\0') {
                    return None;
                }
            }
            Component::CurDir => {}
            // "..", a leading "/", or a Windows prefix all mean "somewhere else".
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    let joined = root.join(rel_path);
    if joined.exists() {
        let real_root = root.canonicalize().ok()?;
        let real = joined.canonicalize().ok()?;
        if !real.starts_with(&real_root) || real == real_root {
            return None;
        }
    }
    Some(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("curio-paths-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(p.join("root")).unwrap();
        std::fs::create_dir_all(p.join("outside")).unwrap();
        std::fs::write(p.join("outside/secret.txt"), b"MY-SSH-KEY").unwrap();
        std::fs::write(p.join("root/ok.png"), b"x").unwrap();
        p
    }

    #[test]
    fn plain_names_resolve() {
        let d = tmp("plain");
        let root = d.join("root");
        assert_eq!(within(&root, "ok.png"), Some(root.join("ok.png")));
        // a name that does not exist yet is still a legal destination
        assert_eq!(within(&root, "new.png"), Some(root.join("new.png")));
        // the dotted naming scheme must survive, including empty components
        assert!(within(&root, "boardwalk.cattacula.night.real.webp").is_some());
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn traversal_is_refused() {
        let d = tmp("trav");
        let root = d.join("root");
        for bad in [
            "../outside/secret.txt",
            "../../outside/secret.txt",
            "a/../../outside/secret.txt",
            "/etc/passwd",
            "..",
            "./../outside/secret.txt",
        ] {
            assert_eq!(within(&root, bad), None, "{bad} was allowed");
        }
        assert_eq!(within(&root, ""), None);
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn a_symlink_planted_inside_is_refused() {
        let d = tmp("symlink");
        let root = d.join("root");
        std::os::unix::fs::symlink(d.join("outside/secret.txt"), root.join("sneak.txt")).unwrap();
        assert_eq!(within(&root, "sneak.txt"), None, "a symlink out of the root was followed");
        // and a directory symlink, which the Crystal version followed all the way out of data/
        std::os::unix::fs::symlink(d.join("outside"), root.join("out")).unwrap();
        assert_eq!(within(&root, "out/secret.txt"), None);
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn the_root_itself_is_not_a_target() {
        let d = tmp("root");
        let root = d.join("root");
        assert_eq!(within(&root, "."), None);
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn a_nested_collection_member_resolves() {
        let d = tmp("nested");
        let root = d.join("root");
        std::fs::create_dir_all(root.join("kit/Textures")).unwrap();
        std::fs::write(root.join("kit/Textures/colormap.png"), b"x").unwrap();
        assert!(within(&root, "kit/Textures/colormap.png").is_some());
        std::fs::remove_dir_all(&d).unwrap();
    }
}
