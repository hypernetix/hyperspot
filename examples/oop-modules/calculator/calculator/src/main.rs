//! OoP binary for calculator module.
//!
//! This binary runs the calculator module as an out-of-process service,
//! registering with the DirectoryService and exposing the gRPC service.
//!
//! Configuration is loaded from:
//! 1. --config CLI argument (passed by master host)
//! 2. MODULE_CONFIG_PATH environment variable (fallback)

mod registered_modules;

shadow_rs::shadow!(shadow);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use clap::Parser;
    use modkit::bootstrap::oop::{OopRunOptions, run_oop_with_options};

    /// OoP calculator module
    #[derive(Parser)]
    #[command(name = "calculator-oop")]
    struct Cli {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<std::path::PathBuf>,

        /// Log verbosity level (-v info, -vv debug, -vvv trace)
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    }

    let cli = Cli::parse();

    // Use CLI config if provided, otherwise fall back to Default (which checks MODULE_CONFIG_PATH env var)
    let opts = OopRunOptions {
        module_name: "calculator".to_string(),
        verbose: cli.verbose,
        config_path: cli.config,
        build_metadata: Some(cf_modkit_build::build_metadata!()),
        ..Default::default()
    };

    run_oop_with_options(opts).await
}
