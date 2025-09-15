use anyhow::Result;
use clap::{Parser, Subcommand};
use mimalloc::MiMalloc;
use runtime::{AppConfig, AppConfigProvider, CliArgs, ConfigProvider};

use std::path::{Path, PathBuf};
use std::sync::Arc;

// Keep sqlx drivers linked (sqlx::any quirk)
#[allow(unused_imports)]
use sqlx::{postgres::Postgres, sqlite::Sqlite};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Adapter to make `AppConfigProvider` implement `modkit::ConfigProvider`.
struct ModkitConfigAdapter(std::sync::Arc<AppConfigProvider>);

impl modkit::ConfigProvider for ModkitConfigAdapter {
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
        self.0.get_module_config(module_name)
    }
}

// Ensure modules are linked and registered via inventory
#[allow(dead_code)]
fn _ensure_modules_linked() {
    // Make sure all modules are linked
    let _ = std::any::type_name::<api_ingress::ApiIngress>();
    #[cfg(feature = "users-info-example")]
    let _ = std::any::type_name::<users_info::UsersInfo>();
}

// Bring runner types & our per-module DB factory
use modkit::runtime::{run, DbOptions, RunOptions, ShutdownOptions};
mod db_factory;

#[allow(dead_code)]
fn _ensure_drivers_linked() {
    // Ensure database drivers are linked for sqlx::any
    let _ = std::any::type_name::<Sqlite>();
    let _ = std::any::type_name::<Postgres>();
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

    /// Port override for HTTP server (overrides config)
    #[arg(short, long)]
    port: Option<u16>,

    /// Print effective configuration (YAML) and exit
    #[arg(long)]
    print_config: bool,

    /// Log verbosity level (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Use mock database (sqlite::memory:) for all modules
    #[arg(long)]
    mock: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server
    Run,
    /// Validate configuration and exit
    Check,
}

#[tokio::main]
async fn main() -> Result<()> {
    _ensure_drivers_linked();

    let cli = Cli::parse();

    // Prepare CLI args that flow into runtime::AppConfig merge logic.
    let args = CliArgs {
        config: cli.config.as_ref().map(|p| p.to_string_lossy().to_string()),
        port: cli.port,
        print_config: cli.print_config,
        verbose: cli.verbose,
        mock: cli.mock,
    };

    // Layered config:
    // 1) defaults -> 2) YAML (if provided) -> 3) env (APP__*) -> 4) CLI overrides
    // Also normalizes + creates server.home_dir.
    let mut config = AppConfig::load_or_default(cli.config.as_deref())?;
    config.apply_cli_overrides(&args);

    // Init logging as early as possible.
    let logging_config = config.logging.as_ref().cloned().unwrap_or_default();
    runtime::logging::init_logging_from_config(&logging_config, Path::new(&config.server.home_dir));

    tracing::info!("HyperSpot Server starting");

    if cli.print_config {
        println!("{}", config.to_yaml()?);
        return Ok(());
    }

    // Dispatch subcommands (default: run)
    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => run_server(config, args).await,
        Commands::Check => check_config(config).await,
    }
}

async fn run_server(config: AppConfig, args: CliArgs) -> Result<()> {
    tracing::info!("Initializing modules…");

    // Bridge AppConfig into ModKit’s ConfigProvider (per-module JSON bag).
    let config_provider = Arc::new(ModkitConfigAdapter(Arc::new(AppConfigProvider::new(
        config.clone(),
    ))));

    // Base dir used by DB factory for file-based SQLite resolution
    let home_dir = PathBuf::from(&config.server.home_dir);

    // Configure DB options: per-module factory or no-DB.
    let db_options = if config.database.is_some() {
        if args.mock {
            tracing::info!("Mock mode enabled: using in-memory SQLite for all modules");
            DbOptions::PerModuleFactory(db_factory::create_mock_per_module_db_factory())
        } else {
            tracing::info!("Using per-module database configuration");
            DbOptions::PerModuleFactory(db_factory::create_per_module_db_factory(
                Arc::new(config.clone()),
                home_dir,
            ))
        }
    } else {
        tracing::warn!("No global database section found; running without databases");
        DbOptions::None
    };

    // Run the ModKit runtime (signals-driven shutdown).
    let run_options = RunOptions {
        modules_cfg: config_provider,
        db: db_options,
        shutdown: ShutdownOptions::Signals,
    };

    run(run_options).await
}

async fn check_config(config: AppConfig) -> Result<()> {
    tracing::info!("Checking configuration…");
    // If load_layered/load_or_default succeeded and home_dir normalized, we’re good.
    println!("Configuration is valid");
    println!("{}", config.to_yaml()?);
    Ok(())
}
