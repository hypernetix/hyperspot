use async_trait::async_trait;
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::ir::ParsedDocument;

/// Trait for document parser backends that can handle specific file types
#[async_trait]
pub trait FileParserBackend: Send + Sync {
    /// Unique identifier for this parser
    fn id(&self) -> &'static str;

    /// File extensions this parser supports (without the dot)
    fn supported_extensions(&self) -> &'static [&'static str];

    /// Parse a file from a local path
    async fn parse_local_path(&self, path: &Path) -> Result<ParsedDocument, DomainError>;

    /// Parse a file from bytes
    async fn parse_bytes(
        &self,
        filename_hint: Option<&str>,
        content_type: Option<&str>,
        bytes: bytes::Bytes,
    ) -> Result<ParsedDocument, DomainError>;
}
