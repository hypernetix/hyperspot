use clap::Args;

#[derive(Args)]
pub struct LintArgs {
    #[arg(long)]
    clippy: bool,
    #[arg(long)]
    dylint: bool,
    #[arg(long)]
    pattern: Option<String>,
}

impl LintArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}
