use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct TemplateArgs {
    #[command(subcommand)]
    command: TemplateCommand,
}

impl TemplateArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.command.run()
    }
}

#[derive(Subcommand)]
pub enum TemplateCommand {
    Init(InitArgs),
    Add(AddArgs),
}

impl TemplateCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            TemplateCommand::Init(args) => args.run(),
            TemplateCommand::Add(args) => args.run(),
        }
    }
}

#[derive(Args)]
pub struct InitArgs {
    path: PathBuf,
}

impl InitArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}

#[derive(Args)]
pub struct AddArgs {
    module: String,
}

impl AddArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}
