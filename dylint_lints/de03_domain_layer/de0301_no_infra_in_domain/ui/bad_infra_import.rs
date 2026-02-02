// simulated_dir=/hyperspot/modules/some_module/domain/service.rs
// Test file for DE0301: No Infrastructure Dependencies in Domain
// This file simulates being in a domain/ directory
#![allow(unused_imports)]
#![allow(dead_code)]

// Should trigger DE0301 - infra in domain
use sea_orm::entity::*;

// Should trigger DE0301 - infra in domain
use sqlx::Pool;

// Should trigger DE0301 - infra in domain
use axum::http::StatusCode;

fn main() {}
