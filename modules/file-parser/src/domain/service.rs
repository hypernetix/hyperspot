use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use modkit_http::HttpClient;
use modkit_macros::domain_model;
use tracing::{debug, info, instrument};

use crate::domain::error::DomainError;
use crate::domain::ir::ParsedDocument;
use crate::domain::parser::FileParserBackend;

/// Mapping of file extensions to MIME types
/// Format: `(extension, mime_type)`
const EXTENSION_MIME_MAPPINGS: &[(&str, &str)] = &[
    ("pdf", "application/pdf"),
    ("html", "text/html"),
    ("htm", "text/html"),
    (
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("webp", "image/webp"),
    ("gif", "image/gif"),
];

/// File parser service that routes to appropriate backends
#[domain_model]
#[derive(Clone)]
pub struct FileParserService {
    parsers: Vec<Arc<dyn FileParserBackend>>,
    config: ServiceConfig,
    /// Shared HTTP client for URL downloads (connection pooling).
    /// `HttpClient` is `Clone + Send + Sync`, no external locking needed.
    http_client: HttpClient,
}

/// Configuration for the file parser service
#[domain_model]
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub max_file_size_bytes: usize,
    pub download_timeout_secs: u64,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 100 * 1024 * 1024, // 100 MB
            download_timeout_secs: 60,
        }
    }
}

/// Information about available parsers
#[domain_model]
#[derive(Debug, Clone)]
pub struct FileParserInfo {
    pub supported_extensions: std::collections::HashMap<String, Vec<String>>,
}

impl FileParserService {
    /// Create a new service with the given parsers and a pre-built HTTP client.
    #[must_use]
    pub fn new(
        parsers: Vec<Arc<dyn FileParserBackend>>,
        config: ServiceConfig,
        http_client: HttpClient,
    ) -> Self {
        Self {
            parsers,
            config,
            http_client,
        }
    }

    /// Get information about available parsers
    #[instrument(skip(self))]
    pub fn info(&self) -> FileParserInfo {
        debug!("Getting parser info");

        let mut supported_extensions = std::collections::HashMap::new();

        for parser in &self.parsers {
            let id = parser.id();
            let extensions: Vec<String> = parser
                .supported_extensions()
                .iter()
                .map(ToString::to_string)
                .collect();
            supported_extensions.insert(id.to_owned(), extensions);
        }

        FileParserInfo {
            supported_extensions,
        }
    }

    /// Parse a file from a local path
    #[instrument(skip(self), fields(path = %path.display()))]
    pub async fn parse_local(&self, path: &Path) -> Result<ParsedDocument, DomainError> {
        info!("Parsing file from local path");

        // Check if file exists
        if !path.exists() {
            return Err(DomainError::file_not_found(path.display().to_string()));
        }

        // Extract extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| DomainError::unsupported_file_type("no extension"))?;

        // Find parser
        let parser = self
            .find_parser_by_extension(extension)
            .ok_or_else(|| DomainError::no_parser_available(extension))?;

        // Parse the file
        let document = parser.parse_local_path(path).await.map_err(|e| {
            tracing::error!(?e, "FileParserService: parse_local failed");
            e
        })?;

        debug!("Successfully parsed file from local path");
        Ok(document)
    }

