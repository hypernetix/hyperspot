// Test file for DE0705: SQL injection vulnerabilities (should trigger)

fn query_user_by_id(user_id: &str) {
    // Should trigger DE0705 - SQL injection
    let _query = "SELECT * FROM users WHERE id = '".to_string() + user_id + "'";
}

fn query_user_by_name(name: &str) {
    // Should trigger DE0705 - SQL injection
    let _query = "SELECT * FROM users WHERE name = ".to_string();
}

fn main() {
    query_user_by_id("1");
    query_user_by_name("admin");
}
