use async_trait::async_trait;
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::ir::{DocumentBuilder, Inline, ParsedBlock, ParsedSource};
use crate::domain::parser::FileParserBackend;

/// Plain text parser that handles text files
pub struct PlainTextParser;

impl PlainTextParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlainTextParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileParserBackend for PlainTextParser {
    fn id(&self) -> &'static str {
        "plain_text"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["txt", "log", "md"]
    }

    async fn parse_local_path(
        &self,
        path: &Path,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let content = tokio::fs::read(path)
            .await
            .map_err(|e| DomainError::io_error(format!("Failed to read file: {}", e)))?;

        let text = String::from_utf8(content)
            .map_err(|e| DomainError::parse_error(format!("Failed to decode UTF-8: {}", e)))?;

        let blocks = text_to_blocks(&text);

        let mut builder = DocumentBuilder::new(ParsedSource::LocalPath(path.display().to_string()))
            .content_type("text/plain")
            .blocks(blocks);

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            builder = builder.title(filename).original_filename(filename);
        }

        Ok(builder.build())
    }

    async fn parse_bytes(
        &self,
        filename_hint: Option<&str>,
        _content_type: Option<&str>,
        bytes: bytes::Bytes,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let text = String::from_utf8(bytes.to_vec())
            .map_err(|e| DomainError::parse_error(format!("Failed to decode UTF-8: {}", e)))?;

        let blocks = text_to_blocks(&text);

        let source = ParsedSource::Uploaded {
            original_name: filename_hint.unwrap_or("unknown.txt").to_string(),
        };

        let mut builder = DocumentBuilder::new(source)
            .content_type("text/plain")
            .blocks(blocks);

        if let Some(filename) = filename_hint {
            builder = builder.title(filename).original_filename(filename);
        }

        Ok(builder.build())
    }
}

/// Convert plain text into blocks by splitting on double newlines
fn text_to_blocks(text: &str) -> Vec<ParsedBlock> {
    text.split("\n\n")
        .filter(|para| !para.trim().is_empty())
        .map(|para| ParsedBlock::Paragraph {
            inlines: vec![Inline::plain(para.trim())],
        })
        .collect()
}
