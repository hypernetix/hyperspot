use std::time::Duration;
use thiserror::Error;

/// Classification of URL validation failures.
///
/// Provides programmatic matching for different failure modes without
/// relying on unstable error message strings.
///
/// # Example
///
/// ```ignore
/// match &err {
///     HttpError::InvalidUri { kind, .. } => match kind {
///         InvalidUriKind::ParseError => println!("Malformed URL syntax"),
///         InvalidUriKind::MissingAuthority => println!("URL needs a host"),
///         InvalidUriKind::MissingScheme => println!("URL needs http:// or https://"),
///         _ => println!("Other URI error"),
///     },
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvalidUriKind {
    /// URL could not be parsed (malformed syntax)
    ParseError,
    /// URL is missing required host/authority component
    MissingAuthority,
    /// URL is missing required scheme (http/https)
    MissingScheme,
}

/// HTTP client error types
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum HttpError {
    /// Request building failed
    #[error("Failed to build request: {0}")]
    RequestBuild(#[from] http::Error),

    /// Invalid header name
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    /// Invalid header value
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    /// Single request attempt timed out
    #[error("Request attempt timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// Total operation deadline exceeded (including all retries)
    #[error("Operation deadline exceeded after {0:?}")]
    DeadlineExceeded(std::time::Duration),

    /// Transport error (network, connection, etc)
    #[error("Transport error: {0}")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// TLS error
    #[error("TLS error: {0}")]
    Tls(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Response body exceeded size limit
    #[error("Response body too large: limit {limit} bytes, got {actual} bytes")]
    BodyTooLarge { limit: usize, actual: usize },

    /// HTTP non-2xx status
    #[error("HTTP {status}: {body_preview}")]
    HttpStatus {
        status: http::StatusCode,
        body_preview: String,
        content_type: Option<String>,
        /// Parsed `Retry-After` header value, if present and valid
        retry_after: Option<Duration>,
    },

    /// JSON parsing error
    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    /// Form URL encoding error
    #[error("Form encoding failed: {0}")]
    FormEncode(#[from] serde_urlencoded::ser::Error),

    /// Service overloaded (concurrency limit reached, fail-fast)
    #[error("Service overloaded: concurrency limit reached")]
    Overloaded,

    /// Internal service failure (buffer worker died, channel closed)
    #[error("Service unavailable: internal failure")]
    ServiceClosed,

    /// Invalid URL (failed to parse)
    ///
    /// Use the `kind` field for programmatic matching. The `reason` field contains
    /// a diagnostic message intended for logging only; do not match on its contents
    /// as the format is unstable and may change between releases.
    #[error("Invalid URL '{url}': {reason}")]
    InvalidUri {
        /// The URL that failed to parse
        url: String,
        /// Structured failure classification for programmatic matching
        kind: InvalidUriKind,
        /// Diagnostic message (unstable format, for logging only)
        reason: String,
    },

    /// Invalid URL scheme for transport security configuration
    #[error("URL scheme '{scheme}' not allowed: {reason}")]
    InvalidScheme {
        /// The URL scheme that was rejected
        scheme: String,
        /// Reason the scheme was rejected
        reason: String,
    },
}

impl From<hyper::Error> for HttpError {
    fn from(err: hyper::Error) -> Self {
        HttpError::Transport(Box::new(err))
    }
}

impl From<hyper_util::client::legacy::Error> for HttpError {
    fn from(err: hyper_util::client::legacy::Error) -> Self {
        HttpError::Transport(Box::new(err))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct TestError(&'static str);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for TestError {}

    #[test]
    fn test_transport_error_preserves_source() {
        let inner = TestError("connection refused");
        let err = HttpError::Transport(Box::new(inner));

        // Verify source() returns the inner error
        let source = err.source();
        assert!(source.is_some(), "Transport error should have a source");

        // Verify we can downcast to the original error type
        let source = source.unwrap();
        let downcast = source.downcast_ref::<TestError>();
        assert!(
            downcast.is_some(),
            "Should be able to downcast to TestError"
        );
        assert_eq!(downcast.unwrap().0, "connection refused");
    }

    #[test]
    fn test_tls_error_preserves_source() {
        let inner = TestError("certificate expired");
        let err = HttpError::Tls(Box::new(inner));

        let source = err.source();
        assert!(source.is_some(), "TLS error should have a source");

        let source = source.unwrap();
        let downcast = source.downcast_ref::<TestError>();
        assert!(downcast.is_some());
        assert_eq!(downcast.unwrap().0, "certificate expired");
    }

    #[test]
    fn test_error_chain_traversal() {
        let inner = TestError("root cause");
        let err = HttpError::Transport(Box::new(inner));

        // Count errors in chain
        let mut count = 0;
        let mut current: Option<&(dyn Error + 'static)> = Some(&err);
        while let Some(e) = current {
            count += 1;
            current = e.source();
        }

        assert_eq!(
            count, 2,
            "Should have 2 errors in chain: HttpError and TestError"
        );
    }
}
