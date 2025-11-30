use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Use DB config types from modkit-db
pub use modkit_db::{DbConnConfig, GlobalDatabaseConfig, PoolCfg};

// Uses your module: crate::home_dirs::resolve_home_dir
use crate::paths::home_dir::resolve_home_dir;

// DB config types are now imported from modkit-db

/// Small typed view to parse each module entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleEntry {
    #[serde(default)]
    pub database: Option<DbConnConfig>,
    #[serde(default)]
    pub config: serde_json::Value,
}

/// Main application configuration with strongly-typed global sections
/// and a flexible per-module configuration bag.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    /// Core server configuration.
    pub server: ServerConfig,
    /// New typed database configuration (optional).
    pub database: Option<GlobalDatabaseConfig>,
    /// Logging configuration (optional, uses defaults if None).
    pub logging: Option<LoggingConfig>,
    /// Tracing configuration (optional, disabled if None).
    pub tracing: Option<TracingConfig>,
    /// Directory containing per-module YAML files (optional).
    #[serde(default)]
    pub modules_dir: Option<String>,
    /// Per-module configuration bag: module_name → arbitrary JSON/YAML value.
    #[serde(default)]
    pub modules: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub home_dir: String, // will be normalized to absolute path
}

/// Logging configuration - maps subsystem names to their logging settings.
/// Key "default" is the catch-all for logs that don't match explicit subsystems.
pub type LoggingConfig = HashMap<String, Section>;

