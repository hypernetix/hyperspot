//! REST DTOs for the Types Registry module.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use gts::GtsIdSegment;
use types_registry_sdk::{GtsEntity, RegisterResult, RegisterSummary, SegmentMatchScope};

/// DTO for a GTS ID segment.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GtsIdSegmentDto {
    /// Vendor component of the segment.
    pub vendor: String,
    /// Package component of the segment.
    pub package: String,
    /// Namespace component of the segment.
    pub namespace: String,
    /// Type name component of the segment.
    pub type_name: String,
    /// Major version number.
    pub ver_major: u32,
}

impl From<&GtsIdSegment> for GtsIdSegmentDto {
    fn from(segment: &GtsIdSegment) -> Self {
        Self {
            vendor: segment.vendor.clone(),
            package: segment.package.clone(),
            namespace: segment.namespace.clone(),
            type_name: segment.type_name.clone(),
            ver_major: segment.ver_major,
        }
    }
}

/// Response DTO for a GTS entity.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GtsEntityDto {
    /// Deterministic UUID generated from the GTS ID.
    pub id: Uuid,
    /// The full GTS identifier string.
    pub gts_id: String,
    /// All parsed segments from the GTS ID.
    pub segments: Vec<GtsIdSegmentDto>,
    /// Whether this entity is a schema (type definition).
    ///
    /// - `true`: This is a type definition (GTS ID ends with `~`)
    /// - `false`: This is an instance (GTS ID does not end with `~`)
    pub is_schema: bool,
    /// The entity content (schema for types, object for instances).
    pub content: serde_json::Value,
    /// Optional description of the entity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl From<GtsEntity> for GtsEntityDto {
    fn from(entity: GtsEntity) -> Self {
        Self {
            id: entity.id,
            gts_id: entity.gts_id.clone(),
            segments: entity.segments.iter().map(GtsIdSegmentDto::from).collect(),
            is_schema: entity.is_schema,
            content: entity.content.clone(),
            description: entity.description.clone(),
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
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "status")]
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
pub struct RegisterEntitiesResponse {
    /// Summary of the registration operation.
    pub summary: RegisterSummaryDto,
    /// Results for each entity in the request.
    pub results: Vec<RegisterResultDto>,
}

/// Summary of a batch registration operation.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
pub struct ListEntitiesQuery {
    /// Optional wildcard pattern for GTS ID matching.
    #[serde(default)]
    pub pattern: Option<String>,
    /// Filter by schema type: true for types, false for instances.
    #[serde(default)]
    pub is_schema: Option<bool>,
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

        if let Some(is_schema) = self.is_schema {
            query = query.with_is_type(is_schema);
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
pub struct ListEntitiesResponse {
    /// The list of entities.
    pub entities: Vec<GtsEntityDto>,
    /// Total count of entities returned.
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gts::GtsIdSegment;
    use types_registry_sdk::TypesRegistryError;

    #[test]
    fn test_gts_entity_dto_from_entity() {
        let segment = GtsIdSegment::new(0, 0, "acme.core.events.user_created.v1~").unwrap();
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.events.user_created.v1~",
            vec![segment],
            true, // is_schema
            serde_json::json!({"type": "object"}),
            Some("A user created event".to_owned()),
        );

        let dto: GtsEntityDto = entity.into();
        assert_eq!(dto.gts_id, "gts.acme.core.events.user_created.v1~");
        assert!(dto.is_schema);
        assert_eq!(dto.segments.len(), 1);
        assert_eq!(dto.segments[0].vendor, "acme");
        assert_eq!(dto.segments[0].package, "core");
        assert_eq!(dto.segments[0].namespace, "events");
        assert_eq!(dto.segments[0].type_name, "user_created");
        assert_eq!(dto.segments[0].ver_major, 1);
        assert_eq!(dto.description, Some("A user created event".to_owned()));
    }

    #[test]
    fn test_gts_entity_dto_instance() {
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.events.user_created.v1~acme.core.instances.instance1.v1",
            vec![],
            false, // is_schema
            serde_json::json!({"data": "value"}),
            None,
        );

        let dto: GtsEntityDto = entity.into();
        assert!(!dto.is_schema);
        assert!(dto.segments.is_empty());
        assert_eq!(dto.description, None);
    }

