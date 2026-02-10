/// Format an [`modkit_http::HttpError`] into a human-readable message with a
/// context prefix.
///
/// The prefix identifies the caller context (e.g. `"JWKS"`, `"OAuth2 token"`)
/// and is prepended to every message so log output is immediately attributable.
///
/// This function is the single place that handles the exhaustive (plus
/// `#[non_exhaustive]` catch-all) match on `HttpError`, shared by JWKS key
/// fetching, `OAuth2` token acquisition, and any future HTTP-based provider.
///
/// # Security
///
/// `HttpStatus` errors include only the status code — the response body is
/// deliberately excluded to prevent server-side diagnostics from leaking
/// into logs or error messages.
#[must_use]
pub fn format_http_error(e: &modkit_http::HttpError, prefix: &str) -> String {
    use modkit_http::HttpError;

    match e {
        HttpError::HttpStatus { status, .. } => {
            format!("{prefix} HTTP {status}")
        }
        HttpError::Json(err) => format!("{prefix} JSON parse failed: {err}"),
        HttpError::Timeout(duration) => {
            format!("{prefix} request timed out after {duration:?}")
        }
        HttpError::DeadlineExceeded(duration) => {
            format!("{prefix} total deadline exceeded after {duration:?}")
        }
        HttpError::Transport(err) => format!("{prefix} transport error: {err}"),
        HttpError::BodyTooLarge { limit, actual } => {
            format!("{prefix} response too large: limit {limit} bytes, got {actual} bytes")
        }
        HttpError::Tls(err) => format!("{prefix} TLS error: {err}"),
        HttpError::RequestBuild(err) => format!("{prefix} request build failed: {err}"),
        HttpError::InvalidHeaderName(err) => format!("{prefix} invalid header name: {err}"),
        HttpError::InvalidHeaderValue(err) => format!("{prefix} invalid header value: {err}"),
        HttpError::FormEncode(err) => format!("{prefix} form encode error: {err}"),
        HttpError::Overloaded => format!("{prefix} request rejected: service overloaded"),
        HttpError::ServiceClosed => format!("{prefix} service unavailable"),
        HttpError::InvalidUri { url, reason, .. } => {
            format!("{prefix} invalid URL '{url}': {reason}")
        }
        HttpError::InvalidScheme { scheme, reason } => {
            format!("{prefix} invalid scheme '{scheme}': {reason}")
        }
        // Future variants (HttpError is #[non_exhaustive]) — omit detail
        // to avoid leaking sensitive data from unknown Display impls.
        _ => format!("{prefix} request failed"),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn http_status_without_body() {
        let err = modkit_http::HttpError::HttpStatus {
            status: http::StatusCode::NOT_FOUND,
            body_preview: String::new(),
            content_type: None,
            retry_after: None,
        };
        let msg = format_http_error(&err, "TEST");
        assert_eq!(msg, "TEST HTTP 404 Not Found");
    }

    #[test]
    fn http_status_with_body_excludes_body() {
        let err = modkit_http::HttpError::HttpStatus {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            body_preview: "something broke".into(),
            content_type: None,
            retry_after: None,
        };
        let msg = format_http_error(&err, "JWKS");
        // body_preview must NOT appear in the output (security)
        assert_eq!(msg, "JWKS HTTP 500 Internal Server Error");
        assert!(!msg.contains("something broke"));
    }

    #[test]
    fn timeout_error() {
        let err = modkit_http::HttpError::Timeout(Duration::from_secs(30));
        let msg = format_http_error(&err, "OAuth2 token");
        assert_eq!(msg, "OAuth2 token request timed out after 30s");
    }

    #[test]
    fn overloaded_error() {
        let err = modkit_http::HttpError::Overloaded;
        let msg = format_http_error(&err, "PREFIX");
        assert_eq!(msg, "PREFIX request rejected: service overloaded");
    }

    #[test]
    fn service_closed_error() {
        let err = modkit_http::HttpError::ServiceClosed;
        let msg = format_http_error(&err, "PREFIX");
        assert_eq!(msg, "PREFIX service unavailable");
    }

    #[test]
    fn prefix_propagated_to_all_variants() {
        // Verify the prefix appears in output for a sample of variants
        let cases: Vec<modkit_http::HttpError> = vec![
            modkit_http::HttpError::Overloaded,
            modkit_http::HttpError::ServiceClosed,
            modkit_http::HttpError::Timeout(Duration::from_secs(1)),
        ];
        for err in &cases {
            let msg = format_http_error(err, "CTX");
            assert!(msg.starts_with("CTX "), "Expected prefix 'CTX' in: {msg}");
        }
    }
}
