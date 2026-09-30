//! A COPY SET ASIDE, ALWAYS.
//!
//! There are two kinds of change you can make to an asset — edit its bytes, or rename it — and only
//! one of them is reversible for free. Editing is destructive the moment your editor writes: by the
//! time `sync` notices, the previous bytes are already gone. They survive only because something
//! copied them BEFORE the edit, which is what `backup/current/` is for. It mirrors `assets/` as of
//! the last sync, so the next sync always has yesterday's version in hand to file away.
//!
//! That is exactly how the deleted `objects/` worked. What it did not do was remember what anything
//! used to be CALLED: 749 of its objects had no name pointing at them and no record of ever having
//! had one, so "version history, and it's free" was retention without recall. Here a replaced
//! version keeps its name and gains a date.
//!
//! READ-ONLY, because the threat is a tool saving over it rather than you deliberately deciding to.
//! `chmod a-w` stops GIMP, `cp`, and a stray shell redirect; it does not stop `chmod`, and it is not
//! meant to. Note that a rename still works on a read-only file — `rename` needs write permission on
//! the DIRECTORY, not the file — which is what lets a version be filed into history without ever
//! becoming writable.
//!
//! REFLINKS, NOT HARDLINKS, and never hardlinks. A hardlink is one inode under two names, so an
//! in-place edit through either one rewrites both; that is the corruption this store was carefully
//! built to avoid. A reflink is a separate inode sharing extents, so it costs nothing until
//! something diverges. Measured on this store: a read-only reflink mirror of 1051 files and 1.4 GB
//! apparent cost 504 KB and 27 ms.

use anyhow::{Context, Result};
use std::fs;
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// `FICLONE` — ask the filesystem to share extents rather than copy bytes. XFS with reflink=1,
/// btrfs, bcachefs and OpenZFS 2.2+ oblige; ext4 refuses and we copy for real.
///
/// This is `_IOW(0x94, 9, int)`: direction 1 << 30, size 4 << 16, type 0x94 << 8, number 9. Worth
/// writing out because the first version of this line had the digits transposed as 0x40094009,
/// which is a perfectly valid-looking number that simply fails — so every "reflink" silently became
/// a full byte copy. Nothing broke, it just quietly stopped being free.
const FICLONE: libc::c_ulong = (1 << 30) | (4 << 16) | (0x94 << 8) | 9;

/// True when the copy shared extents rather than duplicating bytes. Only of interest for reporting.
#[derive(Debug, PartialEq, Eq)]
pub enum How {
    Reflink,
    FullCopy,
}

/// Copy `src` to `dst`, sharing extents if the filesystem can, and leave the result read-only.
///
/// Deliberately not a hardlink. See the module note.
pub fn set_aside(src: &Path, dst: &Path) -> Result<How> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    // A read-only destination from a previous run would refuse to be opened for writing.
    if dst.exists() {
        make_writable(dst)?;
        fs::remove_file(dst)?;
    }

    let how = match clone_extents(src, dst) {
        Ok(()) => How::Reflink,
        Err(_) => {
            // No reflink here — a different filesystem, or ext4. Still correct, just no longer free.
            let _ = fs::remove_file(dst);
            fs::copy(src, dst).with_context(|| {
                format!("copying {} to {}", src.display(), dst.display())
            })?;
            How::FullCopy
        }
    };
    make_read_only(dst)?;
    Ok(how)
}

fn clone_extents(src: &Path, dst: &Path) -> Result<()> {
    let s = fs::File::open(src)?;
    let d = fs::OpenOptions::new().write(true).create(true).truncate(true).open(dst)?;
    // SAFETY: both descriptors are open and owned for the duration of the call; FICLONE only reads
    // the source and writes the destination's extent map.
    let rc = unsafe { libc::ioctl(d.as_raw_fd(), FICLONE, s.as_raw_fd()) };
    if rc == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().into())
    }
}

pub fn make_read_only(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    let mode = perms.mode() & 0o7777;
    perms.set_mode(mode & !0o222);
    fs::set_permissions(path, perms)?;
    Ok(())
}

pub fn make_writable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    let mode = perms.mode() & 0o7777;
    perms.set_mode(mode | 0o200);
    fs::set_permissions(path, perms)?;
    Ok(())
}

