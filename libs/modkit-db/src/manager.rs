//! Database manager for per-module database connections.
//!
//! The `DbManager` is responsible for:
//! - Loading global database configuration from Figment
//! - Building and caching database handles per module
//! - Merging global server configurations with module-specific settings

use crate::config::{DbConnConfig, GlobalDatabaseConfig};
use crate::options::build_db_handle;
use crate::{DbError, DbHandle, Result};
use dashmap::DashMap;
use figment::Figment;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Central database manager that handles per-module database connections.
pub struct DbManager {
    /// Global database configuration loaded from Figment
    global: Option<GlobalDatabaseConfig>,
    /// Figment instance for reading module configurations
    figment: Figment,
    /// Base home directory for modules
    home_dir: PathBuf,
    /// Cache of database handles per module
    cache: DashMap<String, Arc<DbHandle>>,
}

impl DbManager {
    /// Create a new `DbManager` from a Figment configuration.
    ///
    /// # Errors
    /// Returns an error if the configuration cannot be parsed.
    pub fn from_figment(figment: Figment, home_dir: PathBuf) -> Result<Self> {
        // Parse global database configuration from "db.*" section
        let all_data: serde_json::Value = figment
            .extract()
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

        let global: Option<GlobalDatabaseConfig> = all_data
            .get("database")
            .and_then(|db| serde_json::from_value(db.clone()).ok());

        Ok(Self {
            global,
            figment,
            home_dir,
            cache: DashMap::new(),
        })
    }

    /// Get a database handle for the specified module.
    /// Returns cached handle if available, otherwise builds a new one.
    ///
    /// # Errors
    /// Returns an error if the database connection cannot be established.
    pub async fn get(&self, module: &str) -> Result<Option<Arc<DbHandle>>> {
        // Check cache first
        if let Some(handle) = self.cache.get(module) {
            return Ok(Some(handle.clone()));
        }

        // Build new handle
        if let Some(handle) = self.build_for_module(module).await? {
            // Use entry API to handle race conditions properly
            match self.cache.entry(module.to_string()) {
                dashmap::mapref::entry::Entry::Occupied(entry) => {
                    // Another thread beat us to it, return the cached version
                    Ok(Some(entry.get().clone()))
                }
                dashmap::mapref::entry::Entry::Vacant(entry) => {
                    // We're first, insert our handle
                    entry.insert(handle.clone());
                    Ok(Some(handle))
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Build a database handle for the specified module.
    async fn build_for_module(&self, module: &str) -> Result<Option<Arc<DbHandle>>> {
        // Read module database configuration from Figment
        let module_data: serde_json::Value = self
            .figment
            .extract()
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

        let module_cfg: Option<DbConnConfig> = module_data
            .get("modules")
            .and_then(|modules| modules.get(module))
            .and_then(|m| m.get("database"))
            .and_then(|db| serde_json::from_value(db.clone()).ok());

        let Some(mut cfg) = module_cfg else {
            tracing::debug!(
                module = %module,
                "Module has no database configuration; skipping"
            );
            return Ok(None);
        };

        // If module references a global server, merge configurations
        if let Some(server_name) = &cfg.server {
            let server_cfg = self
                .global
                .as_ref()
                .and_then(|g| g.servers.get(server_name))
                .ok_or_else(|| {
                    DbError::InvalidConfig(format!(
                        "Referenced server '{server_name}' not found in global database configuration"
                    ))
                })?;

            cfg = Self::merge_server_into_module(cfg, server_cfg.clone());
        }

        // Finalize SQLite paths if needed
        let module_home_dir = self.home_dir.join(module);
        cfg = self.finalize_sqlite_paths(cfg, &module_home_dir)?;

        // Build the database handle
        let handle = build_db_handle(cfg, self.global.as_ref()).await?;

        tracing::info!(
            module = %module,
            engine = ?handle.engine(),
            dsn = %crate::options::redact_credentials_in_dsn(Some(handle.dsn())),
            "Built database handle for module"
        );

        Ok(Some(Arc::new(handle)))
    }

    /// Merge global server configuration into module configuration.
    /// Module fields override server fields. Params maps are merged with module taking precedence.
    fn merge_server_into_module(
        mut module_cfg: DbConnConfig,
        server_cfg: DbConnConfig,
    ) -> DbConnConfig {
        // Start with server config as base, then apply module overrides

        // DSN: module takes precedence
        if module_cfg.dsn.is_none() {
            module_cfg.dsn = server_cfg.dsn;
        }

        // Individual fields: module takes precedence
        if module_cfg.host.is_none() {
            module_cfg.host = server_cfg.host;
        }
        if module_cfg.port.is_none() {
            module_cfg.port = server_cfg.port;
        }
        if module_cfg.user.is_none() {
            module_cfg.user = server_cfg.user;
        }
        if module_cfg.password.is_none() {
            module_cfg.password = server_cfg.password;
        }
        if module_cfg.dbname.is_none() {
            module_cfg.dbname = server_cfg.dbname;
        }

        // Params: merge maps with module taking precedence
        match (&mut module_cfg.params, server_cfg.params) {
            (Some(module_params), Some(server_params)) => {
                // Merge server params first, then module params (module overrides)
                for (key, value) in server_params {
                    module_params.entry(key).or_insert(value);
                }
            }
            (None, Some(server_params)) => {
                module_cfg.params = Some(server_params);
            }
            _ => {} // Module has params or server has none - keep module params
        }

        // Pool: module takes precedence
        if module_cfg.pool.is_none() {
            module_cfg.pool = server_cfg.pool;
        }

        // Note: file, path, and server fields are module-only and not merged

        module_cfg
    }

    /// Finalize `SQLite` paths by resolving relative file paths to absolute paths.
    fn finalize_sqlite_paths(
        &self,
        mut cfg: DbConnConfig,
        module_home: &Path,
    ) -> Result<DbConnConfig> {
        // If file is specified, convert to absolute path under module home
        if let Some(file) = &cfg.file {
            let absolute_path = module_home.join(file);

            // Check auto_provision setting
            let auto_provision = self
                .global
                .as_ref()
                .and_then(|g| g.auto_provision)
                .unwrap_or(true);

            if auto_provision {
                // Create all necessary directories
                if let Some(parent) = absolute_path.parent() {
                    std::fs::create_dir_all(parent).map_err(DbError::Io)?;
                }
            } else if let Some(parent) = absolute_path.parent() {
                // When auto_provision is false, check if the directory exists
                if !parent.exists() {
                    return Err(DbError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!(
                            "Directory does not exist and auto_provision is disabled: {}",
                            parent.display()
                        ),
                    )));
                }
            }

            cfg.path = Some(absolute_path);
            cfg.file = None; // Clear file since path takes precedence and we can't have both
        }

        // If path is relative, make it absolute relative to module home
        if let Some(path) = &cfg.path {
            if path.is_relative() {
                cfg.path = Some(module_home.join(path));
            }
        }

        Ok(cfg)
    }
}
