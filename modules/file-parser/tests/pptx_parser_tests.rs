#![allow(clippy::unwrap_used, clippy::expect_used, clippy::use_debug)]

use file_parser::domain::parser::FileParserBackend;
use file_parser::infra::parsers::pptx_parser::PptxParser;
use std::path::PathBuf;

/// Helper to get the path to test data files
fn get_test_file_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("testing/e2e/testdata/pptx")
        .join(filename)
}

#[tokio::test]
async fn test_pptx_parser_basic_info() {
    let parser = PptxParser::new();

    assert_eq!(parser.id(), "pptx");
    assert_eq!(parser.supported_extensions(), &["pptx"]);
}

#[tokio::test]
async fn test_pptx_parser_with_simple_file() {
    let parser = PptxParser::new();
    let test_file = get_test_file_path("simple_presentation.pptx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse PPTX file: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");

    // Verify document metadata
    assert!(document.meta.original_filename.is_some());
    assert_eq!(
        document.meta.original_filename.as_deref(),
        Some("simple_presentation.pptx")
    );
    assert_eq!(
        document.meta.content_type.as_deref(),
        Some("application/vnd.openxmlformats-officedocument.presentationml.presentation")
    );
}

#[tokio::test]
async fn test_pptx_parser_with_multislide_file() {
    let parser = PptxParser::new();
    let test_file = get_test_file_path("multi_slide.pptx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse multi-slide PPTX file: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");

    // Count heading blocks (one per slide)
    let heading_count = document
        .blocks
        .iter()
        .filter(|b| {
            matches!(
                b,
                file_parser::domain::ir::ParsedBlock::Heading { level: 2, .. }
            )
        })
        .count();

    assert!(
        heading_count >= 2,
        "Multi-slide file should have at least 2 slide headings, found {heading_count}"
    );

    // Check for page breaks between slides
    let page_break_count = document
        .blocks
        .iter()
        .filter(|b| matches!(b, file_parser::domain::ir::ParsedBlock::PageBreak))
        .count();

    assert!(
        page_break_count >= 1,
        "Multi-slide file should have page breaks between slides"
    );
}

#[tokio::test]
async fn test_pptx_parser_nonexistent_file() {
    let parser = PptxParser::new();
    let test_file = PathBuf::from("/nonexistent/path/to/file.pptx");

    let result = parser.parse_local_path(&test_file).await;

    assert!(result.is_err(), "Should fail for non-existent file");
}

#[tokio::test]
async fn test_pptx_parser_invalid_pptx_bytes() {
    let parser = PptxParser::new();
    let invalid_bytes = bytes::Bytes::from_static(b"This is not a valid PPTX file content");

    let result = parser
        .parse_bytes(Some("invalid.pptx"), None, invalid_bytes)
        .await;

    assert!(result.is_err(), "Should fail for invalid PPTX bytes");
}

#[tokio::test]
async fn test_pptx_parser_parse_bytes() {
    let parser = PptxParser::new();
    let test_file = get_test_file_path("simple_presentation.pptx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let file_content = std::fs::read(&test_file).expect("Failed to read test file");
    let bytes = bytes::Bytes::from(file_content);

    let result = parser
        .parse_bytes(Some("simple_presentation.pptx"), None, bytes)
        .await;

    assert!(
        result.is_ok(),
        "Failed to parse PPTX bytes: {:?}",
        result.err()
    );

    let document = result.unwrap();
    assert!(!document.blocks.is_empty(), "Document should have blocks");
}

#[tokio::test]
async fn test_pptx_parser_extracts_text() {
    let parser = PptxParser::new();
    let test_file = get_test_file_path("simple_presentation.pptx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;
    let document = result.expect("Failed to parse PPTX");

    // Find paragraph blocks with text content
    let paragraph_count = document
        .blocks
        .iter()
        .filter(|b| matches!(b, file_parser::domain::ir::ParsedBlock::Paragraph { .. }))
        .count();

    assert!(
        paragraph_count >= 1,
        "PPTX should contain at least one paragraph block, found {paragraph_count}"
    );
}

#[tokio::test]
async fn test_pptx_parser_with_tables() {
    let parser = PptxParser::new();
    let test_file = get_test_file_path("presentation_with_table.pptx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse PPTX with tables: {:?}",
        result.err()
    );

    let document = result.unwrap();

    // Find table blocks
    let table_count = document
        .blocks
        .iter()
        .filter(|b| matches!(b, file_parser::domain::ir::ParsedBlock::Table(_)))
        .count();

    assert!(
        table_count >= 1,
        "PPTX with tables should contain at least one table block, found {table_count}"
    );
}

#[tokio::test]
async fn test_pptx_parser_with_lists() {
    let parser = PptxParser::new();
    let test_file = get_test_file_path("presentation_with_list.pptx");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse PPTX with lists: {:?}",
        result.err()
    );

    let document = result.unwrap();

    // Find list item blocks
    let list_count = document
        .blocks
        .iter()
        .filter(|b| matches!(b, file_parser::domain::ir::ParsedBlock::ListItem { .. }))
        .count();

    assert!(
        list_count >= 1,
        "PPTX with lists should contain at least one list item block, found {list_count}"
    );
}
