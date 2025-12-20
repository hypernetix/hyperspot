use modkit_macros::module;
use tokio_util::sync::CancellationToken;
use anyhow::Result;

#[module(name="x", capabilities=[stateful], lifecycle(entry="serve", await_ready))]
pub struct X;

impl X {
    // Wrong signature: missing ReadySignal parameter → the generated call won't match.
    async fn serve(&self, _cancel: CancellationToken) -> Result<()> { Ok(()) }
}

#[async_trait::async_trait]
impl modkit::Module for X {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> { Ok(()) }
}

fn main() {}
