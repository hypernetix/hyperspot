use crate::domain::error::DomainError;
use crate::domain::ir::{
    DocumentBuilder, Inline, ParsedBlock, ParsedSource, TableBlock, TableCell, TableRow,
};
use crate::domain::parser::FileParserBackend;
use async_trait::async_trait;
use calamine::{Data, Reader, Xls, Xlsb, Xlsx, open_workbook_auto};
use std::path::Path;

/// XLSX/XLS parser that extracts data from Excel spreadsheets using calamine
pub struct XlsxParser;

/// File extension constants
const EXT_XLS: &str = "xls";
const EXT_XLSX: &str = "xlsx";
const EXT_XLSM: &str = "xlsm";
const EXT_XLSB: &str = "xlsb";

/// Supported file extensions for Excel formats
const SUPPORTED_EXTENSIONS: &[&str] = &[EXT_XLSX, EXT_XLS, EXT_XLSM, EXT_XLSB];

/// MIME type constants
const MIME_TYPE_XLS: &str = "application/vnd.ms-excel";
const MIME_TYPE_XLSX: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
const MIME_TYPE_XLSM: &str = "application/vnd.ms-excel.sheet.macroEnabled.12";
const MIME_TYPE_XLSB: &str = "application/vnd.ms-excel.sheet.binary.macroEnabled.12";

/// Default MIME type for Excel files
const DEFAULT_MIME_TYPE: &str = MIME_TYPE_XLSX;

impl XlsxParser {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Determine MIME type from file extension
    fn mime_type_from_extension(extension: &str) -> Option<&'static str> {
        match extension.to_lowercase().as_str() {
            EXT_XLS => Some(MIME_TYPE_XLS),
            EXT_XLSX => Some(MIME_TYPE_XLSX),
            EXT_XLSM => Some(MIME_TYPE_XLSM),
            EXT_XLSB => Some(MIME_TYPE_XLSB),
            _ => None,
        }
    }

    /// Determine MIME type from filename or provided content type
    fn determine_mime_type(
        filename_hint: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<String, DomainError> {
        // Priority 1: Use provided content-type if it matches an Excel type
        if let Some(ct) = content_type {
            let is_excel_type = matches!(
                ct,
                MIME_TYPE_XLS | MIME_TYPE_XLSX | MIME_TYPE_XLSM | MIME_TYPE_XLSB
            );
            if is_excel_type {
                return Ok(ct.to_owned());
            }
        }

        // Priority 2: Infer from filename extension
        if let Some(filename) = filename_hint
            && let Some(ext) = Path::new(filename).extension().and_then(|s| s.to_str())
            && let Some(mime) = Self::mime_type_from_extension(ext)
        {
            return Ok(mime.to_owned());
        }

        Err(DomainError::unsupported_file_type(
            "Unable to determine Excel format",
        ))
    }
}

impl Default for XlsxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileParserBackend for XlsxParser {
    fn id(&self) -> &'static str {
        EXT_XLSX
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        SUPPORTED_EXTENSIONS
    }

    async fn parse_local_path(
        &self,
        path: &Path,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let path_buf = path.to_path_buf();

        let blocks =
            tokio::task::spawn_blocking(move || -> Result<Vec<ParsedBlock>, DomainError> {
                parse_spreadsheet_from_path(&path_buf)
            })
            .await
            .map_err(|e| DomainError::parse_error(format!("Task join error: {e}")))??;

        let filename = path.file_name().and_then(|s| s.to_str());
        let content_type = Self::determine_mime_type(filename, None)
            .unwrap_or_else(|_| DEFAULT_MIME_TYPE.to_owned());

        let mut builder = DocumentBuilder::new(ParsedSource::LocalPath(path.display().to_string()))
            .content_type(content_type)
            .blocks(blocks);

        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            builder = builder.title(filename).original_filename(filename);
        }

        Ok(builder.build())
    }

    async fn parse_bytes(
        &self,
        filename_hint: Option<&str>,
        content_type: Option<&str>,
        bytes: bytes::Bytes,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        let filename = filename_hint.unwrap_or("unknown.xlsx").to_owned();

        // Determine MIME type
        let mime_type = Self::determine_mime_type(Some(&filename), content_type)?;

        let blocks =
            tokio::task::spawn_blocking(move || -> Result<Vec<ParsedBlock>, DomainError> {
                parse_spreadsheet_from_bytes(&bytes)
            })
            .await
            .map_err(|e| DomainError::parse_error(format!("Task join error: {e}")))??;

        let source = ParsedSource::Uploaded {
            original_name: filename.clone(),
        };

        let mut builder = DocumentBuilder::new(source)
            .content_type(mime_type)
            .blocks(blocks);

        builder = builder.title(&filename).original_filename(&filename);

        Ok(builder.build())
    }
}

