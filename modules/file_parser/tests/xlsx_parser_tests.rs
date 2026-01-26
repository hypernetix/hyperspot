#![allow(clippy::unwrap_used, clippy::expect_used, clippy::use_debug)]

// NOTE: Very large sheet tests (memory behavior with millions of cells) are better suited
// for E2E tests due to test file size and execution time. Unit tests here cover correctness
// of parsing features like merged cells and formulas.

use file_parser::domain::parser::FileParserBackend;
use file_parser::infra::parsers::xlsx_parser::XlsxParser;
use std::path::PathBuf;

/// Helper to get the path to test data files
fn get_test_file_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("testing/e2e/testdata/xlsx")
        .join(filename)
}

#[tokio::test]
async fn test_xlsx_parser_basic_info() {
    let parser = XlsxParser::new();

    assert_eq!(parser.id(), "xlsx");
    assert_eq!(
        parser.supported_extensions(),
        &["xlsx", "xls", "xlsm", "xlsb"]
    );
}

#[tokio::test]
async fn test_xlsx_parser_with_simple_file() {
    let parser = XlsxParser::new();
    let test_file = get_test_file_path("simple_data.xlsx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse XLSX file: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");

    // Verify document metadata
    assert!(document.meta.original_filename.is_some());
    assert_eq!(
        document.meta.original_filename.as_deref(),
        Some("simple_data.xlsx")
    );
    assert_eq!(
        document.meta.content_type.as_deref(),
        Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    );
}

#[tokio::test]
async fn test_xlsx_parser_with_multisheet_file() {
    let parser = XlsxParser::new();
    let test_file = get_test_file_path("multi_sheet.xlsx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse multi-sheet XLSX file: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");

    // Count heading blocks (one per sheet)
    let heading_count = document
        .blocks
        .iter()
        .filter(|b| matches!(b, file_parser::domain::ir::ParsedBlock::Heading { .. }))
        .count();

    assert!(
        heading_count >= 2,
        "Multi-sheet file should have at least 2 sheet headings, found {heading_count}"
    );
}

#[tokio::test]
async fn test_xlsx_parser_nonexistent_file() {
    let parser = XlsxParser::new();
    let test_file = PathBuf::from("/nonexistent/path/to/file.xlsx");

    let result = parser.parse_local_path(&test_file).await;

    assert!(result.is_err(), "Should fail for non-existent file");
}

#[tokio::test]
async fn test_xlsx_parser_invalid_xlsx_bytes() {
    let parser = XlsxParser::new();
    let invalid_bytes = bytes::Bytes::from_static(b"This is not a valid XLSX file content");

    let result = parser
        .parse_bytes(Some("invalid.xlsx"), None, invalid_bytes)
        .await;

    assert!(result.is_err(), "Should fail for invalid XLSX bytes");
}

#[tokio::test]
async fn test_xlsx_parser_parse_bytes() {
    let parser = XlsxParser::new();
    let test_file = get_test_file_path("simple_data.xlsx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let file_content = std::fs::read(&test_file).expect("Failed to read test file");
    let bytes = bytes::Bytes::from(file_content);

    let result = parser
        .parse_bytes(Some("simple_data.xlsx"), None, bytes)
        .await;

    assert!(
        result.is_ok(),
        "Failed to parse XLSX bytes: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");
}

#[tokio::test]
async fn test_xlsx_parser_parse_bytes_unrecognized_format() {
    let parser = XlsxParser::new();
    // Random bytes that don't match any Excel magic bytes
    let invalid_bytes = bytes::Bytes::from_static(b"This is not a valid Excel file content");

    let result = parser
        .parse_bytes(Some("test.xlsx"), None, invalid_bytes)
        .await;

    // Should fail with unrecognized format error (magic bytes don't match)
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err());
    assert!(
        err_msg.contains("Unrecognized") || err_msg.contains("format"),
        "Error should mention unrecognized format: {err_msg}"
    );
}

