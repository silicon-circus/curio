//! WHICH STORES, AND WHAT CURIO MAY DO TO EACH.
//!
//! curio holds no assets. Each store is a DataDungeon reached over its JSON API — `dd-serve` beside a
//! database, or the API mounted inside the app that owns it — and curio is told where they are in
//! one file:
//!
//!     # ~/.config/curio/curio.toml
//!     [[store]]
//!     name = "pirateship"
//!     url = "http://127.0.0.1:7077"
//!     read_token = "…"        # if the store asks for one
//!     write_token = "…"       # only if you mean to curate it
//!
//! THE TOKENS LIVE HERE AND NOWHERE ELSE. Not in the page, not in a URL, not in the browser's
//! storage: curio's server attaches them on the way through. A store with no write token is
//! read-only in curio whatever the update switch says, and DataDungeon would refuse the write
//! anyway, so the switch only ever matters for stores that were deliberately given one.
//!
//! A list rather than one URL because that is what DataDungeon is in production — each app's store
//! lives on that app's server.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// curio's allocated slot.
pub const DEFAULT_PORT: u16 = 19463;

/// LOOPBACK, BECAUSE CURIO HOLDS EVERY STORE'S TOKENS. Anything that can reach this port can read
/// every store configured here, and write to any store with a write token while update mode is on.
/// Reach it from elsewhere through an SSH tunnel, not by binding wider.
pub const DEFAULT_BIND: &str = "127.0.0.1";

/// How long update mode survives without being used.
pub const DEFAULT_IDLE_MINUTES: u64 = 10;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub bind: String,
    pub idle_minutes: u64,
    pub thumbs: PathBuf,
    pub thumbs_max_bytes: u64,
    pub stores: Vec<Store>,
    /// Where it was read from, for the startup message.
    pub source: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    pub name: String,
    pub url: String,
    pub read_token: Option<String>,
    pub write_token: Option<String>,
}

impl Store {
    pub fn writable(&self) -> bool {
        self.write_token.is_some()
    }
}

/// The file as written. Everything but the stores has a default.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    port: Option<u16>,
    bind: Option<String>,
    idle_minutes: Option<u64>,
    thumbs_max_mb: Option<u64>,
    #[serde(default)]
    store: Vec<Store>,
}

impl Config {
    /// `--config PATH`, else `CURIO_CONFIG`, else `$XDG_CONFIG_HOME/curio/curio.toml`.
    pub fn default_path() -> PathBuf {
        env_path("CURIO_CONFIG").unwrap_or_else(|| xdg_dir("XDG_CONFIG_HOME", ".config").join("curio/curio.toml"))
    }

    pub fn load(path: &Path) -> Result<Config> {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        warn_if_readable_by_others(path);
        let mut cfg = Config::parse(&text).with_context(|| format!("in {}", path.display()))?;
        cfg.source = path.to_path_buf();
        Ok(cfg)
    }

    pub fn parse(text: &str) -> Result<Config> {
        let f: File = toml::from_str(text)?;
        let mut seen = std::collections::HashSet::new();
        for s in &f.store {
            if !valid_name(&s.name) {
                bail!("store name {:?} must be lowercase letters, digits and dashes", s.name);
            }
            if !seen.insert(s.name.clone()) {
                bail!("store {:?} is listed twice", s.name);
            }
            if !(s.url.starts_with("http://") || s.url.starts_with("https://")) {
                bail!("store {:?}: url must start with http:// or https://", s.name);
            }
            reqwest::Url::parse(&s.url).with_context(|| format!("store {:?}: bad url", s.name))?;
        }
        let port = env_parse("CURIO_PORT").or(f.port).unwrap_or(DEFAULT_PORT);
        let bind = std::env::var("CURIO_BIND").ok().filter(|b| !b.is_empty())
            .or(f.bind).unwrap_or_else(|| DEFAULT_BIND.to_string());
        Ok(Config {
            port,
            bind,
            idle_minutes: f.idle_minutes.unwrap_or(DEFAULT_IDLE_MINUTES).max(1),
            thumbs: env_path("CURIO_CACHE").unwrap_or_else(|| xdg_dir("XDG_CACHE_HOME", ".cache").join("curio/thumbs")),
            thumbs_max_bytes: env_parse("CURIO_CACHE_MAX_MB").or(f.thumbs_max_mb).unwrap_or(1024) << 20,
            stores: f.store,
            source: PathBuf::new(),
        })
    }

    pub fn store(&self, name: &str) -> Option<&Store> {
        self.stores.iter().find(|s| s.name == name)
    }
}

/// The name goes into curio's own URLs, so it is kept to what needs no escaping.
fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// The file holds tokens; say so if anyone else on the machine can read them. A warning rather than a
/// refusal, because the fix is one chmod and the user may have a reason.
fn warn_if_readable_by_others(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(m) = std::fs::metadata(path) {
            if m.permissions().mode() & 0o077 != 0 {
                eprintln!("curio: {} holds tokens and others can read it — chmod 600 it", path.display());
            }
        }
    }
}

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

fn env_parse<T: std::str::FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok().and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_keep_their_order_and_tokens() {
        let cfg = Config::parse(r#"
            idle_minutes = 5
            [[store]]
            name = "pirateship"
            url = "http://127.0.0.1:7077"
            [[store]]
            name = "cattacula"
            url = "https://cattacula.example/dd"
            read_token = "r"
            write_token = "w"
        "#).unwrap();
        assert_eq!(cfg.stores.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["pirateship", "cattacula"]);
        assert!(!cfg.store("pirateship").unwrap().writable());
        assert!(cfg.store("cattacula").unwrap().writable());
        assert_eq!(cfg.idle_minutes, 5);
    }

    #[test]
    fn defaults_are_loopback_and_the_allocated_port() {
        let cfg = Config::parse("").unwrap();
        if std::env::var("CURIO_BIND").is_err() {
            assert_eq!(cfg.bind, "127.0.0.1");
        }
        if std::env::var("CURIO_PORT").is_err() {
            assert_eq!(cfg.port, 19463);
        }
        assert!(cfg.stores.is_empty());
    }

    #[test]
    fn mistakes_are_refused_rather_than_guessed_at() {
        let twice = "[[store]]\nname = \"a\"\nurl = \"http://x\"\n[[store]]\nname = \"a\"\nurl = \"http://y\"\n";
        assert!(Config::parse(twice).is_err(), "a duplicate name");
        assert!(Config::parse("[[store]]\nname = \"A b\"\nurl = \"http://x\"\n").is_err(), "a name needing escapes");
        assert!(Config::parse("[[store]]\nname = \"a\"\nurl = \"x:7077\"\n").is_err(), "a url with no scheme");
        assert!(Config::parse("[[store]]\nname = \"a\"\nurl = \"http://x\"\nwrite_tokn = \"w\"\n").is_err(),
            "a misspelt key must not silently leave a store read-only");
    }
}