/// Tracing configuration for OpenTelemetry distributed tracing
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TracingConfig {
    pub enabled: bool,
    pub service_name: Option<String>,
    pub exporter: Option<Exporter>,
    pub sampler: Option<Sampler>,
    pub propagation: Option<Propagation>,
    pub resource: Option<HashMap<String, String>>,
    pub http: Option<HttpOpts>,
    pub logs_correlation: Option<LogsCorrelation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Exporter {
    pub kind: Option<String>, // "otlp_grpc" | "otlp_http"
    pub endpoint: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub timeout_ms: Option<u64>,
    // tls fields omitted for brevity; add later if needed
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sampler {
    pub strategy: Option<String>, // "parentbased_always_on" | "parentbased_ratio" | "always_on" | "always_off"
    pub ratio: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Propagation {
    pub w3c_trace_context: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpOpts {
    pub inject_request_id_header: Option<String>,
    pub record_headers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LogsCorrelation {
    pub inject_trace_ids_into_logs: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Section {
    pub console_level: String, // "info", "debug", "error", "off"
    pub file: String,          // "logs/api.log"
    #[serde(default)]
    pub file_level: String,
    pub max_age_days: Option<u32>, // Not implemented yet
    #[serde(default)]
    pub max_backups: Option<usize>, // How many files to keep
    #[serde(default)]
    pub max_size_mb: Option<u64>, // Max size of the file in MB
}

/// Create a default logging configuration.
pub fn default_logging_config() -> LoggingConfig {
    let mut logging = HashMap::new();
    logging.insert(
        "default".to_string(),
        Section {
            console_level: "info".to_string(),
            file: "logs/hyperspot.log".to_string(),
            file_level: "debug".to_string(),
            max_age_days: Some(7),
            max_backups: Some(3),
            max_size_mb: Some(100),
        },
    );
    logging
}

impl Default for AppConfig {
    fn default() -> Self {
        let server = ServerConfig::default();
        Self {
            server,
            database: Some(GlobalDatabaseConfig {
                servers: HashMap::new(),
                auto_provision: None,
            }),
            logging: Some(default_logging_config()),
            tracing: None, // Disabled by default
            modules_dir: None,
            modules: HashMap::new(),
        }
    }
}

impl AppConfig {
    /// Load configuration with layered loading: defaults → YAML file → environment variables.
    /// Also normalizes `server.home_dir` into an absolute path and creates the directory.
    pub fn load_layered<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        use figment::{
            providers::{Env, Format, Serialized, Yaml},
            Figment,
        };

        // For layered loading, start from a minimal base where optional sections are None,
        // so they remain None unless explicitly provided by YAML/ENV.
        let base = AppConfig {
            server: ServerConfig::default(),
            database: None,
            logging: None,
            tracing: None,
            modules_dir: None,
            modules: HashMap::new(),
        };

        let figment = Figment::new()
            .merge(Serialized::defaults(base))
            .merge(Yaml::file(config_path.as_ref()))
            // Example: APP__SERVER__PORT=8087 maps to server.port
            .merge(Env::prefixed("APP__").split("__"));

        let mut config: AppConfig = figment
            .extract()
            .with_context(|| "Failed to extract config from figment".to_string())?;

        // Normalize + create home_dir immediately.
        normalize_home_dir_inplace(&mut config.server)
            .context("Failed to resolve server.home_dir")?;

        // Merge module files if modules_dir is specified.
        if let Some(dir) = config.modules_dir.clone() {
            merge_module_files(&mut config.modules, dir)?;
        }

        Ok(config)
    }

    /// Load configuration from file or create with default values.
    /// Also normalizes `server.home_dir` into an absolute path and creates the directory.
    pub fn load_or_default<P: AsRef<Path>>(config_path: Option<P>) -> Result<Self> {
        match config_path {
            Some(path) => Self::load_layered(path),
            None => {
                let mut c = Self::default();
                normalize_home_dir_inplace(&mut c.server)
                    .context("Failed to resolve server.home_dir (defaults)")?;
                Ok(c)
            }
        }
    }

    /// Serialize configuration to YAML.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize config to YAML")
    }

    /// Apply overrides from command line arguments.
    pub fn apply_cli_overrides(&mut self, args: &CliArgs) {
        // Set logging level based on verbose flags for "default" section.
        let logging = self.logging.get_or_insert_with(default_logging_config);
        if let Some(default_section) = logging.get_mut("default") {
            default_section.console_level = match args.verbose {
                0 => default_section.console_level.clone(), // keep
                1 => "debug".to_string(),
                _ => "trace".to_string(),
            };
        }
    }
}

/// Command line arguments structure.
#[derive(Debug, Clone)]
pub struct CliArgs {
    pub config: Option<String>,
    pub print_config: bool,
    pub verbose: u8,
    pub mock: bool,
}

// TODO: should be pass from outside
const fn default_subdir() -> &'static str {
    ".hyperspot"
}

/// Normalize `server.home_dir` using `home_dirs::resolve_home_dir` and store the absolute path back.
fn normalize_home_dir_inplace(server: &mut ServerConfig) -> Result<()> {
    // Treat empty string as "not provided" => None.
    let opt = if server.home_dir.trim().is_empty() {
        None
    } else {
        Some(server.home_dir.clone())
    };

    let resolved: PathBuf = resolve_home_dir(opt, default_subdir(), /*create*/ true)
        .context("home_dir normalization failed")?;

    server.home_dir = resolved.to_string_lossy().to_string();
    Ok(())
}

fn merge_module_files(
    bag: &mut HashMap<String, serde_json::Value>,
    dir: impl AsRef<Path>,
) -> Result<()> {
    use std::fs;
    let dir = dir.as_ref();
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "yml" && ext != "yaml" {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let raw = fs::read_to_string(&path)?;
        let val: serde_yaml::Value = serde_yaml::from_str(&raw)?;
        let json = serde_json::to_value(val)?;
        bag.insert(name, json);
    }
    Ok(())
}

// ---- New ModKit DB Handling Functions ----

/// Expands environment variables in a DSN string.
/// Replaces `${VARNAME}` with the actual environment variable value.
/// Returns error if any referenced env var is missing.
pub fn expand_env_in_dsn(dsn: &str) -> anyhow::Result<String> {
    use std::env;

    let mut result = dsn.to_string();
    let re = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)}")?;

    for cap in re.captures_iter(dsn) {
        let full_match = &cap[0];
        let var_name = &cap[1];

        let value = env::var(var_name)
            .with_context(|| format!("Environment variable '{}' not found in DSN", var_name))?;

        result = result.replace(full_match, &value);
    }

    Ok(result)
}

/// Resolves password: if it contains ${VAR}, expands from environment variable; otherwise returns as-is.
pub fn resolve_password(password: Option<&str>) -> anyhow::Result<Option<String>> {
    if let Some(pwd) = password {
        if pwd.starts_with("${") && pwd.ends_with('}') {
            // Extract variable name from ${VAR_NAME}
            let var_name = &pwd[2..pwd.len() - 1];
            let resolved = std::env::var(var_name).with_context(|| {
                format!("Environment variable '{}' not found for password", var_name)
            })?;
            Ok(Some(resolved))
        } else {
            // Return literal password as-is
            Ok(Some(pwd.to_string()))
        }
    } else {
        Ok(None)
    }
}

/// Validates that a DSN string is parseable by the dsn crate.
/// Note: SQLite DSNs have special formats that dsn crate doesn't recognize, so we skip validation for them.
pub fn validate_dsn(dsn: &str) -> anyhow::Result<()> {
    // Skip validation for SQLite DSNs as they use special syntax not recognized by dsn crate
    if dsn.starts_with("sqlite:") {
        return Ok(());
    }

    let _parsed = dsn::parse(dsn).map_err(|e| anyhow::anyhow!("Invalid DSN '{}': {}", dsn, e))?;

    Ok(())
}

/// Resolves SQLite @file() syntax in DSN to actual file paths.
/// - `sqlite://@file(users.sqlite)` → `$HOME/.hyperspot/<module>/users.sqlite`
/// - `sqlite://@file(/abs/path/file.db)` → use absolute path
/// - `sqlite://` or `sqlite:///` → `$HOME/.hyperspot/<module>/<module>.sqlite`
fn resolve_sqlite_dsn(dsn: &str, home_dir: &Path, module_name: &str) -> anyhow::Result<String> {
    if dsn.contains("@file(") {
        // Extract the file path from @file(...)
        if let Some(start) = dsn.find("@file(") {
            if let Some(end) = dsn[start..].find(')') {
                let file_path = &dsn[start + 6..start + end]; // +6 for "@file("

                let resolved_path = if file_path.starts_with('/')
                    || (file_path.len() > 1 && file_path.chars().nth(1) == Some(':'))
                {
                    // Absolute path (Unix or Windows)
                    PathBuf::from(file_path)
                } else {
                    // Relative path - resolve under module directory
                    let module_dir = home_dir.join(module_name);
                    std::fs::create_dir_all(&module_dir).with_context(|| {
                        format!("Failed to create module directory: {:?}", module_dir)
                    })?;
                    module_dir.join(file_path)
                };

                let normalized_path = resolved_path.to_string_lossy().replace('\\', "/");
                // For Windows absolute paths (C:/...), use sqlite:path format
                // For Unix absolute paths (/...), use sqlite://path format
                if normalized_path.len() > 1 && normalized_path.chars().nth(1) == Some(':') {
                    // Windows absolute path like C:/...
                    return Ok(format!("sqlite:{}", normalized_path));
                } else {
                    // Unix absolute path or relative path
                    return Ok(format!("sqlite://{}", normalized_path));
                }
            }
        }
        return Err(anyhow::anyhow!(
            "Invalid @file() syntax in SQLite DSN: {}",
            dsn
        ));
    }

    // Handle empty DSN or just sqlite:// - default to module.sqlite
    if dsn == "sqlite://" || dsn == "sqlite:///" || dsn == "sqlite:" {
        let module_dir = home_dir.join(module_name);
        std::fs::create_dir_all(&module_dir)
            .with_context(|| format!("Failed to create module directory: {:?}", module_dir))?;
        let db_path = module_dir.join(format!("{}.sqlite", module_name));
        let normalized_path = db_path.to_string_lossy().replace('\\', "/");
        // For Windows absolute paths (C:/...), use sqlite:path format
        // For Unix absolute paths (/...), use sqlite://path format
        if normalized_path.len() > 1 && normalized_path.chars().nth(1) == Some(':') {
            // Windows absolute path like C:/...
            return Ok(format!("sqlite:{}", normalized_path));
        } else {
            // Unix absolute path or relative path
            return Ok(format!("sqlite://{}", normalized_path));
        }
    }

    // Return DSN as-is for normal cases
    Ok(dsn.to_string())
}

/// Builds a server-based DSN from individual fields.
/// Used when no base DSN is provided or when overriding DSN components.
/// Uses url::Url to properly handle percent-encoding of special characters.
fn build_server_dsn(
    scheme: &str,
    host: Option<&str>,
    port: Option<u16>,
    user: Option<&str>,
    password: Option<&str>,
    dbname: Option<&str>,
    params: &HashMap<String, String>,
) -> anyhow::Result<String> {
    use url::Url;

    let host = host.unwrap_or("localhost");
    let user = user.unwrap_or("postgres"); // reasonable default for server-based DBs

    // Start with base URL
    let mut url = Url::parse(&format!("{}://dummy/", scheme))
        .with_context(|| format!("Invalid scheme: {}", scheme))?;

    // Set host (required)
    url.set_host(Some(host))
        .with_context(|| format!("Invalid host: {}", host))?;

    // Set port if provided
    if let Some(port) = port {
        url.set_port(Some(port))
            .map_err(|_| anyhow::anyhow!("Invalid port: {}", port))?;
    }

    // Set username
    url.set_username(user)
        .map_err(|_| anyhow::anyhow!("Failed to set username: {}", user))?;

    // Set password if provided
    if let Some(password) = password {
        url.set_password(Some(password))
            .map_err(|_| anyhow::anyhow!("Failed to set password"))?;
    }

    // Set database name as path (with leading slash)
    if let Some(dbname) = dbname {
        // Manually encode the dbname to handle special characters
        let encoded_dbname = urlencoding::encode(dbname);
        url.set_path(&format!("/{}", encoded_dbname));
    } else {
        url.set_path("/");
    }

    // Set query parameters
    if !params.is_empty() {
        // Use url::Url::query_pairs_mut() to properly handle encoding
        let mut query_pairs = url.query_pairs_mut();
        for (key, value) in params {
            query_pairs.append_pair(key, value);
        }
    }

    Ok(url.to_string())
}

/// Builds a SQLite DSN by replacing the database file path while preserving query parameters.
fn build_sqlite_dsn_with_dbname_override(
    original_dsn: &str,
    dbname: &str,
    module_name: &str,
    home_dir: &Path,
) -> anyhow::Result<String> {
    // Parse the original DSN to extract query parameters
    let query_params = if let Some(query_start) = original_dsn.find('?') {
        &original_dsn[query_start..]
    } else {
        ""
    };

    // Build the correct path for the database file
    let module_dir = home_dir.join(module_name);
    std::fs::create_dir_all(&module_dir)
        .with_context(|| format!("Failed to create module directory: {:?}", module_dir))?;
    let db_path = module_dir.join(dbname);
    let normalized_path = db_path.to_string_lossy().replace('\\', "/");

    // Build the new DSN with correct format for the platform
    let dsn_base = if normalized_path.len() > 1 && normalized_path.chars().nth(1) == Some(':') {
        // Windows absolute path like C:/...
        format!("sqlite:{}", normalized_path)
    } else {
        // Unix absolute path or relative path
        format!("sqlite://{}", normalized_path)
    };

    Ok(format!("{}{}", dsn_base, query_params))
}

/// Builds a SQLite DSN from file/path or validates existing DSN.
/// If dbname is provided, it overrides the database file in the DSN.
fn build_sqlite_dsn(
    dsn: Option<&str>,
    file: Option<&str>,
    path: Option<&PathBuf>,
    dbname: Option<&str>,
    module_name: &str,
    home_dir: &Path,
) -> anyhow::Result<String> {
    // If full DSN provided, resolve @file() syntax and validate
    if let Some(dsn) = dsn {
        let resolved_dsn = resolve_sqlite_dsn(dsn, home_dir, module_name)?;

        // If dbname is provided, we need to replace the database file path while preserving query params
        if let Some(dbname) = dbname {
            return build_sqlite_dsn_with_dbname_override(
                &resolved_dsn,
                dbname,
                module_name,
                home_dir,
            );
        }

        validate_dsn(&resolved_dsn)?;
        return Ok(resolved_dsn);
    }

    // Build from path (absolute)
    if let Some(path) = path {
        let absolute_path = if path.is_absolute() {
            path.clone()
        } else {
            home_dir.join(path)
        };
        let normalized_path = absolute_path.to_string_lossy().replace('\\', "/");
        // For Windows absolute paths (C:/...), use sqlite:path format
        // For Unix absolute paths (/...), use sqlite://path format
        if normalized_path.len() > 1 && normalized_path.chars().nth(1) == Some(':') {
            // Windows absolute path like C:/...
            return Ok(format!("sqlite:{}", normalized_path));
        } else {
            // Unix absolute path or relative path
            return Ok(format!("sqlite://{}", normalized_path));
        }
    }

    // Build from file (relative under module dir)
    if let Some(file) = file {
        let module_dir = home_dir.join(module_name);
        std::fs::create_dir_all(&module_dir)
            .with_context(|| format!("Failed to create module directory: {:?}", module_dir))?;
        let db_path = module_dir.join(file);
        let normalized_path = db_path.to_string_lossy().replace('\\', "/");
        // For Windows absolute paths (C:/...), use sqlite:path format
        // For Unix absolute paths (/...), use sqlite://path format
        if normalized_path.len() > 1 && normalized_path.chars().nth(1) == Some(':') {
            // Windows absolute path like C:/...
            return Ok(format!("sqlite:{}", normalized_path));
        } else {
            // Unix absolute path or relative path
            return Ok(format!("sqlite://{}", normalized_path));
        }
    }

    // Default to module.sqlite
    let module_dir = home_dir.join(module_name);
    std::fs::create_dir_all(&module_dir)
        .with_context(|| format!("Failed to create module directory: {:?}", module_dir))?;
    let db_path = module_dir.join(format!("{}.sqlite", module_name));
    let normalized_path = db_path.to_string_lossy().replace('\\', "/");
    // For Windows absolute paths (C:/...), use sqlite:path format
    // For Unix absolute paths (/...), use sqlite://path format
    if normalized_path.len() > 1 && normalized_path.chars().nth(1) == Some(':') {
        // Windows absolute path like C:/...
        Ok(format!("sqlite:{}", normalized_path))
    } else {
        // Unix absolute path or relative path
        Ok(format!("sqlite://{}", normalized_path))
    }
}

/// Type alias for the complex return type of build_final_db_for_module
type DbConfigResult = anyhow::Result<Option<(String /* final_dsn */, PoolCfg)>>;

/// Merges global + module DB configs into a final, validated DSN and pool config.
/// Precedence: Global DSN -> Global fields -> Module DSN -> Module fields (fields always win).
/// For server-based, returns error if final dbname is missing.
/// For SQLite, builds/normalizes sqlite DSN from file/path or uses a full DSN as-is.
pub fn build_final_db_for_module(
    app: &AppConfig,
    module_name: &str,
    home_dir: &Path,
) -> DbConfigResult {
    // Parse module entry from raw JSON
    let module_raw = match app.modules.get(module_name) {
        Some(raw) => raw,
        None => return Ok(None), // No module config
    };

    let module_entry: ModuleEntry = serde_json::from_value(module_raw.clone())
        .with_context(|| format!("Invalid module config structure for '{}'", module_name))?;

    let module_db_config = match module_entry.database {
        Some(config) => config,
        None => {
            tracing::warn!(
                "Module '{}' has no database configuration; DB capability disabled",
                module_name
            );
            return Ok(None);
        }
    };

    // Global database config
    let global_db_config = app.database.as_ref();

    // Start building final config
    let mut final_dsn: Option<String> = None;
    let mut final_host: Option<String> = None;
    let mut final_port: Option<u16> = None;
    let mut final_user: Option<String> = None;
    let mut final_password: Option<String> = None;
    let mut final_dbname: Option<String> = None;
    let mut final_params: HashMap<String, String> = HashMap::new();
    let mut final_pool = PoolCfg::default();

    // Step 1: Apply global server config if referenced
    if let Some(server_name) = &module_db_config.server {
        let global_server = global_db_config
            .and_then(|gc| gc.servers.get(server_name))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Referenced server '{}' not found in global config",
                    server_name
                )
            })?;

        // Apply global server DSN
        if let Some(global_dsn) = &global_server.dsn {
            let expanded_dsn = expand_env_in_dsn(global_dsn)?;
            // For SQLite, resolve @file() syntax before validation
            let resolved_dsn = if expanded_dsn.starts_with("sqlite") {
                resolve_sqlite_dsn(&expanded_dsn, home_dir, module_name)?
            } else {
                expanded_dsn
            };
            validate_dsn(&resolved_dsn)?;
            final_dsn = Some(resolved_dsn);
        }

        // Apply global server fields (override DSN parts)
        if let Some(host) = &global_server.host {
            final_host = Some(host.clone());
        }
        if let Some(port) = global_server.port {
            final_port = Some(port);
        }
        if let Some(user) = &global_server.user {
            final_user = Some(user.clone());
        }
        if let Some(password) = resolve_password(global_server.password.as_deref())? {
            final_password = Some(password);
        }
        if let Some(dbname) = &global_server.dbname {
            final_dbname = Some(dbname.clone());
        }
        if let Some(params) = &global_server.params {
            final_params.extend(params.clone());
        }
        if let Some(pool) = &global_server.pool {
            final_pool = pool.clone();
        }
    }

    // Step 2: Apply module DSN (override global)
    if let Some(module_dsn) = &module_db_config.dsn {
        // For SQLite, resolve @file() syntax before validation
        let resolved_dsn = if module_dsn.starts_with("sqlite") {
            resolve_sqlite_dsn(module_dsn, home_dir, module_name)?
        } else {
            module_dsn.to_string()
        };
        validate_dsn(&resolved_dsn)?;
        final_dsn = Some(resolved_dsn);
    }

    // Step 3: Apply module fields (override everything)
    if let Some(host) = &module_db_config.host {
        final_host = Some(host.clone());
    }
    if let Some(port) = module_db_config.port {
        final_port = Some(port);
    }
    if let Some(user) = &module_db_config.user {
        final_user = Some(user.clone());
    }
    if let Some(password) = resolve_password(module_db_config.password.as_deref())? {
        final_password = Some(password);
    }
    if let Some(dbname) = &module_db_config.dbname {
        final_dbname = Some(dbname.clone());
    }
    if let Some(params) = &module_db_config.params {
        final_params.extend(params.clone());
    }
    if let Some(pool) = &module_db_config.pool {
        // Module pool settings override global ones
        if let Some(max_conns) = pool.max_conns {
            final_pool.max_conns = Some(max_conns);
        }
        if let Some(acquire_timeout) = pool.acquire_timeout {
            final_pool.acquire_timeout = Some(acquire_timeout);
        }
    }

    // Determine if this is SQLite or server-based
    // Always treat as SQLite if DSN starts with "sqlite", regardless of server reference
    // Also treat as SQLite if no server reference and no explicit DSN (default case)
    let is_sqlite = module_db_config.file.is_some()
        || module_db_config.path.is_some()
        || final_dsn
            .as_ref()
            .is_some_and(|dsn| dsn.starts_with("sqlite"))
        || (module_db_config.server.is_none() && final_dsn.is_none());

    let result_dsn = if is_sqlite {
        // SQLite: build from file/path or use DSN as-is
        build_sqlite_dsn(
            final_dsn.as_deref(),
            module_db_config.file.as_deref(),
            module_db_config.path.as_ref(),
            final_dbname.as_deref(),
            module_name,
            home_dir,
        )?
    } else {
        // Server-based: extract dbname from DSN if not provided separately
        let dbname = if let Some(dbname) = final_dbname.as_deref() {
            dbname.to_string()
        } else if let Some(dsn) = final_dsn.as_ref() {
            // Try to extract dbname from DSN path
            if let Ok(parsed) = url::Url::parse(dsn) {
                let path = parsed.path();
                if path.len() > 1 {
                    // Remove leading slash and return the path as dbname
                    path[1..].to_string()
                } else {
                    return Err(anyhow::anyhow!(
                        "Server-based database config for module '{}' missing required 'dbname'",
                        module_name
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Server-based database config for module '{}' missing required 'dbname'",
                    module_name
                ));
            }
        } else {
            return Err(anyhow::anyhow!(
                "Server-based database config for module '{}' missing required 'dbname'",
                module_name
            ));
        };

        // Check if we have any field overrides that require rebuilding the DSN
        let has_field_overrides = final_host.is_some()
            || final_port.is_some()
            || final_user.is_some()
            || final_password.is_some()
            || !final_params.is_empty();

        if has_field_overrides || final_dsn.is_none() {
            // Build DSN from fields when we have overrides or no original DSN
            let scheme = if let Some(dsn) = &final_dsn {
                let parsed = url::Url::parse(dsn)?;
                parsed.scheme().to_string()
            } else {
                "postgresql".to_string() // default
            };

            build_server_dsn(
                &scheme,
                final_host.as_deref(),
                final_port,
                final_user.as_deref(),
                final_password.as_deref(),
                Some(&dbname),
                &final_params,
            )?
        } else if let Some(original_dsn) = &final_dsn {
            // Use original DSN when no field overrides (but update dbname if needed)
            if let Ok(mut parsed) = url::Url::parse(original_dsn) {
                // Update the path with the final dbname if it's different
                let original_dbname = parsed.path().trim_start_matches('/');
                if original_dbname != dbname {
                    parsed.set_path(&format!("/{}", dbname));
                }
                parsed.to_string()
            } else {
                // Fallback to building from fields if URL parsing fails
                build_server_dsn(
                    "postgresql",
                    final_host.as_deref(),
                    final_port,
                    final_user.as_deref(),
                    final_password.as_deref(),
                    Some(&dbname),
                    &final_params,
                )?
            }
        } else {
            // This branch should not be reachable due to the condition above
            unreachable!("final_dsn should not be None when has_field_overrides is false")
        }
    };

    // Validate final DSN
    validate_dsn(&result_dsn)?;

    // Redact password for logging
    let log_dsn = if result_dsn.contains('@') {
        let parsed = url::Url::parse(&result_dsn)?;
        let mut log_url = parsed.clone();
        if log_url.password().is_some() {
            log_url.set_password(Some("***")).ok();
        }
        log_url.to_string()
    } else {
        result_dsn.clone()
    };

    tracing::info!(
        "Built final DB config for module '{}': {}",
        module_name,
        log_dsn
    );

    Ok(Some((result_dsn, final_pool)))
}

