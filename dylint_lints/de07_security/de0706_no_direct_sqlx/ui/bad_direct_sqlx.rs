// Test file for DE0706: No Direct sqlx Usage
// This file demonstrates BAD patterns that should trigger the lint
#![allow(unused_imports)]
#![allow(dead_code)]

// Should trigger DE0706 - sqlx
use sqlx;

// Should trigger DE0706 - sqlx
use sqlx::Error;

fn main() {
    // These imports should all be flagged by the lint
}