    #[test]
    fn test_gts_entity_dto_with_multiple_segments() {
        let segment1 = GtsIdSegment::new(0, 0, "acme.core.models.user.v1~").unwrap();
        let segment2 = GtsIdSegment::new(1, 30, "acme.core.instances.user1.v1").unwrap();
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.models.user.v1~acme.core.instances.user1.v1",
            vec![segment1, segment2],
            false, // is_schema
            serde_json::json!({"userId": "user-001"}),
            None,
        );

        let dto: GtsEntityDto = entity.into();
        assert!(!dto.is_schema);
        assert_eq!(dto.segments.len(), 2);
        // First segment (type)
        assert_eq!(dto.segments[0].vendor, "acme");
        assert_eq!(dto.segments[0].type_name, "user");
        // Second segment (instance)
        assert_eq!(dto.segments[1].vendor, "acme");
        assert_eq!(dto.segments[1].type_name, "user1");
    }

    #[test]
    fn test_gts_entity_dto_with_different_vendors_in_segments() {
        // Instance where type and instance have different vendors
        let segment1 = GtsIdSegment::new(0, 0, "acme.core.models.product.v1~").unwrap();
        let segment2 = GtsIdSegment::new(1, 32, "globex.retail.instances.prod1.v1").unwrap();
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.models.product.v1~globex.retail.instances.prod1.v1",
            vec![segment1, segment2],
            false, // is_schema
            serde_json::json!({"productId": "prod-001"}),
            None,
        );

        let dto: GtsEntityDto = entity.into();
        assert_eq!(dto.segments.len(), 2);
        // Type segment from vendor "acme"
        assert_eq!(dto.segments[0].vendor, "acme");
        assert_eq!(dto.segments[0].package, "core");
        assert_eq!(dto.segments[0].namespace, "models");
        assert_eq!(dto.segments[0].type_name, "product");
        assert_eq!(dto.segments[0].ver_major, 1);
        // Instance segment from different vendor "globex"
        assert_eq!(dto.segments[1].vendor, "globex");
        assert_eq!(dto.segments[1].package, "retail");
        assert_eq!(dto.segments[1].namespace, "instances");
        assert_eq!(dto.segments[1].type_name, "prod1");
        assert_eq!(dto.segments[1].ver_major, 1);
    }

    #[test]
    fn test_gts_id_segment_dto_serialization() {
        let segment = GtsIdSegment::new(0, 0, "acme.billing.invoices.invoice.v2~").unwrap();
        let dto = GtsIdSegmentDto::from(&segment);

        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["vendor"], "acme");
        assert_eq!(json["package"], "billing");
        assert_eq!(json["namespace"], "invoices");
        assert_eq!(json["type_name"], "invoice");
        assert_eq!(json["ver_major"], 2);
    }

    #[test]
    fn test_gts_entity_dto_segments_serialization() {
        let segment1 = GtsIdSegment::new(0, 0, "vendor_a.pkg1.ns1.type1.v1~").unwrap();
        let segment2 = GtsIdSegment::new(1, 28, "vendor_b.pkg2.ns2.inst1.v2").unwrap();
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.vendor_a.pkg1.ns1.type1.v1~vendor_b.pkg2.ns2.inst1.v2",
            vec![segment1, segment2],
            false, // is_schema
            serde_json::json!({}),
            None,
        );

        let dto: GtsEntityDto = entity.into();
        let json = serde_json::to_value(&dto).unwrap();

        let json_segments = json["segments"].as_array().unwrap();
        assert_eq!(json_segments.len(), 2);
        assert_eq!(json_segments[0]["vendor"], "vendor_a");
        assert_eq!(json_segments[1]["vendor"], "vendor_b");
    }

    #[test]
    fn test_list_entities_query_to_list_query() {
        let dto = ListEntitiesQuery {
            #[allow(unknown_lints)]
            #[allow(de0901_gts_string_pattern)]
            pattern: Some("gts.acme.*".to_owned()),
            is_schema: Some(true),
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
    fn test_list_entities_query_is_schema_false() {
        let dto = ListEntitiesQuery {
            pattern: None,
            is_schema: Some(false),
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
    fn test_list_entities_query_invalid_segment_scope() {
        let dto = ListEntitiesQuery {
            pattern: None,
            is_schema: None,
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
            true, // is_schema
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
            gts_id: Some("gts.x.core.events.test.v1~".to_owned()),
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
