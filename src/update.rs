//! THE BIG RED SWITCH.
//!
//! curio is read-only until somebody turns update mode on, and it turns itself off again: after a
//! few idle minutes, and whenever the page is reloaded. A curation tool left open in a forgotten tab
//! should not be one stray click from deleting an asset.
//!
//! THE SWITCH IS A KEY, AND THE BROWSER HOLDS IT ONLY IN MEMORY. Turning update mode on hands the
//! page a random key; every write must carry it back in `X-Curio-Update`. The page keeps it in a
//! JavaScript variable — never in storage — so a reload forgets it, and that is the whole of
//! "locks on reload". Nothing has to detect the reload.
//!
//! The server is the authority on idleness. The page counts down too, so it can show the switch
//! going off, but a page that missed its timer still has its write refused here. Any request
//! carrying a valid key counts as activity, so browsing while unlocked keeps it unlocked; only
//! leaving it alone lets it lapse.
//!
//! One key at a time. Unlocking in a second tab replaces the first tab's key, so that tab's next
//! write is refused and it shows itself locked. Two tabs both able to write is not a state worth
//! supporting for a tool with one user.

use std::sync::Mutex;
use std::time::{Duration, Instant};

pub const HEADER: &str = "x-curio-update";

pub struct UpdateMode {
    idle: Duration,
    held: Mutex<Option<Held>>,
}

struct Held {
    key: String,
    last: Instant,
}

impl UpdateMode {
    pub fn new(idle: Duration) -> Self {
        UpdateMode { idle, held: Mutex::new(None) }
    }

    pub fn idle(&self) -> Duration {
        self.idle
    }

    /// Turn update mode on, and return the key that keeps it on.
    pub fn unlock(&self) -> String {
        self.unlock_at(Instant::now())
    }

    pub fn lock(&self) {
        *self.held.lock().unwrap() = None;
    }

    /// Whether `key` holds update mode on. A key that does is refreshed, because using it is the
    /// activity that idleness is measured against.
    pub fn hold(&self, key: &str) -> bool {
        self.hold_at(key, Instant::now())
    }

    fn unlock_at(&self, now: Instant) -> String {
        let key = random_key();
        *self.held.lock().unwrap() = Some(Held { key: key.clone(), last: now });
        key
    }

    fn hold_at(&self, key: &str, now: Instant) -> bool {
        let mut held = self.held.lock().unwrap();
        let Some(h) = held.as_mut() else { return false };
        if now.duration_since(h.last) >= self.idle {
            *held = None;
            return false;
        }
        if !same(h.key.as_bytes(), key.as_bytes()) {
            return false;
        }
        h.last = now;
        true
    }
}

fn random_key() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the system's random source");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Compared without an early exit, so the time taken says nothing about how much of a guess was
/// right.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDLE: Duration = Duration::from_secs(600);

    #[test]
    fn locked_until_unlocked() {
        let u = UpdateMode::new(IDLE);
        assert!(!u.hold(""), "no key opens a switch nobody turned on");
        assert!(!u.hold("anything"));
        let key = u.unlock();
        assert_eq!(key.len(), 32);
        assert!(u.hold(&key));
        assert!(!u.hold("0".repeat(32).as_str()), "only the key that was handed out");
    }

    #[test]
    fn idleness_locks_it_and_use_keeps_it_open() {
        let u = UpdateMode::new(IDLE);
        let t0 = Instant::now();
        let key = u.unlock_at(t0);
        assert!(u.hold_at(&key, t0 + Duration::from_secs(599)), "used just inside the window");
        assert!(u.hold_at(&key, t0 + Duration::from_secs(1100)), "and that use restarted the clock");
        assert!(!u.hold_at(&key, t0 + Duration::from_secs(1700)), "then left alone past it");
        assert!(!u.hold_at(&key, t0 + Duration::from_secs(1701)), "and once lapsed it stays locked");
    }

    #[test]
    fn locking_and_a_second_unlock_both_end_the_old_key() {
        let u = UpdateMode::new(IDLE);
        let first = u.unlock();
        let second = u.unlock();
        assert_ne!(first, second);
        assert!(!u.hold(&first), "another tab's unlock replaces this one");
        assert!(u.hold(&second));
        u.lock();
        assert!(!u.hold(&second));
    }
}