fn parse_spreadsheet_from_path(path: &Path) -> Result<Vec<ParsedBlock>, DomainError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|e| DomainError::parse_error(format!("Failed to open spreadsheet: {e}")))?;

    Ok(extract_blocks_from_workbook(&mut workbook))
}

fn parse_spreadsheet_from_bytes(bytes: &[u8]) -> Result<Vec<ParsedBlock>, DomainError> {
    let cursor = std::io::Cursor::new(bytes);

    // Try XLSX first as it's the most common
    if let Ok(mut workbook) = Xlsx::new(cursor.clone()) {
        return Ok(extract_blocks_from_workbook(&mut workbook));
    }

    // Try legacy XLS
    if let Ok(mut workbook) = Xls::new(cursor.clone()) {
        return Ok(extract_blocks_from_workbook(&mut workbook));
    }

    // Try binary XLSB
    if let Ok(mut workbook) = Xlsb::new(cursor) {
        return Ok(extract_blocks_from_workbook(&mut workbook));
    }

    Err(DomainError::parse_error(
        "Failed to parse as any supported Excel format (XLSX, XLS, XLSB)",
    ))
}

fn extract_blocks_from_workbook<RS: std::io::Read + std::io::Seek, R: Reader<RS>>(
    workbook: &mut R,
) -> Vec<ParsedBlock> {
    let mut blocks = Vec::new();
    let sheet_names = workbook.sheet_names();

    for sheet_name in sheet_names {
        // Add sheet name as a heading
        blocks.push(ParsedBlock::Heading {
            level: 2,
            inlines: vec![Inline::plain(&sheet_name)],
        });

        // Get the worksheet range
        let range = match workbook.worksheet_range(&sheet_name) {
            Ok(range) => range,
            Err(e) => {
                tracing::warn!("Failed to read sheet '{}': {:?}", sheet_name, e);
                continue;
            }
        };

        // Convert range to table block
        if let Some(table_block) = range_to_table_block(&range) {
            blocks.push(table_block);
        }
    }

    blocks
}

