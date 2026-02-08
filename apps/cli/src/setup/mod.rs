use clap::{Args, Subcommand};

mod config;
mod template;
mod tools;

#[derive(Args)]
pub struct SetupArgs {
    #[command(subcommand)]
    command: SetupCommand,
}

impl SetupArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        self.command.run()
    }
}

#[derive(Subcommand)]
pub enum SetupCommand {
    Tools(tools::ToolsArgs),
    Template(template::TemplateArgs),
    Config(config::ConfigArgs),
}

impl SetupCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            SetupCommand::Tools(args) => args.run(),
            SetupCommand::Template(args) => args.run(),
            SetupCommand::Config(args) => args.run(),
        }
    }
}
