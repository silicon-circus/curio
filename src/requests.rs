//! WHAT HAS ACTUALLY BEEN ASKED FOR.
//!
//! A DEDUP LIST, NOT A LOG. One row per distinct name, because the question is "was this ever
//! fetched", never "was it fetched ninety-four times". A true log of one park session runs to
//! thousands of lines, needs rotating, and writes on the hot path; this is bounded by the number of
//! names that exist and lives in memory between flushes.
//!
//! The reason it exists: curio's cache is not a record of requests. It holds RENDITIONS, so anything
//! served straight out of `assets/` leaves no trace at all — measured on boardwalk's set, the cache
//! would show 49 of 97 names and look complete while missing every webp, glb, wav and mp4 that is
//! stored rather than derived. A payload built from it would be half right.
//!
//! Two columns beyond the name carry most of the value:
//!
//!   status        a name that was asked for and NOT served is the interesting one. That is how a
//!                 reference to `deck.webp` — which exists in the store only as
//!                 `boardwalk.deck.webp` — shows up as a 404 rather than as silence.
//!   derived_from  whether a serve payload needs the webp you would bake, or the png master it came
//!                 from. That is the difference between 20 MB and 950 MB.
//!
//! COVERAGE IS NOT THIS FILE'S PROBLEM, and it is worth being honest about: it records what was
//! exercised. Seasonal art, a setting nobody toggled, a venue nobody walked past — none of it
//! appears. Union it with a static scan of the consumer's source and disagreements between the two
//! are precisely what is worth looking at: the scan sees declared-but-unvisited, this sees names
//! built at runtime that no grep can find.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Seen {
    pub hits: u64,
    pub last: i64,
    pub status: u16,
    pub derived_from: String,
}

pub type Table = BTreeMap<String, Seen>;

pub struct Recorder {
    path: Option<PathBuf>,
    inner: RwLock<State>,
}

struct State {
    table: Table,
    dirty: bool,
}

impl Recorder {
    /// `None` disables recording entirely, for a server that does not want the write.
    pub fn new(path: Option<PathBuf>) -> Self {
        let table = path.as_ref().and_then(|p| load(p).ok()).unwrap_or_default();
        Recorder { path, inner: RwLock::new(State { table, dirty: false }) }
    }

    pub fn enabled(&self) -> bool {
        self.path.is_some()
    }

    /// Note one request. Cheap: a map write under a lock, no I/O.
    pub fn note(&self, name: &str, status: u16, derived_from: Option<&str>) {
        if self.path.is_none() {
            return;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        if let Ok(mut g) = self.inner.write() {
            let e = g.table.entry(name.to_string()).or_default();
            e.hits += 1;
            e.last = now;
            e.status = status;
            // Only overwrite with a real source, so a later cache hit does not erase what it came from.
            if let Some(src) = derived_from {
                e.derived_from = src.to_string();
            }
            g.dirty = true;
        }
    }

    pub fn snapshot(&self) -> Table {
        self.inner.read().map(|g| g.table.clone()).unwrap_or_default()
    }

    /// Write if anything changed. Called on a timer rather than per request.
    pub fn flush(&self) -> Result<()> {
        let Some(path) = &self.path else { return Ok(()) };
        let table = {
            let Ok(mut g) = self.inner.write() else { return Ok(()) };
            if !g.dirty {
                return Ok(());
            }
            g.dirty = false;
            g.table.clone()
        };
        save(path, &table)
    }

    pub fn forget(&self) -> Result<()> {
        if let Ok(mut g) = self.inner.write() {
            g.table.clear();
            g.dirty = true;
        }
        if let Some(path) = &self.path {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }
}

pub fn load(path: &Path) -> Result<Table> {
    let mut t = Table::new();
    let text = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(t),
        Err(e) => return Err(e.into()),
    };
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 4 {
            continue;
        }
        t.insert(f[0].to_string(), Seen {
            hits: f[1].parse().unwrap_or(0),
            last: f[2].parse().unwrap_or(0),
            status: f[3].parse().unwrap_or(0),
            derived_from: f.get(4).map(|s| s.to_string()).unwrap_or_default(),
        });
    }
    Ok(t)
}

