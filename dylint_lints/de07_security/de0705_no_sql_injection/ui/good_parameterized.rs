// Test file for DE0705: safe parameterized queries (should not trigger)

// Should not trigger DE0705 - SQL injection
fn query_user_safe() {
    // Static SQL strings without concatenation are fine
    let _query = "SELECT * FROM users WHERE id = $1";
}

// Should not trigger DE0705 - SQL injection
fn query_with_placeholder() {
    // Parameterized query strings are safe
    let _query = "SELECT * FROM users WHERE name = ? AND status = ?";
}

// Should not trigger DE0705 - SQL injection
fn simple_concatenation() {
    // Non-SQL string concatenation is fine
    let _msg = "Hello, ".to_string() + "World!";
}

fn main() {
    query_user_safe();
    query_with_placeholder();
    simple_concatenation();
}
