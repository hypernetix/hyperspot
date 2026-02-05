#![allow(clippy::unwrap_used, clippy::expect_used, clippy::use_debug)]

use base64::Engine;
use file_parser::domain::ir::ParsedBlock;
use file_parser::domain::parser::FileParserBackend;
use file_parser::infra::parsers::image_parser::ImageParser;
use std::path::PathBuf;

/// Helper to get the path to test data files
fn get_test_file_path(filename: &str) -> PathBuf {
    // Path relative to workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("testing/e2e/testdata/images")
        .join(filename)
}

#[tokio::test]
async fn test_image_parser_basic_info() {
    let parser = ImageParser::new();

    assert_eq!(parser.id(), "image");
    assert_eq!(
        parser.supported_extensions(),
        &["png", "jpg", "jpeg", "webp", "gif"]
    );
}

#[tokio::test]
async fn test_image_parser_png() {
    let parser = ImageParser::new();
    let test_file = get_test_file_path("tiny.png");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse PNG file: {:?}",
        result.err()
    );

    let document = result.unwrap();

    // Verify document metadata
    assert_eq!(document.meta.content_type.as_deref(), Some("image/png"));
    assert_eq!(document.meta.original_filename.as_deref(), Some("tiny.png"));

    // Verify single Image block
    assert_eq!(document.blocks.len(), 1, "Should have exactly one block");
    match &document.blocks[0] {
        ParsedBlock::Image { src, .. } => {
            assert!(src.is_some(), "Image src should be present");
            let data_uri = src.as_ref().unwrap();
            assert!(
                data_uri.starts_with("data:image/png;base64,"),
                "Should be a PNG data URI"
            );

            // Verify base64 can be decoded back to original bytes
            let base64_part = data_uri.strip_prefix("data:image/png;base64,").unwrap();
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(base64_part)
                .expect("Should be valid base64");

            let original_bytes = std::fs::read(&test_file).unwrap();
            assert_eq!(
                decoded, original_bytes,
                "Decoded bytes should match original"
            );
        }
        _ => panic!("Expected Image block, got {:?}", document.blocks[0]),
    }
}

#[tokio::test]
async fn test_image_parser_jpg() {
    let parser = ImageParser::new();
    let test_file = get_test_file_path("tiny.jpg");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse JPG file: {:?}",
        result.err()
    );

    let document = result.unwrap();

    assert_eq!(document.meta.content_type.as_deref(), Some("image/jpeg"));
    assert_eq!(document.meta.original_filename.as_deref(), Some("tiny.jpg"));

    assert_eq!(document.blocks.len(), 1);
    match &document.blocks[0] {
        ParsedBlock::Image { src, .. } => {
            let data_uri = src.as_ref().unwrap();
            assert!(data_uri.starts_with("data:image/jpeg;base64,"));
        }
        _ => panic!("Expected Image block"),
    }
}

#[tokio::test]
async fn test_image_parser_webp() {
    let parser = ImageParser::new();
    let test_file = get_test_file_path("tiny.webp");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse WebP file: {:?}",
        result.err()
    );

    let document = result.unwrap();

    assert_eq!(document.meta.content_type.as_deref(), Some("image/webp"));
    assert_eq!(
        document.meta.original_filename.as_deref(),
        Some("tiny.webp")
    );

    assert_eq!(document.blocks.len(), 1);
    match &document.blocks[0] {
        ParsedBlock::Image { src, .. } => {
            let data_uri = src.as_ref().unwrap();
            assert!(data_uri.starts_with("data:image/webp;base64,"));
        }
        _ => panic!("Expected Image block"),
    }
}

#[tokio::test]
async fn test_image_parser_gif() {
    let parser = ImageParser::new();
    let test_file = get_test_file_path("tiny.gif");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let result = parser.parse_local_path(&test_file).await;

    assert!(
        result.is_ok(),
        "Failed to parse GIF file: {:?}",
        result.err()
    );

    let document = result.unwrap();

    assert_eq!(document.meta.content_type.as_deref(), Some("image/gif"));
    assert_eq!(document.meta.original_filename.as_deref(), Some("tiny.gif"));

    assert_eq!(document.blocks.len(), 1);
    match &document.blocks[0] {
        ParsedBlock::Image { src, .. } => {
            let data_uri = src.as_ref().unwrap();
            assert!(data_uri.starts_with("data:image/gif;base64,"));
        }
        _ => panic!("Expected Image block"),
    }
}

#[tokio::test]
async fn test_image_parser_parse_bytes_with_filename_hint() {
    let parser = ImageParser::new();
    let test_file = get_test_file_path("tiny.png");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let file_bytes = std::fs::read(&test_file).unwrap();
    let bytes = bytes::Bytes::from(file_bytes);

    let result = parser.parse_bytes(Some("uploaded.png"), None, bytes).await;

    assert!(result.is_ok(), "Failed to parse bytes: {:?}", result.err());

    let document = result.unwrap();
    assert_eq!(document.meta.content_type.as_deref(), Some("image/png"));
    assert_eq!(document.blocks.len(), 1);
}

#[tokio::test]
async fn test_image_parser_parse_bytes_with_content_type() {
    let parser = ImageParser::new();
    let test_file = get_test_file_path("tiny.jpg");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let file_bytes = std::fs::read(&test_file).unwrap();
    let bytes = bytes::Bytes::from(file_bytes);

    let result = parser.parse_bytes(None, Some("image/jpeg"), bytes).await;

    assert!(result.is_ok(), "Failed to parse bytes: {:?}", result.err());

    let document = result.unwrap();
    assert_eq!(document.meta.content_type.as_deref(), Some("image/jpeg"));
    assert_eq!(document.blocks.len(), 1);
}

#[tokio::test]
async fn test_image_parser_parse_bytes_no_hints() {
    let parser = ImageParser::new();
    let test_file = get_test_file_path("tiny.png");

    if !test_file.exists() {
        eprintln!("Skipping test: test file not found at {test_file:?}");
        return;
    }

    let file_bytes = std::fs::read(&test_file).unwrap();
    let bytes = bytes::Bytes::from(file_bytes);

    let result = parser.parse_bytes(None, None, bytes).await;

    // Should fail without hints
    assert!(result.is_err(), "Should fail to parse bytes without hints");
}

#[tokio::test]
async fn test_image_parser_unsupported_extension() {
    let parser = ImageParser::new();

    // Create a temporary path with unsupported extension
    let temp_path = PathBuf::from("/tmp/test.bmp");

    let result = parser.parse_local_path(&temp_path).await;

    // Should fail with unsupported file type error
    assert!(result.is_err(), "Should fail for unsupported extension");
}
