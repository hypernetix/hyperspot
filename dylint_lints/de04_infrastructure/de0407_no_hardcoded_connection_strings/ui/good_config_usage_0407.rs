// Test file for DE0407: proper configuration loading (no hardcoded connection strings)

// Should not trigger DE0407 - hardcoded connection
fn setup_database() -> Result<(), Box<dyn std::error::Error>> {
    // Load from environment
    let _pg_url = std::env::var("DATABASE_URL")?;

    // Load from environment with fallback
    let _redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| String::new());

    Ok(())
}

// Should not trigger DE0407 - hardcoded connection
fn get_config_key() -> &'static str {
    // Regular strings are fine
    "database_url"
}

// Should not trigger DE0407 - hardcoded connection
fn log_message() {
    // Messages containing URL-like text but not connection strings
    let _msg = "Please set POSTGRES_URL environment variable";
}

fn main() {
    let _ = setup_database();
    let _ = get_config_key();
    log_message();
}
