use async_trait::async_trait;
use std::io::Write;
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::ir::{DocumentBuilder, Inline, ParsedBlock, ParsedSource};
use crate::domain::parser::FileParserBackend;

/// PDF parser that extracts text from PDF files
/// TODO: Migrate to ferrules when it's available as a library crate
pub struct PdfParser;

impl PdfParser {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for PdfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileParserBackend for PdfParser {
    fn id(&self) -> &'static str {
        "pdf"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["pdf"]
    }

    async fn parse_local_path(
        &self,
        path: &Path,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let path_buf = path.to_path_buf();

        let blocks = tokio::task::spawn_blocking(move || parse_pdf_from_path(&path_buf))
            .await
            .map_err(|e| DomainError::parse_error(format!("Task join error: {e}")))??;

        let mut builder = DocumentBuilder::new(ParsedSource::LocalPath(path.display().to_string()))
            .content_type("application/pdf")
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
        let blocks = tokio::task::spawn_blocking(move || parse_pdf_bytes(&bytes))
            .await
            .map_err(|e| DomainError::parse_error(format!("Task join error: {e}")))??;

        let source = ParsedSource::Uploaded {
            original_name: filename_hint.unwrap_or("unknown.pdf").to_owned(),
        };

        let mut builder = DocumentBuilder::new(source)
            .content_type("application/pdf")
            .blocks(blocks);

        if let Some(filename) = filename_hint {
            builder = builder.title(filename).original_filename(filename);
        }

        Ok(builder.build())
    }
}

fn parse_pdf_from_path(path: &Path) -> Result<Vec<ParsedBlock>, DomainError> {
    // Use pdf-extract for now; TODO: migrate to ferrules when available
    let text = pdf_extract::extract_text(path)
        .map_err(|e| DomainError::parse_error(format!("Failed to extract text from PDF: {e}")))?;

    Ok(text_to_blocks(&text))
}

fn parse_pdf_bytes(bytes: &[u8]) -> Result<Vec<ParsedBlock>, DomainError> {
    // Create a temporary file for pdf-extract (it requires a path)
    let mut temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| DomainError::io_error(format!("Failed to create temp file: {e}")))?;

    temp_file
        .write_all(bytes)
        .map_err(|e| DomainError::io_error(format!("Failed to write to temp file: {e}")))?;

    let text = pdf_extract::extract_text(temp_file.path())
        .map_err(|e| DomainError::parse_error(format!("Failed to extract text from PDF: {e}")))?;

    Ok(text_to_blocks(&text))
}

fn text_to_blocks(text: &str) -> Vec<ParsedBlock> {
    // Split text into paragraphs and add page breaks where appropriate
    let mut blocks = Vec::new();

    // Split by form feed (page break) character or double newlines
    for (idx, chunk) in text.split('\x0C').enumerate() {
        if idx > 0 {
            blocks.push(ParsedBlock::PageBreak);
        }

        // Split each page into paragraphs
        for para in chunk.split("\n\n") {
            let trimmed = para.trim();
            if !trimmed.is_empty() {
                // TODO: improve PDF structure extraction (headings, columns, styles, etc)
                blocks.push(ParsedBlock::Paragraph {
                    inlines: vec![Inline::plain(trimmed)],
                });
            }
        }
    }

    blocks
}
