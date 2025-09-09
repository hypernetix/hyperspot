use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use db::{ConnectOpts, DbHandle};
use mimalloc::MiMalloc;
use runtime::{AppConfig, AppConfigProvider, CliArgs, ConfigProvider, DatabaseConfig};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Adapter to make AppConfigProvider implement modkit::ConfigProvider
struct ModkitConfigAdapter(std::sync::Arc<AppConfigProvider>);

impl modkit::ConfigProvider for ModkitConfigAdapter {
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
        self.0.get_module_config(module_name)
    }
}

use modkit::runtime::{run, DbFactory, DbOptions, RunOptions, ShutdownOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

// Ensure modules are linked and registered via inventory
#[allow(dead_code)]
fn _ensure_modules_linked() {
    // Make sure all modules are linked
    let _ = std::any::type_name::<api_ingress::ApiIngress>();
    #[cfg(feature = "users-info-example")]
    let _ = std::any::type_name::<users_info::UsersInfo>();
    let _ = std::any::type_name::<settings::Settings>();
}

// Force SQLx driver registration for Any driver (workaround for SQLx 0.8)
#[allow(unused_imports)]
use sqlx::{postgres::Postgres, sqlite::Sqlite};

#[allow(dead_code)]
fn _ensure_drivers_linked() {
    // Make sure database drivers are linked for sqlx::any
    let _ = std::any::type_name::<Sqlite>();
    let _ = std::any::type_name::<Postgres>();
}

/// Expand a sqlite DSN into an absolute-path DSN using a base directory.
/// - Keeps "sqlite::memory:" as-is.
/// - Normalizes backslashes into forward slashes (important on Windows).
fn absolutize_sqlite_dsn(dsn: &str, base_dir: &Path, create_dirs: bool) -> Result<String> {
    if dsn.eq_ignore_ascii_case("sqlite::memory:") || dsn.eq_ignore_ascii_case("sqlite://:memory:")
    {
        return Ok("sqlite::memory:".to_string());
    }
    let db_path = dsn
        .strip_prefix("sqlite://")
        .ok_or_else(|| anyhow!("DSN must start with sqlite:// (got: {})", dsn))?;

    let (path_str, query) = match db_path.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (db_path, None),
    };

    let mut p = PathBuf::from(path_str);
    if p.as_os_str().is_empty() {
        return Err(anyhow!("Empty SQLite path in DSN"));
    }
    if p.is_relative() {
        p = base_dir.join(p);
    }

    if let Some(dir) = p.parent() {
        if create_dirs {
            std::fs::create_dir_all(dir)?;
        }
    }

    // Rebuild DSN with absolute path and normalized slashes
    let mut out = String::from("sqlite://");
    out.push_str(&p.to_string_lossy().replace('\\', "/"));
    if let Some(q) = query {
        out.push('?');
        out.push_str(q);
    }
    Ok(out)
}

/// HyperSpot Server - modular platform for AI services
#[derive(Parser)]
#[command(name = "hyperspot-server")]
#[command(about = "HyperSpot Server - modular platform for AI services")]
#[command(version = "0.1.0")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Port for HTTP server (overrides config)
    #[arg(short, long)]
    port: Option<u16>,

    /// Print current configuration and exit
    #[arg(long)]
    print_config: bool,

    /// Log verbosity level (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Use mock database
    #[arg(long)]
    mock: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server
    Run,
    /// Check configuration
    Check,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Link SQLx drivers (Any driver quirk in 0.8)
    _ensure_drivers_linked();

    let cli = Cli::parse();

    // CLI args passed down to config/app
    let args = CliArgs {
        config: cli.config.as_ref().map(|p| p.to_string_lossy().to_string()),
        port: cli.port,
        print_config: cli.print_config,
        verbose: cli.verbose,
        mock: cli.mock,
    };

    // Load configuration (normalized home_dir is applied inside)
    let mut config = AppConfig::load_or_default(cli.config.as_deref())?;

    // Apply CLI overrides (port / verbosity)
    config.apply_cli_overrides(&args);

    // Initialize logging
    let logging_config = config.logging.as_ref().cloned().unwrap_or_default();
    runtime::logging::init_logging_from_config(&logging_config, Path::new(&config.server.home_dir));
    tracing::info!("HyperSpot Server starting");
    println!("Effective configuration:\n{:#?}", config.server);

    // Print config and exit if requested
    if cli.print_config {
        println!("{}", config.to_yaml()?);
        return Ok(());
    }

    // Execute command
    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => run_server(config, args).await,
        Commands::Check => check_config(config).await,
    }
}

