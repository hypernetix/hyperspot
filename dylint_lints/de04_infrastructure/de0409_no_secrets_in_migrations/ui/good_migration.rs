// simulated_dir=/hyperspot/migrations/
// Test file for DE0409: safe migration patterns (should not trigger)

// Should not trigger DE0409 - secret in migration
fn create_table() {
    let _sql = "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255))";
}

// Should not trigger DE0409 - secret in migration
fn add_column() {
    let _sql = "ALTER TABLE users ADD COLUMN email VARCHAR(255)";
}

// Should not trigger DE0409 - secret in migration
fn create_index() {
    let _sql = "CREATE INDEX idx_users_email ON users(email)";
}

fn main() {
    create_table();
    add_column();
    create_index();
}