    /// Parse a file from bytes
    #[instrument(
        skip(self, bytes),
        fields(filename_hint = ?filename_hint, content_type = ?content_type, size = bytes.len())
    )]
    pub async fn parse_bytes(
        &self,
        filename_hint: Option<&str>,
        content_type: Option<&str>,
        bytes: Bytes,
    ) -> Result<ParsedDocument, DomainError> {
        info!("Parsing uploaded file");

        // Check file size
        if bytes.len() > self.config.max_file_size_bytes {
            return Err(DomainError::invalid_request(format!(
                "File size {} exceeds maximum of {} bytes",
                bytes.len(),
                self.config.max_file_size_bytes
            )));
        }

        // Determine extension by priority:
        // 1. From filename (if provided and has extension)
        // 2. From Content-Type (if provided and recognized)
        // 3. Error if both fail
        let extension_from_name = filename_hint
            .and_then(|name| Path::new(name).extension())
            .and_then(|s| s.to_str())
            .map(ToString::to_string);

        let extension = if let Some(ext) = extension_from_name {
            // Priority 1: Use extension from filename
            ext
        } else if let Some(ct) = content_type {
            // Priority 2: Try to infer from Content-Type
            if let Some(ext) = Self::extension_from_content_type(ct) {
                ext
            } else {
                return Err(DomainError::unsupported_file_type(
                    "no extension and unknown content-type",
                ));
            }
        } else {
            // Both failed
            return Err(DomainError::unsupported_file_type(
                "no extension and no content-type",
            ));
        };

        // NOTE: For direct uploads (parse_bytes), we do NOT validate MIME type.
        // This allows uploads with filename="document.pdf" and Content-Type="application/octet-stream"
        // to succeed. MIME validation is only enforced for parse_url (remote downloads).

        // Find parser
        let parser = self
            .find_parser_by_extension(&extension)
            .ok_or_else(|| DomainError::no_parser_available(&extension))?;

        // Parse the file
        let document = parser
            .parse_bytes(filename_hint, content_type, bytes)
            .await
            .map_err(|e| {
                tracing::error!(?e, "FileParserService: parse_bytes failed");
                e
            })?;

        debug!("Successfully parsed uploaded file");
        Ok(document)
    }

    /// Parse a file from a URL
    #[instrument(skip(self), fields(url = %url))]
    pub async fn parse_url(&self, url: &url::Url) -> Result<ParsedDocument, DomainError> {
        info!("Parsing file from URL");

        // Extract extension from URL path
        let path = Path::new(url.path());
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| DomainError::unsupported_file_type("no extension in URL"))?;

        // Find parser
        let parser = self
            .find_parser_by_extension(extension)
            .ok_or_else(|| DomainError::no_parser_available(extension))?;

        // Download file
        debug!("Downloading file from URL");
        let response = self
            .http_client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| {
                tracing::error!(?e, "FileParserService: failed to download file");
                DomainError::download_error(format!("Failed to download file: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(?status, "FileParserService: HTTP error during download");
            return Err(DomainError::download_error(format!("HTTP error: {status}")));
        }

        let content_type = response
            .headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(ToString::to_string);

        // Validate MIME type if present
        if let Some(ref ct) = content_type {
            Self::validate_mime_type(extension, ct)?;
        }

        let bytes = response.bytes().await.map_err(|e| {
            tracing::error!(?e, "FileParserService: failed to read response bytes");
            DomainError::download_error(format!("Failed to read response: {e}"))
        })?;

        // Defense-in-depth: HttpClient's max_body_size should enforce this limit during
        // download, but we check again here in case the client config changes or as a
        // safeguard against implementation differences.
        if bytes.len() > self.config.max_file_size_bytes {
            return Err(DomainError::invalid_request(format!(
                "File size {} exceeds maximum of {} bytes",
                bytes.len(),
                self.config.max_file_size_bytes
            )));
        }

        // Parse the downloaded file
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(ToString::to_string);
        let document = parser
            .parse_bytes(file_name.as_deref(), content_type.as_deref(), bytes)
            .await
            .map_err(|e| {
                tracing::error!(?e, "FileParserService: parse_url failed during parsing");
                e
            })?;

        debug!("Successfully parsed file from URL");
        Ok(document)
    }

    /// Extract file extension from Content-Type header
    #[must_use]
    pub fn extension_from_content_type(ct: &str) -> Option<String> {
        let mime: mime::Mime = ct.parse().ok()?;
        let essence = mime.essence_str();

        // Special case: application/xhtml+xml maps to html
        if essence == "application/xhtml+xml" {
            return Some("html".to_owned());
        }

        // Find extension by matching MIME type
        EXTENSION_MIME_MAPPINGS
            .iter()
            .find(|(_, mime_type)| *mime_type == essence)
            .map(|(ext, _)| (*ext).to_owned())
    }

    /// Validate MIME type against expected type for extension
    fn validate_mime_type(extension: &str, content_type: &str) -> Result<(), DomainError> {
        // Parse MIME type
        let mime: mime::Mime = content_type.parse().map_err(|_| {
            DomainError::invalid_request(format!("Invalid content-type: {content_type}"))
        })?;

        let mime_str = mime.essence_str();
        let extension_lower = extension.to_lowercase();

        // Find expected MIME type(s) for this extension
        let expected_mimes: Vec<&str> = EXTENSION_MIME_MAPPINGS
            .iter()
            .filter(|(ext, _)| *ext == extension_lower.as_str())
            .map(|(_, mime_type)| *mime_type)
            .collect();

        if expected_mimes.is_empty() {
            // Unknown extension - allow it
            return Ok(());
        }

        // Check if actual MIME matches any expected MIME
        // Special case: also accept application/xhtml+xml for html
        let is_valid = expected_mimes.contains(&mime_str)
            || (extension_lower == "html" && mime_str == "application/xhtml+xml")
            || (extension_lower == "htm" && mime_str == "application/xhtml+xml");

        if !is_valid {
            tracing::warn!(
                extension = extension,
                expected = ?expected_mimes,
                actual = mime_str,
                "MIME type mismatch"
            );
            return Err(DomainError::invalid_request(format!(
                "Content-Type {mime_str} does not match expected type(s) {expected_mimes:?} for .{extension}"
            )));
        }

        Ok(())
    }

    /// Find a parser by file extension
    fn find_parser_by_extension(&self, ext: &str) -> Option<Arc<dyn FileParserBackend>> {
        let ext_lower = ext.to_lowercase();
        self.parsers
            .iter()
            .find(|p| {
                p.supported_extensions()
                    .iter()
                    .any(|e| e.to_lowercase() == ext_lower)
            })
            .cloned()
    }
}
