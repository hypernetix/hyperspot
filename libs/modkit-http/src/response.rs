use crate::error::HttpError;
use crate::security::ERROR_BODY_PREVIEW_LIMIT;
use bytes::Bytes;
use http::{HeaderMap, Response, StatusCode};
use http_body::Frame;
use http_body_util::BodyExt;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime};

/// Parse `Retry-After` header value into a `Duration`.
///
/// Supports two formats per RFC 7231:
/// - Seconds: "120" → 120 seconds
/// - HTTP-date (RFC 1123): "Wed, 21 Oct 2015 07:28:00 GMT" → duration until that time
///
/// Returns `None` if:
/// - Header is missing
/// - Value cannot be parsed as integer or HTTP-date
/// - Parsed duration is negative (time already passed or negative seconds)
pub fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(http::header::RETRY_AFTER)?.to_str().ok()?;
    let trimmed = value.trim();

    // First, try to parse as seconds (most common format)
    if let Ok(seconds) = trimmed.parse::<i64>() {
        if seconds < 0 {
            return None;
        }
        return Some(Duration::from_secs(seconds.cast_unsigned()));
    }

    // Fall back to HTTP-date format (RFC 1123)
    parse_http_date(trimmed)
}

/// Parse HTTP-date (RFC 1123) and return duration until that time.
/// Returns `None` if the date is in the past or cannot be parsed.
fn parse_http_date(value: &str) -> Option<Duration> {
    let parsed = httpdate::parse_http_date(value).ok()?;
    let now = SystemTime::now();

    // Return duration until the parsed time (None if already passed)
    parsed.duration_since(now).ok()
}

/// Type alias for the boxed response body that supports decompression.
///
/// This type can hold either a raw body or a decompressed body (gzip/br/deflate).
/// The body is type-erased to allow the decompression layer to work transparently.
pub type ResponseBody =
    http_body_util::combinators::BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>;

pin_project! {
    /// Body wrapper that enforces size limits during streaming.
    ///
    /// Created by [`HttpResponse::into_limited_body()`]. Tracks bytes read
    /// and returns [`HttpError::BodyTooLarge`] if the limit is exceeded.
    ///
    /// Unlike reading the body with [`HttpResponse::bytes()`] or [`HttpResponse::json()`],
    /// this allows incremental processing while still enforcing limits.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use http_body_util::BodyExt;
    ///
    /// let response = client.get("https://example.com/large-file").send().await?;
    /// let mut body = response.into_limited_body();
    ///
    /// while let Some(frame) = body.frame().await {
    ///     let frame = frame?; // Returns BodyTooLarge if limit exceeded
    ///     if let Some(chunk) = frame.data_ref() {
    ///         process_chunk(chunk);
    ///     }
    /// }
    /// ```
    pub struct LimitedBody {
        #[pin]
        inner: ResponseBody,
        limit: usize,
        read: usize,
    }
}

impl LimitedBody {
    /// Creates a new `LimitedBody` wrapping the given body with the specified limit.
    #[must_use]
    pub fn new(inner: ResponseBody, limit: usize) -> Self {
        Self {
            inner,
            limit,
            read: 0,
        }
    }

    /// Returns the number of bytes read so far.
    #[must_use]
    pub fn bytes_read(&self) -> usize {
        self.read
    }

    /// Returns the configured size limit.
    #[must_use]
    pub fn limit(&self) -> usize {
        self.limit
    }
}

impl http_body::Body for LimitedBody {
    type Data = Bytes;
    type Error = HttpError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();

