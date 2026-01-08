use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    pub trace_id: String,
}

impl ProblemDetails {
    pub fn new(
        problem_type: &str,
        title: &str,
        status: u16,
        detail: &str,
        instance: Option<String>,
    ) -> Self {
        Self {
            problem_type: problem_type.to_string(),
            title: title.to_string(),
            status,
            detail: detail.to_string(),
            instance,
            trace_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn routing_error(detail: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/routing-failed",
            "Routing Failed",
            404,
            detail,
            instance,
        )
    }

    pub fn invalid_identifier(detail: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/invalid-identifier",
            "Invalid GTS Identifier",
            400,
            detail,
            instance,
        )
    }

    pub fn authentication_error(detail: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/authentication-failed",
            "Authentication Failed",
            401,
            detail,
            instance,
        )
    }

    pub fn authorization_error(detail: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/authorization-failed",
            "Authorization Failed",
            403,
            detail,
            instance,
        )
    }

    pub fn read_only_entity(entity_id: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/read-only-entity",
            "Read-Only Entity",
            403,
            &format!(
                "Entity '{}' is read-only. It was provisioned through configuration files and cannot be modified via the API.",
                entity_id
            ),
            instance,
        )
    }

    pub fn validation_error(detail: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/validation-failed",
            "Validation Failed",
            400,
            detail,
            instance,
        )
    }

    pub fn invalid_odata_query(detail: &str, available_fields: &[String], instance: Option<String>) -> Self {
        let fields_list = available_fields.join(", ");
        let full_detail = format!(
            "{}. Available indexed fields: [{}]",
            detail, fields_list
        );
        Self::new(
            "https://hyperspot.dev/problems/invalid-query",
            "Invalid OData Query",
            400,
            &full_detail,
            instance,
        )
    }

    pub fn invalid_json_patch(detail: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/invalid-patch",
            "Invalid JSON Patch",
            400,
            detail,
            instance,
        )
    }

    pub fn service_error(detail: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/service-unavailable",
            "Service Unavailable",
            503,
            detail,
            instance,
        )
    }

    pub fn domain_feature_unavailable(gts_type: &str, instance: Option<String>) -> Self {
        Self::new(
            "https://hyperspot.dev/problems/feature-unavailable",
            "Domain Feature Unavailable",
            503,
            &format!(
                "Domain feature for GTS type '{}' is temporarily unavailable. Please retry later.",
                gts_type
            ),
            instance,
        )
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

#[derive(Debug)]
pub enum GtsCoreError {
    UnknownGtsType { gts_type: String, instance: String },
    InvalidIdentifier { detail: String, instance: String },
    MissingJwt { instance: String },
    InvalidJwt { detail: String, instance: String },
    ExpiredJwt { instance: String },
    ReadOnlyEntity { entity_id: String, instance: String },
    InvalidOdataQuery { detail: String, available_fields: Vec<String>, instance: String },
    UnsupportedField { field: String, available_fields: Vec<String>, instance: String },
    InvalidJsonPatch { detail: String, instance: String },
    PatchPathRestricted { path: String, instance: String },
    DomainFeatureUnavailable { gts_type: String, instance: String },
    DomainFeatureError { detail: String, instance: String },
}

impl GtsCoreError {
    pub fn to_problem_details(&self) -> ProblemDetails {
        match self {
            GtsCoreError::UnknownGtsType { gts_type, instance } => {
                ProblemDetails::routing_error(
                    &format!("No domain feature registered for GTS type '{}'", gts_type),
                    Some(instance.clone()),
                )
            }
            GtsCoreError::InvalidIdentifier { detail, instance } => {
                ProblemDetails::invalid_identifier(detail, Some(instance.clone()))
            }
            GtsCoreError::MissingJwt { instance } => {
                ProblemDetails::authentication_error(
                    "Missing Authorization header. JWT token is required.",
                    Some(instance.clone()),
                )
            }
            GtsCoreError::InvalidJwt { detail, instance } => {
                ProblemDetails::authentication_error(detail, Some(instance.clone()))
            }
            GtsCoreError::ExpiredJwt { instance } => {
                ProblemDetails::authentication_error(
                    "JWT token has expired. Please refresh your token.",
                    Some(instance.clone()),
                )
            }
            GtsCoreError::ReadOnlyEntity { entity_id, instance } => {
                ProblemDetails::read_only_entity(entity_id, Some(instance.clone()))
            }
            GtsCoreError::InvalidOdataQuery { detail, available_fields, instance } => {
                ProblemDetails::invalid_odata_query(detail, available_fields, Some(instance.clone()))
            }
            GtsCoreError::UnsupportedField { field, available_fields, instance } => {
                ProblemDetails::invalid_odata_query(
                    &format!("Field '{}' is not indexed and cannot be used in $filter", field),
                    available_fields,
                    Some(instance.clone()),
                )
            }
            GtsCoreError::InvalidJsonPatch { detail, instance } => {
                ProblemDetails::invalid_json_patch(detail, Some(instance.clone()))
            }
            GtsCoreError::PatchPathRestricted { path, instance } => {
                ProblemDetails::invalid_json_patch(
                    &format!(
                        "JSON Patch path '{}' is not allowed. Only paths starting with '/entity/' are permitted.",
                        path
                    ),
                    Some(instance.clone()),
                )
            }
            GtsCoreError::DomainFeatureUnavailable { gts_type, instance } => {
                ProblemDetails::domain_feature_unavailable(gts_type, Some(instance.clone()))
            }
            GtsCoreError::DomainFeatureError { detail, instance } => {
                ProblemDetails::service_error(detail, Some(instance.clone()))
            }
        }
    }
}

impl IntoResponse for GtsCoreError {
    fn into_response(self) -> Response {
        self.to_problem_details().into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_error_format() {
        let problem = ProblemDetails::routing_error(
            "No domain feature registered for GTS type 'gts.hypernetix.hyperspot.analytics.unknown.v1~'",
            Some("/api/analytics/v1/gts/gts.hypernetix.hyperspot.analytics.unknown.v1~test.v1".to_string()),
        );

        assert_eq!(problem.status, 404);
        assert_eq!(problem.title, "Routing Failed");
        assert_eq!(problem.problem_type, "https://hyperspot.dev/problems/routing-failed");
        assert!(problem.detail.contains("gts.hypernetix.hyperspot.analytics.unknown.v1~"));
        assert!(!problem.trace_id.is_empty());
    }

    #[test]
    fn test_authentication_error_format() {
        let problem = ProblemDetails::authentication_error(
            "JWT signature validation failed",
            Some("/api/analytics/v1/gts/test".to_string()),
        );

        assert_eq!(problem.status, 401);
        assert_eq!(problem.title, "Authentication Failed");
        assert!(!problem.trace_id.is_empty());
    }

    #[test]
    fn test_read_only_entity_error() {
        let problem = ProblemDetails::read_only_entity(
            "gts.hypernetix.hyperspot.ax.query.v1~",
            Some("/api/analytics/v1/gts/gts.hypernetix.hyperspot.ax.query.v1~".to_string()),
        );

        assert_eq!(problem.status, 403);
        assert_eq!(problem.title, "Read-Only Entity");
        assert!(problem.detail.contains("configuration files"));
        assert!(!problem.trace_id.is_empty());
    }

    #[test]
    fn test_invalid_odata_query_with_fields() {
        let available_fields = vec![
            "entity/name".to_string(),
            "entity/age".to_string(),
            "entity/email".to_string(),
        ];
        let problem = ProblemDetails::invalid_odata_query(
            "Field 'entity/unsupported' is not indexed",
            &available_fields,
            Some("/api/analytics/v1/gts".to_string()),
        );

        assert_eq!(problem.status, 400);
        assert_eq!(problem.title, "Invalid OData Query");
        assert!(problem.detail.contains("Available indexed fields"));
        assert!(problem.detail.contains("entity/name"));
        assert!(!problem.trace_id.is_empty());
    }

    #[test]
    fn test_service_unavailable_error() {
        let problem = ProblemDetails::domain_feature_unavailable(
            "gts.test.type.v1~",
            Some("/api/analytics/v1/gts/test".to_string()),
        );

        assert_eq!(problem.status, 503);
        assert_eq!(problem.title, "Domain Feature Unavailable");
        assert!(problem.detail.contains("temporarily unavailable"));
        assert!(problem.detail.contains("retry"));
        assert!(!problem.trace_id.is_empty());
    }

    #[test]
    fn test_gts_core_error_to_problem_details() {
        let error = GtsCoreError::UnknownGtsType {
            gts_type: "gts.unknown.v1~".to_string(),
            instance: "/api/analytics/v1/gts/test".to_string(),
        };

        let problem = error.to_problem_details();
        assert_eq!(problem.status, 404);
        assert!(problem.detail.contains("gts.unknown.v1~"));
    }

    #[test]
    fn test_patch_path_restricted_error() {
        let error = GtsCoreError::PatchPathRestricted {
            path: "/id".to_string(),
            instance: "/api/analytics/v1/gts/test".to_string(),
        };

        let problem = error.to_problem_details();
        assert_eq!(problem.status, 400);
        assert!(problem.detail.contains("'/entity/'"));
        assert!(problem.detail.contains("/id"));
    }

    #[test]
    fn test_trace_id_uniqueness() {
        let problem1 = ProblemDetails::routing_error("test", None);
        let problem2 = ProblemDetails::routing_error("test", None);
        
        assert_ne!(problem1.trace_id, problem2.trace_id);
    }
}
