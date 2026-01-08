//! Retry intent and related types.
//!
//! OAGW performs no implicit retries — retries occur only when explicitly
//! requested by the caller via `RetryIntent`.

use std::sync::atomic::AtomicU32;
use std::sync::Arc;

/// Retry intent specifying how OAGW should handle retries.
///
/// Default is no retry (`max_attempts: 1`).
#[derive(Debug, Clone)]
pub struct RetryIntent {
    /// Maximum number of attempts (1 = no retry).
    pub max_attempts: u32,
    /// Conditions under which to retry.
    pub retry_on: Vec<RetryOn>,
    /// Scope for retry (same link, different link, or reroute).
    pub scope: RetryScope,
    /// Whether to allow strategy re-selection when switching links.
    pub allow_strategy_reselect: bool,
    /// Backoff strategy between retries.
    pub backoff: BackoffStrategy,
    /// Optional shared budget limiting total retries.
    pub budget: Option<Arc<RetryBudget>>,
}

impl Default for RetryIntent {
    fn default() -> Self {
        Self {
            max_attempts: 1, // No retry by default
            retry_on: Vec::new(),
            scope: RetryScope::SameLink,
            allow_strategy_reselect: false,
            backoff: BackoffStrategy::None,
            budget: None,
        }
    }
}

impl RetryIntent {
    /// Create a retry intent with the specified max attempts.
    #[must_use]
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Add a retry condition.
    #[must_use]
    pub fn retry_on(mut self, condition: RetryOn) -> Self {
        self.retry_on.push(condition);
        self
    }

    /// Set the retry scope.
    #[must_use]
    pub fn with_scope(mut self, scope: RetryScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set the backoff strategy.
    #[must_use]
    pub fn with_backoff(mut self, backoff: BackoffStrategy) -> Self {
        self.backoff = backoff;
        self
    }

    /// Set a shared retry budget.
    #[must_use]
    pub fn with_budget(mut self, budget: Arc<RetryBudget>) -> Self {
        self.budget = Some(budget);
        self
    }
}

/// Condition for triggering a retry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryOn {
    /// Retry on timeout errors.
    Timeout,
    /// Retry on connection errors.
    ConnectError,
    /// Retry on specific HTTP status class.
    StatusClass(StatusClass),
    /// Retry on specific GTS error ID.
    ErrorGtsId(String),
}

/// HTTP status class for retry conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    /// 5xx server errors.
    C5xx,
    /// 429 Too Many Requests.
    C429,
}

/// Scope for retry attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetryScope {
    /// Retry using the same link.
    #[default]
    SameLink,
    /// Retry using a different link if available.
    DifferentLink,
    /// Re-run the full route selection strategy.
    Reroute,
}

/// Backoff strategy between retry attempts.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum BackoffStrategy {
    /// No delay between retries.
    #[default]
    None,
    /// Constant delay.
    Constant {
        /// Delay in milliseconds.
        delay_ms: u64,
    },
    /// Linear backoff.
    Linear {
        /// Initial delay in milliseconds.
        initial_ms: u64,
        /// Increment per attempt in milliseconds.
        increment_ms: u64,
        /// Maximum delay in milliseconds.
        max_ms: u64,
    },
    /// Exponential backoff.
    Exponential {
        /// Initial delay in milliseconds.
        initial_ms: u64,
        /// Multiplier per attempt.
        multiplier: f64,
        /// Maximum delay in milliseconds.
        max_ms: u64,
    },
}

impl BackoffStrategy {
    /// Calculate delay for a given attempt (0-indexed).
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        match self {
            Self::None => 0,
            Self::Constant { delay_ms } => *delay_ms,
            Self::Linear {
                initial_ms,
                increment_ms,
                max_ms,
            } => {
                let delay = initial_ms + (u64::from(attempt) * increment_ms);
                delay.min(*max_ms)
            }
            Self::Exponential {
                initial_ms,
                multiplier,
                max_ms,
            } => {
                // Allow precision loss for backoff calculation - acceptable for timing
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
                let delay = (*initial_ms as f64) * multiplier.powi(attempt as i32);
                // Truncation is intentional - we want milliseconds as integer
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let delay_ms = delay as u64;
                delay_ms.min(*max_ms)
            }
        }
    }
}

/// Shared budget to limit total retries across multiple calls.
#[derive(Debug)]
pub struct RetryBudget {
    /// Maximum retries allowed in the time window.
    pub max_retries: AtomicU32,
    /// Time window in seconds.
    pub time_window_sec: u64,
    /// Minimum retry rate guarantee.
    pub min_retries_per_sec: f64,
}

impl RetryBudget {
    /// Create a new retry budget.
    #[must_use]
    pub fn new(max_retries: u32, time_window_sec: u64, min_retries_per_sec: f64) -> Self {
        Self {
            max_retries: AtomicU32::new(max_retries),
            time_window_sec,
            min_retries_per_sec,
        }
    }

    /// Try to acquire a retry from the budget.
    ///
    /// Returns `true` if a retry is allowed, `false` otherwise.
    pub fn try_acquire(&self) -> bool {
        // TODO(v3): Implement proper budget tracking with time window
        // For now, simple atomic decrement
        let current = self.max_retries.load(std::sync::atomic::Ordering::Relaxed);
        if current > 0 {
            self.max_retries
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_retry_intent() {
        let intent = RetryIntent::default();
        assert_eq!(intent.max_attempts, 1);
        assert!(intent.retry_on.is_empty());
    }

    #[test]
    fn test_exponential_backoff() {
        let backoff = BackoffStrategy::Exponential {
            initial_ms: 100,
            multiplier: 2.0,
            max_ms: 5000,
        };

        assert_eq!(backoff.delay_for_attempt(0), 100);
        assert_eq!(backoff.delay_for_attempt(1), 200);
        assert_eq!(backoff.delay_for_attempt(2), 400);
        assert_eq!(backoff.delay_for_attempt(6), 5000); // Capped at max
    }
}
