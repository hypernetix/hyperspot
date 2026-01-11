// simulated_dir=/hyperspot/migrations/
// Test file for DE0409: hardcoded secrets in migrations (should trigger)

fn create_admin_user() {
    // Should trigger DE0409 - secret in migration
    let _sql = "INSERT INTO users (name, password='secret123') VALUES ('admin')";
}

fn configure_api() {
    // Should trigger DE0409 - secret in migration
    let _config = "api_key=sk_live_1234567890abcdef";
}

fn setup_database() {
    // Should trigger DE0409 - secret in migration
    let _conn = "postgres://user:password@localhost:5432/db";
}

fn main() {
    create_admin_user();
    configure_api();
    setup_database();
}
