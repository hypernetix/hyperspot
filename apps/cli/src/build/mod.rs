use crate::common::CommonArgs;
use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    #[arg(short = 'r', long)]
    release: bool,
    #[command(flatten)]
    common_args: CommonArgs,
}

impl BuildArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}
