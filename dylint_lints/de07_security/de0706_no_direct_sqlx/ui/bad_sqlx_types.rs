// Test file for DE0706: No Direct sqlx Usage via type annotations
// This file demonstrates BAD patterns using qualified paths in types
#![allow(unused_imports)]
#![allow(dead_code)]

struct Database {
    // Should trigger DE0706 - sqlx
    err: sqlx::Error,
}

// Should trigger DE0706 - sqlx
fn handle_error(err: &sqlx::Error) {}

// Should trigger DE0706 - sqlx
type DbError = sqlx::Error;

enum DbResult {
    // Should trigger DE0706 - sqlx
    Err(sqlx::Error),
}

// Should trigger DE0706 - sqlx
fn nested_generic() -> Option<sqlx::Error> {
    None
}

fn main() {}
