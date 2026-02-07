// Test that module names using snake_case are rejected

use modkit::Module;

#[modkit::module(
    name = "file_parser",  // Should fail: uses snake_case instead of kebab-case
    capabilities = []
)]
pub struct TestModule;

impl Module for TestModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

fn main() {}
