//! `OpenAPI` registry for schema and operation management
//!
//! This module provides a standalone `OpenAPI` registry that collects operation specs
//! and schemas, and builds a complete `OpenAPI` document from them.

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

/// `OpenAPI` document metadata (title, version, description)
#[derive(Debug, Clone)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

impl Default for OpenApiInfo {
    fn default() -> Self {
        Self {
            title: "API Documentation".to_owned(),
            version: "0.1.0".to_owned(),
            description: None,
        }
    }
}

/// `OpenAPI` registry trait for operation and schema registration
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

/// Helper function to call `ensure_schema` with proper type information
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

/// Implementation of `OpenAPI` registry with lock-free data structures
pub struct OpenApiRegistryImpl {
    /// Store operation specs keyed by "METHOD:path"
    pub operation_specs: DashMap<String, operation_builder::OperationSpec>,
    /// Store schema components using arc-swap for lock-free reads
    pub components_registry: ArcSwap<HashMap<String, RefOr<Schema>>>,
}

impl OpenApiRegistryImpl {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            operation_specs: DashMap::new(),
            components_registry: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    /// Build `OpenAPI` specification from registered operations and components.
    ///
    /// # Arguments
    /// * `info` - `OpenAPI` document metadata (title, version, description)
    ///
    /// # Errors
    /// Returns an error if the `OpenAPI` specification cannot be built.
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

            // Vendor extensions
            let mut ext = utoipa::openapi::extensions::Extensions::default();

            // Rate limit
            if let Some(rl) = spec.rate_limit.as_ref() {
                ext.insert("x-rate-limit-rps".to_owned(), serde_json::json!(rl.rps));
                ext.insert("x-rate-limit-burst".to_owned(), serde_json::json!(rl.burst));
                ext.insert(
                    "x-in-flight-limit".to_owned(),
                    serde_json::json!(rl.in_flight),
                );
            }

            // Pagination
            if let Some(pagination) = spec.vendor_extensions.x_odata_filter.as_ref() {
                if let Ok(value) = serde_json::to_value(pagination) {
                    ext.insert("x-odata-filter".to_owned(), value);
                }
            }
            if let Some(pagination) = spec.vendor_extensions.x_odata_orderby.as_ref() {
                if let Ok(value) = serde_json::to_value(pagination) {
                    ext.insert("x-odata-orderby".to_owned(), value);
                }
            }

