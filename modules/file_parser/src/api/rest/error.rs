use http::StatusCode;
use modkit::api::problem::Problem;

use crate::domain::error::DomainError;

/// Convert domain errors to HTTP Problem responses
pub fn domain_error_to_problem(err: DomainError) -> Problem {
    match err {
        DomainError::FileNotFound { path } => Problem::new(
            StatusCode::NOT_FOUND,
            "File Not Found",
            format!("File not found: {path}"),
        ),

        DomainError::UnsupportedFileType { extension } => Problem::new(
            StatusCode::BAD_REQUEST,
            "Unsupported File Type",
            format!("Unsupported file type: {extension}"),
        ),

        DomainError::NoParserAvailable { extension } => Problem::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "No Parser Available",
            format!("No parser available for extension: {extension}"),
        ),

        DomainError::ParseError { message } => {
            Problem::new(StatusCode::UNPROCESSABLE_ENTITY, "Parse Error", message)
        }

        DomainError::IoError { message } => {
            Problem::new(StatusCode::INTERNAL_SERVER_ERROR, "IO Error", message)
        }

        DomainError::InvalidUrl { url } => Problem::new(
            StatusCode::BAD_REQUEST,
            "Invalid URL",
            format!("Invalid URL: {url}"),
        ),

        DomainError::DownloadError { message } => {
            Problem::new(StatusCode::BAD_GATEWAY, "Download Error", message)
        }

        DomainError::InvalidRequest { message } => {
            Problem::new(StatusCode::BAD_REQUEST, "Invalid Request", message)
        }
    }
}

/// Implement Into<Problem> for `DomainError` so `?` works in handlers
impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        domain_error_to_problem(e)
    }
}