        match this.inner.poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    *this.read += data.len();
                    if *this.read > *this.limit {
                        return Poll::Ready(Some(Err(HttpError::BodyTooLarge {
                            limit: *this.limit,
                            actual: *this.read,
                        })));
                    }
                }
                Poll::Ready(Some(Ok(frame)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(HttpError::Transport(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// HTTP response wrapper with body-reading helpers
///
/// Provides a reqwest-like API for reading response bodies:
/// - `resp.error_for_status()?` - Check status without reading body
/// - `resp.bytes().await?` - Read raw bytes
/// - `resp.checked_bytes().await?` - Read bytes with status check
/// - `resp.json::<T>().await?` - Parse as JSON with status check
///
/// All body reads enforce the configured `max_body_size` limit.
#[derive(Debug)]
pub struct HttpResponse {
    pub(crate) inner: Response<ResponseBody>,
    pub(crate) max_body_size: usize,
}

impl HttpResponse {
    /// Get the response status code
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Get the response headers
    #[must_use]
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Consume the wrapper and return the inner response with boxed body
    ///
    /// Useful for advanced callers who need direct access to the response.
    /// Note: The body has already been through the decompression layer,
    /// so it contains decompressed bytes if the server sent compressed data.
    #[must_use]
    pub fn into_inner(self) -> Response<ResponseBody> {
        self.inner
    }

    /// Check status and return error for non-2xx responses
    ///
    /// Does NOT read the response body. For non-2xx status, returns
    /// `HttpError::HttpStatus` with an empty body preview.
    ///
    /// # Errors
    ///
    /// Returns `HttpError::HttpStatus` if the response status is not 2xx.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let resp = client.get("https://example.com/api").send().await?;
    /// let resp = resp.error_for_status()?;  // Fails if not 2xx
    /// let body = resp.bytes().await?;
    /// ```
    pub fn error_for_status(self) -> Result<Self, HttpError> {
        if self.inner.status().is_success() {
            return Ok(self);
        }

        let content_type = self
            .inner
            .headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let retry_after = parse_retry_after(self.inner.headers());

        Err(HttpError::HttpStatus {
            status: self.inner.status(),
            body_preview: String::new(),
            content_type,
            retry_after,
        })
    }

    /// Read response body as bytes without status check
    ///
    /// Enforces `max_body_size` limit.
    ///
    /// # Errors
    /// Returns `HttpError::BodyTooLarge` if body exceeds limit.
    pub async fn bytes(self) -> Result<Bytes, HttpError> {
        read_body_limited_impl(self.inner, self.max_body_size).await
    }

    /// Read response body as bytes with status check
    ///
    /// Returns `HttpError::HttpStatus` for non-2xx responses (with body preview).
    /// Enforces `max_body_size` limit for successful responses.
    ///
    /// # Errors
    /// Returns `HttpError::HttpStatus` if status is not 2xx.
    /// Returns `HttpError::BodyTooLarge` if body exceeds limit.
    pub async fn checked_bytes(self) -> Result<Bytes, HttpError> {
        checked_body_impl(self.inner, self.max_body_size).await
    }

    /// Parse response body as JSON with status check
    ///
    /// Equivalent to `resp.checked_bytes().await?` followed by JSON parsing.
    ///
    /// # Errors
    /// Returns `HttpError::HttpStatus` if status is not 2xx.
    /// Returns `HttpError::BodyTooLarge` if body exceeds limit.
    /// Returns `HttpError::Json` if parsing fails.
    pub async fn json<T: DeserializeOwned>(self) -> Result<T, HttpError> {
        let body_bytes = checked_body_impl(self.inner, self.max_body_size).await?;
        let value = serde_json::from_slice(&body_bytes)?;
        Ok(value)
    }

    /// Read response body as text (UTF-8) with status check
    ///
    /// Equivalent to `resp.checked_bytes().await?` followed by UTF-8 conversion.
    /// Invalid UTF-8 sequences are replaced with the Unicode replacement character.
    ///
    /// # Errors
    /// Returns `HttpError::HttpStatus` if status is not 2xx.
    /// Returns `HttpError::BodyTooLarge` if body exceeds limit.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let body = client
    ///     .get("https://example.com/text")
    ///     .send()
    ///     .await?
    ///     .text()
    ///     .await?;
    /// println!("Response: {}", body);
    /// ```
    pub async fn text(self) -> Result<String, HttpError> {
        let body_bytes = checked_body_impl(self.inner, self.max_body_size).await?;
        Ok(String::from_utf8_lossy(&body_bytes).into_owned())
    }

    /// Returns the response body as a stream for incremental processing.
    ///
    /// # Warning: No Size Limit Enforcement
    ///
    /// This method does **NOT** enforce `max_body_size`. For untrusted responses
    /// (especially compressed), prefer [`into_limited_body()`](Self::into_limited_body)
    /// to protect against decompression bombs and memory exhaustion.
    ///
    /// Unlike `bytes()`, `json()`, or `text()`, this method does NOT:
    /// - Check the HTTP status code (use `error_for_status()` first if needed)
    /// - Enforce the `max_body_size` limit (caller is responsible for limiting)
    /// - Buffer the entire body in memory
    ///
    /// Use this only when:
    /// - You trust the response source AND have external size limits
    /// - You're implementing custom streaming logic with your own limits
    /// - Performance is critical and you can guarantee bounded responses
    ///
    /// # Example
    ///
    /// ```ignore
    /// use http_body_util::BodyExt;
    ///
    /// let response = client.get("https://example.com/large-file").send().await?;
    ///
    /// // Check status first (optional)
    /// if !response.status().is_success() {
    ///     return Err(/* handle error */);
    /// }
    ///
    /// // Get the body stream (WARNING: no size limit!)
    /// let mut body = response.into_body();
    ///
    /// // Process frames incrementally
    /// while let Some(frame) = body.frame().await {
    ///     let frame = frame?;
    ///     if let Some(chunk) = frame.data_ref() {
    ///         process_chunk(chunk);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn into_body(self) -> ResponseBody {
        self.inner.into_body()
    }

    /// Returns the response body as a size-limited stream.
    ///
    /// Unlike [`into_body()`](Self::into_body), this method wraps the body in a
    /// [`LimitedBody`] that enforces the configured `max_body_size` limit during
    /// streaming. This protects against decompression bombs where a small compressed
    /// payload expands to gigabytes of memory.
    ///
    /// The limit is enforced on **decompressed** bytes, so a 1KB gzip payload that
    /// decompresses to 1GB will be rejected.
    ///
    /// # Errors
    ///
    /// When the limit is exceeded, the next `poll_frame()` call returns
    /// `HttpError::BodyTooLarge`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use http_body_util::BodyExt;
    ///
    /// let response = client.get("https://example.com/large-file").send().await?;
    /// let mut body = response.into_limited_body();
    ///
    /// while let Some(frame) = body.frame().await {
    ///     let frame = frame?; // Returns BodyTooLarge if limit exceeded
    ///     if let Some(chunk) = frame.data_ref() {
    ///         process_chunk(chunk);
    ///     }
    /// }
    ///
    /// println!("Total bytes read: {}", body.bytes_read());
    /// ```
    #[must_use]
    pub fn into_limited_body(self) -> LimitedBody {
        LimitedBody::new(self.inner.into_body(), self.max_body_size)
    }

    /// Returns the configured max body size for this response.
    ///
    /// This is the limit that would be applied by `bytes()`, `checked_bytes()`,
    /// `json()`, and `text()` methods.
    #[must_use]
    pub fn max_body_size(&self) -> usize {
        self.max_body_size
    }
}

/// Internal implementation of `checked_body` that doesn't capture `&self`
pub async fn checked_body_impl(
    response: Response<ResponseBody>,
    max_body_size: usize,
) -> Result<Bytes, HttpError> {
    let status = response.status();
    let content_type = response
        .headers()
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    if !status.is_success() {
        // Parse Retry-After header before consuming response
        let retry_after = parse_retry_after(response.headers());

        // Read limited preview for error message
        // Handle BodyTooLarge gracefully - don't let it hide the HTTP status error
        let preview_limit = max_body_size.min(ERROR_BODY_PREVIEW_LIMIT);
        let body_preview = match read_body_limited_impl(response, preview_limit).await {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(HttpError::BodyTooLarge { .. }) => "<body too large for preview>".to_owned(),
            Err(e) => return Err(e), // Propagate transport errors
        };

        return Err(HttpError::HttpStatus {
            status,
            body_preview,
            content_type,
            retry_after,
        });
    }

    read_body_limited_impl(response, max_body_size).await
}

/// Internal implementation of `read_body_limited` that doesn't capture `&self`
///
/// This function reads from the (potentially decompressed) response body,
/// enforcing the byte limit on decompressed data. This protects against
/// decompression bombs where a small compressed payload expands to gigabytes.
pub async fn read_body_limited_impl(
    response: Response<ResponseBody>,
    limit: usize,
) -> Result<Bytes, HttpError> {
    let (_parts, body) = response.into_parts();

    let mut collected = Vec::new();
    let mut body = std::pin::pin!(body);

    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(HttpError::Transport)?;
        if let Some(chunk) = frame.data_ref() {
            if collected.len() + chunk.len() > limit {
                return Err(HttpError::BodyTooLarge {
                    limit,
                    actual: collected.len() + chunk.len(),
                });
            }
            collected.extend_from_slice(chunk);
        }
    }

    Ok(Bytes::from(collected))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_retry_after_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::RETRY_AFTER, "120".parse().unwrap());

        let result = parse_retry_after(&headers);
        assert_eq!(result, Some(Duration::from_secs(120)));
    }

    #[test]
    fn test_parse_retry_after_seconds_with_whitespace() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::RETRY_AFTER, "  60  ".parse().unwrap());

        let result = parse_retry_after(&headers);
        assert_eq!(result, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_parse_retry_after_missing() {
        let headers = HeaderMap::new();
        let result = parse_retry_after(&headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_retry_after_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::RETRY_AFTER, "not-a-number".parse().unwrap());

        let result = parse_retry_after(&headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_retry_after_http_date_in_past() {
        let mut headers = HeaderMap::new();
        // HTTP-date in the past returns None
        headers.insert(
            http::header::RETRY_AFTER,
            "Wed, 21 Oct 2015 07:28:00 GMT".parse().unwrap(),
        );

        let result = parse_retry_after(&headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_retry_after_http_date_in_future() {
        let mut headers = HeaderMap::new();
        // Create a date 60 seconds in the future
        let future_time = SystemTime::now() + Duration::from_secs(60);
        let http_date = httpdate::fmt_http_date(future_time);
        headers.insert(http::header::RETRY_AFTER, http_date.parse().unwrap());

        let result = parse_retry_after(&headers);
        assert!(result.is_some());
        // Should be approximately 60 seconds (with some tolerance for test execution)
        let duration = result.unwrap();
        assert!(duration.as_secs() >= 58 && duration.as_secs() <= 62);
    }

    #[test]
    fn test_parse_retry_after_negative_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::RETRY_AFTER, "-5".parse().unwrap());

        let result = parse_retry_after(&headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_retry_after_zero() {
        let mut headers = HeaderMap::new();
        headers.insert(http::header::RETRY_AFTER, "0".parse().unwrap());

        let result = parse_retry_after(&headers);
        assert_eq!(result, Some(Duration::from_secs(0)));
    }
}
