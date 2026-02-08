use clap::Args;

#[derive(Args)]
pub struct ToolsArgs {
    #[arg(long)]
    upgrade: bool,
    #[arg(long, value_delimiter = ',')]
    install: Option<Vec<String>>,
    #[arg(long)]
    install_yolo: bool,
    #[arg(short, long)]
    verbose: bool,
}

impl ToolsArgs {
    pub fn run(&self) -> anyhow::Result<()> {
        unimplemented!("Not implemented yet")
    }
}
