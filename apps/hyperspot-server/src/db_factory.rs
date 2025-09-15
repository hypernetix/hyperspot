//! Per-module database factory wiring.
//!
//! This module bridges `runtime::AppConfig` + module name → `db::DbHandle`.
//! It lives in the server crate on purpose, so core libraries (`modkit`, `runtime`)
//! stay free from sqlx/db dependencies and app-specific wiring.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use db::{ConnectOpts, DbHandle};
use modkit::runtime::PerModuleDbFactory;
use runtime::AppConfig;

/// Build a per-module DB factory backed by the new config model.
///
/// For each module name, we compute the *final DSN* and *pool settings* using
/// `runtime::config::build_final_db_for_module`, then open a `DbHandle`.
/// If a module has no `database:` section, we return `Ok(None)` (no DB capability).
pub fn create_per_module_db_factory(
    config: Arc<AppConfig>,
    home_dir: PathBuf,
) -> PerModuleDbFactory {
    Box::new(move |module_name: &str| {
        let config = config.clone();
        let home_dir = home_dir.clone();
        let module_name = module_name.to_string();

        Box::pin(async move {
            match runtime::config::build_final_db_for_module(&config, &module_name, &home_dir)? {
                Some((final_dsn, pool_cfg)) => {
                    let connect_opts = ConnectOpts {
                        max_conns: pool_cfg.max_conns,
                        acquire_timeout: pool_cfg.acquire_timeout,
                        // For SQLite file DSNs we want dirs to exist; no-op for server DBs.
                        create_sqlite_dirs: true,
                        ..Default::default()
                    };

                    let redacted = redact_dsn_password(&final_dsn);
                    tracing::info!(
                        module = %module_name,
                        dsn = %redacted,
                        "Connecting module database"
                    );

                    // For SQLite file databases, ensure the file exists before connecting
                    if final_dsn.starts_with("sqlite:") && !final_dsn.contains("memory:") {
                        if let Some(file_path_with_query) = final_dsn.strip_prefix("sqlite:") {
                            // Remove query parameters to get the actual file path
                            let file_path =
                                if let Some(query_start) = file_path_with_query.find('?') {
                                    &file_path_with_query[..query_start]
                                } else {
                                    file_path_with_query
                                };

                            let path = std::path::Path::new(file_path);

                            // Create parent directories if they don't exist
                            if let Some(parent_dir) = path.parent() {
                                if !parent_dir.exists() {
                                    tracing::debug!(
                                        "Creating SQLite database directory: {:?}",
                                        parent_dir
                                    );
                                    std::fs::create_dir_all(parent_dir).map_err(|e| {
                                        anyhow::anyhow!(
                                            "Failed to create directory {:?}: {}",
                                            parent_dir,
                                            e
                                        )
                                    })?;
                                }
                            }

                            // Create empty database file if it doesn't exist
                            if !path.exists() {
                                tracing::debug!("Creating SQLite database file: {:?}", path);
                                std::fs::File::create(path).map_err(|e| {
                                    anyhow::anyhow!(
                                        "Failed to create SQLite file {:?}: {}",
                                        path,
                                        e
                                    )
                                })?;
                            }
                        }
                    }

                    let db_handle =
                        DbHandle::connect(&final_dsn, connect_opts)
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "Failed to connect DB for module '{}': {}",
                                    module_name,
                                    e
                                )
                            })?;

                    Ok(Some(Arc::new(db_handle)))
                }
                None => {
                    tracing::debug!(
                        module = %module_name,
                        "Module has no database configuration; skipping"
                    );
                    Ok(None)
                }
            }
        })
    })
}

/// Create a per-module factory that always returns an in-memory SQLite handle.
///
/// Useful for `--mock` or tests where you still want the DB capability wired,
/// but without touching the filesystem or external servers.
pub fn create_mock_per_module_db_factory() -> PerModuleDbFactory {
    Box::new(move |_module_name: &str| {
        Box::pin(async move {
            let connect_opts = ConnectOpts {
                max_conns: Some(10),
                acquire_timeout: Some(Duration::from_secs(5)),
                create_sqlite_dirs: false,
                ..Default::default()
            };

            tracing::info!("Connecting to mock database: sqlite::memory:");
            let db = DbHandle::connect("sqlite::memory:", connect_opts).await?;
            Ok(Some(Arc::new(db)))
        })
    })
}

