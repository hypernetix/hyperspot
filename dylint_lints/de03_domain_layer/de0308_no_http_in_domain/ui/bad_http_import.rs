// simulated_dir=/hyperspot/modules/some_module/domain/service.rs
// Test file for DE0308: No HTTP in Domain
#![allow(unused_imports)]
#![allow(dead_code)]

// Should trigger DE0308 - HTTP in domain
use http::StatusCode;

// Should trigger DE0308 - HTTP in domain
use axum::http::HeaderMap;

fn main() {}
