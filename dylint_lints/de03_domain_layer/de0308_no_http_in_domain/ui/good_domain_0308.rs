// simulated_dir=/hyperspot/modules/some_module/domain/service.rs
// Test file for DE0308: Good domain code - no HTTP types
#![allow(unused_imports)]
#![allow(dead_code)]

// Should not trigger DE0308 - HTTP in domain
use std::sync::Arc;

// Should not trigger DE0308 - HTTP in domain
use anyhow::Result;

// Domain error enum - this is correct
pub enum DomainResult {
    Success,
    NotFound,
    InvalidData,
}

fn main() {}
