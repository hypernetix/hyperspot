//! Test that domain_model macro rejects types with infrastructure dependencies.
//!
//! This test should fail to compile because `http::StatusCode` does not implement `DomainSafe`.

use modkit_macros::domain_model;

#[domain_model]
pub struct BadModel {
    pub id: String,
    pub status: http::StatusCode, // This should cause a compile error!
}

fn main() {}
