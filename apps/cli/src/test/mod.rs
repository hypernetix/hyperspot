use clap::Args;

#[derive(Args)]
pub struct TestArgs {
    #[arg(long)]
    e2e: bool,
    #[arg(long)]
    module: Option<String>,
    #[arg(long)]
    coverage: bool,
}

impl TestArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}
