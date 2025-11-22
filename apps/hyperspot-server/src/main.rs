use anyhow::Result;
use clap::{Parser, Subcommand};
use figment::Figment;
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
    let _ = std::any::type_name::<grpc_hub::GrpcHub>();
    let _ = std::any::type_name::<directory_service::DirectoryServiceModule>();

    #[cfg(feature = "users-info-example")]
    let _ = std::any::type_name::<users_info::UsersInfo>();
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

    // Build OpenTelemetry layer before logging
    #[cfg(feature = "otel")]
    let otel_layer = config
        .tracing
        .as_ref()
        .and_then(modkit::telemetry::init::init_tracing);
    #[cfg(not(feature = "otel"))]
    let otel_layer = None;

    // Initialize logging + otel in one Registry
    let logging_config = config.logging.as_ref().cloned().unwrap_or_default();
    runtime::logging::init_logging_unified(
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
    println!("Effective configuration:\n{:#?}", config.server);

    // Print config and exit if requested
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
    let _home_dir = PathBuf::from(&config.server.home_dir);

    // Configure DB options: DbManager or no-DB.
    let db_options = if config.database.is_some() {
        if args.mock {
            tracing::info!("Mock mode enabled: using in-memory SQLite for all modules");
            // For mock mode, create a simple figment with mock database config
            let mock_figment = create_mock_figment(&config);
            let home_dir = PathBuf::from(&config.server.home_dir);
            let db_manager = Arc::new(modkit_db::DbManager::from_figment(mock_figment, home_dir)?);
            DbOptions::Manager(db_manager)
        } else {
            tracing::info!("Using DbManager with Figment-based configuration");

            // Create Figment from the current configuration
            let figment = create_figment_from_config(&config)?;
            let home_dir = PathBuf::from(&config.server.home_dir);
            let db_manager = Arc::new(modkit_db::DbManager::from_figment(figment, home_dir)?);

            DbOptions::Manager(db_manager)
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

    let result = run(run_options).await;

    // Graceful shutdown - flush any remaining traces
    #[cfg(feature = "otel")]
    modkit::telemetry::init::shutdown_tracing();

    result
}

async fn check_config(config: AppConfig) -> Result<()> {
    tracing::info!("Checking configuration…");
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

    // Create a mock configuration where all modules get in-memory SQLite
    let mut mock_config = config.clone();

    // Override all module database configurations to use in-memory SQLite
    for module_value in mock_config.modules.values_mut() {
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

    Figment::new().merge(Serialized::defaults(mock_config))
}
