#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for options module functionality.

#[cfg(feature = "pg")]
#[tokio::test]
async fn test_build_db_handle_postgres_missing_dbname() {
    use modkit_db::{DbConnConfig, build_db_handle};
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Postgres),
        server: Some("postgres".to_owned()),
        host: Some("localhost".to_owned()),
        port: Some(5432),
        user: Some("testuser".to_owned()),
        password: Some("testpass".to_owned()),
        // Missing dbname
        ..Default::default()
    };

    let result = build_db_handle(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    println!("Actual error: {error}");
    assert!(
        error
            .to_string()
            .contains("dbname is required for PostgreSQL connections")
    );
}

#[tokio::test]
async fn test_credential_redaction() {
    // This test ensures that sensitive information is not logged
    // We can't easily test the actual logging output, but we can test the function
    use modkit_db::options::redact_credentials_in_dsn;

    let dsn_with_password = Some("postgresql://user:secret@localhost/db");
    let redacted = redact_credentials_in_dsn(dsn_with_password);
    assert!(!redacted.contains("secret"));
    assert!(redacted.contains("***"));

    let dsn_without_password = Some("sqlite::memory:");
    let not_redacted = redact_credentials_in_dsn(dsn_without_password);
    assert_eq!(not_redacted, "sqlite::memory:");

    let no_dsn = redact_credentials_in_dsn(None);
    assert_eq!(no_dsn, "none");
}

#[cfg(feature = "pg")]
#[test]
fn test_display_postgres() {
    use modkit_db::DbConnectOptions;

    let opts = sea_orm::sqlx::postgres::PgConnectOptions::new()
        .host("localhost")
        .port(5432)
        .database("testdb")
        .username("user")
        .password("secret");

    let db_opts = DbConnectOptions::Postgres(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "postgresql://<redacted>@localhost:5432/testdb");
    assert!(!display_str.contains("secret"));
    assert!(!display_str.contains("user"));
}

#[cfg(feature = "pg")]
#[test]
fn test_display_postgres_custom_port() {
    use modkit_db::DbConnectOptions;

    let opts = sea_orm::sqlx::postgres::PgConnectOptions::new()
        .host("db.example.com")
        .port(15432)
        .database("myapp");

    let db_opts = DbConnectOptions::Postgres(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(
        display_str,
        "postgresql://<redacted>@db.example.com:15432/myapp"
    );
}

#[cfg(feature = "pg")]
#[test]
fn test_display_postgres_no_database() {
    use modkit_db::DbConnectOptions;

    let opts = sea_orm::sqlx::postgres::PgConnectOptions::new()
        .host("localhost")
        .port(5432);

    let db_opts = DbConnectOptions::Postgres(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "postgresql://<redacted>@localhost:5432/");
}

#[cfg(feature = "mysql")]
#[test]
fn test_display_mysql() {
    use modkit_db::DbConnectOptions;

    let opts = sea_orm::sqlx::mysql::MySqlConnectOptions::new()
        .host("localhost")
        .port(3306)
        .database("testdb")
        .username("user")
        .password("secret");

    let db_opts = DbConnectOptions::MySql(opts);

    let display_str = format!("{db_opts}");
    assert_eq!(display_str, "mysql://<redacted>@...");
    assert!(!display_str.contains("secret"));
    assert!(!display_str.contains("user"));
    assert!(!display_str.contains("localhost"));
    assert!(!display_str.contains("testdb"));
}
