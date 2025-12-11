use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use axum::http::{Method, StatusCode};
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use parking_lot::Mutex;
use tokio::sync::Semaphore;

use crate::config::ApiIngressConfig;

type RateLimitKey = (Method, String);
type BucketMap = Arc<HashMap<RateLimitKey, Arc<Mutex<TokenBucket>>>>;
type InflightMap = Arc<HashMap<RateLimitKey, Arc<Semaphore>>>;

#[derive(Default, Clone)]
pub struct RateLimiterMap {
    buckets: BucketMap,
    inflight: InflightMap,
}

impl RateLimiterMap {
    pub fn from_specs(specs: &Vec<modkit::api::OperationSpec>, cfg: &ApiIngressConfig) -> Self {
        let mut buckets = HashMap::new();
        let mut inflight = HashMap::new();
        // TODO: Add support for per-route rate limiting
        for spec in specs {
            let (rps, burst, in_flight) = spec
                .rate_limit
                .as_ref()
                .map(|r| (r.rps, r.burst, r.in_flight))
                .unwrap_or((
                    cfg.defaults.rate_limit.rps,
                    cfg.defaults.rate_limit.burst,
                    cfg.defaults.rate_limit.in_flight,
                ));
            let key = (spec.method.clone(), spec.path.clone());
            buckets.insert(
                key.clone(),
                Arc::new(Mutex::new(TokenBucket::new(rps, burst))),
            );
            inflight.insert(key, Arc::new(Semaphore::new(in_flight as usize)));
        }
        Self {
            buckets: Arc::new(buckets),
            inflight: Arc::new(inflight),
        }
    }
}

// TODO: Use tower-governor instead of own implementation
pub async fn rate_limit_middleware(map: RateLimiterMap, req: Request, next: Next) -> Response {
    let method = req.method().clone();
    // Use MatchedPath extension (set by Axum router) for accurate route matching
    let path = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    let key = (method, path);

    if let Some(bucket) = map.buckets.get(&key) {
        let mut b = bucket.lock();
        if !b.allow_now() {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }
    }

    if let Some(sem) = map.inflight.get(&key) {
        match sem.clone().try_acquire_owned() {
            Ok(_permit) => {
                // Allow request; permit is dropped when response future completes
                return next.run(req).await;
            }
            Err(_) => {
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        }
    }

    next.run(req).await
}

struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_per_sec: f64,
    last: Instant,
}

impl TokenBucket {
    fn new(rps: u32, burst: u32) -> Self {
        let cap = burst.max(rps).max(1);
        Self {
            capacity: cap,
            tokens: f64::from(cap),
            refill_per_sec: f64::from(rps.max(1)),
            last: Instant::now(),
        }
    }

    fn allow_now(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        self.last = now;
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(f64::from(self.capacity));
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
