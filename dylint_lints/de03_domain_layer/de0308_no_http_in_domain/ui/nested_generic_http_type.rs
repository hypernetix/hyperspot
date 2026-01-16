// simulated_dir=/hyperspot/modules/some_module/domain/service.rs
#![feature(register_tool)]
#![register_tool(dylint)]
#![allow(dead_code)]

pub struct Config {
    // Should trigger DE0308 - HTTP in domain
    status: Option<http::StatusCode>,
    // Should trigger DE0308 - HTTP in domain
    headers: Vec<http::HeaderMap>,
    // Should trigger DE0308 - HTTP in domain
    response: &'static http::Response<String>,
}

fn main() {}
