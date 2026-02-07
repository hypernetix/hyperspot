// Test that module names with consecutive hyphens are rejected

use modkit::Module;

#[modkit::module(
    name = "file--parser",  // Should fail: consecutive hyphens
    capabilities = []
)]
pub struct TestModule;

impl Module for TestModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

fn main() {}