fn range_to_table_block(range: &calamine::Range<Data>) -> Option<ParsedBlock> {
    let height = range.height();
    let width = range.width();

    if height == 0 || width == 0 {
        return None;
    }

    let mut rows = Vec::with_capacity(height);

    for (row_idx, row) in range.rows().enumerate() {
        let is_header = row_idx == 0;
        let mut cells = Vec::with_capacity(width);

        for cell in row {
            let text = cell_to_string(cell);
            cells.push(TableCell {
                blocks: vec![ParsedBlock::Paragraph {
                    inlines: vec![Inline::plain(text)],
                }],
            });
        }

        rows.push(TableRow { is_header, cells });
    }

    Some(ParsedBlock::Table(TableBlock { rows }))
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) | Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            // Format floats nicely - remove trailing zeros
            if f.fract() == 0.0 {
                format!("{f:.0}")
            } else {
                format!("{f}")
            }
        }
        Data::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_owned(),
        Data::DateTime(dt) => {
            // calamine DateTime is days since 1899-12-30
            // ExcelDateTime implements Display
            format!("{dt}")
        }
        Data::Error(e) => format!("#ERROR: {e:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_type_from_extension() {
        assert_eq!(
            XlsxParser::mime_type_from_extension("xls"),
            Some("application/vnd.ms-excel")
        );
        assert_eq!(
            XlsxParser::mime_type_from_extension("XLS"),
            Some("application/vnd.ms-excel")
        );
        assert_eq!(
            XlsxParser::mime_type_from_extension("xlsx"),
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        );
        assert_eq!(
            XlsxParser::mime_type_from_extension("XLSX"),
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        );
        assert_eq!(
            XlsxParser::mime_type_from_extension("xlsm"),
            Some("application/vnd.ms-excel.sheet.macroEnabled.12")
        );
        assert_eq!(
            XlsxParser::mime_type_from_extension("XLSM"),
            Some("application/vnd.ms-excel.sheet.macroEnabled.12")
        );
        assert_eq!(
            XlsxParser::mime_type_from_extension("xlsb"),
            Some("application/vnd.ms-excel.sheet.binary.macroEnabled.12")
        );
        assert_eq!(
            XlsxParser::mime_type_from_extension("XLSB"),
            Some("application/vnd.ms-excel.sheet.binary.macroEnabled.12")
        );
        assert_eq!(XlsxParser::mime_type_from_extension("pdf"), None);
        assert_eq!(XlsxParser::mime_type_from_extension("unknown"), None);
    }

    #[test]
    fn test_determine_mime_type_from_content_type() {
        let mime = XlsxParser::determine_mime_type(None, Some("application/vnd.ms-excel"))
            .expect("should parse content type");
        assert_eq!(mime, "application/vnd.ms-excel");

        let mime = XlsxParser::determine_mime_type(
            None,
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
        )
        .expect("should parse content type");
        assert_eq!(
            mime,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        );
    }

    #[test]
    fn test_determine_mime_type_from_filename() {
        let mime =
            XlsxParser::determine_mime_type(Some("document.xls"), None).expect("should parse .xls");
        assert_eq!(mime, "application/vnd.ms-excel");

        let mime = XlsxParser::determine_mime_type(Some("document.xlsx"), None)
            .expect("should parse .xlsx");
        assert_eq!(
            mime,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        );
    }

    #[test]
    fn test_determine_mime_type_priority() {
        // Content type should take priority over filename
        let mime = XlsxParser::determine_mime_type(
            Some("document.xlsx"),
            Some("application/vnd.ms-excel"),
        )
        .expect("should parse");
        assert_eq!(mime, "application/vnd.ms-excel");
    }

    #[test]
    fn test_determine_mime_type_unknown() {
        assert!(XlsxParser::determine_mime_type(Some("document.unknown"), None).is_err());
        assert!(XlsxParser::determine_mime_type(Some("document"), None).is_err());
        assert!(XlsxParser::determine_mime_type(None, None).is_err());
        assert!(XlsxParser::determine_mime_type(Some("file.pdf"), None).is_err());
    }

    #[test]
    fn test_supported_extensions() {
        assert_eq!(SUPPORTED_EXTENSIONS, &["xlsx", "xls", "xlsm", "xlsb"]);
    }

    #[test]
    fn test_cell_to_string_empty() {
        assert_eq!(cell_to_string(&Data::Empty), "");
    }

    #[test]
    fn test_cell_to_string_string() {
        assert_eq!(cell_to_string(&Data::String("hello".to_owned())), "hello");
    }

    #[test]
    fn test_cell_to_string_int() {
        assert_eq!(cell_to_string(&Data::Int(42)), "42");
    }

    #[test]
    fn test_cell_to_string_float() {
        assert_eq!(cell_to_string(&Data::Float(2.71)), "2.71");
        assert_eq!(cell_to_string(&Data::Float(42.0)), "42");
    }

    #[test]
    fn test_cell_to_string_bool() {
        assert_eq!(cell_to_string(&Data::Bool(true)), "TRUE");
        assert_eq!(cell_to_string(&Data::Bool(false)), "FALSE");
    }
}
