use async_trait::async_trait;
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::ir::{DocumentBuilder, ParsedBlock, ParsedSource};
use crate::domain::parser::FileParserBackend;

/// DOCX parser that extracts text from Word documents
pub struct DocxParser;

impl DocxParser {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for DocxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileParserBackend for DocxParser {
    fn id(&self) -> &'static str {
        "docx"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["docx"]
    }

    async fn parse_local_path(
        &self,
        path: &Path,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let path_buf = path.to_path_buf();

        let blocks =
            tokio::task::spawn_blocking(move || -> Result<Vec<ParsedBlock>, DomainError> {
                let docx_file = docx_rust::DocxFile::from_file(&path_buf).map_err(|e| {
                    DomainError::parse_error(format!("Failed to open DOCX file: {e}"))
                })?;

                let docx = docx_file
                    .parse()
                    .map_err(|e| DomainError::parse_error(format!("Failed to parse DOCX: {e}")))?;

                Ok(extract_blocks_from_docx(&docx))
            })
            .await
            .map_err(|e| DomainError::parse_error(format!("Task join error: {e}")))??;

        let mut builder = DocumentBuilder::new(ParsedSource::LocalPath(path.display().to_string()))
            .content_type("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
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
                // docx-rust requires file path or Read trait, so we use a temporary file
                let mut temp_file = tempfile::NamedTempFile::new().map_err(|e| {
                    DomainError::io_error(format!("Failed to create temp file: {e}"))
                })?;

                std::io::Write::write_all(&mut temp_file, &bytes).map_err(|e| {
                    DomainError::io_error(format!("Failed to write to temp file: {e}"))
                })?;

                let temp_path = temp_file.path();
                let docx_file = docx_rust::DocxFile::from_file(temp_path).map_err(|e| {
                    DomainError::parse_error(format!("Failed to open DOCX file: {e}"))
                })?;

                let docx = docx_file
                    .parse()
                    .map_err(|e| DomainError::parse_error(format!("Failed to parse DOCX: {e}")))?;

                Ok(extract_blocks_from_docx(&docx))
            })
            .await
            .map_err(|e| DomainError::parse_error(format!("Task join error: {e}")))??;

        let source = ParsedSource::Uploaded {
            original_name: filename_hint.unwrap_or("unknown.docx").to_string(),
        };

        let mut builder = DocumentBuilder::new(source)
            .content_type("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
            .blocks(blocks);

        if let Some(filename) = filename_hint {
            builder = builder.title(filename).original_filename(filename);
        }

        Ok(builder.build())
    }
}

fn extract_blocks_from_docx(docx: &docx_rust::Docx) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();

    for body_content in &docx.document.body.content {
        match body_content {
            docx_rust::document::BodyContent::Paragraph(paragraph) => {
                if let Some(block) = extract_paragraph_block(paragraph) {
                    blocks.push(block);
                }
            }
            docx_rust::document::BodyContent::Table(table) => {
                tracing::trace!("Found table at top level with {} rows", table.rows.len());
                blocks.push(extract_table_block(table));
            }
            // Ignore other body content types for now
            _ => {}
        }
    }

    blocks
}

fn extract_paragraph_block(paragraph: &docx_rust::document::Paragraph) -> Option<ParsedBlock> {
    let inlines = extract_inlines_from_paragraph(paragraph);

    if inlines.is_empty() {
        return None;
    }

    // Detect heading level from paragraph properties
    let heading_level = detect_heading_level(paragraph);

    if let Some(level) = heading_level {
        Some(ParsedBlock::Heading { level, inlines })
    } else {
        Some(ParsedBlock::Paragraph { inlines })
    }
}

fn detect_heading_level(paragraph: &docx_rust::document::Paragraph) -> Option<u8> {
    fn to_level_u8(lvl: i64) -> Option<u8> {
        u8::try_from(lvl.clamp(1, 6)).ok()
    }

    paragraph.property.as_ref().and_then(|property| {
        let fallback = || {
            property
                .outline_lvl
                .as_ref()
                .map(|lvl| lvl.value + 1)
                .map(|lvl| lvl.clamp(1, 6))
                .and_then(|lvl| to_level_u8(lvl as i64))
        };

        // Check paragraph style for heading markers
        property
            .style_id
            .as_ref()
            .map(|style_id| style_id.value.to_lowercase())
            .filter(|style_id| style_id.starts_with("heading"))
            .and_then(|style_id| style_id.chars().nth(7))
            .and_then(|d| d.to_digit(10))
            .map(|lvl| lvl.clamp(1, 6))
            .and_then(|lvl| to_level_u8(i64::from(lvl)))
            .and_then(|_| fallback())
    })
}