            if !ext.is_empty() {
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
                    .content(rb.content_type.to_owned(), content);
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
                                "#/components/schemas/{name}"
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
                Method::POST => HttpMethod::Post,
                Method::PUT => HttpMethod::Put,
                Method::DELETE => HttpMethod::Delete,
                Method::PATCH => HttpMethod::Patch,
                // GET and any other method default to Get
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
                }
                tracing::warn!(%name, "Schema content conflict; overriding with latest");
            }
            reg.insert(name, schema);
        }

        self.components_registry.store(Arc::new(reg));
        root_name.to_owned()
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
        OperationSpec, ParamLocation, ParamSpec, ResponseSpec, VendorExtensions,
    };
    use http::Method;

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
            path: "/test".to_owned(),
            operation_id: Some("test_op".to_owned()),
            summary: Some("Test operation".to_owned()),
            description: None,
            tags: vec![],
            params: vec![],
            request_body: None,
            responses: vec![ResponseSpec {
                status: 200,
                content_type: "application/json",
                description: "Success".to_owned(),
                schema_name: None,
            }],
            handler_id: "get_test".to_owned(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: None,
            vendor_extensions: VendorExtensions::default(),
        };

        registry.register_operation(&spec);
        assert_eq!(registry.operation_specs.len(), 1);
    }

    #[test]
    fn test_build_empty_openapi() {
        let registry = OpenApiRegistryImpl::new();
        let info = OpenApiInfo {
            title: "Test API".to_owned(),
            version: "1.0.0".to_owned(),
            description: Some("Test API Description".to_owned()),
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
            path: "/users/{id}".to_owned(),
            operation_id: Some("get_user".to_owned()),
            summary: Some("Get user by ID".to_owned()),
            description: Some("Retrieves a user by their ID".to_owned()),
            tags: vec!["users".to_owned()],
            params: vec![ParamSpec {
                name: "id".to_owned(),
                location: ParamLocation::Path,
                required: true,
                description: Some("User ID".to_owned()),
                param_type: "string".to_owned(),
            }],
            request_body: None,
            responses: vec![ResponseSpec {
                status: 200,
                content_type: "application/json",
                description: "User found".to_owned(),
                schema_name: None,
            }],
            handler_id: "get_users_id".to_owned(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: None,
            vendor_extensions: VendorExtensions::default(),
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
        let schemas = vec![("TestSchema".to_owned(), RefOr::T(schema))];

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
            path: "/files/v1/upload".to_owned(),
            operation_id: Some("upload_file".to_owned()),
            summary: Some("Upload a file".to_owned()),
            description: Some("Upload raw binary file".to_owned()),
            tags: vec!["upload".to_owned()],
            params: vec![],
            request_body: Some(crate::api::operation_builder::RequestBodySpec {
                content_type: "application/octet-stream",
                description: Some("Raw file bytes".to_owned()),
                schema: RequestBodySchema::Binary,
                required: true,
            }),
            responses: vec![ResponseSpec {
                status: 200,
                content_type: "application/json",
                description: "Upload successful".to_owned(),
                schema_name: None,
            }],
            handler_id: "post_upload".to_owned(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: Some(vec!["application/octet-stream"]),
            vendor_extensions: VendorExtensions::default(),
        };

        registry.register_operation(&spec);
        let info = OpenApiInfo::default();
        let doc = registry.build_openapi(&info).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        // Verify path exists
        let paths = json.get("paths").unwrap();
        assert!(paths.get("/files/v1/upload").is_some());

        // Verify request body has application/octet-stream with binary schema
        let post_op = paths.get("/files/v1/upload").unwrap().get("post").unwrap();
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
    fn test_build_openapi_with_pagination() {
        let registry = OpenApiRegistryImpl::new();

        let mut filter: operation_builder::ODataPagination<
            std::collections::BTreeMap<String, Vec<String>>,
        > = operation_builder::ODataPagination::default();
        filter.allowed_fields.insert(
            "name".to_owned(),
            vec!["eq", "ne", "contains", "startswith", "endswith", "in"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        filter.allowed_fields.insert(
            "age".to_owned(),
            vec!["eq", "ne", "gt", "ge", "lt", "le", "in"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        let mut order_by: operation_builder::ODataPagination<Vec<String>> =
            operation_builder::ODataPagination::default();
        order_by.allowed_fields.push("name asc".to_owned());
        order_by.allowed_fields.push("name desc".to_owned());
        order_by.allowed_fields.push("age asc".to_owned());
        order_by.allowed_fields.push("age desc".to_owned());

        let mut spec = OperationSpec {
            method: Method::GET,
            path: "/test".to_owned(),
            operation_id: Some("test_op".to_owned()),
            summary: Some("Test".to_owned()),
            description: None,
            tags: vec![],
            params: vec![],
            request_body: None,
            responses: vec![ResponseSpec {
                status: 200,
                content_type: "application/json",
                description: "OK".to_owned(),
                schema_name: None,
            }],
            handler_id: "get_test".to_owned(),
            sec_requirement: None,
            is_public: false,
            rate_limit: None,
            allowed_request_content_types: None,
            vendor_extensions: VendorExtensions::default(),
        };
        spec.vendor_extensions.x_odata_filter = Some(filter);
        spec.vendor_extensions.x_odata_orderby = Some(order_by);

        registry.register_operation(&spec);
        let info = OpenApiInfo::default();
        let doc = registry.build_openapi(&info).unwrap();
        let json = serde_json::to_value(&doc).unwrap();

        let paths = json.get("paths").unwrap();
        let op = paths.get("/test").unwrap().get("get").unwrap();

        let filter_ext = op
            .get("x-odata-filter")
            .expect("x-odata-filter should be present");

        let allowed_fields = filter_ext.get("allowedFields").unwrap();
        assert!(allowed_fields.get("name").is_some());
        assert!(allowed_fields.get("age").is_some());

        let order_ext = op
            .get("x-odata-orderby")
            .expect("x-odata-orderby should be present");

        let allowed_order = order_ext.get("allowedFields").unwrap().as_array().unwrap();
        assert!(allowed_order.iter().any(|v| v.as_str() == Some("name asc")));
        assert!(allowed_order.iter().any(|v| v.as_str() == Some("age desc")));
    }
}
