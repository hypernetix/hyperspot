// simulated_dir=/hyperspot/modules/some_module/contract/
// Should trigger DE0103 - HTTP types in contract
use http::StatusCode;
// Should trigger DE0103 - HTTP types in contract
use http::Method;
// Should trigger DE0103 - HTTP types in contract
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
