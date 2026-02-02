use async_trait::async_trait;
use pptx_to_md::{ListElement, ParserConfig, PptxContainer, SlideElement, TableElement};
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::ir::{
    DocumentBuilder, Inline, ParsedBlock, ParsedSource, TableBlock, TableCell, TableRow,
};
use crate::domain::parser::FileParserBackend;

/// PPTX parser that extracts text from `PowerPoint` presentations using `pptx-to-md`
pub struct PptxParser;

impl PptxParser {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for PptxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileParserBackend for PptxParser {
    fn id(&self) -> &'static str {
        "pptx"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["pptx"]
    }

    async fn parse_local_path(
        &self,
        path: &Path,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let path_buf = path.to_path_buf();

        let blocks =
            tokio::task::spawn_blocking(move || -> Result<Vec<ParsedBlock>, DomainError> {
                parse_pptx_from_path(&path_buf)
            })
            .await
            .map_err(|e| DomainError::parse_error(format!("Task join error: {e}")))??;

        let mut builder = DocumentBuilder::new(ParsedSource::LocalPath(path.display().to_string()))
            .content_type(
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            )
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
        let blocks =
            tokio::task::spawn_blocking(move || -> Result<Vec<ParsedBlock>, DomainError> {
                parse_pptx_from_bytes(&bytes)
            })
            .await
            .map_err(|e| DomainError::parse_error(format!("Task join error: {e}")))??;

        let filename = filename_hint.unwrap_or("unknown.pptx");

        let source = ParsedSource::Uploaded {
            original_name: filename.to_owned(),
        };

        let mut builder = DocumentBuilder::new(source)
            .content_type(
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            )
            .blocks(blocks);

        builder = builder.title(filename).original_filename(filename);

        Ok(builder.build())
    }
}

fn parse_pptx_from_path(path: &Path) -> Result<Vec<ParsedBlock>, DomainError> {
    let config = ParserConfig::builder()
        .extract_images(false)
        .include_slide_comment(false)
        .build();

    let mut container = PptxContainer::open(path, config)
        .map_err(|e| DomainError::parse_error(format!("Failed to open PPTX: {e}")))?;

    let slides = container
        .parse_all()
        .map_err(|e| DomainError::parse_error(format!("Failed to parse PPTX slides: {e}")))?;

    Ok(extract_blocks_from_slides(&slides))
}

// TODO(scalability): `pptx-to-md` only supports file paths, not `Read` streams.
// At scale, writing temp files for every parse is inefficient (disk I/O, cleanup overhead).
// Consider contributing stream support upstream or switching to a library that supports in-memory parsing.
fn parse_pptx_from_bytes(bytes: &[u8]) -> Result<Vec<ParsedBlock>, DomainError> {
    let mut temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| DomainError::io_error(format!("Failed to create temp file: {e}")))?;

    std::io::Write::write_all(&mut temp_file, bytes)
        .map_err(|e| DomainError::io_error(format!("Failed to write to temp file: {e}")))?;

    std::io::Write::flush(&mut temp_file)
        .map_err(|e| DomainError::io_error(format!("Failed to flush temp file: {e}")))?;

    let temp_path = temp_file.into_temp_path();
    parse_pptx_from_path(&temp_path)
}

fn extract_blocks_from_slides(slides: &[pptx_to_md::Slide]) -> Vec<ParsedBlock> {
    let mut blocks = Vec::with_capacity(slides.len());

    for (slide_idx, slide) in slides.iter().enumerate() {
        // Add slide separator as heading
        blocks.push(ParsedBlock::Heading {
            level: 2,
            inlines: vec![Inline::plain(format!("Slide {}", slide_idx + 1))],
        });

        // Process slide elements
        for element in &slide.elements {
            match element {
                SlideElement::Text(text, _pos) => {
                    let content: String = text
                        .runs
                        .iter()
                        .map(|run| run.text.as_str())
                        .collect::<Vec<_>>()
                        .join("");
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        blocks.push(ParsedBlock::Paragraph {
                            inlines: vec![Inline::plain(trimmed)],
                        });
                    }
                }
                SlideElement::Table(table, _pos) => {
                    if let Some(table_block) = convert_pptx_table(table) {
                        blocks.push(table_block);
                    }
                }
                SlideElement::List(list, _pos) => {
                    blocks.extend(convert_pptx_list(list));
                }
                SlideElement::Image(..) | SlideElement::Unknown => {
                    // Skip images and unknown elements - focused on text extraction
                }
            }
        }

        // Add page break between slides (except after last slide)
        if slide_idx < slides.len() - 1 {
            blocks.push(ParsedBlock::PageBreak);
        }
    }

    blocks
}

fn convert_pptx_table(table: &TableElement) -> Option<ParsedBlock> {
    if table.rows.is_empty() {
        return None;
    }

    let mut rows = Vec::with_capacity(table.rows.len());

    for (row_idx, row) in table.rows.iter().enumerate() {
        let is_header = row_idx == 0;
        let mut cells = Vec::with_capacity(row.cells.len());

        for cell in &row.cells {
            let cell_text: String = cell
                .runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<Vec<_>>()
                .join("");
            cells.push(TableCell {
                blocks: vec![ParsedBlock::Paragraph {
                    inlines: vec![Inline::plain(cell_text.trim())],
                }],
            });
        }

        rows.push(TableRow { is_header, cells });
    }

    Some(ParsedBlock::Table(TableBlock { rows }))
}

fn convert_pptx_list(list: &ListElement) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();

    for item in &list.items {
        // Extract text from runs
        let text: String = item
            .runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        let trimmed = text.trim();

        if !trimmed.is_empty() {
            blocks.push(ParsedBlock::ListItem {
                level: u8::try_from(item.level).unwrap_or(0),
                ordered: item.is_ordered,
                blocks: vec![ParsedBlock::Paragraph {
                    inlines: vec![Inline::plain(trimmed)],
                }],
            });
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_id() {
        let parser = PptxParser::new();
        assert_eq!(parser.id(), "pptx");
    }

    #[test]
    fn test_supported_extensions() {
        let parser = PptxParser::new();
        assert_eq!(parser.supported_extensions(), &["pptx"]);
    }
}
