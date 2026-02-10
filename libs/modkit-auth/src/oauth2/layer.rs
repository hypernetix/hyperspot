use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use http::header::{AUTHORIZATION, HeaderName};
use http::{HeaderValue, Request, Response};
use tower::{Layer, Service};

use super::token::Token;
use modkit_http::HttpError;

/// Tower layer that injects a bearer token into outbound HTTP requests.
///
/// Wraps an [`Token`] handle and sets the `Authorization: Bearer <token>`
/// header (or a custom header) on every request before forwarding it to the
/// inner service.
#[derive(Clone, Debug)]
pub struct BearerAuthLayer {
    token: Token,
    header_name: HeaderName,
}

impl BearerAuthLayer {
    /// Create a layer that injects `Authorization: Bearer <token>`.
    #[must_use]
    pub fn new(token: Token) -> Self {
        Self {
            token,
            header_name: AUTHORIZATION,
        }
    }

    /// Create a layer that injects `<header_name>: Bearer <token>`.
    #[must_use]
    pub fn with_header_name(token: Token, header_name: HeaderName) -> Self {
        Self { token, header_name }
    }
}

impl<S> Layer<S> for BearerAuthLayer {
    type Service = BearerAuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BearerAuthService {
            inner,
            token: self.token.clone(),
            header_name: self.header_name.clone(),
        }
    }
}

/// Tower service that injects a bearer token header before forwarding the
/// request to the inner service.
///
/// Created by [`BearerAuthLayer`].
#[derive(Clone, Debug)]
pub struct BearerAuthService<S> {
    inner: S,
    token: Token,
    header_name: HeaderName,
}

