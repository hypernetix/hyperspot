use async_trait::async_trait;
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::ir::{DocumentBuilder, Inline, ParsedBlock, ParsedSource};
use crate::domain::parser::FileParserBackend;

/// Stub parser that provides placeholder parsing for various file types
/// This is a temporary implementation until proper parsers are integrated
pub struct StubParser;

impl StubParser {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for StubParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileParserBackend for StubParser {
    fn id(&self) -> &'static str {
        "generic_stub"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["doc", "rtf", "odt", "xls", "xlsx", "ppt", "pptx"]
    }

    async fn parse_local_path(
        &self,
        path: &Path,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let content = tokio::fs::read(path)
            .await
            .map_err(|e| DomainError::io_error(format!("Failed to read file: {e}")))?;

        let bytes = bytes::Bytes::from(content);
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        Ok(Self::parse_bytes_internal(
            file_name,
            &bytes,
            ParsedSource::LocalPath(path.display().to_string()),
        ))
    }

    async fn parse_bytes(
        &self,
        filename_hint: Option<&str>,
        _content_type: Option<&str>,
        bytes: bytes::Bytes,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let file_name = filename_hint.unwrap_or("unknown");
        Ok(Self::parse_bytes_internal(
            file_name,
            &bytes,
            ParsedSource::Uploaded {
                original_name: file_name.to_owned(),
            },
        ))
    }
}

impl StubParser {
    fn parse_bytes_internal(
        file_name: &str,
        bytes: &bytes::Bytes,
        source: ParsedSource,
    ) -> crate::domain::ir::ParsedDocument {
        // Try UTF-8 decode, fall back to base64 if that fails
        let text = if let Ok(s) = String::from_utf8(bytes.to_vec()) {
            format!(
                "[STUB PARSER] Content extracted from {} ({} bytes)\n\nRaw text preview:\n{}",
                file_name,
                bytes.len(),
                s.chars().take(500).collect::<String>()
            )
        } else {
            // Binary file, provide base64 preview
            use base64::Engine;
            let b64 =
                base64::engine::general_purpose::STANDARD.encode(&bytes[..bytes.len().min(300)]);
            format!(
                "[STUB PARSER] Binary content from {} ({} bytes)\n\nBase64 preview (first 300 bytes):\n{}",
                file_name,
                bytes.len(),
                b64
            )
        };

        let blocks = vec![ParsedBlock::Paragraph {
            inlines: vec![Inline::plain(text)],
        }];

        DocumentBuilder::new(source)
            .title(file_name)
            .original_filename(file_name)
            .content_type("application/octet-stream")
            .stub(true) // Mark this as stub output
            .blocks(blocks)
            .build()
    }
}
