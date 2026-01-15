use async_trait::async_trait;
use std::path::Path;

use crate::domain::error::DomainError;
use crate::domain::ir::{DocumentBuilder, ParsedBlock, ParsedSource};
use crate::domain::parser::FileParserBackend;

/// Image parser that handles standard image formats
///
/// Treats images as opaque binary payloads and encodes them as base64 data URIs.
/// No decoding, resizing, or re-encoding is performed.
///
/// Supported formats:
/// - PNG (.png, image/png)
/// - JPEG (.jpg, .jpeg, image/jpeg)
/// - WebP (.webp, image/webp)
/// - GIF (.gif, image/gif)
///
/// # File Size Limits
/// Individual image files are limited to 50 MB to prevent memory exhaustion.
/// The service layer enforces an additional global file size limit (configurable,
/// defaults to 100 MB) that applies to all file types.
///
/// # Note on GIF format
/// This parser accepts GIF files but does **not** validate whether they are static or animated.
/// Animated GIFs are generally unsuitable for LLM consumption (they contain multiple frames,
/// but only the first frame would typically be useful). Future implementations may add validation
/// to reject animated GIFs or extract only the first frame.
pub struct ImageParser;

/// Supported file extensions for image formats
const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif"];

/// Maximum allowed size for individual image files in MB (for display)
const MAX_IMAGE_SIZE_MB: u64 = 50;

/// Maximum allowed size for individual image files in bytes
const MAX_IMAGE_SIZE_BYTES: u64 = MAX_IMAGE_SIZE_MB * 1024 * 1024;

impl ImageParser {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Determine MIME type from file extension
    fn mime_type_from_extension(extension: &str) -> Option<&'static str> {
        if extension.eq_ignore_ascii_case("png") {
            return Some("image/png");
        }
        if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") {
            return Some("image/jpeg");
        }
        if extension.eq_ignore_ascii_case("webp") {
            return Some("image/webp");
        }
        if extension.eq_ignore_ascii_case("gif") {
            return Some("image/gif");
        }
        None
    }

    /// Determine MIME type from filename or provided content type
    fn determine_mime_type(
        filename_hint: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<String, DomainError> {
        // Priority 1: Use provided content-type if it's an image type
        if let Some(ct) = content_type {
            if ct.starts_with("image/") {
                return Ok(ct.to_owned());
            }
        }

        // Priority 2: Infer from filename extension
        if let Some(filename) = filename_hint {
            if let Some(ext) = Path::new(filename).extension().and_then(|s| s.to_str()) {
                if let Some(mime) = Self::mime_type_from_extension(ext) {
                    return Ok(mime.to_owned());
                }
            }
        }

        Err(DomainError::unsupported_file_type(
            "Unable to determine image MIME type",
        ))
    }

    /// Build a data URI from raw image bytes
    fn build_data_uri(mime_type: &str, bytes: &[u8]) -> String {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        format!("data:{mime_type};base64,{encoded}")
    }
}

impl Default for ImageParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileParserBackend for ImageParser {
    fn id(&self) -> &'static str {
        "image"
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        SUPPORTED_EXTENSIONS
    }

    async fn parse_local_path(
        &self,
        path: &Path,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        // Check file size before reading
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| DomainError::io_error(format!("Failed to read file metadata: {e}")))?;

        if metadata.len() > MAX_IMAGE_SIZE_BYTES {
            return Err(DomainError::invalid_request(format!(
                "Image file too large: {} bytes (max {} MB)",
                metadata.len(),
                MAX_IMAGE_SIZE_MB
            )));
        }

        // Read file bytes
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| DomainError::io_error(format!("Failed to read image file: {e}")))?;

        // Determine MIME type from extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| DomainError::unsupported_file_type("no extension"))?;

        let mime_type = Self::mime_type_from_extension(extension)
            .ok_or_else(|| DomainError::unsupported_file_type(extension))?;

        // Build data URI
        let data_uri = Self::build_data_uri(mime_type, &bytes);

        // Extract filename
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(ToString::to_string);

        // Build document with single Image block
        let document = DocumentBuilder::new(ParsedSource::LocalPath(path.display().to_string()))
            .content_type(mime_type)
            .original_filename(filename.unwrap_or_else(|| "unknown".to_owned()))
            .blocks(vec![ParsedBlock::Image {
                alt: None,
                title: None,
                src: Some(data_uri),
            }])
            .build();

        Ok(document)
    }

    async fn parse_bytes(
        &self,
        filename_hint: Option<&str>,
        content_type: Option<&str>,
        bytes: bytes::Bytes,
    ) -> Result<crate::domain::ir::ParsedDocument, DomainError> {
        // Check file size
        #[allow(clippy::cast_possible_truncation)]
        if bytes.len() > MAX_IMAGE_SIZE_BYTES as usize {
            return Err(DomainError::invalid_request(format!(
                "Image file too large: {} bytes (max {} MB)",
                bytes.len(),
                MAX_IMAGE_SIZE_MB
            )));
        }

        // Determine MIME type
        let mime_type = Self::determine_mime_type(filename_hint, content_type)?;

        // Build data URI
        let data_uri = Self::build_data_uri(&mime_type, &bytes);

        // Determine source
        let source = if let Some(name) = filename_hint {
            ParsedSource::Uploaded {
                original_name: name.to_owned(),
            }
        } else {
            ParsedSource::Uploaded {
                original_name: "unknown".to_owned(),
            }
        };

        // Build document with single Image block
        let document = DocumentBuilder::new(source)
            .content_type(mime_type)
            .original_filename(filename_hint.unwrap_or("unknown"))
            .blocks(vec![ParsedBlock::Image {
                alt: None,
                title: None,
                src: Some(data_uri),
            }])
            .build();

        Ok(document)
    }
}
