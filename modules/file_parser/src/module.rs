use std::sync::Arc;

use async_trait::async_trait;
use modkit::api::OpenApiRegistry;
use modkit::{Module, ModuleCtx, RestApiCapability};
use tracing::{debug, info};

use crate::config::FileParserConfig;
use crate::domain::parsers::{DocxParser, HtmlParser, PdfParser, PlainTextParser, StubParser};
use crate::domain::service::{FileParserService, ServiceConfig};

/// Main module struct for file parsing
#[modkit::module(
    name = "file_parser",
    capabilities = [rest]
)]
pub struct FileParserModule {
    // Keep the service behind ArcSwap for cheap read-mostly access.
    service: arc_swap::ArcSwapOption<FileParserService>,
}

impl Default for FileParserModule {
    fn default() -> Self {
        Self {
            service: arc_swap::ArcSwapOption::from(None),
        }
    }
}

impl Clone for FileParserModule {
    fn clone(&self) -> Self {
        Self {
            service: arc_swap::ArcSwapOption::new(self.service.load().as_ref().map(Clone::clone)),
        }
    }
}

#[async_trait]
impl Module for FileParserModule {
    #[allow(clippy::cast_possible_truncation)]
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        const BYTES_IN_MB: u64 = 1024_u64 * 1024;

        info!("Initializing file_parser module");

        // Load module configuration
        let cfg: FileParserConfig = ctx.config()?;
        debug!(
            "Loaded file_parser config: max_file_size_mb={}, download_timeout_secs={}",
            cfg.max_file_size_mb, cfg.download_timeout_secs
        );

        // Build parser backends
        let parsers: Vec<Arc<dyn crate::domain::parser::FileParserBackend>> = vec![
            Arc::new(PlainTextParser::new()),
            Arc::new(HtmlParser::new()),
            Arc::new(PdfParser::new()),
            Arc::new(DocxParser::new()),
            Arc::new(StubParser::new()),
        ];

        info!("Registered {} parser backends", parsers.len());

        // Create service config from module config
        let service_config = ServiceConfig {
            max_file_size_bytes: usize::try_from(cfg.max_file_size_mb * BYTES_IN_MB)
                .unwrap_or(usize::MAX),
            download_timeout_secs: cfg.download_timeout_secs,
        };

        // Create file parser service
        let file_parser_service = Arc::new(FileParserService::new(parsers, service_config));

        // Store service for REST usage
        self.service.store(Some(file_parser_service));

        info!("FileParserService initialized successfully");
        Ok(())
    }
}

impl RestApiCapability for FileParserModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        info!("Registering file_parser REST routes");

        let service = self
            .service
            .load()
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Service not initialized"))?
            .clone();

        let router = crate::api::rest::routes::register_routes(router, openapi, service);

        info!("File parser REST routes registered successfully");
        Ok(router)
    }
}
