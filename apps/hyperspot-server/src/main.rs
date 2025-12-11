mod registered_modules;

use anyhow::Result;
use clap::{Parser, Subcommand};
use figment::Figment;
use mimalloc::MiMalloc;
use modkit_bootstrap::{AppConfig, AppConfigProvider, CliArgs, ConfigProvider};

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

// Bring runner types & our per-module DB factory
use modkit::runtime::{run, DbOptions, RunOptions, ShutdownOptions};

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

    if let Some(ref path) = cli.config {
        let path_str = path.to_string_lossy();
        if !Path::new(path).is_file() {
            anyhow::bail!("config file does not exist: {}", path_str);
        }
    }

    // Prepare CLI args that flow into runtime::AppConfig merge logic.
    let args = CliArgs {
        config: cli.config.as_ref().map(|p| p.to_string_lossy().to_string()),
        print_config: cli.print_config,
        verbose: cli.verbose,
        mock: cli.mock,
    };

    // Layered config:
    // 1) defaults -> 2) YAML (if provided) -> 3) env (APP__*) -> 4) CLI overrides
    // Also normalizes + creates server.home_dir.
    let mut config = AppConfig::load_or_default(cli.config.as_deref())?;
    config.apply_cli_overrides(&args);

    // Build OpenTelemetry layer before logging
    #[cfg(feature = "otel")]
    let otel_layer = config
        .tracing
        .as_ref()
        .and_then(modkit::telemetry::init::init_tracing);
    #[cfg(not(feature = "otel"))]
    let otel_layer = None;

    // Initialize logging + otel in one Registry
    let logging_config = config.logging.clone().unwrap_or_default();
    modkit_bootstrap::logging::init_logging_unified(
        &logging_config,
        Path::new(&config.server.home_dir),
        otel_layer,
    );

    // One-time connectivity probe
    #[cfg(feature = "otel")]
    if let Some(tc) = config.tracing.as_ref() {
        if let Err(e) = modkit::telemetry::init::otel_connectivity_probe(tc).await {
            tracing::error!(error = %e, "OTLP connectivity probe failed");
        }
    }

    // Smoke test span to confirm traces flow to Jaeger
    tracing::info_span!("startup_check", app = "hyperspot").in_scope(|| {
        tracing::info!("startup span alive - traces should be visible in Jaeger");
    });

    tracing::info!("HyperSpot Server starting");

    // Print config and exit if requested
    if cli.print_config {
        println!("Effective configuration:\n{}", config.to_yaml()?);
        return Ok(());
    }

    // Dispatch subcommands (default: run)
    match cli.command.unwrap_or(Commands::Run) {
        Commands::Run => run_server(config, args).await,
        Commands::Check => check_config(config).await,
    }
}

async fn check_config(config: AppConfig) -> Result<()> {
    tracing::info!("Checking configuration...");
    // If load_layered/load_or_default succeeded and home_dir normalized, we're good.
    println!("Configuration is valid");
    println!("{}", config.to_yaml()?);
    Ok(())
}

/// Create a Figment from the loaded AppConfig for use with DbManager.
fn create_figment_from_config(config: &AppConfig) -> Result<Figment> {
    use figment::providers::Serialized;

    // Convert the AppConfig back to a Figment that DbManager can use
    // We serialize the config and then parse it back as a Figment
    let figment = Figment::new().merge(Serialized::defaults(config));

    Ok(figment)
}

/// Create a mock Figment for testing with in-memory SQLite databases.
fn create_mock_figment(config: &AppConfig) -> Figment {
    use figment::providers::Serialized;

    let mut mock_config = config.clone();
    override_modules_with_mock_db(&mut mock_config);

    Figment::new().merge(Serialized::defaults(mock_config))
}

/// Override all module database configurations with in-memory SQLite
fn override_modules_with_mock_db(config: &mut AppConfig) {
    for module_value in config.modules.values_mut() {
        if let Some(obj) = module_value.as_object_mut() {
            obj.insert(
                "database".to_string(),
                serde_json::json!({
                    "dsn": "sqlite::memory:",
                    "params": {
                        "journal_mode": "WAL"
                    }
                }),
            );
        }
    }
}

/// Build config provider from AppConfig
fn build_config_provider(config: AppConfig) -> Arc<dyn modkit::ConfigProvider> {
    Arc::new(ModkitConfigAdapter(Arc::new(AppConfigProvider::new(
        config,
    ))))
}

/// Resolve database options based on configuration and args
fn resolve_db_options(config: &AppConfig, args: &CliArgs) -> Result<DbOptions> {
    if config.database.is_none() {
        tracing::warn!("No global database section found; running without databases");
        return Ok(DbOptions::None);
    }

    if args.mock {
        tracing::info!("Mock mode enabled: using in-memory SQLite for all modules");
        let mock_figment = create_mock_figment(config);
        let home_dir = PathBuf::from(&config.server.home_dir);
        let db_manager = Arc::new(modkit_db::DbManager::from_figment(mock_figment, home_dir)?);
        return Ok(DbOptions::Manager(db_manager));
    }

    tracing::info!("Using DbManager with Figment-based configuration");
    let figment = create_figment_from_config(config)?;
    let home_dir = PathBuf::from(&config.server.home_dir);
    let db_manager = Arc::new(modkit_db::DbManager::from_figment(figment, home_dir)?);
    Ok(DbOptions::Manager(db_manager))
}

async fn run_server(config: AppConfig, args: CliArgs) -> Result<()> {
    tracing::info!("Initializing modules...");

    // Build config provider and resolve database options
    let config_provider = build_config_provider(config.clone());
    let db_options = resolve_db_options(&config, &args)?;

    // Run the ModKit runtime with signal-driven shutdown
    let run_options = RunOptions {
        modules_cfg: config_provider,
        db: db_options,
        shutdown: ShutdownOptions::Signals,
    };

    let result = run(run_options).await;

    // Graceful shutdown - flush any remaining traces
    #[cfg(feature = "otel")]
    modkit::telemetry::init::shutdown_tracing();

    result
}