#[tokio::test]
async fn test_xlsx_parser_parse_bytes_with_ole_magic_but_invalid() {
    let parser = XlsxParser::new();
    // OLE magic bytes (D0 CF 11 E0) but invalid content after
    let mut invalid_bytes = vec![0xD0, 0xCF, 0x11, 0xE0];
    invalid_bytes.extend_from_slice(b"invalid content after magic");
    let invalid_bytes = bytes::Bytes::from(invalid_bytes);

    let result = parser
        .parse_bytes(Some("test.xls"), None, invalid_bytes)
        .await;

    // Should fail with XLS-specific error (magic bytes matched, but content invalid)
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err());
    assert!(
        err_msg.contains("XLS") || err_msg.contains("xls"),
        "Error should mention XLS: {err_msg}"
    );
}

#[tokio::test]
async fn test_xlsx_parser_extracts_tables() {
    let parser = XlsxParser::new();
    let test_file = get_test_file_path("simple_data.xlsx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;
    let document = result.expect("Failed to parse XLSX");

    // Find table blocks
    let table_count = document
        .blocks
        .iter()
        .filter(|b| matches!(b, file_parser::domain::ir::ParsedBlock::Table(_)))
        .count();

    assert!(
        table_count >= 1,
        "XLSX should contain at least one table block, found {table_count}"
    );
}

#[tokio::test]
async fn test_xlsx_parser_merged_cells() {
    let parser = XlsxParser::new();
    let test_file = get_test_file_path("merged_cells.xlsx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse XLSX with merged cells: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");

    // Find table blocks and verify structure
    let tables: Vec<_> = document
        .blocks
        .iter()
        .filter_map(|b| {
            if let file_parser::domain::ir::ParsedBlock::Table(t) = b {
                Some(t)
            } else {
                None
            }
        })
        .collect();

    assert!(!tables.is_empty(), "Should have at least one table");

    // Merged cells: calamine returns the value in the top-left cell of the merge,
    // and empty strings for other cells in the merged range.
    // Verify we can parse the file without errors and get expected row count.
    let first_table = tables[0];
    assert!(
        first_table.rows.len() >= 3,
        "Merged cells test file should have at least 3 rows, found {}",
        first_table.rows.len()
    );
}

#[tokio::test]
async fn test_xlsx_parser_formula_cells() {
    let parser = XlsxParser::new();
    let test_file = get_test_file_path("formula_cells.xlsx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse XLSX with formulas: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");

    // Find table blocks
    let tables: Vec<_> = document
        .blocks
        .iter()
        .filter_map(|b| {
            if let file_parser::domain::ir::ParsedBlock::Table(t) = b {
                Some(t)
            } else {
                None
            }
        })
        .collect();

    assert!(!tables.is_empty(), "Should have at least one table");

    // Calamine reads cached formula results from xlsx files.
    // Note: Files created programmatically (e.g., openpyxl) without Excel
    // may not have cached values, resulting in empty cells for formulas.
    // Files saved by Excel will have computed values cached.
    let first_table = tables[0];

    // Extract all cell text content for verification
    let cell_texts: Vec<String> = first_table
        .rows
        .iter()
        .flat_map(|row| {
            row.cells.iter().filter_map(|cell| {
                cell.blocks.first().and_then(|block| {
                    if let file_parser::domain::ir::ParsedBlock::Paragraph { inlines } = block {
                        inlines.first().and_then(|inline| {
                            if let file_parser::domain::ir::Inline::Text { text, .. } = inline {
                                Some(text.clone())
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                })
            })
        })
        .collect();

    // Verify we can read the regular (non-formula) cells
    assert!(
        cell_texts.iter().any(|t| t == "10"),
        "Should find value 10 in cells, found: {cell_texts:?}"
    );
    assert!(
        cell_texts.iter().any(|t| t == "20"),
        "Should find value 20 in cells, found: {cell_texts:?}"
    );

    // Formula text (starting with =) should NOT appear in output
    // Calamine returns either computed values or empty, never raw formula text
    assert!(
        !cell_texts.iter().any(|t| t.starts_with('=')),
        "Raw formula text should not appear in output, found cells: {cell_texts:?}"
    );
}
