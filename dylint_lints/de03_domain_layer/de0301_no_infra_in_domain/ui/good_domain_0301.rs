// simulated_dir=/hyperspot/modules/some_module/domain/service.rs
// Test file for DE0301: No Infrastructure Dependencies in Domain
// This file simulates being in a domain/ directory - should NOT trigger
#![allow(unused_imports)]
#![allow(dead_code)]

// Should not trigger DE0301 - infra in domain
use std::sync::Arc;

// Should not trigger DE0301 - infra in domain
use uuid::Uuid;

// Should not trigger DE0301 - infra in domain
use anyhow::Result;

// Domain trait - this is correct
pub trait UsersRepository: Send + Sync {
    fn find_by_id(&self, id: Uuid) -> Result<()>;
}

fn main() {}
