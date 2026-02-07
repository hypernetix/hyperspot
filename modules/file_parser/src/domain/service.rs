use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use tracing::{debug, info, instrument, warn};

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
    /// Optional base directory for local file parsing.
    /// When set, only files within this directory (and its subdirectories)
    /// can be read via `parse_local`. This prevents path traversal attacks.
    pub allowed_local_base_dir: Option<std::path::PathBuf>,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 100 * 1024 * 1024, // 100 MB
            download_timeout_secs: 60,
            allowed_local_base_dir: None,
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
    #[must_use]
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

        // Path traversal protection: validate and canonicalize the path
        let canonical_path = Self::validate_local_path(
            path,
            self.config.allowed_local_base_dir.as_deref(),
        )?;
        let path = canonical_path.as_path();

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

        // SSRF protection: validate URL scheme and target address
        Self::validate_url_for_ssrf(url)?;

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
                DomainError::download_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let response = client.get(url.as_str()).send().await.map_err(|e| {
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
            .get("content-type")
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

    /// Validate that a URL is safe to fetch (SSRF protection).
    ///
    /// Rejects:
    /// - Non-HTTP(S) schemes (e.g. `file://`, `ftp://`, `gopher://`)
    /// - URLs targeting loopback addresses (`127.0.0.0/8`, `::1`)
    /// - URLs targeting private network ranges (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`)
    /// - URLs targeting link-local addresses (`169.254.0.0/16` — includes cloud metadata endpoints)
    fn validate_url_for_ssrf(url: &url::Url) -> Result<(), DomainError> {
        // Only allow http and https schemes
        match url.scheme() {
            "http" | "https" => {}
            scheme => {
                warn!(scheme = scheme, "Rejected URL with disallowed scheme");
                return Err(DomainError::invalid_url(format!(
                    "Only http and https schemes are allowed, got: {scheme}"
                )));
            }
        }

        // Resolve the hostname and check all resulting IPs
        let host = url
            .host_str()
            .ok_or_else(|| DomainError::invalid_url("URL has no host"))?;

        // Block well-known internal hostnames
        let host_lower = host.to_lowercase();
        if host_lower == "localhost"
            || host_lower == "metadata.google.internal"
            || host_lower.ends_with(".internal")
        {
            warn!(host = host, "Rejected URL targeting internal hostname");
            return Err(DomainError::invalid_url(format!(
                "URLs targeting internal hosts are not allowed: {host}"
            )));
        }

        // If the host is an IP address literal, validate it directly
        if let Some(url::Host::Ipv4(ip)) = url.host() {
            if Self::is_private_ipv4(ip) {
                warn!(%ip, "Rejected URL targeting private IPv4 address");
                return Err(DomainError::invalid_url(
                    "URLs targeting private or internal IP addresses are not allowed",
                ));
            }
        }

        if let Some(url::Host::Ipv6(ip)) = url.host() {
            if Self::is_private_ipv6(ip) {
                warn!(%ip, "Rejected URL targeting private IPv6 address");
                return Err(DomainError::invalid_url(
                    "URLs targeting private or internal IP addresses are not allowed",
                ));
            }
        }

        // Perform DNS resolution and check all resolved IPs
        let port = url.port_or_known_default().unwrap_or(80);
        let addr_str = format!("{host}:{port}");
        if let Ok(addrs) = std::net::ToSocketAddrs::to_socket_addrs(&addr_str.as_str()) {
            for addr in addrs {
                match addr.ip() {
                    IpAddr::V4(ip) if Self::is_private_ipv4(ip) => {
                        warn!(%ip, resolved_from = host, "Rejected URL: hostname resolves to private IPv4");
                        return Err(DomainError::invalid_url(
                            "URLs targeting private or internal IP addresses are not allowed",
                        ));
                    }
                    IpAddr::V6(ip) if Self::is_private_ipv6(ip) => {
                        warn!(%ip, resolved_from = host, "Rejected URL: hostname resolves to private IPv6");
                        return Err(DomainError::invalid_url(
                            "URLs targeting private or internal IP addresses are not allowed",
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Check if an IPv4 address is in a private or reserved range.
    fn is_private_ipv4(ip: std::net::Ipv4Addr) -> bool {
        ip.is_loopback()             // 127.0.0.0/8
            || ip.is_private()       // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
            || ip.is_link_local()    // 169.254.0.0/16 (cloud metadata: 169.254.169.254)
            || ip.is_broadcast()     // 255.255.255.255
            || ip.is_unspecified()   // 0.0.0.0
    }

    /// Check if an IPv6 address is in a private or reserved range.
    fn is_private_ipv6(ip: std::net::Ipv6Addr) -> bool {
        ip.is_loopback()           // ::1
            || ip.is_unspecified() // ::
    }

    /// Validate that a local file path is safe to read (path traversal protection).
    ///
    /// Rejects paths containing `..` components and verifies the canonical path
    /// is within the configured `allowed_local_base_dir` (if set).
    fn validate_local_path(
        path: &Path,
        allowed_base: Option<&Path>,
    ) -> Result<std::path::PathBuf, DomainError> {
        // Reject paths with .. components before canonicalization
        for component in path.components() {
            if matches!(component, std::path::Component::ParentDir) {
                warn!(path = %path.display(), "Rejected path containing '..' traversal");
                return Err(DomainError::invalid_request(
                    "Path traversal ('..') is not allowed in file paths",
                ));
            }
        }

        // Canonicalize the path (resolves symlinks, normalizes)
        let canonical = path.canonicalize().map_err(|e| {
            DomainError::file_not_found(format!("{}: {e}", path.display()))
        })?;

        // If an allowed base directory is configured, verify the path is within it
        if let Some(base) = allowed_base {
            let canonical_base = base.canonicalize().map_err(|e| {
                tracing::error!(base = %base.display(), error = %e, "Allowed base directory is invalid");
                DomainError::io_error(format!(
                    "Server misconfiguration: allowed_local_base_dir '{}' is invalid: {e}",
                    base.display()
                ))
            })?;

            if !canonical.starts_with(&canonical_base) {
                warn!(
                    path = %canonical.display(),
                    base = %canonical_base.display(),
                    "Rejected path outside allowed base directory"
                );
                return Err(DomainError::invalid_request(format!(
                    "File path must be within the allowed directory: {}",
                    canonical_base.display()
                )));
            }
        }

        Ok(canonical)
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
