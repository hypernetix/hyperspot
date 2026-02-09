use std::fmt;

use zeroize::{Zeroize, ZeroizeOnDrop};

/// Opaque wrapper around a secret string value.
///
/// `Debug` and `Display` both print `[REDACTED]` — the inner value is never
/// exposed through formatting traits.  Use [`expose`](Self::expose) for
/// controlled access when constructing HTTP headers or form bodies.
///
/// On [`Drop`] the backing buffer is securely zeroed via the [`zeroize`] crate.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString(String);

impl SecretString {
    /// Create a new `SecretString` from a plain value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Provide read-only access to the underlying secret.
    ///
    /// Callers must not log, store, or otherwise persist the returned slice.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl Clone for SecretString {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use zeroize::Zeroize;

    #[test]
    fn debug_is_redacted() {
        let s = SecretString::new("hunter2");
        assert_eq!(format!("{s:?}"), "[REDACTED]");
    }

    #[test]
    fn display_is_redacted() {
        let s = SecretString::new("hunter2");
        assert_eq!(format!("{s}"), "[REDACTED]");
    }

    #[test]
    fn debug_does_not_contain_secret() {
        let secret = "super-secret-value-12345";
        let s = SecretString::new(secret);
        let dbg = format!("{s:?}");
        assert!(!dbg.contains(secret), "Debug must not contain the secret");
    }

    #[test]
    fn expose_returns_original_value() {
        let s = SecretString::new("hunter2");
        assert_eq!(s.expose(), "hunter2");
    }

    #[test]
    fn clone_preserves_value() {
        let s = SecretString::new("value");
        #[allow(clippy::redundant_clone)]
        let c = s.clone();
        assert_eq!(c.expose(), "value");
    }

    #[test]
    fn zeroize_clears_buffer() {
        let mut s = SecretString::new("sensitive");
        assert_eq!(s.expose(), "sensitive");

        s.zeroize();
        assert!(s.0.is_empty(), "buffer should be empty after zeroize");
    }
}
