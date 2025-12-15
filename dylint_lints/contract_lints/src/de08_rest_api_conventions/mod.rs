//! DE08: REST API Convention Checks
//!
//! Linters to enforce REST API guidelines and HTTP best practices.

pub mod de0801_api_endpoint_version;

pub use de0801_api_endpoint_version::DE0801_API_ENDPOINT_MUST_HAVE_VERSION;
