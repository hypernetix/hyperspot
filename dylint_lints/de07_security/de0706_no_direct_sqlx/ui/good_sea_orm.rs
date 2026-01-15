// Test file for DE0706: No Direct sqlx Usage
// This file demonstrates GOOD patterns that should NOT trigger the lint
#![allow(unused_imports)]
#![allow(dead_code)]

// Using sea_orm is allowed
use sea_orm::EntityTrait;

fn main() {
    // Sea-ORM usage is the preferred pattern
}
