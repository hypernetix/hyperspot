//! Compile-fail test: SecureConn cannot be cloned to bypass transaction scope.
//!
//! This test verifies that attempting to clone a SecureConn inside a transaction
//! closure fails at compile time, preventing the transaction bypass vulnerability.

use modkit_db::secure::SecureConn;

fn attempt_clone(conn: SecureConn) {
    // This should fail to compile: SecureConn does not implement Clone
    let _cloned = conn.clone();
}

fn main() {}
