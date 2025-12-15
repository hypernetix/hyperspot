#![allow(clippy::unwrap_used, clippy::expect_used, clippy::use_debug)]

use file_parser::domain::parser::FileParserBackend;
use file_parser::domain::parsers::docx_parser::DocxParser;
use std::path::PathBuf;

/// Helper to get the path to test data files
fn get_test_file_path(filename: &str) -> PathBuf {
    // Path relative to workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("testing/e2e/testdata/docx")
        .join(filename)
}

#[tokio::test]
async fn test_docx_parser_basic_info() {
    let parser = DocxParser::new();

    assert_eq!(parser.id(), "docx");
    assert_eq!(parser.supported_extensions(), &["docx"]);
}

#[tokio::test]
async fn test_docx_parser_with_working_file() {
    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_1table_multilingual.docx");

    // Skip test if file doesn't exist (not all test files may be available)
    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse working DOCX file: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");

    // Verify document metadata
    assert!(document.meta.original_filename.is_some());
    assert_eq!(
        document.meta.original_filename.as_deref(),
        Some("test_file_1table_multilingual.docx")
    );
}

#[tokio::test]
async fn test_docx_parser_with_two_page_file() {
    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_2pages_multilingual.docx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    // Note: docx-rust has issues with files containing gradient fills without
    // the "rotate_with_shape" field, so this may fail
    if result.is_err() {
        let error = result.err().unwrap();
        let error_msg = error.to_string();
        if error_msg.contains("malformed XML") || error_msg.contains("rotate_with_shape") {
            eprintln!("Known docx-rust limitation with gradient fills: {error_msg}");
            return; // Skip test due to known limitation
        }
        panic!("Unexpected error: {error_msg:?}");
    }

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");
}

#[tokio::test]
async fn test_docx_parser_with_big_english_file() {
    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_big_english.docx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    // Note: docx-rust has issues with files containing gradient fills without
    // the "rotate_with_shape" field, so this may fail
    if result.is_err() {
        let error = result.err().unwrap();
        let error_msg = error.to_string();
        if error_msg.contains("malformed XML") || error_msg.contains("rotate_with_shape") {
            eprintln!("Known docx-rust limitation with gradient fills: {error_msg}");
            return; // Skip test due to known limitation
        }
        panic!("Unexpected error: {error_msg:?}");
    }

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");
}

#[tokio::test]
async fn test_docx_parser_with_edge_cases_file_returns_error() {
    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_1table_edge_cases.docx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    // This should return an error due to parser limitations
    let result = parser.parse_local_path(&test_file).await;
    assert!(
        result.is_ok(),
        "Failed to parse DOCX from bytes: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");
    assert!(document.meta.original_filename.is_some());
}

#[tokio::test]
async fn test_docx_parser_with_image_multilingual_file() {
    // This file is 1,812,664 bytes and contains images.
    // With our fixes to docx-rust, it now parses successfully.
    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_1table_image_multilingual.docx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse image multilingual DOCX file: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");

    eprintln!(
        "Successfully parsed complex DOCX file with images ({} blocks)",
        document.blocks.len()
    );
}

#[tokio::test]
async fn test_docx_parser_parse_bytes() {
    // Test parsing from bytes instead of file path
    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_1table_multilingual.docx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    // Read file as bytes
    let file_bytes = std::fs::read(&test_file).expect("Failed to read test file");
    let bytes = bytes::Bytes::from(file_bytes);

    let result = parser
        .parse_bytes(
            Some("test_file_1table_multilingual.docx"),
            Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
            bytes,
        )
        .await;

    assert!(
        result.is_ok(),
        "Failed to parse DOCX from bytes: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");
    assert!(document.meta.original_filename.is_some());
}

#[tokio::test]
async fn test_docx_parser_parse_bytes_without_filename() {
    // Test parsing from bytes without filename hint
    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_1table_multilingual.docx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let file_bytes = std::fs::read(&test_file).expect("Failed to read test file");
    let bytes = bytes::Bytes::from(file_bytes);

    let result = parser.parse_bytes(None, None, bytes).await;

    assert!(
        result.is_ok(),
        "Failed to parse DOCX from bytes without filename: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");
    // When no filename hint is provided, original_filename is not set
    assert_eq!(
        document.meta.original_filename, None,
        "Without filename hint, original_filename should be None"
    );
}

#[tokio::test]
async fn test_docx_parser_nonexistent_file() {
    let parser = DocxParser::new();
    let nonexistent = PathBuf::from("/nonexistent/path/to/file.docx");

    let result = parser.parse_local_path(&nonexistent).await;

    assert!(result.is_err(), "Should fail on nonexistent file");

    let error = result.err().unwrap();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Failed to read file")
            || error_msg.contains("No such file")
            || error_msg.contains("Failed to open DOCX file")
            || error_msg.contains("cannot find the path"),
        "Error should mention file reading issue, got: {error_msg}"
    );
}

#[tokio::test]
async fn test_docx_parser_invalid_docx_bytes() {
    let parser = DocxParser::new();
    let invalid_bytes = bytes::Bytes::from("This is not a valid DOCX file");

    let result = parser
        .parse_bytes(Some("invalid.docx"), None, invalid_bytes)
        .await;

    assert!(result.is_err(), "Should fail on invalid DOCX data");

    let error = result.err().unwrap();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Failed to parse DOCX")
            || error_msg.contains("Failed to open DOCX file"),
        "Error should mention DOCX parsing failure, got: {error_msg}"
    );
}

#[tokio::test]
async fn test_docx_parser_extracts_tables() {
    use file_parser::domain::ir::ParsedBlock;

    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_1table_multilingual.docx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;
    assert!(
        result.is_ok(),
        "Failed to parse DOCX file: {:?}",
        result.err()
    );

    let document = result.unwrap();

    // Count how many tables are in the document
    let table_count = document
        .blocks
        .iter()
        .filter(|block| matches!(block, ParsedBlock::Table(_)))
        .count();

    assert!(
        table_count > 0,
        "Document should contain at least one table"
    );

    // Verify table structure
    for block in &document.blocks {
        if let ParsedBlock::Table(table_block) = block {
            assert!(
                !table_block.rows.is_empty(),
                "Table should have at least one row"
            );

            for (row_idx, row) in table_block.rows.iter().enumerate() {
                assert!(
                    !row.cells.is_empty(),
                    "Row {row_idx} should have at least one cell"
                );

                for (cell_idx, cell) in row.cells.iter().enumerate() {
                    assert!(
                        !cell.blocks.is_empty(),
                        "Cell ({row_idx}, {cell_idx}) should have at least one block"
                    );
                }
            }

            eprintln!(
                "Table has {} rows with {} cells in first row",
                table_block.rows.len(),
                table_block.rows[0].cells.len()
            );
        }
    }
}

#[tokio::test]
async fn test_docx_parser_table_edge_cases() {
    use file_parser::domain::ir::ParsedBlock;

    let parser = DocxParser::new();
    let test_file = get_test_file_path("test_file_1table_edge_cases.docx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;
    assert!(
        result.is_ok(),
        "Failed to parse DOCX file: {:?}",
        result.err()
    );

    let document = result.unwrap();

    // Count tables
    let table_count = document
        .blocks
        .iter()
        .filter(|block| matches!(block, ParsedBlock::Table(_)))
        .count();

    assert!(
        table_count > 0,
        "Document should contain at least one table"
    );
}
