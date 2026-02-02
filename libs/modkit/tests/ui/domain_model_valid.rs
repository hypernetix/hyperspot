//! Test that domain_model macro works correctly with valid domain types.
//!
//! This test should compile successfully.

use modkit_macros::domain_model;
use uuid::Uuid;

#[domain_model]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub active: bool,
}

#[domain_model]
pub struct UserId(Uuid);

#[domain_model]
pub enum OrderStatus {
    Pending,
    Confirmed { order_id: Uuid },
    Shipped(String),
    Delivered,
}

fn main() {
    // Verify that DomainModel trait is implemented
    fn assert_domain_model<T: modkit::domain::DomainModel>() {}

    assert_domain_model::<User>();
    assert_domain_model::<UserId>();
    assert_domain_model::<OrderStatus>();
}
