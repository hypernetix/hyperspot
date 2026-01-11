#![allow(dead_code, unused_variables)]

fn load_from_env() {
    // Should not trigger DE0703 - loading from environment
    let api_key = std::env::var("API_KEY").unwrap();
    
    // Should not trigger DE0703 - loading from config
    let config_path = "/path/to/config";
}

fn short_strings() {
    // Should not trigger DE0703 - too short
    let short = "hello";
    let tiny = "ok";
}

fn normal_text() {
    // Should not trigger DE0703 - normal text
    let message = "Processing user request";
    let url = "https://example.com/api/users";
}

fn sql_queries() {
    // Should not trigger DE0703 - SQL queries
    let query1 = "SELECT * FROM users WHERE id = $1";
    let query2 = "INSERT INTO logs (message) VALUES ($1)";
    let query3 = "UPDATE settings SET value = $1";
}

fn documentation_examples() {
    // Should not trigger DE0703 - documentation with e.g.
    let help_text = "deps must be an array, e.g. deps = [\"db\", \"auth\"]";
}

fn paths_and_identifiers() {
    // Should not trigger DE0703 - file paths and identifiers
    let path = "/var/log/application.log";
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    let hex = "0x1234567890abcdef";
}

fn main() {}
