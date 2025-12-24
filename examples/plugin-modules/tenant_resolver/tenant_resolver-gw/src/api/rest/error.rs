use modkit::api::problem::Problem;

use crate::domain::error::DomainError;

impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        use http::StatusCode;

        match e {
            DomainError::PluginNotFound { vendor } => Problem::new(
                StatusCode::NOT_FOUND,
                "PluginNotFound",
                format!("No plugin instances found for vendor '{vendor}'"),
            ),
            DomainError::PluginClientNotFound { gts_id } => Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "PluginClientNotFound",
                format!("Plugin client not found in ClientHub for '{gts_id}'"),
            ),
            DomainError::InvalidPluginInstance { gts_id, reason } => Problem::new(
                StatusCode::BAD_REQUEST,
                "InvalidPluginInstance",
                format!("Invalid plugin instance '{gts_id}': {reason}"),
            ),
            DomainError::TypesRegistryUnavailable(reason) => Problem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "TypesRegistryUnavailable",
                reason,
            ),
            DomainError::TenantNotFound(msg) => {
                Problem::new(StatusCode::NOT_FOUND, "TenantNotFound", msg)
            }
            DomainError::PermissionDenied(msg) => {
                Problem::new(StatusCode::FORBIDDEN, "PermissionDenied", msg)
            }
            DomainError::Internal(reason) => {
                Problem::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal", reason)
            }
        }
    }
}
