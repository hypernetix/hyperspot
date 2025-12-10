//! OpenAPI registry for schema and operation management.
//!
//! This module acts as a central repository for API operations and data schemas.
//! It allows distributed registration of endpoints (e.g., from different modules or handlers)
//! and their associated data types.
//!
//! # Flow
//! 1. **Registration**: Handlers register their operations via `register_operation`.
//! 2. **Schema Collection**: Data types used in requests/responses are registered via `ensure_schema`.
//! 3. **Generation**: The `build_openapi` method aggregates all registered data into a standard OpenAPI 3.0 document.
//!
//! This design decouples the definition of API logic from the generation of documentation,
//! enabling automatic discovery and documentation of available endpoints.

use anyhow::Result;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::openapi::{
    content::ContentBuilder,
    info::InfoBuilder,
    path::{
        HttpMethod, OperationBuilder as UOperationBuilder, ParameterBuilder, ParameterIn,
        PathItemBuilder, PathsBuilder,
    },
    request_body::RequestBodyBuilder,
    response::{ResponseBuilder, ResponsesBuilder},
    schema::{ComponentsBuilder, ObjectBuilder, Schema, SchemaFormat, SchemaType},
    security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    OpenApi, OpenApiBuilder, Ref, RefOr, Required,
};

use crate::api::{operation_builder, problem};

/// Type alias for schema collections used in API operations.
type SchemaCollection = Vec<(String, RefOr<Schema>)>;

/// OpenAPI document metadata (title, version, description)
#[derive(Debug, Clone)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

impl Default for OpenApiInfo {
    fn default() -> Self {
        Self {
            title: "API Documentation".to_string(),
            version: "0.1.0".to_string(),
            description: None,
        }
    }
}

/// Interface for registering API operations and schemas.
///
/// This trait allows different parts of the application to register their API capabilities
/// without knowing the concrete storage mechanism. It is typically used by
/// route handlers or module initialization logic to declare what endpoints they expose.
pub trait OpenApiRegistry: Send + Sync {
    /// Register an API operation specification
    fn register_operation(&self, spec: &operation_builder::OperationSpec);

    /// Ensure schema for a type (including transitive dependencies) is registered
    /// under components and return the canonical component name for `$ref`.
    /// This is a type-erased version for dyn compatibility.
    fn ensure_schema_raw(&self, name: &str, schemas: SchemaCollection) -> String;

    /// Downcast support for accessing the concrete implementation if needed.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Registers a type and its dependencies into the registry.
///
/// This helper ensures that a type `T` (which implements `ToSchema`) and all its
/// nested types are added to the registry's component section. It returns the
/// component name, which can be used to reference the schema in operations.
pub fn ensure_schema<T: utoipa::ToSchema + utoipa::PartialSchema + 'static>(
    registry: &dyn OpenApiRegistry,
) -> String {
    use utoipa::PartialSchema;

    // 1) Canonical component name for T as seen by utoipa
    let root_name = T::name().to_string();

    // 2) Always insert T's own schema first (actual object, not a ref)
    //    This avoids self-referential components.
    let mut collected: SchemaCollection = vec![(root_name.clone(), <T as PartialSchema>::schema())];

    // 3) Collect and append all referenced schemas (dependencies) of T
    T::schemas(&mut collected);

    // 4) Pass to registry for insertion
    registry.ensure_schema_raw(&root_name, collected)
}

/// Thread-safe implementation of the OpenAPI registry.
///
/// Uses `DashMap` for concurrent operation registration and `ArcSwap` for
/// lock-free schema reads, making it suitable for use in a multi-threaded
/// web server environment where operations might be registered during startup
/// or dynamically.
pub struct OpenApiRegistryImpl {
    /// Store operation specs keyed by "METHOD:path"
    pub operation_specs: DashMap<String, operation_builder::OperationSpec>,
    /// Store schema components using arc-swap for lock-free reads
    pub components_registry: ArcSwap<HashMap<String, RefOr<Schema>>>,
}

