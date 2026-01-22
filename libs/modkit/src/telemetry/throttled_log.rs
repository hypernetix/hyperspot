//! Lock-free throttled logging helper.
//!
//! Provides a reusable mechanism to limit log frequency without
//! performing any logging itself.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// A lock-free helper that decides whether logging is allowed at the current moment.
///
/// Uses monotonic time (`Instant`) and atomic operations to ensure correct
/// behavior under concurrency without any locks or allocations on the hot path.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use modkit::telemetry::ThrottledLog;
///
/// let throttle = ThrottledLog::new(Duration::from_secs(10));
///
/// if throttle.should_log() {
///     // Perform logging here
/// }
/// ```
pub struct ThrottledLog {
    /// Monotonic start time for computing elapsed milliseconds.
    start: Instant,
    /// Next allowed log time in milliseconds since `start`.
    next_log_ms: AtomicU64,
    /// Throttle interval in milliseconds.
    throttle_ms: u64,
}

fn u64_millis(d: Duration) -> u64 {
    let ms: u128 = d.as_millis();
    u64::try_from(ms).unwrap_or(u64::MAX)
}

impl ThrottledLog {
    /// Creates a new throttled log helper with the given throttle interval.
    #[must_use]
    pub fn new(throttle: Duration) -> Self {
        Self {
            start: Instant::now(),
            next_log_ms: AtomicU64::new(0),
            throttle_ms: u64_millis(throttle),
        }
    }

    /// Returns `true` if logging is allowed at the current moment.
    ///
    /// Uses compare-and-swap to ensure that under concurrent calls,
    /// only one caller per throttle interval receives `true`.
    pub fn should_log(&self) -> bool {
        let now_ms = u64_millis(self.start.elapsed());
        let next = self.next_log_ms.load(Ordering::Relaxed);

        if now_ms < next {
            return false;
        }

        let new_next = now_ms.saturating_add(self.throttle_ms);
        self.next_log_ms
            .compare_exchange(next, new_next, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_call_returns_true() {
        let throttle = ThrottledLog::new(Duration::from_secs(10));
        assert!(throttle.should_log());
    }

    #[test]
    fn second_call_within_interval_returns_false() {
        let throttle = ThrottledLog::new(Duration::from_secs(10));
        assert!(throttle.should_log());
        assert!(!throttle.should_log());
    }
}