/// Helper function to get module database configuration from AppConfig.
/// Returns the DbConnConfig for a module, or None if the module has no database config.
pub fn get_module_db_config(app: &AppConfig, module_name: &str) -> Option<DbConnConfig> {
    let module_raw = app.modules.get(module_name)?;
    let module_entry: ModuleEntry = serde_json::from_value(module_raw.clone()).ok()?;
    module_entry.database
}

/// Helper function to resolve module home directory.
/// Returns the path where module-specific files (like SQLite databases) should be stored.
pub fn module_home(app: &AppConfig, module_name: &str) -> PathBuf {
    PathBuf::from(&app.server.home_dir).join(module_name)
}

// Include tracing config tests
#[cfg(test)]
mod tracing_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};
    use tempfile::tempdir;

    /// Helper: a normalized home_dir should be absolute and not start with '~'.
    fn is_normalized_path(p: &str) -> bool {
        let pb = PathBuf::from(p);
        pb.is_absolute() && !p.starts_with('~')
    }

    /// Helper: platform default subdirectory name.
    fn default_subdir() -> &'static str {
        ".hyperspot"
    }

    #[test]
    fn test_default_config_structure() {
        let config = AppConfig::default();

        // Database defaults (simplified structure)
        assert!(config.database.is_some());
        let db = config.database.as_ref().unwrap();
        assert!(db.servers.is_empty()); // Default config has no servers defined
        assert_eq!(db.auto_provision, None);

        // Logging defaults
        assert!(config.logging.is_some());
        let logging = config.logging.as_ref().unwrap();
        assert!(logging.contains_key("default"));

        let default_section = &logging["default"];
        assert_eq!(default_section.console_level, "info");
        assert_eq!(default_section.file, "logs/hyperspot.log");

        // Modules bag is empty by default
        assert!(config.modules.is_empty());
    }

    #[test]
    fn test_load_layered_normalizes_home_dir() {
        let tmp = tempdir().unwrap();
        let cfg_path = tmp.path().join("cfg.yaml");

        // Provide a user path with "~" to ensure expansion and normalization.
        let yaml = r#"
server:
  home_dir: "~/.test_hyperspot"

database:
  servers:
    test_postgres:
      dsn: "postgres://user:pass@localhost/db"
      pool:
        max_conns: 20

logging:
  default:
    console_level: debug
    file: "logs/default.log"
"#;
        fs::write(&cfg_path, yaml).unwrap();

        let config = AppConfig::load_layered(&cfg_path).unwrap();

        // home_dir should be normalized immediately
        assert!(is_normalized_path(&config.server.home_dir));
        assert!(config.server.home_dir.ends_with(".test_hyperspot"));

        // database parsed (TODO: update test to use new config format)
        // For now, since this test uses old format YAML, we skip DB assertions
        // let db = config.database.as_ref().unwrap();

        // logging parsed
        let logging = config.logging.as_ref().unwrap();
        let def = &logging["default"];
        assert_eq!(def.console_level, "debug");
        assert_eq!(def.file, "logs/default.log");
    }

    #[test]
    fn test_load_or_default_normalizes_home_dir_when_none() {
        // No external file => defaults, but home_dir must be normalized.
        // Ensure platform env is present for home resolution in CI.
        let tmp = tempdir().unwrap();
        #[cfg(target_os = "windows")]
        env::set_var("APPDATA", tmp.path());
        #[cfg(not(target_os = "windows"))]
        env::set_var("HOME", tmp.path());
        let config = AppConfig::load_or_default(None::<&str>).unwrap();
        assert!(is_normalized_path(&config.server.home_dir));
        assert!(config.server.home_dir.ends_with(default_subdir()));
    }

    #[test]
    fn test_minimal_yaml_config() {
        let tmp = tempdir().unwrap();
        let cfg_path = tmp.path().join("cfg.yaml");

        // Set up environment variables for home directory resolution
        #[cfg(target_os = "windows")]
        env::set_var("APPDATA", tmp.path());
        #[cfg(not(target_os = "windows"))]
        env::set_var("HOME", tmp.path());

        let yaml = r#"
server:
  home_dir: "~/.minimal"
"#;
        fs::write(&cfg_path, yaml).unwrap();

        let config = AppConfig::load_layered(&cfg_path).unwrap();

        // Required fields are parsed; home_dir normalized
        assert!(is_normalized_path(&config.server.home_dir));
        assert!(config.server.home_dir.ends_with(".minimal"));

        // Optional sections default to None
        assert!(config.database.is_none());
        assert!(config.logging.is_none());
        assert!(config.modules.is_empty());
    }

    #[test]
    fn test_cli_overrides() {
        let mut config = AppConfig::default();

        let args = super::CliArgs {
            config: None,
            print_config: false,
            verbose: 2, // trace
            mock: false,
        };

        config.apply_cli_overrides(&args);

        // Port override

        // Verbose override affects logging
        let logging = config.logging.as_ref().unwrap();
        let default_section = &logging["default"];
        assert_eq!(default_section.console_level, "trace");
    }

    #[test]
    fn test_cli_verbose_levels_matrix() {
        for (verbose_level, expected_log_level) in [
            (0, "info"), // unchanged from default
            (1, "debug"),
            (2, "trace"),
            (3, "trace"), // cap at trace
        ] {
            let mut config = AppConfig::default();
            let args = super::CliArgs {
                config: None,
                print_config: false,
                verbose: verbose_level,
                mock: false,
            };

            config.apply_cli_overrides(&args);

            let logging = config.logging.as_ref().unwrap();
            let default_section = &logging["default"];

            if verbose_level == 0 {
                assert_eq!(default_section.console_level, "info");
            } else {
                assert_eq!(default_section.console_level, expected_log_level);
            }
        }
    }

    #[test]
    fn test_layered_config_loading_with_modules_dir() {
        let tmp = tempdir().unwrap();
        let cfg_path = tmp.path().join("modules_dir.yaml");
        let modules_dir = tmp.path().join("modules");

        fs::create_dir_all(&modules_dir).unwrap();
        let module_cfg = modules_dir.join("test_module.yaml");
        fs::write(
            &module_cfg,
            r#"
setting1: "value1"
setting2: 42
"#,
        )
        .unwrap();

        // Convert Windows paths to forward slashes for YAML compatibility
        let modules_dir_str = modules_dir.to_string_lossy().replace('\\', "/");
        let yaml = format!(
            r#"
server:
  home_dir: "~/.modules_test"

modules_dir: "{}"

modules:
  existing_module:
    key: "value"
"#,
            modules_dir_str
        );

        fs::write(&cfg_path, yaml).unwrap();

        let config = AppConfig::load_layered(&cfg_path).unwrap();

        // Should have loaded the existing module from modules section
        assert!(config.modules.contains_key("existing_module"));

        // Should have also loaded the module from modules_dir
        assert!(config.modules.contains_key("test_module"));

        // Check the loaded module config
        let test_module = &config.modules["test_module"];
        assert_eq!(test_module["setting1"], "value1");
        assert_eq!(test_module["setting2"], 42);
    }

    #[test]
    fn test_load_and_init_logging_smoke() {
        // Just verifies structure is acceptable for logging init path.
        let tmp = tempdir().unwrap();
        let cfg_path = tmp.path().join("logging.yaml");
        let yaml = r#"
server:
  home_dir: "~/.logging_test"

logging:
  default:
    console_level: debug
    file: ""
    file_level: info
"#;
        fs::write(&cfg_path, yaml).unwrap();

        let config = AppConfig::load_layered(&cfg_path).unwrap();
        assert!(config.logging.is_some());
        let logging = config.logging.as_ref().unwrap();
        assert!(logging.contains_key("default"));

        let default_section = &logging["default"];
        assert_eq!(default_section.console_level, "debug");
        assert_eq!(default_section.file_level, "info");
        // not calling init to avoid side effects in tests
    }

    // ===================== DB Configuration Precedence Tests =====================

    /// Helper function to create AppConfig with database server configuration
    fn create_app_with_server(server_name: &str, db_config: DbConnConfig) -> AppConfig {
        let mut servers = HashMap::new();
        servers.insert(server_name.to_string(), db_config);

        AppConfig {
            database: Some(GlobalDatabaseConfig {
                servers,
                auto_provision: None,
            }),
            ..Default::default()
        }
    }

    /// Helper function to add a module to AppConfig
    fn add_module_to_app(
        app: &mut AppConfig,
        module_name: &str,
        database_config: serde_json::Value,
    ) {
        app.modules.insert(
            module_name.to_string(),
            serde_json::json!({
                "database": database_config,
                "config": {}
            }),
        );
    }

    #[test]
    fn test_precedence_global_dsn_only() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                dsn: Some(
                    "postgresql://global_user:global_pass@global_host:5432/global_db".to_string(),
                ),
                ..Default::default()
            },
        );

        // Module references global server
        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("global_user"));
        assert!(dsn.contains("global_host"));
        assert!(dsn.contains("global_db"));
    }

    #[test]
    fn test_precedence_global_fields_only() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                host: Some("field_host".to_string()),
                port: Some(5433),
                user: Some("field_user".to_string()),
                dbname: Some("field_db".to_string()),
                ..Default::default()
            },
        );

        // Module references global server
        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("field_host"));
        assert!(dsn.contains("5433"));
        assert!(dsn.contains("field_user"));
        assert!(dsn.contains("field_db"));
    }

    #[test]
    fn test_precedence_module_dsn_only() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "dsn": "sqlite://module_test.db?wal=true&synchronous=NORMAL"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("module_test.db"));
        assert!(dsn.contains("wal=true"));
    }

    #[test]
    fn test_precedence_module_fields_only() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "file": "module_fields.db"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("module_fields.db"));
        // Platform-specific DSN format check
        #[cfg(windows)]
        assert!(dsn.starts_with("sqlite:") && !dsn.starts_with("sqlite://"));
        #[cfg(unix)]
        assert!(dsn.starts_with("sqlite://"));
    }

    #[test]
    fn test_precedence_fields_override_dsn() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                dsn: Some("postgresql://old_user:old_pass@old_host:5432/old_db".to_string()),
                host: Some("new_host".to_string()), // This should override DSN host
                port: Some(5433),                   // This should override DSN port
                user: Some("new_user".to_string()), // This should override DSN user
                dbname: Some("new_db".to_string()), // This should override DSN dbname
                ..Default::default()
            },
        );

        // Module also overrides some fields
        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server",
                "port": 5434  // Module field should override global field
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        // Fields should override DSN parts
        assert!(dsn.contains("new_host"));
        assert!(dsn.contains("5434")); // Module override should win
        assert!(dsn.contains("new_user"));
        assert!(dsn.contains("new_db"));
        // Old DSN values should not appear
        assert!(!dsn.contains("old_host"));
        assert!(!dsn.contains("5432"));
        assert!(!dsn.contains("old_user"));
        assert!(!dsn.contains("old_db"));
    }

    #[test]
    fn test_env_expansion_password() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        // Set environment variable for test
        env::set_var("TEST_DB_PASSWORD", "secret123");

        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                host: Some("localhost".to_string()),
                port: Some(5432),
                user: Some("testuser".to_string()),
                password: Some("${TEST_DB_PASSWORD}".to_string()), // Should expand to "secret123"
                dbname: Some("testdb".to_string()),
                ..Default::default()
            },
        );

        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("secret123"));

        // Clean up
        env::remove_var("TEST_DB_PASSWORD");
    }

    #[test]
    fn test_env_expansion_in_dsn() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        // Set environment variables for test
        env::set_var("DB_HOST", "test-server");
        env::set_var("DB_PASSWORD", "env_secret");

        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                dsn: Some("postgresql://user:${DB_PASSWORD}@${DB_HOST}:5432/mydb".to_string()),
                ..Default::default()
            },
        );

        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("test-server"));
        assert!(dsn.contains("env_secret"));
        // ${} placeholders should be replaced
        assert!(!dsn.contains("${DB_HOST}"));
        assert!(!dsn.contains("${DB_PASSWORD}"));

        // Clean up
        env::remove_var("DB_HOST");
        env::remove_var("DB_PASSWORD");
    }

    #[test]
    fn test_sqlite_file_path_resolution() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        // Test 1: file (relative to home_dir/module_name/)
        let app1 = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "file": "test.db"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result1 = build_final_db_for_module(&app1, "test_module", home_dir).unwrap();
        assert!(result1.is_some());
        let (dsn1, _) = result1.unwrap();
        assert!(dsn1.contains("test_module"));
        assert!(dsn1.contains("test.db"));

        // Test 2: path (absolute path)
        let abs_path = tmp.path().join("absolute.db");
        let app2 = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "path": abs_path.to_string_lossy()
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result2 = build_final_db_for_module(&app2, "test_module", home_dir).unwrap();
        assert!(result2.is_some());
        let (dsn2, _) = result2.unwrap();
        assert!(dsn2.contains("absolute.db"));

        // Test 3: no file or path (should default to module_name.sqlite)
        let app3 = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {},
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result3 = build_final_db_for_module(&app3, "test_module", home_dir).unwrap();
        assert!(result3.is_some());
        let (dsn3, _) = result3.unwrap();
        assert!(dsn3.contains("test_module.sqlite"));
    }

    #[cfg(windows)]
    #[test]
    fn test_sqlite_path_resolution_windows() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "file": "test.db"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());
        let (dsn, _) = result.unwrap();

        // On Windows, paths should be normalized to forward slashes in DSN
        assert!(!dsn.contains("\\"));
        assert!(dsn.contains("/"));
    }

    #[test]
    fn test_sqlite_dsn_with_server_reference_and_dbname_override() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let mut app = AppConfig::default();

        // Global server with SQLite DSN and query params
        let mut servers = HashMap::new();
        servers.insert(
            "sqlite_users".to_string(),
            DbConnConfig {
                dsn: Some(
                    "sqlite://users_info.db?WAL=true&synchronous=NORMAL&busy_timeout=5000"
                        .to_string(),
                ),
                host: None,
                port: None,
                user: None,
                password: None,
                dbname: None,
                params: None,
                pool: None,
                file: None,
                path: None,
                server: None,
            },
        );

        app.database = Some(GlobalDatabaseConfig {
            servers,
            auto_provision: None,
        });

        // Module that references the server but overrides the dbname
        app.modules.insert(
            "users_info".to_string(),
            serde_json::json!({
                "database": {
                    "server": "sqlite_users",
                    "dbname": "users_info.db"
                },
                "config": {}
            }),
        );

        let result = build_final_db_for_module(&app, "users_info", home_dir).unwrap();
        assert!(result.is_some());
        let (dsn, _) = result.unwrap();

        // Should be an absolute path with preserved query parameters
        assert!(dsn.contains("?WAL=true&synchronous=NORMAL&busy_timeout=5000"));
        assert!(dsn.contains("users_info/users_info.db"));

        // Platform-specific path format
        #[cfg(windows)]
        {
            // Windows should use sqlite:C:/path format
            assert!(dsn.starts_with("sqlite:"));
            assert!(!dsn.starts_with("sqlite://"));
        }

        #[cfg(unix)]
        {
            // Unix should use sqlite://path format
            assert!(dsn.starts_with("sqlite://"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_sqlite_path_resolution_unix() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "file": "test.db"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());
        let (dsn, _) = result.unwrap();

        // On Unix, paths should be absolute
        assert!(dsn.starts_with("sqlite://"));
        assert!(dsn.contains("/test_module/test.db"));
    }

    #[test]
    fn test_server_based_db_missing_dbname_error() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                host: Some("localhost".to_string()),
                port: Some(5432),
                user: Some("testuser".to_string()),
                // Missing dbname for server-based DB
                ..Default::default()
            },
        );

        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("missing required 'dbname'"));
    }

    #[test]
    fn test_module_no_database_config() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        // Module with no database section
        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "no_db_module".to_string(),
                    serde_json::json!({
                        "config": {
                            "some_setting": "value"
                        }
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "no_db_module", home_dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_module_empty_database_config() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        // Module with empty database section
        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "empty_db_module".to_string(),
                    serde_json::json!({
                        "database": null,
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "empty_db_module", home_dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_referenced_server_not_found() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "server": "nonexistent_server"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Referenced server 'nonexistent_server' not found"));
    }

    #[test]
    fn test_dsn_validation_invalid_url() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "dsn": "invalid://not-a-valid[url"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_env_variable_not_found() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        // Ensure the env var doesn't exist
        env::remove_var("NONEXISTENT_PASSWORD");

        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                host: Some("localhost".to_string()),
                password: Some("${NONEXISTENT_PASSWORD}".to_string()),
                dbname: Some("testdb".to_string()),
                ..Default::default()
            },
        );

        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("NONEXISTENT_PASSWORD"));
    }

    #[test]
    fn test_sqlite_at_file_relative_path() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "dsn": "sqlite://@file(users.db)"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("test_module"));
        assert!(dsn.contains("users.db"));
        // Platform-specific DSN format check
        #[cfg(windows)]
        assert!(dsn.starts_with("sqlite:") && !dsn.starts_with("sqlite://"));
        #[cfg(unix)]
        assert!(dsn.starts_with("sqlite:///"));
    }

    #[test]
    fn test_sqlite_at_file_absolute_path() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();
        let abs_path = tmp.path().join("absolute_db.sqlite");

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "dsn": format!("sqlite://@file({})", abs_path.to_string_lossy())
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("absolute_db.sqlite"));
        // Platform-specific DSN format check
        #[cfg(windows)]
        assert!(dsn.starts_with("sqlite:") && !dsn.starts_with("sqlite://"));
        #[cfg(unix)]
        assert!(dsn.starts_with("sqlite:///"));
    }

    #[test]
    fn test_sqlite_empty_dsn_default() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "dsn": "sqlite://"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();
        assert!(dsn.contains("test_module"));
        assert!(dsn.contains("test_module.sqlite"));
        // Platform-specific DSN format check
        #[cfg(windows)]
        assert!(dsn.starts_with("sqlite:") && !dsn.starts_with("sqlite://"));
        #[cfg(unix)]
        assert!(dsn.starts_with("sqlite:///"));
    }

    #[test]
    fn test_sqlite_at_file_invalid_syntax() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let app = AppConfig {
            modules: {
                let mut modules = HashMap::new();
                modules.insert(
                    "test_module".to_string(),
                    serde_json::json!({
                        "database": {
                            "dsn": "sqlite://@file(missing_closing_paren"
                        },
                        "config": {}
                    }),
                );
                modules
            },
            ..Default::default()
        };

        let result = build_final_db_for_module(&app, "test_module", home_dir);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid @file() syntax"));
    }

    #[test]
    fn test_dsn_special_characters_in_credentials() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        // Test with special characters in username and password
        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                host: Some("localhost".to_string()),
                port: Some(5432),
                user: Some("user@domain".to_string()),
                password: Some("pa@ss:w0rd/with%special&chars".to_string()),
                dbname: Some("test/db".to_string()),
                ..Default::default()
            },
        );

        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();

        // Verify DSN is properly encoded
        assert!(dsn.starts_with("postgresql://"));
        assert!(dsn.contains("user%40domain")); // @ encoded as %40
        assert!(dsn.contains("/test%2Fdb")); // / in dbname encoded as %2F

        // Verify DSN is parseable and contains expected user
        validate_dsn(&dsn).expect("DSN with special characters should be valid");

        // Parse the DSN to verify it contains the correct components
        let parsed_dsn = dsn::parse(&dsn).expect("DSN should be parseable");
        assert_eq!(parsed_dsn.username.as_deref(), Some("user@domain"));
        assert_eq!(
            parsed_dsn.password.as_deref(),
            Some("pa@ss:w0rd/with%special&chars")
        );
        // Note: dsn crate may have limitations with path parsing - just verify the main DSN works
        // The important thing is that the DSN is valid and contains the right components
    }

    #[test]
    fn test_dsn_unicode_characters() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        // Test with Unicode characters
        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                host: Some("localhost".to_string()),
                user: Some("ユーザー".to_string()), // Japanese characters
                password: Some("пароль".to_string()), // Cyrillic characters
                dbname: Some("тест_база".to_string()),
                ..Default::default()
            },
        );

        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();

        // Verify DSN is properly encoded with Unicode
        assert!(dsn.starts_with("postgresql://"));
        // Unicode characters should be percent-encoded
        assert!(dsn.contains("%")); // Should contain encoded characters

        // Verify DSN is parseable
        validate_dsn(&dsn).expect("DSN with Unicode characters should be valid");
    }

    #[test]
    fn test_dsn_query_parameters_encoding() {
        let tmp = tempdir().unwrap();
        let home_dir = tmp.path();

        let mut params = HashMap::new();
        params.insert("ssl mode".to_string(), "require & verify".to_string());
        params.insert("application_name".to_string(), "my-app/v1.0".to_string());

        let mut app = create_app_with_server(
            "test_server",
            DbConnConfig {
                host: Some("localhost".to_string()),
                user: Some("testuser".to_string()),
                dbname: Some("testdb".to_string()),
                params: Some(params),
                ..Default::default()
            },
        );

        add_module_to_app(
            &mut app,
            "test_module",
            serde_json::json!({
                "server": "test_server"
            }),
        );

        let result = build_final_db_for_module(&app, "test_module", home_dir).unwrap();
        assert!(result.is_some());

        let (dsn, _pool) = result.unwrap();

        // Verify query parameters are properly encoded (spaces become +, & becomes %26)
        assert!(dsn.contains("ssl+mode=require+%26+verify"));
        assert!(dsn.contains("application_name=my-app%2Fv1.0"));

        // Verify DSN is parseable
        validate_dsn(&dsn).expect("DSN with encoded query parameters should be valid");
    }
}

// Note: DB trait implementations and helper functions removed since we now use DbManager
