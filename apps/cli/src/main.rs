use clap::{Parser, Subcommand};

mod build;
mod common;
mod lint;
mod run;
mod setup;
mod test;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(name = "cyberfabric")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup(setup::SetupArgs),
    Lint(lint::LintArgs),
    Test(test::TestArgs),
    Run(run::RunArgs),
    Build(build::BuildArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup(setup) => setup.run(),
        Commands::Lint(lint) => lint.run(),
        Commands::Test(test) => test.run(),
        Commands::Run(run) => run.run(),
        Commands::Build(build) => build.run(),
    }
}
