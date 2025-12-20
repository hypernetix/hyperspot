//! REST DTOs for the Types Registry module.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use types_registry_sdk::{
    GtsEntity, GtsEntityKind, RegisterResult, RegisterSummary, SegmentMatchScope,
};

/// Response DTO for a GTS entity.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GtsEntityDto {
    /// Deterministic UUID generated from the GTS ID.
    pub id: Uuid,
    /// The full GTS identifier string.
    pub gts_id: String,
    /// The kind of entity: "type" or "instance".
    pub kind: String,
    /// The entity content (schema for types, object for instances).
    pub content: serde_json::Value,
    /// Optional description of the entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Vendor from the primary segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// Package from the primary segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Namespace from the primary segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

impl From<GtsEntity> for GtsEntityDto {
    fn from(entity: GtsEntity) -> Self {
        Self {
            id: entity.id,
            gts_id: entity.gts_id.clone(),
            kind: match entity.kind {
                GtsEntityKind::Type => "type".to_owned(),
                GtsEntityKind::Instance => "instance".to_owned(),
            },
            content: entity.content.clone(),
            description: entity.description.clone(),
            vendor: entity.vendor().map(ToOwned::to_owned),
            package: entity.package().map(ToOwned::to_owned),
            namespace: entity.namespace().map(ToOwned::to_owned),
        }
    }
}

/// Request DTO for registering GTS entities.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RegisterEntitiesRequest {
    /// Array of GTS entities to register.
    pub entities: Vec<serde_json::Value>,
}

/// Result of registering a single entity.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum RegisterResultDto {
    /// Successfully registered entity.
    #[serde(rename = "ok")]
    Ok {
        /// The registered entity.
        entity: GtsEntityDto,
    },
    /// Failed to register entity.
    #[serde(rename = "error")]
    Error {
        /// The GTS ID that was attempted, if available.
        #[serde(skip_serializing_if = "Option::is_none")]
        gts_id: Option<String>,
        /// Error message.
        error: String,
    },
}

impl From<RegisterResult> for RegisterResultDto {
    fn from(result: RegisterResult) -> Self {
        match result {
            RegisterResult::Ok(entity) => Self::Ok {
                entity: entity.into(),
            },
            RegisterResult::Err { gts_id, error } => Self::Error {
                gts_id,
                error: error.to_string(),
            },
        }
    }
}

/// Response DTO for batch registration.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterEntitiesResponse {
    /// Summary of the registration operation.
    pub summary: RegisterSummaryDto,
    /// Results for each entity in the request.
    pub results: Vec<RegisterResultDto>,
}

/// Summary of a batch registration operation.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterSummaryDto {
    /// Total number of entities processed.
    pub total: usize,
    /// Number of successfully registered entities.
    pub succeeded: usize,
    /// Number of failed registrations.
    pub failed: usize,
}

impl From<RegisterSummary> for RegisterSummaryDto {
    fn from(summary: RegisterSummary) -> Self {
        Self {
            total: summary.total(),
            succeeded: summary.succeeded,
            failed: summary.failed,
        }
    }
}

/// Query parameters for listing GTS entities.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListEntitiesQuery {
    /// Optional wildcard pattern for GTS ID matching.
    #[serde(default)]
    pub pattern: Option<String>,
    /// Filter for entity kind: "type" or "instance".
    #[serde(default)]
    pub kind: Option<String>,
    /// Filter by vendor.
    #[serde(default)]
    pub vendor: Option<String>,
    /// Filter by package.
    #[serde(default)]
    pub package: Option<String>,
    /// Filter by namespace.
    #[serde(default)]
    pub namespace: Option<String>,
    /// Segment match scope: "primary" or "any" (default).
    #[serde(default)]
    pub segment_scope: Option<String>,
}

impl ListEntitiesQuery {
    /// Converts this DTO to the SDK `ListQuery`.
    #[must_use]
    pub fn to_list_query(&self) -> types_registry_sdk::ListQuery {
        let mut query = types_registry_sdk::ListQuery::default();

        if let Some(ref pattern) = self.pattern {
            query = query.with_pattern(pattern);
        }

        if let Some(ref kind) = self.kind {
            match kind.as_str() {
                "type" => query = query.with_is_type(true),
                "instance" => query = query.with_is_type(false),
                _ => {}
            }
        }

        if let Some(ref vendor) = self.vendor {
            query = query.with_vendor(vendor);
        }

        if let Some(ref package) = self.package {
            query = query.with_package(package);
        }

        if let Some(ref namespace) = self.namespace {
            query = query.with_namespace(namespace);
        }

        if let Some(ref scope) = self.segment_scope {
            match scope.as_str() {
                "primary" => query = query.with_segment_scope(SegmentMatchScope::Primary),
                "any" => query = query.with_segment_scope(SegmentMatchScope::Any),
                _ => {}
            }
        }

        query
    }
}

