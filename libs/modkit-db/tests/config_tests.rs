#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for configuration types and serialization.

use modkit_db::{DbConnConfig, GlobalDatabaseConfig, PoolCfg};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn test_dbconnconfig_serialization() {
    let config = DbConnConfig {
        engine: None,
        dsn: Some("postgresql://user:pass@localhost/db".to_owned()),
        host: Some("localhost".to_owned()),
        port: Some(5432),
        user: Some("testuser".to_owned()),
        password: Some("testpass".to_owned()),
        dbname: Some("testdb".to_owned()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("ssl".to_owned(), "require".to_owned());
            params
        }),
        file: Some("test.db".to_owned()),
        path: Some(PathBuf::from("/tmp/test.db")),
        pool: Some(PoolCfg {
            max_conns: Some(10),
            acquire_timeout: Some(Duration::from_secs(30)),
            ..Default::default()
        }),
        server: Some("test_server".to_owned()),
    };

    // Test serialization to JSON
    let json = serde_json::to_string_pretty(&config).expect("Failed to serialize to JSON");
    assert!(json.contains("postgresql://user:pass@localhost/db"));
    assert!(json.contains("test_server"));

    // Test deserialization from JSON
    let deserialized: DbConnConfig =
        serde_json::from_str(&json).expect("Failed to deserialize from JSON");
    assert_eq!(deserialized.dsn, config.dsn);
    assert_eq!(deserialized.server, config.server);
    assert_eq!(deserialized.host, config.host);
    assert_eq!(deserialized.port, config.port);
}

#[test]
fn test_dbconnconfig_defaults() {
    let config = DbConnConfig::default();
    assert!(config.engine.is_none());
    assert!(config.dsn.is_none());
    assert!(config.host.is_none());
    assert!(config.port.is_none());
    assert!(config.user.is_none());
    assert!(config.password.is_none());
    assert!(config.dbname.is_none());
    assert!(config.params.is_none());
    assert!(config.file.is_none());
    assert!(config.path.is_none());
    assert!(config.pool.is_none());
    assert!(config.server.is_none());
}

#[test]
fn test_globaldatabaseconfig_serialization() {
    let mut servers = HashMap::new();
    servers.insert(
        "postgres_main".to_owned(),
        DbConnConfig {
            host: Some("db.example.com".to_owned()),
            port: Some(5432),
            user: Some("appuser".to_owned()),
            password: Some("${DB_PASSWORD}".to_owned()),
            dbname: Some("maindb".to_owned()),
            params: Some({
                let mut params = HashMap::new();
                params.insert("sslmode".to_owned(), "require".to_owned());
                params
            }),
            pool: Some(PoolCfg {
                max_conns: Some(20),
                acquire_timeout: Some(Duration::from_secs(10)),
                ..Default::default()
            }),
            ..Default::default()
        },
    );

    let global_config = GlobalDatabaseConfig {
        servers,
        auto_provision: Some(true),
    };

    // Test serialization to YAML (more readable for config files)
    let yaml = serde_saphyr::to_string(&global_config).expect("Failed to serialize to YAML");
    assert!(yaml.contains("postgres_main"));
    assert!(yaml.contains("db.example.com"));
    assert!(yaml.contains("${DB_PASSWORD}"));

    // Test deserialization from YAML
    let deserialized: GlobalDatabaseConfig =
        serde_saphyr::from_str(&yaml).expect("Failed to deserialize from YAML");
    assert_eq!(deserialized.auto_provision, Some(true));
    assert!(deserialized.servers.contains_key("postgres_main"));

    let server_config = &deserialized.servers["postgres_main"];
    assert_eq!(server_config.host, Some("db.example.com".to_owned()));
    assert_eq!(server_config.port, Some(5432));
}

#[test]
fn test_poolcfg_defaults() {
    let pool = PoolCfg::default();
    assert!(pool.max_conns.is_none());
    assert!(pool.acquire_timeout.is_none());
}

#[test]
fn test_poolcfg_with_humantime() {
    // Test that humantime_serde works correctly
    let json = r#"{
        "max_conns": 15,
        "acquire_timeout": "45s"
    }"#;

    let pool: PoolCfg = serde_json::from_str(json).expect("Failed to deserialize PoolCfg");
    assert_eq!(pool.max_conns, Some(15));
    assert_eq!(pool.acquire_timeout, Some(Duration::from_secs(45)));

    // Test serialization back
    let serialized = serde_json::to_string(&pool).expect("Failed to serialize PoolCfg");
    let deserialized: PoolCfg =
        serde_json::from_str(&serialized).expect("Failed to deserialize again");
    assert_eq!(deserialized.max_conns, Some(15));
    assert_eq!(deserialized.acquire_timeout, Some(Duration::from_secs(45)));
}

#[test]
fn test_deny_unknown_fields() {
    // Test that serde(deny_unknown_fields) works for DbConnConfig
    let json_with_unknown = r#"{
        "dsn": "sqlite::memory:",
        "unknown_field": "should_fail"
    }"#;

    let result: Result<DbConnConfig, _> = serde_json::from_str(json_with_unknown);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("unknown field"));

    // Test that it works for GlobalDatabaseConfig too
    let global_json_with_unknown = r#"{
        "servers": {},
        "auto_provision": true,
        "invalid_field": "should_fail"
    }"#;

    let result: Result<GlobalDatabaseConfig, _> = serde_json::from_str(global_json_with_unknown);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn test_minimal_configs() {
    // Test minimal SQLite config
    let sqlite_config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        file: Some("data.db".to_owned()),
        ..Default::default()
    };
    let json = serde_json::to_string(&sqlite_config).unwrap();
    let deserialized: DbConnConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.file, Some("data.db".to_owned()));

    // Test minimal server reference config
    let server_ref_config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Postgres),
        server: Some("main_db".to_owned()),
        dbname: Some("myapp".to_owned()),
        ..Default::default()
    };
    let json = serde_json::to_string(&server_ref_config).unwrap();
    let deserialized: DbConnConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.server, Some("main_db".to_owned()));
    assert_eq!(deserialized.dbname, Some("myapp".to_owned()));
}
