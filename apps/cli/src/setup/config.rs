use crate::common::CommonArgs;
use clap::Args;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(flatten)]
    common_args: CommonArgs,
}

impl ConfigArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}