/// Move the copy held in `current/` into `history/<stamp>/`, keeping its name.
///
/// This is the whole point of the arrangement: at the moment sync notices a change, `current/` is
/// holding the version from BEFORE it, and filing that is a rename rather than a copy — so it costs
/// nothing and cannot half-succeed.
pub fn archive(current: &Path, history_stamp: &Path, rel_name: &str) -> Result<bool> {
    let from = current.join(rel_name);
    if !from.is_file() {
        return Ok(false);
    }
    let to = history_stamp.join(rel_name);
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&from, &to)
        .with_context(|| format!("filing {} into {}", rel_name, history_stamp.display()))?;
    Ok(true)
}

/// A timestamp directory name. Sortable, and legible without a tool.
pub fn stamp_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    stamp_from_unix(secs)
}

/// Civil time from a Unix second, UTC. Written out rather than pulled in, because one directory
/// name is not worth a date library.
pub fn stamp_from_unix(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400);
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
        y, m, d, tod / 3600, (tod % 3600) / 60, tod % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("curio-backup-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn a_copy_is_set_aside_read_only_with_the_same_bytes() {
        let d = tmp("aside");
        let src = d.join("a.png");
        fs::write(&src, b"original bytes").unwrap();
        let dst = d.join("current/a.png");
        let how = set_aside(&src, &dst).unwrap();
        assert_eq!(fs::read(&dst).unwrap(), b"original bytes");
        // Reflink on a copy-on-write filesystem, a real copy elsewhere; both are correct, so this
        // asserts only that one of them happened. Run with TMPDIR on the store's own filesystem to
        // see which: on the XFS this store lives on, it is Reflink.
        println!("set_aside used {how:?}");
        assert!(matches!(how, How::Reflink | How::FullCopy), "{how:?}");
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(&dst).unwrap().permissions().mode() & 0o222, 0,
            "the copy must not be writable");
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn the_copy_refuses_an_overwrite_but_can_still_be_filed() {
        let d = tmp("readonly");
        let src = d.join("a.png");
        fs::write(&src, b"v1").unwrap();
        let held = d.join("current/a.png");
        set_aside(&src, &held).unwrap();

        // this is the GIMP-save threat, and it must fail
        assert!(fs::write(&held, b"clobbered").is_err(), "a read-only copy was overwritten");
        assert_eq!(fs::read(&held).unwrap(), b"v1");

        // ...while filing it into history still works, because rename needs the directory
        let stamp = d.join("history/2026-09-30T12-00-00");
        assert!(archive(&d.join("current"), &stamp, "a.png").unwrap());
        assert_eq!(fs::read(stamp.join("a.png")).unwrap(), b"v1");
        assert!(!held.exists(), "current/ should no longer hold it");
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn setting_aside_again_replaces_the_read_only_copy() {
        let d = tmp("refresh");
        let src = d.join("a.png");
        let held = d.join("current/a.png");
        fs::write(&src, b"v1").unwrap();
        set_aside(&src, &held).unwrap();
        fs::write(&src, b"v2 is longer").unwrap();
        set_aside(&src, &held).unwrap();
        assert_eq!(fs::read(&held).unwrap(), b"v2 is longer");
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn archiving_something_not_held_is_not_an_error() {
        let d = tmp("absent");
        fs::create_dir_all(d.join("current")).unwrap();
        assert!(!archive(&d.join("current"), &d.join("history/x"), "never-seen.png").unwrap());
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn collection_members_keep_their_shape() {
        let d = tmp("nested");
        let src = d.join("kit/Textures/colormap.png");
        fs::create_dir_all(src.parent().unwrap()).unwrap();
        fs::write(&src, b"atlas").unwrap();
        let held = d.join("current/kit/Textures/colormap.png");
        set_aside(&src, &held).unwrap();
        assert!(held.is_file());
        let stamp = d.join("history/s");
        assert!(archive(&d.join("current"), &stamp, "kit/Textures/colormap.png").unwrap());
        assert!(stamp.join("kit/Textures/colormap.png").is_file());
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn stamps_are_sortable_and_legible() {
        assert_eq!(stamp_from_unix(0), "1970-01-01T00-00-00");
        assert_eq!(stamp_from_unix(1_790_000_000), "2026-09-21T14-13-20");
        // sortable as text is the property that matters for pruning by date
        let mut v = vec![stamp_from_unix(1_790_000_100), stamp_from_unix(1_790_000_000)];
        v.sort();
        assert_eq!(v[0], stamp_from_unix(1_790_000_000));
    }
}