/// Redact password component from DSN for safe logging.
/// If parsing fails, returns the original string.
fn redact_dsn_password(dsn: &str) -> String {
    if let Ok(mut parsed) = url::Url::parse(dsn) {
        if parsed.password().is_some() {
            let _ = parsed.set_password(Some("***"));
            return parsed.to_string();
        }
    }
    dsn.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;
    use tempfile::tempdir;
    use tokio::time::{timeout, Duration as TokioDuration};

    // Helper: minimal AppConfig with temp home_dir
    fn make_base_appconfig(home: &Path) -> AppConfig {
        let mut cfg = AppConfig::default();
        cfg.server.home_dir = home.to_string_lossy().to_string();
        // Ensure we have a global database section present (even if empty),
        // because main codepath checks `config.database.is_some()`
        if cfg.database.is_none() {
            cfg.database = Some(runtime::config::GlobalDatabaseConfig {
                servers: std::collections::HashMap::new(),
                auto_provision: None,
            });
        }
        cfg
    }

    #[test]
    fn redact_dsn_password_masks_secret() {
        let dsn = "postgresql://user:pass@localhost:5432/dbname?sslmode=require";
        let redacted = super::redact_dsn_password(dsn);
        assert!(redacted.contains("user:***@localhost"));
        assert!(!redacted.contains("user:pass@localhost"));
    }

    #[test]
    fn redact_dsn_password_no_password_kept() {
        let dsn = "postgresql://user@localhost:5432/dbname";
        let redacted = super::redact_dsn_password(dsn);
        assert_eq!(redacted, dsn);
    }

    #[tokio::test]
    async fn mock_factory_returns_handle() -> anyhow::Result<()> {
        let factory = super::create_mock_per_module_db_factory();

        // Guard against hanging in CI
        let fut = factory("any-module");
        let res = timeout(TokioDuration::from_secs(5), fut).await??;

        assert!(res.is_some(), "mock factory must return Some(DbHandle)");
        Ok(())
    }

    #[tokio::test]
    async fn per_module_factory_sqlite_file_minimal() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let home_dir = tmp.path().to_path_buf();

        // Build minimal AppConfig and add a module with SQLite "file" config
        let mut app = make_base_appconfig(&home_dir);
        app.modules.insert(
            "users_info".to_string(),
            json!({
                "database": {
                    "file": "test.db"
                },
                "config": { }
            }),
        );

        let factory = super::create_per_module_db_factory(Arc::new(app), home_dir.clone());

        // Guard with timeout to prevent hangs in CI
        let fut = factory("users_info");
        let opt = timeout(TokioDuration::from_secs(10), fut).await??;

        let handle = opt.expect("factory should return Some(DbHandle) for sqlite file");
        drop(handle);
        Ok(())
    }

    #[test]
    fn test_sqlite_path_extraction_with_query_params() -> anyhow::Result<()> {
        // Test the SQLite path extraction logic with query parameters
        let dsn = "sqlite:C:/Users/Mike/.hyperspot/users_info/users_info.db?WAL=true&synchronous=NORMAL&busy_timeout=5000";

        if let Some(file_path_with_query) = dsn.strip_prefix("sqlite:") {
            // Remove query parameters to get the actual file path
            let file_path = if let Some(query_start) = file_path_with_query.find('?') {
                &file_path_with_query[..query_start]
            } else {
                file_path_with_query
            };

            let path = std::path::Path::new(file_path);

            // Verify the path is correct
            assert_eq!(
                file_path,
                "C:/Users/Mike/.hyperspot/users_info/users_info.db"
            );
            assert_eq!(path.file_name().unwrap(), "users_info.db");
            assert_eq!(
                path.parent().unwrap().to_string_lossy(),
                "C:/Users/Mike/.hyperspot/users_info"
            );
        } else {
            panic!("Failed to extract path from DSN");
        }

        Ok(())
    }

    #[tokio::test]
    async fn per_module_factory_no_db_config_returns_none() -> anyhow::Result<()> {
        let tmp = tempdir()?;
        let home_dir = tmp.path().to_path_buf();

        let mut app = make_base_appconfig(&home_dir);
        // Module present but WITHOUT "database" section → should be treated as no DB capability
        app.modules.insert(
            "api_ingress".to_string(),
            json!({
                "config": {
                    "bind_addr": "127.0.0.1:8087",
                    "enable_docs": true,
                    "cors_enabled": true
                }
            }),
        );

        let factory = super::create_per_module_db_factory(Arc::new(app), home_dir.clone());
        let fut = factory("api_ingress");
        let res = timeout(TokioDuration::from_secs(5), fut).await??;

        assert!(res.is_none(), "expected None for module without DB section");
        Ok(())
    }

    #[tokio::test]
    #[cfg(windows)]
    #[ignore = "enable locally to verify Windows absolute path handling"]
    async fn per_module_factory_sqlite_file_windows_absolute() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let mut abs = tmp.path().join("users_info");
        std::fs::create_dir_all(&abs)?;
        abs.push("test.db");

        let mut app = make_base_appconfig(tmp.path());
        app.modules.insert(
            "users_info".to_string(),
            json!({
                "database": {
                    "path": abs.to_string_lossy()
                },
                "config": {}
            }),
        );

        let factory = super::create_per_module_db_factory(Arc::new(app), tmp.path().to_path_buf());
        let fut = factory("users_info");
        let opt = tokio::time::timeout(std::time::Duration::from_secs(10), fut).await??;
        assert!(opt.is_some());
        Ok(())
    }
}
