// Test that module names starting with hyphen are rejected

use modkit::Module;

#[modkit::module(
    name = "-parser",  // Should fail: starts with hyphen
    capabilities = []
)]
pub struct TestModule;

impl Module for TestModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

fn main() {}