/// Response DTO for listing GTS entities.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListEntitiesResponse {
    /// The list of entities.
    pub entities: Vec<GtsEntityDto>,
    /// Total count of entities returned.
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use types_registry_sdk::{GtsEntityKind, GtsIdSegment, TypesRegistryError};

    #[test]
    fn test_gts_entity_dto_from_entity() {
        let segment = GtsIdSegment::new(0, 0, "acme.core.events.user_created.v1~").unwrap();
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.events.user_created.v1~",
            vec![segment],
            GtsEntityKind::Type,
            serde_json::json!({"type": "object"}),
            Some("A user created event".to_owned()),
        );

        let dto: GtsEntityDto = entity.into();
        assert_eq!(dto.gts_id, "gts.acme.core.events.user_created.v1~");
        assert_eq!(dto.kind, "type");
        assert_eq!(dto.vendor, Some("acme".to_owned()));
        assert_eq!(dto.package, Some("core".to_owned()));
        assert_eq!(dto.namespace, Some("events".to_owned()));
        assert_eq!(dto.description, Some("A user created event".to_owned()));
    }

    #[test]
    fn test_gts_entity_dto_instance() {
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.events.user_created.v1~instance1",
            vec![],
            GtsEntityKind::Instance,
            serde_json::json!({"data": "value"}),
            None,
        );

        let dto: GtsEntityDto = entity.into();
        assert_eq!(dto.kind, "instance");
        assert_eq!(dto.vendor, None);
        assert_eq!(dto.description, None);
    }

    #[test]
    fn test_list_entities_query_to_list_query() {
        let dto = ListEntitiesQuery {
            pattern: Some("gts.acme.*".to_owned()),
            kind: Some("type".to_owned()),
            vendor: Some("acme".to_owned()),
            package: None,
            namespace: None,
            segment_scope: Some("primary".to_owned()),
        };

        let query = dto.to_list_query();
        assert_eq!(query.pattern, Some("gts.acme.*".to_owned()));
        assert_eq!(query.is_type, Some(true));
        assert_eq!(query.vendor, Some("acme".to_owned()));
        assert_eq!(query.segment_scope, SegmentMatchScope::Primary);
    }

    #[test]
    fn test_list_entities_query_instance_kind() {
        let dto = ListEntitiesQuery {
            pattern: None,
            kind: Some("instance".to_owned()),
            vendor: None,
            package: Some("core".to_owned()),
            namespace: Some("events".to_owned()),
            segment_scope: Some("any".to_owned()),
        };

        let query = dto.to_list_query();
        assert_eq!(query.is_type, Some(false));
        assert_eq!(query.package, Some("core".to_owned()));
        assert_eq!(query.namespace, Some("events".to_owned()));
        assert_eq!(query.segment_scope, SegmentMatchScope::Any);
    }

    #[test]
    fn test_list_entities_query_unknown_kind() {
        let dto = ListEntitiesQuery {
            pattern: None,
            kind: Some("unknown".to_owned()),
            vendor: None,
            package: None,
            namespace: None,
            segment_scope: Some("invalid".to_owned()),
        };

        let query = dto.to_list_query();
        assert_eq!(query.is_type, None);
        assert_eq!(query.segment_scope, SegmentMatchScope::Any);
    }

    #[test]
    fn test_list_entities_query_default() {
        let dto = ListEntitiesQuery::default();
        let query = dto.to_list_query();
        assert_eq!(query.pattern, None);
        assert_eq!(query.is_type, None);
        assert_eq!(query.vendor, None);
    }

    #[test]
    fn test_register_result_dto_ok() {
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.test.pkg.ns.type.v1~",
            vec![],
            GtsEntityKind::Type,
            serde_json::json!({}),
            None,
        );
        let result = RegisterResult::Ok(entity);
        let dto: RegisterResultDto = result.into();
        assert!(matches!(dto, RegisterResultDto::Ok { .. }));
    }

    #[test]
    fn test_register_result_dto_err() {
        let result: RegisterResult = RegisterResult::Err {
            gts_id: Some("gts.test~".to_owned()),
            error: TypesRegistryError::validation_failed("test error"),
        };
        let dto: RegisterResultDto = result.into();
        assert!(matches!(dto, RegisterResultDto::Error { .. }));
    }

    #[test]
    fn test_register_summary_dto() {
        let summary = RegisterSummary {
            succeeded: 5,
            failed: 2,
        };
        let dto: RegisterSummaryDto = summary.into();
        assert_eq!(dto.total, 7);
        assert_eq!(dto.succeeded, 5);
        assert_eq!(dto.failed, 2);
    }
}
