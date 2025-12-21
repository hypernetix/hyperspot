// simulated_dir=/hyperspot/modules/some_module/contract/
use http::StatusCode;
use http::Method;
use axum::http::HeaderMap;

#[allow(dead_code)]
pub struct OrderResult {
    pub status: StatusCode,
}

#[allow(dead_code)]
pub struct RequestInfo {
    pub method: Method,
    pub headers: HeaderMap,
}

fn main() {}
