//! WHAT CURIO KNOWS ABOUT assets/ WITHOUT RE-READING 1.4 GB.
//!
//! One row per asset: the name, the hash of its bytes, its size, its mtime, and its inode. Only
//! files whose size or mtime moved get re-hashed, because hashing the whole store on every boot
//! would cost ten seconds to learn nothing 99% of the time.
//!
//! THE HASH IS AN IDENTIFIER, NOT A FILENAME. This is the whole reason it is still here after
//! objects/ was deleted: it is the only thing that survives a rename. There are two kinds of change
//! you can make to an asset and the manifest tells them apart —
//!
//!   edited          name the same, hash different          -> an edit
//!   renamed         name gone, its hash under a new name   -> a rename, and the same asset
//!   renamed AND edited                                     -> indistinguishable from delete + add
//!
//! The third case is genuinely ambiguous and is not guessed at: sync reports it and asks, the same
//! way watch/ shows its plan before filing. The inode is stored because it sometimes rescues that
//! case for free — measured, it survives `mv` and survives an in-place truncate-write — but it does
//! NOT survive write-and-rename, which is what GIMP and `magick foo.png foo.png` do, so it is a
//! hint and never a guarantee.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub hash: String,
    pub size: u64,
    pub mtime: i64,
    /// A hint for linking a rename to an edit. Zero when unknown.
    pub inode: u64,
}

/// Sorted by name, so the file on disk is stable and diffable.
pub type Manifest = BTreeMap<String, Entry>;

/// Read the manifest. A missing file is an empty manifest, not an error — that is a first run.
///
/// Four-field rows are accepted without an inode, so a store written by the Crystal version is read
/// rather than discarded.
pub fn load(path: &Path) -> Result<Manifest> {
    let mut rows = Manifest::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(rows),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    for line in BufReader::new(file).lines() {
        let line = line?;
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 {
            continue;
        }
        let name = f[0].to_string();
        rows.insert(name.clone(), Entry {
            name,
            hash: f[1].to_string(),
            size: f[2].parse().unwrap_or(0),
            mtime: f[3].parse().unwrap_or(0),
            inode: f.get(4).and_then(|s| s.parse().ok()).unwrap_or(0),
        });
    }
    Ok(rows)
}

/// Write the manifest via a staging file and a rename, so a reader never sees a half-written one and
/// an interrupted write cannot leave the store without a manifest at all.
pub fn save(path: &Path, manifest: &Manifest) -> Result<()> {
    let stage = path.with_extension("tsv.tmp");
    {
        let mut w = BufWriter::new(File::create(&stage)?);
        for e in manifest.values() {
            writeln!(w, "{}\t{}\t{}\t{}\t{}", e.name, e.hash, e.size, e.mtime, e.inode)?;
        }
        w.flush()?;
    }
    std::fs::rename(&stage, path).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

pub fn sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("hashing {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    // Format the digest bytes directly rather than leaning on a LowerHex impl, which has moved
    // between sha2 releases.
    Ok(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// Everything about a file that can be learned from a stat, which is all sync needs to decide
/// whether the bytes are worth reading.
pub fn stat_of(path: &Path) -> Result<(u64, i64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let m = std::fs::metadata(path)?;
    Ok((m.len(), m.mtime(), m.ino()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("curio-man-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn entry(name: &str, hash: &str) -> Entry {
        Entry { name: name.into(), hash: hash.into(), size: 12, mtime: 1_790_000_000, inode: 42 }
    }

    #[test]
    fn round_trips() {
        let dir = tmp("round");
        let path = dir.join("manifest.tsv");
        let mut m = Manifest::new();
        m.insert("b.png".into(), entry("b.png", "bb"));
        m.insert("a.png".into(), entry("a.png", "aa"));
        save(&path, &m).unwrap();
        assert_eq!(load(&path).unwrap(), m);
        // sorted on disk, so the file is stable and diffable
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("a.png\t"), "not sorted: {text}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_missing_manifest_is_empty_not_an_error() {
        let dir = tmp("missing");
        assert!(load(&dir.join("nope.tsv")).unwrap().is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reads_the_crystal_versions_four_field_rows() {
        let dir = tmp("legacy");
        let path = dir.join("index.tsv");
        std::fs::write(&path, "a.png\taaa\t100\t1790000000\nb.png\tbbb\t200\t1790000001\n").unwrap();
        let m = load(&path).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m["a.png"].hash, "aaa");
        assert_eq!(m["a.png"].inode, 0, "no inode recorded yet, and that is fine");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn save_is_atomic_and_leaves_no_staging_file() {
        let dir = tmp("atomic");
        let path = dir.join("manifest.tsv");
        let mut m = Manifest::new();
        m.insert("a.png".into(), entry("a.png", "aa"));
        save(&path, &m).unwrap();
        let left: Vec<String> = std::fs::read_dir(&dir).unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(left, vec!["manifest.tsv".to_string()]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn hashes_match_the_reference_value() {
        let dir = tmp("hash");
        let path = dir.join("x");
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(sha256(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn stat_reports_size_and_a_nonzero_inode() {
        let dir = tmp("stat");
        let path = dir.join("x");
        std::fs::write(&path, b"0123456789").unwrap();
        let (size, _mtime, inode) = stat_of(&path).unwrap();
        assert_eq!(size, 10);
        assert!(inode > 0);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
