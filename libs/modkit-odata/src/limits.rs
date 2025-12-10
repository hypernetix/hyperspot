//! Input validation and safety limits for OData parsing
//!
//! This module enforces sane caps to prevent abuse and resource exhaustion:
//! - Maximum `$top` value
//! - Maximum number of `$orderby` fields
//! - Maximum filter expression length
//! - Cursor integrity checks (HMAC signing)

use crate::Error;

/// Default configuration for OData input limits
#[derive(Debug, Clone)]
pub struct ODataLimits {
    /// Maximum value for $top (default: 1000)
    pub max_top: usize,
    /// Maximum number of fields in $orderby (default: 5)
    pub max_orderby_fields: usize,
    /// Maximum length of $filter expression in characters (default: 2000)
    pub max_filter_length: usize,
    /// Whether to enforce HMAC signing on cursors (default: false for now)
    pub require_signed_cursors: bool,
    /// HMAC key for cursor signing (if enabled)
    pub cursor_hmac_key: Option<Vec<u8>>,
}

impl Default for ODataLimits {
    fn default() -> Self {
        Self {
            max_top: 1000,
            max_orderby_fields: 5,
            max_filter_length: 2000,
            require_signed_cursors: false,
            cursor_hmac_key: None,
        }
    }
}

impl ODataLimits {
    /// Create limits with custom values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum $top value
    pub fn with_max_top(mut self, max_top: usize) -> Self {
        self.max_top = max_top;
        self
    }

    /// Set maximum number of $orderby fields
    pub fn with_max_orderby_fields(mut self, max: usize) -> Self {
        self.max_orderby_fields = max;
        self
    }

    /// Set maximum $filter length
    pub fn with_max_filter_length(mut self, max: usize) -> Self {
        self.max_filter_length = max;
        self
    }

    /// Enable HMAC-signed cursors with the given key
    pub fn with_signed_cursors(mut self, key: Vec<u8>) -> Self {
        self.require_signed_cursors = true;
        self.cursor_hmac_key = Some(key);
        self
    }

    /// Validate a $top value against limits
    pub fn validate_top(&self, top: usize) -> Result<(), Error> {
        if top > self.max_top {
            return Err(Error::InvalidLimit);
        }
        Ok(())
    }

    /// Validate a $filter expression length
    pub fn validate_filter(&self, filter: &str) -> Result<(), Error> {
        if filter.len() > self.max_filter_length {
            return Err(Error::InvalidFilter(format!(
                "Filter expression exceeds maximum length of {} characters",
                self.max_filter_length
            )));
        }
        Ok(())
    }

    /// Validate number of $orderby fields
    pub fn validate_orderby_count(&self, count: usize) -> Result<(), Error> {
        if count > self.max_orderby_fields {
            return Err(Error::InvalidOrderByField(format!(
                "Too many orderby fields (max: {})",
                self.max_orderby_fields
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ODataLimits::default();
        assert_eq!(limits.max_top, 1000);
        assert_eq!(limits.max_orderby_fields, 5);
        assert_eq!(limits.max_filter_length, 2000);
        assert!(!limits.require_signed_cursors);
    }

    #[test]
    fn test_validate_top_ok() {
        let limits = ODataLimits::default();
        assert!(limits.validate_top(500).is_ok());
        assert!(limits.validate_top(1000).is_ok());
    }

    #[test]
    fn test_validate_top_exceeds() {
        let limits = ODataLimits::default();
        assert!(limits.validate_top(1001).is_err());
    }

    #[test]
    fn test_validate_filter_ok() {
        let limits = ODataLimits::default();
        assert!(limits.validate_filter("name eq 'John'").is_ok());
    }

    #[test]
    fn test_validate_filter_too_long() {
        let limits = ODataLimits::default();
        let long_filter = "x".repeat(2001);
        assert!(limits.validate_filter(&long_filter).is_err());
    }

    #[test]
    fn test_validate_orderby_count_ok() {
        let limits = ODataLimits::default();
        assert!(limits.validate_orderby_count(3).is_ok());
        assert!(limits.validate_orderby_count(5).is_ok());
    }

    #[test]
    fn test_validate_orderby_count_exceeds() {
        let limits = ODataLimits::default();
        assert!(limits.validate_orderby_count(6).is_err());
    }

    #[test]
    fn test_custom_limits() {
        let limits = ODataLimits::new()
            .with_max_top(100)
            .with_max_orderby_fields(3)
            .with_max_filter_length(500);

        assert_eq!(limits.max_top, 100);
        assert_eq!(limits.max_orderby_fields, 3);
        assert_eq!(limits.max_filter_length, 500);
    }
}