impl OpenApiRegistryImpl {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            operation_specs: DashMap::new(),
            components_registry: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    /// Generates the complete OpenAPI specification.
    ///
    /// Aggregates all registered operations and schemas into a `utoipa::openapi::OpenApi` object.
    /// This includes:
    /// - Converting internal operation specs to OpenAPI paths and operations.
    /// - Mapping parameters, request bodies, and responses.
    /// - collecting all referenced schemas into the `components` section.
    /// - Applying global metadata (title, version, etc.).
    ///
    /// # Arguments
    /// * `info` - OpenAPI document metadata (title, version, description)
    pub fn build_openapi(&self, info: &OpenApiInfo) -> Result<OpenApi> {
        use http::Method;

        // Log operation count for visibility
        let op_count = self.operation_specs.len();
        tracing::info!("Building OpenAPI: found {op_count} registered operations");

        // 1) Paths
        let mut paths = PathsBuilder::new();

        for spec in self.operation_specs.iter().map(|e| e.value().clone()) {
            let mut op = UOperationBuilder::new()
                .operation_id(spec.operation_id.clone().or(Some(spec.handler_id.clone())))
                .summary(spec.summary.clone())
                .description(spec.description.clone());

            for tag in &spec.tags {
                op = op.tag(tag.clone());
            }

            // Vendor extensions for rate limit, if present (string values)
            if let Some(rl) = spec.rate_limit.as_ref() {
                let mut ext = utoipa::openapi::extensions::Extensions::default();
                ext.insert("x-rate-limit-rps".to_string(), serde_json::json!(rl.rps));
                ext.insert(
                    "x-rate-limit-burst".to_string(),
                    serde_json::json!(rl.burst),
                );
                ext.insert(
                    "x-in-flight-limit".to_string(),
                    serde_json::json!(rl.in_flight),
                );
                op = op.extensions(Some(ext));
            }

            // Parameters
            for p in &spec.params {
                let in_ = match p.location {
                    operation_builder::ParamLocation::Path => ParameterIn::Path,
                    operation_builder::ParamLocation::Query => ParameterIn::Query,
                    operation_builder::ParamLocation::Header => ParameterIn::Header,
                    operation_builder::ParamLocation::Cookie => ParameterIn::Cookie,
                };
                let required =
                    if matches!(p.location, operation_builder::ParamLocation::Path) || p.required {
                        Required::True
                    } else {
                        Required::False
                    };

                let schema_type = match p.param_type.as_str() {
                    "integer" => SchemaType::Type(utoipa::openapi::schema::Type::Integer),
                    "number" => SchemaType::Type(utoipa::openapi::schema::Type::Number),
                    "boolean" => SchemaType::Type(utoipa::openapi::schema::Type::Boolean),
                    _ => SchemaType::Type(utoipa::openapi::schema::Type::String),
                };
                let schema = Schema::Object(ObjectBuilder::new().schema_type(schema_type).build());

                let param = ParameterBuilder::new()
                    .name(&p.name)
                    .parameter_in(in_)
                    .required(required)
                    .description(p.description.clone())
                    .schema(Some(schema))
                    .build();

                op = op.parameter(param);
            }

            // Request body
            if let Some(rb) = &spec.request_body {
                let content = match &rb.schema {
                    operation_builder::RequestBodySchema::Ref { schema_name } => {
                        ContentBuilder::new()
                            .schema(Some(RefOr::Ref(Ref::from_schema_name(schema_name.clone()))))
                            .build()
                    }
                    operation_builder::RequestBodySchema::MultipartFile { field_name } => {
                        // Build multipart/form-data schema with a single binary file field
                        // type: object
                        // properties:
                        //   {field_name}: { type: string, format: binary }
                        // required: [ field_name ]
                        let file_schema = Schema::Object(
                            ObjectBuilder::new()
                                .schema_type(SchemaType::Type(
                                    utoipa::openapi::schema::Type::String,
                                ))
                                .format(Some(SchemaFormat::Custom("binary".into())))
                                .build(),
                        );
                        let obj = ObjectBuilder::new()
                            .property(field_name.clone(), file_schema)
                            .required(field_name.clone());
                        let schema = Schema::Object(obj.build());
                        ContentBuilder::new().schema(Some(schema)).build()
                    }
                    operation_builder::RequestBodySchema::Binary => {
                        // Represent raw binary body as type string, format binary.
                        // This is used for application/octet-stream and similar raw binary content.
                        let schema = Schema::Object(
                            ObjectBuilder::new()
                                .schema_type(SchemaType::Type(
                                    utoipa::openapi::schema::Type::String,
                                ))
                                .format(Some(SchemaFormat::Custom("binary".into())))
                                .build(),
                        );

                        ContentBuilder::new().schema(Some(schema)).build()
                    }
                    operation_builder::RequestBodySchema::InlineObject => {
                        // Preserve previous behavior for inline object bodies
                        ContentBuilder::new()
                            .schema(Some(Schema::Object(ObjectBuilder::new().build())))
                            .build()
                    }
                };
                let mut rbld = RequestBodyBuilder::new()
                    .description(rb.description.clone())
                    .content(rb.content_type.to_string(), content);
                if rb.required {
                    rbld = rbld.required(Some(Required::True));
                }
                op = op.request_body(Some(rbld.build()));
            }

            // Responses
            let mut responses = ResponsesBuilder::new();
            for r in &spec.responses {
                let is_json_like = r.content_type == "application/json"
                    || r.content_type == problem::APPLICATION_PROBLEM_JSON
                    || r.content_type == "text/event-stream";
                let resp = if is_json_like {
                    if let Some(name) = &r.schema_name {
                        // Manually build content to preserve the correct content type
                        let content = ContentBuilder::new()
                            .schema(Some(RefOr::Ref(Ref::new(format!(
                                "#/components/schemas/{}",
                                name
                            )))))
                            .build();
                        ResponseBuilder::new()
                            .description(&r.description)
                            .content(r.content_type, content)
                            .build()
                    } else {
                        let content = ContentBuilder::new()
                            .schema(Some(Schema::Object(ObjectBuilder::new().build())))
                            .build();
                        ResponseBuilder::new()
                            .description(&r.description)
                            .content(r.content_type, content)
                            .build()
                    }
                } else {
                    let schema = Schema::Object(
                        ObjectBuilder::new()
                            .schema_type(SchemaType::Type(utoipa::openapi::schema::Type::String))
                            .format(Some(SchemaFormat::Custom(r.content_type.into())))
                            .build(),
                    );
                    let content = ContentBuilder::new().schema(Some(schema)).build();
                    ResponseBuilder::new()
                        .description(&r.description)
                        .content(r.content_type, content)
                        .build()
                };
                responses = responses.response(r.status.to_string(), resp);
            }
            op = op.responses(responses.build());

            // Add security requirement if operation has explicit auth metadata
            if spec.sec_requirement.is_some() {
                let sec_req = utoipa::openapi::security::SecurityRequirement::new(
                    "bearerAuth",
                    Vec::<String>::new(),
                );
                op = op.security(sec_req);
            }

            let method = match spec.method {
                Method::GET => HttpMethod::Get,
                Method::POST => HttpMethod::Post,
                Method::PUT => HttpMethod::Put,
                Method::DELETE => HttpMethod::Delete,
                Method::PATCH => HttpMethod::Patch,
                _ => HttpMethod::Get,
            };

            let item = PathItemBuilder::new().operation(method, op.build()).build();
            // Convert Axum-style path to OpenAPI-style path
            let openapi_path = operation_builder::axum_to_openapi_path(&spec.path);
            paths = paths.path(openapi_path, item);
        }

        // 2) Components (from our registry)
        let mut components = ComponentsBuilder::new();
        for (name, schema) in self.components_registry.load().iter() {
            components = components.schema(name.clone(), schema.clone());
        }

        // Add bearer auth security scheme
        components = components.security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );

