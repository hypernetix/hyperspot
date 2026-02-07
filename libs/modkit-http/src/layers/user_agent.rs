use crate::error::HttpError;
use http::{HeaderValue, Request, Response};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Tower layer that adds User-Agent header to all requests
#[derive(Clone)]
pub struct UserAgentLayer {
    user_agent: HeaderValue,
}

impl UserAgentLayer {
    /// Create a new `UserAgentLayer` with the specified user agent string
    ///
    /// # Errors
    /// Returns `HttpError::InvalidHeaderValue` if the user agent string is not valid
    pub fn try_new(user_agent: impl AsRef<str>) -> Result<Self, HttpError> {
        let user_agent =
            HeaderValue::from_str(user_agent.as_ref()).map_err(HttpError::InvalidHeaderValue)?;
        Ok(Self { user_agent })
    }
}

impl<S> Layer<S> for UserAgentLayer {
    type Service = UserAgentService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        UserAgentService {
            inner,
            user_agent: self.user_agent.clone(),
        }
    }
}

/// Service that adds User-Agent header to requests
#[derive(Clone)]
pub struct UserAgentService<S> {
    inner: S,
    user_agent: HeaderValue,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for UserAgentService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        // Only add User-Agent if not already present
        if !req.headers().contains_key(http::header::USER_AGENT) {
            req.headers_mut()
                .insert(http::header::USER_AGENT, self.user_agent.clone());
        }
        self.inner.call(req)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use bytes::Bytes;
    use http::{Method, Request, Response, StatusCode};
    use http_body_util::Full;
    use tower::ServiceExt;

    /// Test service that asserts the User-Agent header matches the expected value.
    #[derive(Clone)]
    struct CheckUaService {
        expected_ua: HeaderValue,
    }

    impl Service<Request<Full<Bytes>>> for CheckUaService {
        type Response = Response<Full<Bytes>>;
        type Error = Box<dyn std::error::Error + Send + Sync>;
        type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Request<Full<Bytes>>) -> Self::Future {
            let ua = req.headers().get(http::header::USER_AGENT);
            assert_eq!(ua, Some(&self.expected_ua));
            std::future::ready(Ok(Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::new()))
                .unwrap()))
        }
    }

    #[tokio::test]
    async fn test_user_agent_added() {
        let check_service = CheckUaService {
            expected_ua: HeaderValue::from_static("test-agent/1.0"),
        };

        let layer = UserAgentLayer::try_new("test-agent/1.0").unwrap();
        let mut service = layer.layer(check_service);

        let req = Request::builder()
            .method(Method::GET)
            .uri("http://example.com")
            .body(Full::new(Bytes::new()))
            .unwrap();

        service.ready().await.unwrap().call(req).await.unwrap();
    }

    #[tokio::test]
    async fn test_user_agent_not_overwritten() {
        let check_service = CheckUaService {
            expected_ua: HeaderValue::from_static("custom-agent/2.0"),
        };

        let layer = UserAgentLayer::try_new("test-agent/1.0").unwrap();
        let mut service = layer.layer(check_service);

        let req = Request::builder()
            .method(Method::GET)
            .uri("http://example.com")
            .header(http::header::USER_AGENT, "custom-agent/2.0")
            .body(Full::new(Bytes::new()))
            .unwrap();

        service.ready().await.unwrap().call(req).await.unwrap();
    }

    #[test]
    fn test_user_agent_layer_invalid_value() {
        // Control characters are invalid in header values
        let result = UserAgentLayer::try_new("invalid\x00agent");
        assert!(result.is_err());
    }
}