/// Staged and renamed, so a reader never sees half a table and an interrupted write cannot lose one.
pub fn save(path: &Path, table: &Table) -> Result<()> {
    let stage = path.with_extension("tsv.tmp");
    let mut out = String::new();
    for (name, s) in table {
        out.push_str(&format!("{}\t{}\t{}\t{}\t{}\n", name, s.hits, s.last, s.status, s.derived_from));
    }
    std::fs::write(&stage, out)?;
    std::fs::rename(&stage, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("curio-req-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn distinct_names_not_a_line_per_request() {
        let d = tmp("dedup");
        let r = Recorder::new(Some(d.join("requests.tsv")));
        for _ in 0..50 {
            r.note("a.webp", 200, None);
        }
        r.note("b.webp", 200, None);
        let t = r.snapshot();
        assert_eq!(t.len(), 2, "one row per name, not per request");
        assert_eq!(t["a.webp"].hits, 50);
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn a_name_that_was_asked_for_and_not_served_is_recorded() {
        // This is how a reference to `deck.webp` — which exists only as `boardwalk.deck.webp` —
        // becomes visible instead of silent.
        let d = tmp("missing");
        let r = Recorder::new(Some(d.join("requests.tsv")));
        r.note("deck.webp", 404, None);
        assert_eq!(r.snapshot()["deck.webp"].status, 404);
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn the_master_a_rendition_came_from_is_kept_even_after_a_cache_hit() {
        let d = tmp("source");
        let r = Recorder::new(Some(d.join("requests.tsv")));
        r.note("x.webp", 200, Some("x.png"));
        r.note("x.webp", 200, None); // served from cache the second time
        assert_eq!(r.snapshot()["x.webp"].derived_from, "x.png",
            "a cache hit must not erase where it came from");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn survives_a_restart() {
        let d = tmp("persist");
        let p = d.join("requests.tsv");
        let r = Recorder::new(Some(p.clone()));
        r.note("a.webp", 200, Some("a.png"));
        r.note("b.glb", 200, None);
        r.flush().unwrap();

        let again = Recorder::new(Some(p.clone()));
        let t = again.snapshot();
        assert_eq!(t.len(), 2);
        assert_eq!(t["a.webp"].derived_from, "a.png");
        assert_eq!(t["b.glb"].status, 200);
        // and no staging file left behind
        let left: Vec<String> = std::fs::read_dir(&d).unwrap().flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(left, vec!["requests.tsv".to_string()]);
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn flush_does_nothing_when_nothing_changed() {
        let d = tmp("clean");
        let p = d.join("requests.tsv");
        let r = Recorder::new(Some(p.clone()));
        r.note("a.webp", 200, None);
        r.flush().unwrap();
        let first = std::fs::metadata(&p).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        r.flush().unwrap();
        assert_eq!(std::fs::metadata(&p).unwrap().modified().unwrap(), first,
            "an unchanged table should not be rewritten");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn disabled_records_nothing_and_writes_nothing() {
        let r = Recorder::new(None);
        assert!(!r.enabled());
        r.note("a.webp", 200, None);
        assert!(r.snapshot().is_empty());
        r.flush().unwrap();
    }

    #[test]
    fn forget_clears_the_table_and_the_file() {
        let d = tmp("forget");
        let p = d.join("requests.tsv");
        let r = Recorder::new(Some(p.clone()));
        r.note("a.webp", 200, None);
        r.flush().unwrap();
        assert!(p.is_file());
        r.forget().unwrap();
        assert!(r.snapshot().is_empty());
        assert!(!p.exists());
        std::fs::remove_dir_all(&d).unwrap();
    }
}
