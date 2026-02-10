use modkit_macros::domain_model;
use thiserror::Error;

/// Domain-level errors for file parsing operations
#[domain_model]
#[derive(Error, Debug, Clone)]
pub enum DomainError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Unsupported file type: {extension}")]
    UnsupportedFileType { extension: String },

    #[error("No parser available for extension: {extension}")]
    NoParserAvailable { extension: String },

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    #[error("Download error: {message}")]
    DownloadError { message: String },

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
}

impl DomainError {
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    pub fn unsupported_file_type(extension: impl Into<String>) -> Self {
        Self::UnsupportedFileType {
            extension: extension.into(),
        }
    }

    pub fn no_parser_available(extension: impl Into<String>) -> Self {
        Self::NoParserAvailable {
            extension: extension.into(),
        }
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
        }
    }

    pub fn invalid_url(url: impl Into<String>) -> Self {
        Self::InvalidUrl { url: url.into() }
    }

    pub fn download_error(message: impl Into<String>) -> Self {
        Self::DownloadError {
            message: message.into(),
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            message: message.into(),
        }
    }
}