impl<S, B, ResBody> Service<Request<B>> for BearerAuthService<S>
where
    S: Service<Request<B>, Response = Response<ResBody>, Error = HttpError>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
    B: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = Response<ResBody>;
    type Error = HttpError;
    type Future = Pin<Box<dyn Future<Output = Result<Response<ResBody>, HttpError>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let mut bearer_value = match self.token.get() {
            Ok(secret) => {
                let raw = zeroize::Zeroizing::new(format!("Bearer {}", secret.expose()));
                match HeaderValue::from_str(&raw) {
                    Ok(v) => v,
                    Err(e) => return Box::pin(async { Err(HttpError::InvalidHeaderValue(e)) }),
                }
            }
            Err(e) => {
                return Box::pin(async { Err(HttpError::Transport(Box::new(e))) });
            }
        };
        bearer_value.set_sensitive(true);

        req.headers_mut()
            .insert(self.header_name.clone(), bearer_value);

        // Clone-swap pattern (Tower Service contract).
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move { inner.call(req).await })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use bytes::Bytes;
    use http::{Method, Request, Response, StatusCode};
    use http_body_util::Full;
    use httpmock::prelude::*;
    use modkit_utils::SecretString;
    use std::time::Duration;
    use url::Url;

    use crate::oauth2::config::OAuthClientConfig;

    /// Build a test config pointing at the given mock server.
    fn test_config(server: &MockServer) -> OAuthClientConfig {
        OAuthClientConfig {
            token_endpoint: Some(
                Url::parse(&format!("http://localhost:{}/token", server.port())).unwrap(),
            ),
            client_id: "test-client".into(),
            client_secret: SecretString::new("test-secret"),
            http_config: Some(modkit_http::HttpClientConfig::for_testing()),
            jitter_max: Duration::from_millis(0),
            min_refresh_period: Duration::from_millis(100),
            ..Default::default()
        }
    }

    fn token_json(token: &str, expires_in: u64) -> String {
        format!(r#"{{"access_token":"{token}","expires_in":{expires_in},"token_type":"Bearer"}}"#)
    }

    // -- mock inner service ---------------------------------------------------

    /// Mock service that captures request headers and returns 200 OK.
    #[derive(Clone)]
    struct CaptureHeaderService {
        expected_header: HeaderName,
        expected_value: String,
    }

    impl Service<Request<Full<Bytes>>> for CaptureHeaderService {
        type Response = Response<Full<Bytes>>;
        type Error = HttpError;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Request<Full<Bytes>>) -> Self::Future {
            let header = req
                .headers()
                .get(&self.expected_header)
                .expect("expected header not found")
                .to_str()
                .unwrap()
                .to_owned();
            let expected = self.expected_value.clone();

            Box::pin(async move {
                assert_eq!(header, expected);
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Full::new(Bytes::new()))
                    .unwrap())
            })
        }
    }

    // -- trait assertions -----------------------------------------------------

    #[test]
    fn bearer_auth_is_send_sync_clone() {
        fn assert_traits<T: Send + Sync + Clone>() {}
        assert_traits::<BearerAuthLayer>();
        assert_traits::<BearerAuthService<CaptureHeaderService>>();
    }

    // -- header injection -----------------------------------------------------

    #[tokio::test]
    async fn injects_authorization_header() {
        let server = MockServer::start();

        let _mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-layer-test", 3600));
        });

        let token = Token::new(test_config(&server)).await.unwrap();
        let inner = CaptureHeaderService {
            expected_header: AUTHORIZATION,
            expected_value: "Bearer tok-layer-test".into(),
        };

        let layer = BearerAuthLayer::new(token);
        let mut svc = layer.layer(inner);

        let req = Request::builder()
            .method(Method::GET)
            .uri("http://example.com/api")
            .body(Full::new(Bytes::new()))
            .unwrap();

        Service::call(&mut svc, req).await.unwrap();
    }

    #[tokio::test]
    async fn custom_header_name() {
        let server = MockServer::start();

        let _mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-custom-hdr", 3600));
        });

        let token = Token::new(test_config(&server)).await.unwrap();
        let custom_header = HeaderName::from_static("x-api-key");
        let inner = CaptureHeaderService {
            expected_header: custom_header.clone(),
            expected_value: "Bearer tok-custom-hdr".into(),
        };

        let layer = BearerAuthLayer::with_header_name(token, custom_header);
        let mut svc = layer.layer(inner);

        let req = Request::builder()
            .method(Method::GET)
            .uri("http://example.com/api")
            .body(Full::new(Bytes::new()))
            .unwrap();

        Service::call(&mut svc, req).await.unwrap();
    }

    // -- error path -----------------------------------------------------------

    #[tokio::test]
    async fn returns_error_when_token_expired() {
        let server = MockServer::start();

        // Initial token fetch succeeds but with very short TTL.
        let mut success_mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-short-lived", 1));
        });

        let token = Token::new(test_config(&server)).await.unwrap();
        assert_eq!(success_mock.calls(), 1);

        // Remove the success mock; refresh attempts will now fail.
        success_mock.delete();
        let _fail_mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(500)
                .header("content-type", "application/json")
                .body(r#"{"error":"server_error"}"#);
        });

        // Wait for token to expire + refresh to fail.
        tokio::time::sleep(Duration::from_secs(3)).await;

        let inner = CaptureHeaderService {
            expected_header: AUTHORIZATION,
            expected_value: String::new(), // won't be reached
        };

        let layer = BearerAuthLayer::new(token);
        let mut svc = layer.layer(inner);

        let req = Request::builder()
            .method(Method::GET)
            .uri("http://example.com/api")
            .body(Full::new(Bytes::new()))
            .unwrap();

        let err = Service::call(&mut svc, req).await.unwrap_err();
        assert!(
            matches!(err, HttpError::Transport(_)),
            "expected Transport error, got: {err:?}"
        );
    }

    // -- debug safety ---------------------------------------------------------

    #[tokio::test]
    async fn token_value_not_in_debug() {
        let server = MockServer::start();

        let _mock = server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("super-secret-layer", 3600));
        });

        let token = Token::new(test_config(&server)).await.unwrap();
        let layer = BearerAuthLayer::new(token);
        let dbg = format!("{layer:?}");

        assert!(
            !dbg.contains("super-secret-layer"),
            "Debug must not reveal token value: {dbg}"
        );
    }
}