/// Detect DB backend from URL scheme (sqlite/postgres/mysql).
fn detect_from_dsn(cfg: &DatabaseConfig) -> anyhow::Result<&'static str> {
    let raw = cfg.url.trim().to_owned();
    if raw.is_empty() {
        return Err(anyhow!("Database URL not configured"));
    }

    let url = Url::parse(&raw).map_err(|e| anyhow!("Invalid database DSN '{}': {}", raw, e))?;

    match url.scheme() {
        "sqlite" | "sqlite3" => Ok("sqlite"),
        "postgres" | "postgresql" => Ok("postgres"),
        "mysql" | "mariadb" => Ok("mysql"),
        other => Err(anyhow!("Unsupported database type: {}", other)),
    }
}

async fn run_server(config: AppConfig, args: CliArgs) -> Result<()> {
    tracing::info!("Initializing modules...");

    // Provide module configs to modkit
    let config_provider = Arc::new(ModkitConfigAdapter(Arc::new(AppConfigProvider::new(
        config.clone(),
    ))));

    // Base dir for resolving relative sqlite paths (already absolute & created)
    let base_dir = PathBuf::from(&config.server.home_dir);

    // Prepare DB factory if database config exists
    let db_options = if let Some(db_config) = config.database.clone() {
        let factory: DbFactory = Box::new(move || {
            let args = args.clone();
            let db_config = db_config.clone();
            let base_dir = base_dir.clone();

            Box::pin(async move {
                let _backend = detect_from_dsn(&db_config)?;

                // Use URL from config; override with in-memory SQLite when --mock is set
                let config_dsn = db_config.url.trim().to_owned();
                if config_dsn.is_empty() {
                    return Err(anyhow!("Database URL not configured"));
                }

                let mut final_dsn = if args.mock {
                    "sqlite://:memory:".to_string()
                } else {
                    config_dsn
                };

                // Absolutize sqlite DSNs to avoid cwd issues
                if final_dsn.starts_with("sqlite://") {
                    final_dsn = absolutize_sqlite_dsn(&final_dsn, &base_dir, true)?;
                }

                let connect_opts = ConnectOpts {
                    max_conns: db_config.max_conns,
                    acquire_timeout: Some(Duration::from_secs(5)),
                    sqlite_busy_timeout: db_config
                        .busy_timeout_ms
                        .map(|ms| Duration::from_millis(ms as u64)),
                    create_sqlite_dirs: true,
                    ..Default::default()
                };

                tracing::info!("Connecting to database: {}", final_dsn);
                let db = DbHandle::connect(&final_dsn, connect_opts).await?;
                let backend = db.engine();
                tracing::info!("Connected DB backend: {:?}", backend);

                Ok(Arc::new(db))
            })
        });

        DbOptions::Auto(factory)
    } else {
        tracing::warn!("No database configuration found, running without database");
        DbOptions::None
    };

    // Run the server via modkit
    let run_options: RunOptions = RunOptions {
        modules_cfg: config_provider,
        db: db_options,
        shutdown: ShutdownOptions::Signals,
    };

    run(run_options).await
}

async fn check_config(config: AppConfig) -> Result<()> {
    tracing::info!("Checking configuration...");

    // AppConfig::load_* already normalized & created home_dir
    tracing::info!("Configuration is valid");
    println!("Configuration check passed");
    println!("Server config:");
    println!("{}", config.to_yaml()?);

    Ok(())
}
