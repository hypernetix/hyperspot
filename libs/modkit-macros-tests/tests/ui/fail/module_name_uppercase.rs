// Test that module names with uppercase letters are rejected

use modkit::Module;

#[modkit::module(
    name = "FileParser",  // Should fail: contains uppercase letters
    capabilities = []
)]
pub struct TestModule;

impl Module for TestModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

fn main() {}
