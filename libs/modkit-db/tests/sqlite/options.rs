use modkit_db::{DbConnConfig, PoolCfg, build_db};
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_build_db_handle_env_expansion() {
    temp_env::with_var("TEST_SQLITE_SYNC", Some("NORMAL"), || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let config = DbConnConfig {
                engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
                dsn: Some("sqlite::memory:".to_owned()),
                params: Some({
                    let mut params = HashMap::new();
                    // Exercise env expansion in params
                    params.insert("synchronous".to_owned(), "${TEST_SQLITE_SYNC}".to_owned());
                    params
                }),
                ..Default::default()
            };

            let result = build_db(config, None).await;
            assert!(result.is_ok(), "Expected Ok, got: {result:?}");
            let db = result.unwrap();
            assert!(db.conn().is_ok(), "conn() should succeed");
        });
    });
}

#[tokio::test]
async fn test_build_db_handle_sqlite_memory() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("journal_mode".to_owned(), "WAL".to_owned());
            params
        }),
        ..Default::default()
    };

    let result = build_db(config, None).await;
    assert!(result.is_ok());

    let db = result.unwrap();
    assert!(db.conn().is_ok(), "conn() should succeed");
}

#[tokio::test]
async fn test_build_db_handle_sqlite_file() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        path: Some(db_path),
        params: Some({
            let mut params = HashMap::new();
            params.insert("journal_mode".to_owned(), "DELETE".to_owned());
            params.insert("synchronous".to_owned(), "NORMAL".to_owned());
            params
        }),
        ..Default::default()
    };

    let result = build_db(config, None).await;
    assert!(result.is_ok());

    let db = result.unwrap();
    assert!(db.conn().is_ok(), "conn() should succeed");
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_build_db_handle_invalid_env_var() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        password: Some("${NONEXISTENT_VAR}".to_owned()),
        ..Default::default()
    };

    let result = build_db(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("environment variable not found"));
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_build_db_handle_invalid_sqlite_pragma() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("invalid_pragma".to_owned(), "some_value".to_owned());
            params
        }),
        ..Default::default()
    };

    let result = build_db(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("invalid_pragma"));
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_build_db_handle_invalid_journal_mode() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        params: Some({
            let mut params = HashMap::new();
            params.insert("journal_mode".to_owned(), "INVALID_MODE".to_owned());
            params
        }),
        ..Default::default()
    };

    let result = build_db(config, None).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.to_string().contains("journal_mode"));
    assert!(
        error
            .to_string()
            .contains("must be DELETE/WAL/MEMORY/TRUNCATE/PERSIST/OFF")
    );
}

#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_build_db_handle_pool_config() {
    let config = DbConnConfig {
        engine: Some(modkit_db::config::DbEngineCfg::Sqlite),
        dsn: Some("sqlite::memory:".to_owned()),
        pool: Some(PoolCfg {
            max_conns: Some(5),
            acquire_timeout: Some(Duration::from_secs(10)),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = build_db(config, None).await;
    assert!(result.is_ok());

    let db = result.unwrap();
    assert!(db.conn().is_ok(), "conn() should succeed");
}
