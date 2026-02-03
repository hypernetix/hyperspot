use bytes::Bytes;
use http::{Request, Response};
use http_body_util::Full;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Tower layer that adds OpenTelemetry tracing to outbound HTTP requests
///
/// Creates a span for each request with:
/// - `http.method`: The HTTP method
/// - `http.url`: The full URL (string form of URI)
/// - `otel.kind`: "client"
///
/// Records `http.status_code` on response and sets `error=true` for 4xx/5xx.
/// Injects W3C trace context headers when OTEL feature is enabled.
#[derive(Clone, Default)]
pub struct OtelLayer;

impl OtelLayer {
    /// Create a new OTEL tracing layer
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for OtelLayer {
    type Service = OtelService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelService { inner }
    }
}

/// Service that wraps requests with OpenTelemetry tracing spans
#[derive(Clone)]
pub struct OtelService<S> {
    inner: S,
}

impl<S, ResBody> Service<Request<Full<Bytes>>> for OtelService<S>
where
    S: Service<Request<Full<Bytes>>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send,
    S::Error: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Full<Bytes>>) -> Self::Future {
        use tracing::{Instrument, Level};

        let method = req.method().clone();
        let uri = req.uri().clone();

        // Sanitize URL for tracing: remove query string to avoid leaking sensitive params
        let url_str = format!(
            "{}://{}{}",
            uri.scheme_str().unwrap_or("https"),
            uri.authority().map_or("", http::uri::Authority::as_str),
            uri.path()
        );

        // Inject trace context into request headers (no-op when OTEL disabled)
        crate::otel::inject_current_span(req.headers_mut());

        // Swap so we call the instance that was poll_ready'd, leaving a fresh clone
        // for the next poll_ready cycle. This satisfies the Tower Service contract.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let span = tracing::span!(
                Level::INFO, "outgoing_http",
                http.method = %method,
                http.url = %url_str,
                otel.kind = "client",
                http.status_code = tracing::field::Empty,
                error = tracing::field::Empty,
            );

            let result = inner.call(req).instrument(span.clone()).await;

            match &result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    span.record("http.status_code", status);
                    if response.status().is_client_error() || response.status().is_server_error() {
                        span.record("error", true);
                    }
                }
                Err(_) => {
                    span.record("error", true);
                }
            }

            result
        })
    }
}
