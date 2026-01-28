// simulated_dir=/hyperspot/modules/another_module/domain/
// Test file for DE0301: Mixed imports - some valid, some violating
#![allow(unused_imports)]
#![allow(dead_code)]

// Should not trigger DE0301 - infra in domain
use std::collections::HashMap;

// Should trigger DE0301 - infra in domain
use sea_orm::DatabaseConnection;

// Should not trigger DE0301 - infra in domain
use thiserror::Error;

fn main() {}
