//! HTTP security utilities.

/// Maximum body preview size for error messages (8KB).
///
/// When an HTTP request returns a non-2xx status, the response body is included
/// in the error message for debugging. This constant limits how much of the body
/// is read to prevent memory issues with large error responses.
pub const ERROR_BODY_PREVIEW_LIMIT: usize = 8 * 1024;
