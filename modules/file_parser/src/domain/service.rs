use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use tracing::{debug, info, instrument};

use crate::domain::error::DomainError;
use crate::domain::ir::ParsedDocument;
use crate::domain::parser::FileParserBackend;

/// File parser service that routes to appropriate backends
#[derive(Clone)]
pub struct FileParserService {
    parsers: Vec<Arc<dyn FileParserBackend>>,
    config: ServiceConfig,
}

/// Configuration for the file parser service
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
#[derive(Debug, Clone)]
pub struct FileParserInfo {
    pub supported_extensions: std::collections::HashMap<String, Vec<String>>,
}

impl FileParserService {
    /// Create a new service with the given parsers
    pub fn new(parsers: Vec<Arc<dyn FileParserBackend>>, config: ServiceConfig) -> Self {
        Self { parsers, config }
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
                .map(|s| s.to_string())
                .collect();
            supported_extensions.insert(id.to_string(), extensions);
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
            .map(|s| s.to_string());

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
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(
                self.config.download_timeout_secs,
            ))
            .build()
            .map_err(|e| {
                tracing::error!(?e, "FileParserService: failed to create HTTP client");
                DomainError::download_error(format!("Failed to create HTTP client: {}", e))
            })?;

        let response = client.get(url.as_str()).send().await.map_err(|e| {
            tracing::error!(?e, "FileParserService: failed to download file");
            DomainError::download_error(format!("Failed to download file: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            tracing::error!(?status, "FileParserService: HTTP error during download");
            return Err(DomainError::download_error(format!(
                "HTTP error: {}",
                status
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Validate MIME type if present
        if let Some(ref ct) = content_type {
            Self::validate_mime_type(extension, ct)?;
        }

        let bytes = response.bytes().await.map_err(|e| {
            tracing::error!(?e, "FileParserService: failed to read response bytes");
            DomainError::download_error(format!("Failed to read response: {}", e))
        })?;

        // Check file size
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
            .map(|s| s.to_string());
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
    pub fn extension_from_content_type(ct: &str) -> Option<String> {
        let mime: mime::Mime = ct.parse().ok()?;
        let essence = mime.essence_str();

        let ext = match essence {
            "application/pdf" => "pdf",
            "text/html" | "application/xhtml+xml" => "html",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
            _ => return None,
        };

        Some(ext.to_string())
    }

    /// Validate MIME type against expected type for extension
    fn validate_mime_type(extension: &str, content_type: &str) -> Result<(), DomainError> {
        // Parse MIME type
        let mime: mime::Mime = content_type.parse().map_err(|_| {
            DomainError::invalid_request(format!("Invalid content-type: {}", content_type))
        })?;

        let expected = match extension.to_lowercase().as_str() {
            "pdf" => Some("application/pdf"),
            "html" | "htm" => Some("text/html"),
            "docx" => {
                Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
            }
            _ => None, // Allow unknown extensions
        };

        if let Some(expected_type) = expected {
            let mime_str = mime.essence_str();
            // Also accept application/xhtml+xml for html
            let is_valid = if extension == "html" || extension == "htm" {
                mime_str == expected_type || mime_str == "application/xhtml+xml"
            } else {
                mime_str == expected_type
            };

            if !is_valid {
                tracing::warn!(
                    extension = extension,
                    expected = expected_type,
                    actual = mime_str,
                    "MIME type mismatch"
                );
                return Err(DomainError::invalid_request(format!(
                    "Content-Type {} does not match expected type {} for .{}",
                    mime_str, expected_type, extension
                )));
            }
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