fn extract_inlines_from_paragraph(
    paragraph: &docx_rust::document::Paragraph,
) -> Vec<crate::domain::ir::Inline> {
    use crate::domain::ir::{Inline, InlineStyle};

    let mut inlines = Vec::new();
    let mut current_text = String::new();
    let mut current_style = InlineStyle::default();

    for content in &paragraph.content {
        // TODO: Handle hyperlinks when docx-rust supports them
        // For now, hyperlinks are not in the ParagraphContent enum
        let docx_rust::document::ParagraphContent::Run(run) = content else {
            // Handle other paragraph content types as needed
            continue;
        };

        let run_style = extract_style_from_run(run);

        for run_content in &run.content {
            match run_content {
                docx_rust::document::RunContent::Text(text_elem) => {
                    // If style changed, flush current text
                    if run_style != current_style && !current_text.is_empty() {
                        inlines.push(Inline::Text {
                            text: std::mem::take(&mut current_text),
                            style: current_style.clone(),
                        });
                    }

                    current_style = run_style.clone();
                    current_text.push_str(&text_elem.text);
                }
                docx_rust::document::RunContent::Tab(_) => {
                    current_text.push('\t');
                }
                docx_rust::document::RunContent::Break(_) => {
                    current_text.push('\n');
                }
                _ => {
                    // Handle other run content types as needed
                }
            }
        }
    }

    // Flush any remaining text
    if !current_text.is_empty() {
        inlines.push(Inline::Text {
            text: current_text,
            style: current_style,
        });
    }

    inlines
}

fn extract_style_from_run(run: &docx_rust::document::Run) -> crate::domain::ir::InlineStyle {
    use crate::domain::ir::InlineStyle;

    let mut style = InlineStyle::default();

    if let Some(ref property) = run.property {
        if let Some(ref bold) = property.bold {
            style.bold = bold.value.unwrap_or(false);
        }
        if let Some(ref italics) = property.italics {
            style.italic = italics.value.unwrap_or(false);
        }
        if let Some(ref underline) = property.underline {
            style.underline = underline.val.is_some();
        }
        if let Some(ref strike) = property.strike {
            style.strike = strike.value.unwrap_or(false);
        }
        // Check for monospace fonts as code indicator
        if let Some(ref fonts) = property.fonts {
            if let Some(ref ascii) = fonts.ascii {
                let font_lower = ascii.to_lowercase();
                if font_lower.contains("consolas")
                    || font_lower.contains("courier")
                    || font_lower.contains("mono")
                {
                    style.code = true;
                }
            }
        }
    }

    style
}

fn extract_table_block(table: &docx_rust::document::Table) -> ParsedBlock {
    use crate::domain::ir::{Inline, TableBlock, TableCell, TableRow};
    use docx_rust::formatting::OnOffOnlyType;

    let mut rows = Vec::new();
    let mut total_cells = 0;

    for row in &table.rows {
        // Check if this is a header row
        // OnOffOnlyType::On indicates a header row
        let is_header = match &row.property.table_header {
            Some(table_header) => matches!(table_header.value, Some(OnOffOnlyType::On)),
            None => false,
        };

        let mut cells = Vec::new();

        for cell_content in &row.cells {
            // TableRow.cells contains TableRowContent enum
            if let docx_rust::document::TableRowContent::TableCell(cell) = cell_content {
                total_cells += 1;
                let mut cell_blocks = Vec::new();

                for content in &cell.content {
                    match content {
                        docx_rust::document::TableCellContent::Paragraph(para) => {
                            if let Some(block) = extract_paragraph_block(para) {
                                cell_blocks.push(block);
                            }
                        } // Note: Nested tables are not yet supported in docx-rs
                          // The Table variant in TableCellContent is commented out in the library
                          // If/when docx-rs adds support, we can uncomment the code below:
                          //
                          // docx_rust::document::TableCellContent::Table(nested_table) => {
                          //     tracing::debug!("DOCX debug: found nested table in row {} with {} rows",
                          //         row_idx, nested_table.rows.len());
                          //     cell_blocks.push(extract_table_block(nested_table));
                          // }
                    }
                }

                // If no blocks extracted, add an empty paragraph with a plain text space
                // This ensures cells always have content for proper rendering
                if cell_blocks.is_empty() {
                    cell_blocks.push(ParsedBlock::Paragraph {
                        inlines: vec![Inline::plain("")],
                    });
                }

                cells.push(TableCell {
                    blocks: cell_blocks,
                });
            }
        }

        rows.push(TableRow { is_header, cells });
    }

    tracing::trace!(
        "Extracted table with {} rows, {} total cells",
        rows.len(),
        total_cells
    );

    ParsedBlock::Table(TableBlock { rows })
}
