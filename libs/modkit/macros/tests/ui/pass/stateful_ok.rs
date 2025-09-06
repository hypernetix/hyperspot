// Minimal stateful module with lifecycle (no ready)
use modkit_macros::module;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

#[derive(Default)]
#[module(name = "demo", caps = [stateful], lifecycle(entry = "serve", stop_timeout = "1s"))]
pub struct Demo;

impl Demo {
    async fn serve(&self, _cancel: CancellationToken) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl modkit::Module for Demo {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

fn main() {}
