//! Time, injected. The core never reads the system clock directly, so a
//! scenario test advances thirty minutes in a microsecond.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// An instant, as milliseconds since 1970-01-01 UTC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(pub i64);

impl Timestamp {
    #[must_use]
    pub const fn unix_millis(self) -> i64 {
        self.0
    }
}

/// Where the core reads the time.
pub trait Clock: Send + Sync {
    fn now(&self) -> Timestamp;
}

/// The system's clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| {
                i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
            });
        Timestamp(millis)
    }
}

/// A clock that moves only when told to: fake mode's and the tests'.
#[derive(Debug)]
pub struct ManualClock {
    millis: AtomicI64,
}

impl ManualClock {
    #[must_use]
    pub const fn new(start: Timestamp) -> ManualClock {
        ManualClock {
            millis: AtomicI64::new(start.0),
        }
    }

    /// Moves the clock forward.
    pub fn advance_millis(&self, millis: i64) {
        self.millis.fetch_add(millis, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Timestamp {
        Timestamp(self.millis.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_manual_clock_moves_only_when_told() {
        let clock = ManualClock::new(Timestamp(1_000));
        assert_eq!(clock.now(), Timestamp(1_000));
        clock.advance_millis(30 * 60 * 1_000);
        assert_eq!(clock.now(), Timestamp(1_801_000));
    }

    #[test]
    fn the_system_clock_is_after_2026() {
        // 2026-01-01T00:00:00Z.
        assert!(SystemClock.now().unix_millis() > 1_767_225_600_000);
    }
}
