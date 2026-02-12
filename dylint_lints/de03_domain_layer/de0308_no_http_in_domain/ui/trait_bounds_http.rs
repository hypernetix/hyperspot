// simulated_dir=/hyperspot/modules/some_module/domain/service.rs
#![feature(register_tool)]
#![register_tool(dylint)]
#![allow(dead_code)]

use std::fmt::Display;

// Should trigger DE0308 - HTTP in domain
pub fn make_status() -> impl Display + axum::http::header::IntoHeaderName {
    "content-type"
}

// Should trigger DE0308 - HTTP in domain
pub fn make_header() -> impl axum::http::header::IntoHeaderName {
    "x-custom"
}

fn main() {}
