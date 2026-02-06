// Test that valid kebab-case module names are accepted

#[modkit::module(
    name = "file-parser",  // Valid kebab-case
    capabilities = []
)]
#[derive(Default)]
pub struct FileParserModule;

#[async_trait::async_trait]
impl modkit::Module for FileParserModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

#[modkit::module(
    name = "simple-user-settings",  // Valid kebab-case with multiple hyphens
    capabilities = []
)]
#[derive(Default)]
pub struct SettingsModule;

#[async_trait::async_trait]
impl modkit::Module for SettingsModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

#[modkit::module(
    name = "api-gateway",  // Valid kebab-case
    capabilities = []
)]
#[derive(Default)]
pub struct ApiGatewayModule;

#[async_trait::async_trait]
impl modkit::Module for ApiGatewayModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

#[modkit::module(
    name = "module-v2",  // Valid kebab-case with digit
    capabilities = []
)]
#[derive(Default)]
pub struct ModuleV2;

#[async_trait::async_trait]
impl modkit::Module for ModuleV2 {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

#[modkit::module(
    name = "system",  // Valid single word (no hyphens needed)
    capabilities = []
)]
#[derive(Default)]
pub struct SystemModule;

#[async_trait::async_trait]
impl modkit::Module for SystemModule {
    async fn init(&self, _ctx: &modkit::ModuleCtx) -> anyhow::Result<()> {
        Ok(())
    }
}

fn main() {}