        // 3) Info & final OpenAPI doc
        let openapi_info = InfoBuilder::new()
            .title(&info.title)
            .version(&info.version)
            .description(info.description.clone())
            .build();

        let openapi = OpenApiBuilder::new()
            .info(openapi_info)
            .paths(paths.build())
            .components(Some(components.build()))
            .build();

        Ok(openapi)
    }
}

impl Default for OpenApiRegistryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenApiRegistry for OpenApiRegistryImpl {
    fn register_operation(&self, spec: &operation_builder::OperationSpec) {
        let operation_key = format!("{}:{}", spec.method.as_str(), spec.path);
        self.operation_specs
            .insert(operation_key.clone(), spec.clone());

        tracing::debug!(
            handler_id = %spec.handler_id,
            method = %spec.method.as_str(),
            path = %spec.path,
            summary = %spec.summary.as_deref().unwrap_or("No summary"),
            operation_key = %operation_key,
            "Registered API operation in registry"
        );
    }

    fn ensure_schema_raw(&self, root_name: &str, schemas: SchemaCollection) -> String {
        // Snapshot & copy-on-write
        let current = self.components_registry.load();
        let mut reg = (**current).clone();

        for (name, schema) in schemas {
            // Conflict policy: identical → no-op; different → warn & override
            if let Some(existing) = reg.get(&name) {
                let a = serde_json::to_value(existing).ok();
                let b = serde_json::to_value(&schema).ok();
                if a == b {
                    continue; // Skip identical schemas
                } else {
                    tracing::warn!(%name, "Schema content conflict; overriding with latest");
                }
            }
            reg.insert(name, schema);
        }

        self.components_registry.store(Arc::new(reg));
        root_name.to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::api::operation_builder::{
        OperationSpec, ParamLocation, ParamSpec, RateLimitSpec, ResponseSpec,
    };
    use http::Method;

    // Helper function to create a minimal operation spec for testing
    fn minimal_op(path: &str, handler_id: &str) -> OperationSpec {
        OperationSpec {
            method: Method::GET,
            path: path.to_string(),
            operation_id: Some(handler_id.to_string()),
            summary: None,
            description: None,
            tags: vec![],
            params: vec![],
            request_body: None,
            responses: vec![ResponseSpec {
                status: 200,
                content_type: "application/json",
                description: "Success".to_string(),
                schema_name: None,
            }],
            handler_id: handler_id.to_string(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: None,
        }
    }

    // Test schema struct for schema linking tests
    #[derive(utoipa::ToSchema, serde::Serialize)]
    struct TestModel {
        id: i32,
        name: String,
    }

    #[test]
    fn test_registry_creation() {
        let registry = OpenApiRegistryImpl::new();
        assert_eq!(registry.operation_specs.len(), 0);
        assert_eq!(registry.components_registry.load().len(), 0);
    }

    #[test]
    fn test_register_operation() {
        let registry = OpenApiRegistryImpl::new();
        let spec = OperationSpec {
            method: Method::GET,
            path: "/test".to_string(),
            operation_id: Some("test_op".to_string()),
            summary: Some("Test operation".to_string()),
            description: None,
            tags: vec![],
            params: vec![],
            request_body: None,
            responses: vec![ResponseSpec {
                status: 200,
                content_type: "application/json",
                description: "Success".to_string(),
                schema_name: None,
            }],
            handler_id: "get_test".to_string(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: None,
        };

        registry.register_operation(&spec);
        assert_eq!(registry.operation_specs.len(), 1);
    }

    #[test]
    fn test_build_empty_openapi() {
        let registry = OpenApiRegistryImpl::new();
        let info = OpenApiInfo {
            title: "Test API".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test API Description".to_string()),
        };
        let doc = registry.build_openapi(&info).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify it's valid OpenAPI document structure
        assert!(json.get("openapi").is_some());
        assert!(json.get("info").is_some());
        assert!(json.get("paths").is_some());

        // Verify info section
        let openapi_info = json.get("info").unwrap();
        assert_eq!(openapi_info.get("title").unwrap(), "Test API");
        assert_eq!(openapi_info.get("version").unwrap(), "1.0.0");
        assert_eq!(
            openapi_info.get("description").unwrap(),
            "Test API Description"
        );
    }

    #[test]
    fn test_build_openapi_with_operation() {
        let registry = OpenApiRegistryImpl::new();
        let spec = OperationSpec {
            method: Method::GET,
            path: "/users/{id}".to_string(),
            operation_id: Some("get_user".to_string()),
            summary: Some("Get user by ID".to_string()),
            description: Some("Retrieves a user by their ID".to_string()),
            tags: vec!["users".to_string()],
            params: vec![ParamSpec {
                name: "id".to_string(),
                location: ParamLocation::Path,
                required: true,
                description: Some("User ID".to_string()),
                param_type: "string".to_string(),
            }],
            request_body: None,
            responses: vec![ResponseSpec {
                status: 200,
                content_type: "application/json",
                description: "User found".to_string(),
                schema_name: None,
            }],
            handler_id: "get_users_id".to_string(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: None,
        };

        registry.register_operation(&spec);
        let info = OpenApiInfo::default();
        let doc = registry.build_openapi(&info).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify path exists
        let paths = json.get("paths").unwrap();
        assert!(paths.get("/users/{id}").is_some());

        // Verify operation details
        let get_op = paths.get("/users/{id}").unwrap().get("get").unwrap();
        assert_eq!(get_op.get("operationId").unwrap(), "get_user");
        assert_eq!(get_op.get("summary").unwrap(), "Get user by ID");
    }

    #[test]
    fn test_ensure_schema_raw() {
        let registry = OpenApiRegistryImpl::new();
        let schema = Schema::Object(ObjectBuilder::new().build());
        let schemas = vec![("TestSchema".to_string(), RefOr::T(schema))];

        let name = registry.ensure_schema_raw("TestSchema", schemas);
        assert_eq!(name, "TestSchema");
        assert_eq!(registry.components_registry.load().len(), 1);
    }

    #[test]
    fn test_build_openapi_with_binary_request() {
        use crate::api::operation_builder::RequestBodySchema;

        let registry = OpenApiRegistryImpl::new();
        let spec = OperationSpec {
            method: Method::POST,
            path: "/upload".to_string(),
            operation_id: Some("upload_file".to_string()),
            summary: Some("Upload a file".to_string()),
            description: Some("Upload raw binary file".to_string()),
            tags: vec!["upload".to_string()],
            params: vec![],
            request_body: Some(crate::api::operation_builder::RequestBodySpec {
                content_type: "application/octet-stream",
                description: Some("Raw file bytes".to_string()),
                schema: RequestBodySchema::Binary,
                required: true,
            }),
            responses: vec![ResponseSpec {
                status: 200,
                content_type: "application/json",
                description: "Upload successful".to_string(),
                schema_name: None,
            }],
            handler_id: "post_upload".to_string(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: Some(vec!["application/octet-stream"]),
        };

        registry.register_operation(&spec);
        let info = OpenApiInfo::default();
        let doc = registry.build_openapi(&info).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify path exists
        let paths = json.get("paths").unwrap();
        assert!(paths.get("/upload").is_some());

        // Verify request body has application/octet-stream with binary schema
        let post_op = paths.get("/upload").unwrap().get("post").unwrap();
        let request_body = post_op.get("requestBody").unwrap();
        let content = request_body.get("content").unwrap();
        let octet_stream = content
            .get("application/octet-stream")
            .expect("application/octet-stream content type should exist");

        // Verify schema is type: string, format: binary
        let schema = octet_stream.get("schema").unwrap();
        assert_eq!(schema.get("type").unwrap(), "string");
        assert_eq!(schema.get("format").unwrap(), "binary");

        // Verify required flag
        assert_eq!(request_body.get("required").unwrap(), true);
    }

    #[test]
    fn operation_references_registered_schema() {
        let registry = OpenApiRegistryImpl::new();

        // Register schema
        ensure_schema::<TestModel>(&registry);

        // Register operation that references the schema
        let mut op = minimal_op("/model", "get_model");
        op.responses[0].schema_name = Some("TestModel".to_string());
        registry.register_operation(&op);

        // Build and verify
        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify schema exists in components
        assert!(
            json.pointer("/components/schemas/TestModel").is_some(),
            "Schema should be registered in components"
        );

        // Verify operation references the schema
        let ref_path =
            json.pointer("/paths/~1model/get/responses/200/content/application~1json/schema/$ref");
        assert!(ref_path.is_some(), "Operation should reference schema");
        assert_eq!(
            ref_path.unwrap().as_str().unwrap(),
            "#/components/schemas/TestModel",
            "Reference should point to correct schema"
        );
    }

    #[test]
    fn security_requirement_generates_bearer_auth_scheme() {
        let registry = OpenApiRegistryImpl::new();

        // Register operation with security requirement
        let mut op = minimal_op("/secure", "secure_op");
        op.sec_requirement = Some(crate::api::operation_builder::OperationSecRequirement {
            resource: "test".to_string(),
            action: "read".to_string(),
        });
        registry.register_operation(&op);

        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify security scheme is defined in components
        let security_scheme = json.pointer("/components/securitySchemes/bearerAuth");
        assert!(
            security_scheme.is_some(),
            "Bearer auth security scheme should be defined"
        );

        // Verify scheme type
        let scheme_type = json.pointer("/components/securitySchemes/bearerAuth/type");
        assert_eq!(
            scheme_type.unwrap().as_str().unwrap(),
            "http",
            "Security scheme should be http type"
        );

        // Verify operation has security requirement
        let operation_security = json.pointer("/paths/~1secure/get/security");
        assert!(
            operation_security.is_some(),
            "Operation should have security requirement"
        );

        // Verify bearerAuth is in the security array
        let security_array = operation_security.unwrap().as_array().unwrap();
        assert!(
            !security_array.is_empty(),
            "Security array should not be empty"
        );
        assert!(
            security_array[0].get("bearerAuth").is_some(),
            "Security should reference bearerAuth"
        );
    }

    #[test]
    fn rate_limit_metadata_generates_vendor_extensions() {
        let registry = OpenApiRegistryImpl::new();

        // Register operation with rate limit
        let mut op = minimal_op("/limited", "limited_op");
        op.rate_limit = Some(RateLimitSpec {
            rps: 10,
            burst: 20,
            in_flight: 5,
        });
        registry.register_operation(&op);

        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify vendor extensions are present
        let operation = json.pointer("/paths/~1limited/get").unwrap();

        assert_eq!(
            operation.get("x-rate-limit-rps").unwrap().as_u64().unwrap(),
            10,
            "Rate limit RPS should be in extensions"
        );
        assert_eq!(
            operation
                .get("x-rate-limit-burst")
                .unwrap()
                .as_u64()
                .unwrap(),
            20,
            "Rate limit burst should be in extensions"
        );
        assert_eq!(
            operation
                .get("x-in-flight-limit")
                .unwrap()
                .as_u64()
                .unwrap(),
            5,
            "In-flight limit should be in extensions"
        );
    }

    #[test]
    fn schema_conflict_uses_latest_version() {
        let registry = OpenApiRegistryImpl::new();

        // Register first version of schema
        let schema_v1 = Schema::Object(
            ObjectBuilder::new()
                .schema_type(SchemaType::Type(utoipa::openapi::schema::Type::String))
                .build(),
        );
        registry.ensure_schema_raw(
            "ConflictSchema",
            vec![("ConflictSchema".into(), RefOr::T(schema_v1))],
        );

        // Register second version (different schema)
        let schema_v2 = Schema::Object(
            ObjectBuilder::new()
                .schema_type(SchemaType::Type(utoipa::openapi::schema::Type::Integer))
                .build(),
        );
        registry.ensure_schema_raw(
            "ConflictSchema",
            vec![("ConflictSchema".into(), RefOr::T(schema_v2))],
        );

        // Verify the latest version is stored
        let stored = registry.components_registry.load();
        let final_schema = stored.get("ConflictSchema").unwrap();

        let json = serde_json::to_value(final_schema).unwrap();
        assert_eq!(
            json.get("type").unwrap().as_str().unwrap(),
            "integer",
            "Schema should be updated to latest version (integer)"
        );
    }

    #[test]
    fn schema_identical_registration_skips_duplicate() {
        let registry = OpenApiRegistryImpl::new();

        // Register schema twice with identical content
        let schema = Schema::Object(
            ObjectBuilder::new()
                .schema_type(SchemaType::Type(utoipa::openapi::schema::Type::String))
                .build(),
        );

        registry.ensure_schema_raw(
            "IdenticalSchema",
            vec![("IdenticalSchema".into(), RefOr::T(schema.clone()))],
        );
        registry.ensure_schema_raw(
            "IdenticalSchema",
            vec![("IdenticalSchema".into(), RefOr::T(schema))],
        );

        // Verify only one entry exists
        let stored = registry.components_registry.load();
        assert_eq!(stored.len(), 1, "Should only have one schema entry");
        assert!(
            stored.contains_key("IdenticalSchema"),
            "Schema should be present"
        );
    }

    #[test]
    fn multiple_operations_with_different_methods_on_same_path() {
        let registry = OpenApiRegistryImpl::new();

        // Register GET operation
        let get_op = minimal_op("/resource", "get_resource");
        registry.register_operation(&get_op);

        // Register POST operation on same path
        let mut post_op = minimal_op("/resource", "create_resource");
        post_op.method = Method::POST;
        registry.register_operation(&post_op);

        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify both operations exist on the same path
        let path = json.pointer("/paths/~1resource").unwrap();
        assert!(path.get("get").is_some(), "GET operation should exist");
        assert!(path.get("post").is_some(), "POST operation should exist");

        // Verify operation IDs are correct
        assert_eq!(
            path.get("get")
                .unwrap()
                .get("operationId")
                .unwrap()
                .as_str()
                .unwrap(),
            "get_resource"
        );
        assert_eq!(
            path.get("post")
                .unwrap()
                .get("operationId")
                .unwrap()
                .as_str()
                .unwrap(),
            "create_resource"
        );
    }

    #[test]
    fn axum_path_conversion_to_openapi_format() {
        let registry = OpenApiRegistryImpl::new();

        // Register operation with Axum-style wildcard path parameter
        let op = minimal_op("/static/{*file_path}", "get_static");
        registry.register_operation(&op);

        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify wildcard path is normalized to OpenAPI format
        assert!(
            json.pointer("/paths/~1static~1{file_path}").is_some(),
            "Wildcard paths should drop the * when converted to OpenAPI"
        );
    }

    #[test]
    fn request_body_multipart_file_generates_proper_schema() {
        use crate::api::operation_builder::RequestBodySchema;

        let registry = OpenApiRegistryImpl::new();
        let mut op = minimal_op("/upload", "upload_handler");
        op.method = Method::POST;
        op.request_body = Some(crate::api::operation_builder::RequestBodySpec {
            content_type: "multipart/form-data",
            description: Some("File upload".to_string()),
            schema: RequestBodySchema::MultipartFile {
                field_name: "document".to_string(),
            },
            required: true,
        });

        registry.register_operation(&op);
        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify multipart/form-data content type
        let content = json
            .pointer("/paths/~1upload/post/requestBody/content/multipart~1form-data")
            .expect("Multipart content should exist");

        // Verify schema structure
        let schema = content.get("schema").unwrap();
        assert_eq!(
            schema.get("type").unwrap().as_str().unwrap(),
            "object",
            "Multipart schema should be object type"
        );

        // Verify field is present
        let properties = schema.get("properties").unwrap();
        assert!(
            properties.get("document").is_some(),
            "Field 'document' should be in properties"
        );

        // Verify field is binary
        let field_schema = properties.get("document").unwrap();
        assert_eq!(
            field_schema.get("type").unwrap().as_str().unwrap(),
            "string"
        );
        assert_eq!(
            field_schema.get("format").unwrap().as_str().unwrap(),
            "binary"
        );

        // Verify required array
        let required = schema.get("required").unwrap().as_array().unwrap();
        assert!(required.contains(&serde_json::json!("document")));
    }

    #[test]
    fn request_body_inline_object_generates_empty_schema() {
        use crate::api::operation_builder::RequestBodySchema;

        let registry = OpenApiRegistryImpl::new();
        let mut op = minimal_op("/dynamic", "dynamic_handler");
        op.method = Method::POST;
        op.request_body = Some(crate::api::operation_builder::RequestBodySpec {
            content_type: "application/json",
            description: Some("Dynamic object".to_string()),
            schema: RequestBodySchema::InlineObject,
            required: false,
        });

        registry.register_operation(&op);
        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify inline object generates empty object schema
        let schema = json
            .pointer("/paths/~1dynamic/post/requestBody/content/application~1json/schema")
            .expect("Request body schema should exist");

        assert_eq!(
            schema.get("type").unwrap().as_str().unwrap(),
            "object",
            "Inline object should be object type"
        );
    }

    #[test]
    fn response_with_non_json_content_type() {
        let registry = OpenApiRegistryImpl::new();
        let mut op = minimal_op("/download", "download_file");
        op.responses = vec![ResponseSpec {
            status: 200,
            content_type: "application/pdf",
            description: "PDF document".to_string(),
            schema_name: None,
        }];

        registry.register_operation(&op);
        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify non-JSON response uses string type with custom format
        let response = json
            .pointer("/paths/~1download/get/responses/200")
            .expect("Response should exist");

        let content = response
            .get("content")
            .unwrap()
            .get("application/pdf")
            .expect("PDF content type should exist");

        let schema = content.get("schema").unwrap();
        assert_eq!(schema.get("type").unwrap().as_str().unwrap(), "string");
        assert_eq!(
            schema.get("format").unwrap().as_str().unwrap(),
            "application/pdf"
        );
    }

    #[test]
    fn response_without_schema_generates_empty_object() {
        let registry = OpenApiRegistryImpl::new();
        let mut op = minimal_op("/status", "get_status");
        op.responses = vec![ResponseSpec {
            status: 200,
            content_type: "application/json",
            description: "Status response".to_string(),
            schema_name: None,
        }];

        registry.register_operation(&op);
        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify response without schema_name generates empty object
        let schema = json
            .pointer("/paths/~1status/get/responses/200/content/application~1json/schema")
            .expect("Response schema should exist");

        assert_eq!(
            schema.get("type").unwrap().as_str().unwrap(),
            "object",
            "Default schema should be empty object"
        );
    }

    #[test]
    fn http_methods_put_delete_patch_are_supported() {
        let registry = OpenApiRegistryImpl::new();

        // PUT
        let mut put_op = minimal_op("/item", "update_item");
        put_op.method = Method::PUT;
        registry.register_operation(&put_op);

        // DELETE
        let mut delete_op = minimal_op("/item", "delete_item");
        delete_op.method = Method::DELETE;
        registry.register_operation(&delete_op);

        // PATCH
        let mut patch_op = minimal_op("/item", "patch_item");
        patch_op.method = Method::PATCH;
        registry.register_operation(&patch_op);

        let doc = registry.build_openapi(&OpenApiInfo::default()).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        let path = json.pointer("/paths/~1item").expect("Path should exist");

        assert!(
            path.get("put").is_some(),
            "PUT operation should be registered"
        );
        assert!(
            path.get("delete").is_some(),
            "DELETE operation should be registered"
        );
        assert!(
            path.get("patch").is_some(),
            "PATCH operation should be registered"
        );

        // Verify operation IDs
        assert_eq!(
            path.get("put")
                .unwrap()
                .get("operationId")
                .unwrap()
                .as_str()
                .unwrap(),
            "update_item"
        );
        assert_eq!(
            path.get("delete")
                .unwrap()
                .get("operationId")
                .unwrap()
                .as_str()
                .unwrap(),
            "delete_item"
        );
        assert_eq!(
            path.get("patch")
                .unwrap()
                .get("operationId")
                .unwrap()
                .as_str()
                .unwrap(),
            "patch_item"
        );
    }
}
