//! Type-safe API operation builder with compile-time guarantees
//!
//! This module implements a type-state builder pattern that ensures:
//! - `register()` cannot be called unless a handler is set
//! - `register()` cannot be called unless at least one response is declared
//! - Descriptive methods remain available at any stage
//! - No panics or unwraps in production hot paths
//! - Request body support (`json_request`, `json_request_schema`) so POST/PUT calls are invokable in UI
//! - Schema-aware responses (`json_response_with_schema`)
//! - Typed Router state `S` usage pattern: pass a state type once via `Router::with_state`,
//!   then use plain function handlers (no per-route closures that capture/clones).
//! - Optional `method_router(...)` for advanced use (layers/middleware on route level).

use axum::{handler::Handler, routing::MethodRouter, Router};
use http::Method;
use std::marker::PhantomData;

use crate::api::problem;

/// Convert OpenAPI-style path placeholders to Axum 0.8+ style path parameters.
///
/// Axum 0.8+ uses `{id}` for path parameters and `{*path}` for wildcards, which is the same as `OpenAPI`.
/// However, `OpenAPI` wildcards are just `{path}` without the asterisk.
/// This function converts `OpenAPI` wildcards to Axum wildcards by detecting common wildcard names.
///
/// # Examples
///
/// ```
/// # use modkit::api::operation_builder::normalize_to_axum_path;
/// assert_eq!(normalize_to_axum_path("/users/{id}"), "/users/{id}");
/// assert_eq!(normalize_to_axum_path("/projects/{project_id}/items/{item_id}"), "/projects/{project_id}/items/{item_id}");
/// // Note: Most paths don't need normalization in Axum 0.8+
/// ```
#[must_use]
pub fn normalize_to_axum_path(path: &str) -> String {
    // In Axum 0.8+, the path syntax is {param} for parameters and {*wildcard} for wildcards
    // which is the same as OpenAPI except wildcards need the asterisk prefix.
    // For now, we just pass through the path as-is since OpenAPI and Axum 0.8 use the same syntax
    // for regular parameters. Wildcards need special handling if used.
    path.to_owned()
}

/// Convert Axum 0.8+ style path parameters to OpenAPI-style placeholders.
///
/// Removes the asterisk prefix from Axum wildcards `{*path}` to make them OpenAPI-compatible `{path}`.
///
/// # Examples
///
/// ```
/// # use modkit::api::operation_builder::axum_to_openapi_path;
/// assert_eq!(axum_to_openapi_path("/users/{id}"), "/users/{id}");
/// assert_eq!(axum_to_openapi_path("/static/{*path}"), "/static/{path}");
/// ```
#[must_use]
pub fn axum_to_openapi_path(path: &str) -> String {
    // In Axum 0.8+, wildcards are {*name} but OpenAPI expects {name}
    // Regular parameters are the same in both
    path.replace("{*", "{")
}

/// Type-state markers for compile-time enforcement
pub mod state {
    /// Marker for missing required components
    #[derive(Debug, Clone, Copy)]
    pub struct Missing;

    /// Marker for present required components
    #[derive(Debug, Clone, Copy)]
    pub struct Present;

    /// Marker for auth requirement not yet set
    #[derive(Debug, Clone, Copy)]
    pub struct AuthNotSet;

    /// Marker for auth requirement set (either `require_auth` or public)
    #[derive(Debug, Clone, Copy)]
    pub struct AuthSet;
}

/// Internal trait mapping handler state to the concrete router slot type.
/// For `Missing` there is no router slot; for `Present` it is `MethodRouter<S>`.
/// Private sealed trait to enforce the implementation is only visible within this module.
mod sealed {
    pub trait Sealed {}
    pub trait SealedAuth {}
}

pub trait HandlerSlot<S>: sealed::Sealed {
    type Slot;
}

/// Sealed trait for auth state markers
pub trait AuthState: sealed::SealedAuth {}

impl sealed::Sealed for Missing {}
impl sealed::Sealed for Present {}

impl sealed::SealedAuth for state::AuthNotSet {}
impl sealed::SealedAuth for state::AuthSet {}

impl AuthState for state::AuthNotSet {}
impl AuthState for state::AuthSet {}

impl<S> HandlerSlot<S> for Missing {
    type Slot = ();
}
impl<S> HandlerSlot<S> for Present {
    type Slot = MethodRouter<S>;
}

pub use state::{AuthNotSet, AuthSet, Missing, Present};

/// Parameter specification for API operations
#[derive(Clone, Debug)]
pub struct ParamSpec {
    pub name: String,
    pub location: ParamLocation,
    pub required: bool,
    pub description: Option<String>,
    pub param_type: String, // JSON Schema type (string, integer, etc.)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamLocation {
    Path,
    Query,
    Header,
    Cookie,
}

/// Request body schema variants for different kinds of request bodies
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestBodySchema {
    /// Reference to a component schema in `#/components/schemas/{schema_name}`
    Ref { schema_name: String },
    /// Multipart form with a single file field
    MultipartFile { field_name: String },
    /// Raw binary body (e.g. application/octet-stream), represented as
    /// type: string, format: binary in `OpenAPI`.
    Binary,
    /// A generic inline object schema with no predefined properties
    InlineObject,
}

/// Request body specification for API operations
#[derive(Clone, Debug)]
pub struct RequestBodySpec {
    pub content_type: &'static str,
    pub description: Option<String>,
    /// The schema for this request body
    pub schema: RequestBodySchema,
    /// Whether request body is required (`OpenAPI` default is `false`).
    pub required: bool,
}

/// Response specification for API operations
#[derive(Clone, Debug)]
pub struct ResponseSpec {
    pub status: u16,
    pub content_type: &'static str,
    pub description: String,
    /// Name of a registered component schema (if any).
    pub schema_name: Option<String>,
}

/// Security requirement for an operation (resource:action pattern)
#[derive(Clone, Debug)]
pub struct OperationSecRequirement {
    pub resource: String,
    pub action: String,
}

/// Simplified operation specification for the type-safe builder
#[derive(Clone, Debug)]
pub struct OperationSpec {
    pub method: Method,
    pub path: String,
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub params: Vec<ParamSpec>,
    pub request_body: Option<RequestBodySpec>,
    pub responses: Vec<ResponseSpec>,
    /// Internal handler id; can be used by registry/generator to map a handler identity
    pub handler_id: String,
    /// Security requirement for this operation (if any)
    pub sec_requirement: Option<OperationSecRequirement>,
    /// Explicitly mark route as public (no auth required)
    pub is_public: bool,
    /// Optional rate & concurrency limits for this operation
    pub rate_limit: Option<RateLimitSpec>,
    /// Optional whitelist of allowed request Content-Type values (without parameters).
    /// Example: Some(vec!["application/json", "multipart/form-data", "application/pdf"])
    /// When set, ingress middleware will enforce these types and return HTTP 415 for
    /// requests with disallowed Content-Type headers. This is independent of the
    /// request body schema and should not be used to create synthetic request bodies.
    pub allowed_request_content_types: Option<Vec<&'static str>>,
}

/// Per-operation rate & concurrency limit specification
#[derive(Clone, Debug, Default)]
pub struct RateLimitSpec {
    /// Target steady-state requests per second
    pub rps: u32,
    /// Maximum burst size (token bucket capacity)
    pub burst: u32,
    /// Maximum number of in-flight requests for this route
    pub in_flight: u32,
}

//
pub trait OperationBuilderODataExt<S, H, R> {
    /// Adds optional `$filter` query parameter to `OpenAPI`.
    #[must_use]
    fn with_odata_filter(self) -> Self;

    /// Same as above but with explicit description (e.g., allowed fields).
    #[must_use]
    fn with_odata_filter_doc(self, description: impl Into<String>) -> Self;
}

impl<S, H, R, A> OperationBuilderODataExt<S, H, R> for OperationBuilder<H, R, S, A>
where
    H: HandlerSlot<S>,
    A: AuthState,
{
    fn with_odata_filter(mut self) -> Self {
        self.spec.params.push(ParamSpec {
            name: "$filter".to_owned(),
            location: ParamLocation::Query,
            required: false,
            description: Some("OData v4 filter expression".to_owned()),
            param_type: "string".to_owned(),
        });
        self
    }

    fn with_odata_filter_doc(mut self, description: impl Into<String>) -> Self {
        self.spec.params.push(ParamSpec {
            name: "$filter".to_owned(),
            location: ParamLocation::Query,
            required: false,
            description: Some(description.into()),
            param_type: "string".to_owned(),
        });
        self
    }
}

// Re-export from openapi_registry for backward compatibility
pub use crate::api::openapi_registry::{ensure_schema, OpenApiRegistry};

/// Type-safe operation builder with compile-time guarantees.
///
/// Generic parameters:
/// - `H`: Handler state (Missing | Present)
/// - `R`: Response state (Missing | Present)
/// - `S`: Router state type (what you put into `Router::with_state(S)`).
/// - `A`: Auth state (`AuthNotSet` | `AuthSet`)
#[must_use]
pub struct OperationBuilder<H = Missing, R = Missing, S = (), A = AuthNotSet>
where
    H: HandlerSlot<S>,
    A: AuthState,
{
    spec: OperationSpec,
    method_router: <H as HandlerSlot<S>>::Slot,
    _has_handler: PhantomData<H>,
    _has_response: PhantomData<R>,
    #[allow(clippy::type_complexity)]
    _state: PhantomData<fn() -> S>, // Zero-sized marker for type-state pattern
    _auth_state: PhantomData<A>,
}

// -------------------------------------------------------------------------------------------------
// Constructors — starts with both handler and response missing, auth not set
// -------------------------------------------------------------------------------------------------
impl<S> OperationBuilder<Missing, Missing, S, AuthNotSet> {
    /// Create a new operation builder with an HTTP method and path
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        let path_str = path.into();
        let handler_id = format!(
            "{}:{}",
            method.as_str().to_lowercase(),
            path_str.replace(['/', '{', '}'], "_")
        );

        Self {
            spec: OperationSpec {
                method,
                path: path_str,
                operation_id: None,
                summary: None,
                description: None,
                tags: Vec::new(),
                params: Vec::new(),
                request_body: None,
                responses: Vec::new(),
                handler_id,
                sec_requirement: None,
                is_public: false,
                rate_limit: None,
                allowed_request_content_types: None,
            },
            method_router: (), // no router in Missing state
            _has_handler: PhantomData,
            _has_response: PhantomData,
            _state: PhantomData,
            _auth_state: PhantomData,
        }
    }

    /// Convenience constructor for GET requests
    pub fn get(path: impl Into<String>) -> Self {
        let path_str = path.into();
        Self::new(Method::GET, normalize_to_axum_path(&path_str))
    }

    /// Convenience constructor for POST requests
    pub fn post(path: impl Into<String>) -> Self {
        let path_str = path.into();
        Self::new(Method::POST, normalize_to_axum_path(&path_str))
    }

    /// Convenience constructor for PUT requests
    pub fn put(path: impl Into<String>) -> Self {
        let path_str = path.into();
        Self::new(Method::PUT, normalize_to_axum_path(&path_str))
    }

    /// Convenience constructor for DELETE requests
    pub fn delete(path: impl Into<String>) -> Self {
        let path_str = path.into();
        Self::new(Method::DELETE, normalize_to_axum_path(&path_str))
    }

    /// Convenience constructor for PATCH requests
    pub fn patch(path: impl Into<String>) -> Self {
        let path_str = path.into();
        Self::new(Method::PATCH, normalize_to_axum_path(&path_str))
    }
}

// -------------------------------------------------------------------------------------------------
// Descriptive methods — available at any stage
// -------------------------------------------------------------------------------------------------
impl<H, R, S, A> OperationBuilder<H, R, S, A>
where
    H: HandlerSlot<S>,
    A: AuthState,
{
    /// Inspect the spec (primarily for tests)
    pub fn spec(&self) -> &OperationSpec {
        &self.spec
    }

    /// Set the operation ID
    pub fn operation_id(mut self, id: impl Into<String>) -> Self {
        self.spec.operation_id = Some(id.into());
        self
    }

    /// Require per-route rate and concurrency limits.
    /// Stores metadata for the ingress to enforce.
    pub fn require_rate_limit(&mut self, rps: u32, burst: u32, in_flight: u32) -> &mut Self {
        self.spec.rate_limit = Some(RateLimitSpec {
            rps,
            burst,
            in_flight,
        });
        self
    }

    /// Set the operation summary
    pub fn summary(mut self, text: impl Into<String>) -> Self {
        self.spec.summary = Some(text.into());
        self
    }

    /// Set the operation description
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.spec.description = Some(text.into());
        self
    }

    /// Add a tag to the operation
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.spec.tags.push(tag.into());
        self
    }

    /// Add a parameter to the operation
    pub fn param(mut self, param: ParamSpec) -> Self {
        self.spec.params.push(param);
        self
    }

    /// Add a path parameter with type inference (defaults to string)
    pub fn path_param(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.spec.params.push(ParamSpec {
            name: name.into(),
            location: ParamLocation::Path,
            required: true,
            description: Some(description.into()),
            param_type: "string".to_owned(),
        });
        self
    }

    /// Add a query parameter (defaults to string)
    pub fn query_param(
        mut self,
        name: impl Into<String>,
        required: bool,
        description: impl Into<String>,
    ) -> Self {
        self.spec.params.push(ParamSpec {
            name: name.into(),
            location: ParamLocation::Query,
            required,
            description: Some(description.into()),
            param_type: "string".to_owned(),
        });
        self
    }

    /// Add a typed query parameter with explicit `OpenAPI` type
    pub fn query_param_typed(
        mut self,
        name: impl Into<String>,
        required: bool,
        description: impl Into<String>,
        param_type: impl Into<String>,
    ) -> Self {
        self.spec.params.push(ParamSpec {
            name: name.into(),
            location: ParamLocation::Query,
            required,
            description: Some(description.into()),
            param_type: param_type.into(),
        });
        self
    }

    /// Attach a JSON request body by *schema name* that you've already registered.
    /// This variant sets a description (`Some(desc)`) and marks the body as **required**.
    pub fn json_request_schema(
        mut self,
        schema_name: impl Into<String>,
        desc: impl Into<String>,
    ) -> Self {
        self.spec.request_body = Some(RequestBodySpec {
            content_type: "application/json",
            description: Some(desc.into()),
            schema: RequestBodySchema::Ref {
                schema_name: schema_name.into(),
            },
            required: true,
        });
        self
    }

    /// Attach a JSON request body by *schema name* with **no** description (`None`).
    /// Marks the body as **required**.
    pub fn json_request_schema_no_desc(mut self, schema_name: impl Into<String>) -> Self {
        self.spec.request_body = Some(RequestBodySpec {
            content_type: "application/json",
            description: None,
            schema: RequestBodySchema::Ref {
                schema_name: schema_name.into(),
            },
            required: true,
        });
        self
    }

    /// Attach a JSON request body and auto-register its schema using `utoipa`.
    /// This variant sets a description (`Some(desc)`) and marks the body as **required**.
    pub fn json_request<T>(
        mut self,
        registry: &dyn OpenApiRegistry,
        desc: impl Into<String>,
    ) -> Self
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(registry);
        self.spec.request_body = Some(RequestBodySpec {
            content_type: "application/json",
            description: Some(desc.into()),
            schema: RequestBodySchema::Ref { schema_name: name },
            required: true,
        });
        self
    }

    /// Attach a JSON request body (auto-register schema) with **no** description (`None`).
    /// Marks the body as **required**.
    pub fn json_request_no_desc<T>(mut self, registry: &dyn OpenApiRegistry) -> Self
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(registry);
        self.spec.request_body = Some(RequestBodySpec {
            content_type: "application/json",
            description: None,
            schema: RequestBodySchema::Ref { schema_name: name },
            required: true,
        });
        self
    }

    /// Make the previously attached request body **optional** (if any).
    pub fn request_optional(mut self) -> Self {
        if let Some(rb) = &mut self.spec.request_body {
            rb.required = false;
        }
        self
    }

    /// Configure a multipart/form-data file upload request.
    ///
    /// This is a convenience helper for file upload endpoints that:
    /// - Sets the request body content type to "multipart/form-data"
    /// - Sets a description for the request body
    /// - Configures an inline object schema with a binary file field
    /// - Restricts allowed Content-Type to only "multipart/form-data"
    ///
    /// The file field will be documented in `OpenAPI` as a binary string with the
    /// given field name. This generates the correct `OpenAPI` schema for UI tools
    /// like Stoplight to display a file upload control.
    ///
    /// # Arguments
    /// * `field_name` - Name of the multipart form field (e.g., "file")
    /// * `description` - Optional description for the request body
    ///
    /// # Example
    /// ```rust,ignore
    /// OperationBuilder::post("/files/v1/upload")
    ///     .operation_id("upload_file")
    ///     .summary("Upload a file")
    ///     .multipart_file_request("file", Some("File to upload"))
    ///     .handler(upload_handler)
    ///     .json_response(200, "Upload successful")
    ///     .register(router, &api);
    /// ```
    pub fn multipart_file_request(mut self, field_name: &str, description: Option<&str>) -> Self {
        // Set request body with multipart/form-data content type
        self.spec.request_body = Some(RequestBodySpec {
            content_type: "multipart/form-data",
            description: description
                .map(|s| format!("{s} (expects field '{field_name}' with file data)")),
            schema: RequestBodySchema::MultipartFile {
                field_name: field_name.to_owned(),
            },
            required: true,
        });

        // Also configure MIME type validation
        self.spec.allowed_request_content_types = Some(vec!["multipart/form-data"]);

        self
    }

    /// Configure the request body as raw binary (application/octet-stream).
    ///
    /// This is intended for endpoints that accept the entire request body
    /// as a file or arbitrary bytes, without multipart form encoding.
    ///
    /// The `OpenAPI` schema will be:
    /// ```yaml
    /// requestBody:
    ///   required: true
    ///   content:
    ///     application/octet-stream:
    ///       schema:
    ///         type: string
    ///         format: binary
    /// ```
    ///
    /// Tools like Stoplight will render this as a single file upload control
    /// for the entire body.
    ///
    /// # Arguments
    /// * `description` - Optional description for the request body
    ///
    /// # Example
    /// ```rust,ignore
    /// OperationBuilder::post("/files/v1/upload")
    ///     .operation_id("upload_file")
    ///     .summary("Upload a file")
    ///     .octet_stream_request(Some("Raw file bytes to parse"))
    ///     .handler(upload_handler)
    ///     .json_response(200, "Upload successful")
    ///     .register(router, &api);
    /// ```
    pub fn octet_stream_request(mut self, description: Option<&str>) -> Self {
        self.spec.request_body = Some(RequestBodySpec {
            content_type: "application/octet-stream",
            description: description.map(ToString::to_string),
            schema: RequestBodySchema::Binary,
            required: true,
        });

        // Also configure MIME type validation
        self.spec.allowed_request_content_types = Some(vec!["application/octet-stream"]);

        self
    }

    /// Configure allowed request MIME types for this operation.
    ///
    /// This attaches a whitelist of allowed Content-Type values (without parameters),
    /// which will be enforced by ingress middleware. If a request arrives with a
    /// Content-Type that is not in this list, ingress will return HTTP 415.
    ///
    /// This is independent of the request body schema - it only configures ingress
    /// validation and does not affect `OpenAPI` request body specifications.
    ///
    /// # Example
    /// ```rust,ignore
    /// OperationBuilder::post("/files/v1/upload")
    ///     .operation_id("upload_file")
    ///     .allow_content_types(&["multipart/form-data", "application/pdf"])
    ///     .handler(upload_handler)
    ///     .json_response(200, "Upload successful")
    ///     .register(router, &api);
    /// ```
    pub fn allow_content_types(mut self, types: &[&'static str]) -> Self {
        self.spec.allowed_request_content_types = Some(types.to_vec());
        self
    }
}

// -------------------------------------------------------------------------------------------------
// Auth requirement setting — transitions AuthNotSet -> AuthSet
// -------------------------------------------------------------------------------------------------
impl<H, R, S> OperationBuilder<H, R, S, AuthNotSet>
where
    H: HandlerSlot<S>,
{
    /// Require authentication with a specific resource:action permission.
    ///
    /// This method transitions from `AuthNotSet` to `AuthSet` state.
    ///
    /// # Example
    /// ```rust,ignore
    /// OperationBuilder::get("/users-info/v1/users")
    ///     .require_auth("users", "read")
    ///     .handler(list_users)
    ///     .json_response(200, "List of users")
    ///     .register(router, &api);
    /// ```
    pub fn require_auth(
        mut self,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> OperationBuilder<H, R, S, AuthSet> {
        self.spec.sec_requirement = Some(OperationSecRequirement {
            resource: resource.into(),
            action: action.into(),
        });
        self.spec.is_public = false;
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: self._has_response,
            _state: self._state,
            _auth_state: PhantomData,
        }
    }

    /// Mark this route as public (no authentication required).
    ///
    /// This explicitly opts out of the `require_auth_by_default` setting.
    /// This method transitions from `AuthNotSet` to `AuthSet` state.
    ///
    /// # Example
    /// ```rust,ignore
    /// OperationBuilder::get("/users-info/v1/health")
    ///     .public()
    ///     .handler(health_check)
    ///     .json_response(200, "OK")
    ///     .register(router, &api);
    /// ```
    pub fn public(mut self) -> OperationBuilder<H, R, S, AuthSet> {
        self.spec.is_public = true;
        self.spec.sec_requirement = None;
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: self._has_response,
            _state: self._state,
            _auth_state: PhantomData,
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Handler setting — transitions Missing -> Present for handler
// -------------------------------------------------------------------------------------------------
impl<R, S, A> OperationBuilder<Missing, R, S, A>
where
    S: Clone + Send + Sync + 'static,
    A: AuthState,
{
    /// Set the handler for this operation (function handlers are recommended).
    ///
    /// This transitions the builder from `Missing` to `Present` handler state.
    pub fn handler<F, T>(self, h: F) -> OperationBuilder<Present, R, S, A>
    where
        F: Handler<T, S> + Clone + Send + 'static,
        T: 'static,
    {
        let method_router = match self.spec.method {
            Method::GET => axum::routing::get(h),
            Method::POST => axum::routing::post(h),
            Method::PUT => axum::routing::put(h),
            Method::DELETE => axum::routing::delete(h),
            Method::PATCH => axum::routing::patch(h),
            _ => axum::routing::any(|| async { axum::http::StatusCode::METHOD_NOT_ALLOWED }),
        };

        OperationBuilder {
            spec: self.spec,
            method_router, // concrete MethodRouter<S> in Present state
            _has_handler: PhantomData::<Present>,
            _has_response: self._has_response,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }

    /// Alternative path: provide a pre-composed `MethodRouter<S>` yourself
    /// (useful to attach per-route middleware/layers).
    pub fn method_router(self, mr: MethodRouter<S>) -> OperationBuilder<Present, R, S, A> {
        OperationBuilder {
            spec: self.spec,
            method_router: mr, // concrete MethodRouter<S> in Present state
            _has_handler: PhantomData::<Present>,
            _has_response: self._has_response,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Response setting — transitions Missing -> Present for response (first response)
// -------------------------------------------------------------------------------------------------
impl<H, S, A> OperationBuilder<H, Missing, S, A>
where
    H: HandlerSlot<S>,
    A: AuthState,
{
    /// Add a raw response spec (transitions from Missing to Present).
    pub fn response(mut self, resp: ResponseSpec) -> OperationBuilder<H, Present, S, A> {
        self.spec.responses.push(resp);
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }

    /// Add a JSON response (transitions from Missing to Present).
    pub fn json_response(
        mut self,
        status: http::StatusCode,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S, A> {
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type: "application/json",
            description: description.into(),
            schema_name: None,
        });
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }

    /// Add a JSON response with a registered schema (transitions from Missing to Present).
    pub fn json_response_with_schema<T>(
        mut self,
        registry: &dyn OpenApiRegistry,
        status: http::StatusCode,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S, A>
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(registry);
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type: "application/json",
            description: description.into(),
            schema_name: Some(name),
        });
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }

    /// Add a text response with a custom content type (transitions from Missing to Present).
    ///
    /// # Arguments
    /// * `status` - HTTP status code
    /// * `description` - Description of the response
    /// * `content_type` - **Pure media type without parameters** (e.g., `"text/plain"`, `"text/markdown"`)
    ///
    /// # Important
    /// The `content_type` must be a pure media type **without parameters** like `; charset=utf-8`.
    /// `OpenAPI` media type keys cannot include parameters. Use `"text/markdown"` instead of
    /// `"text/markdown; charset=utf-8"`. Actual HTTP response headers in handlers should still
    /// include the charset parameter.
    pub fn text_response(
        mut self,
        status: http::StatusCode,
        description: impl Into<String>,
        content_type: &'static str,
    ) -> OperationBuilder<H, Present, S, A> {
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type,
            description: description.into(),
            schema_name: None,
        });
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }

    /// Add an HTML response (transitions from Missing to Present).
    pub fn html_response(
        mut self,
        status: http::StatusCode,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S, A> {
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type: "text/html",
            description: description.into(),
            schema_name: None,
        });
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }

    /// Add an RFC 9457 `application/problem+json` response (transitions from Missing to Present).
    pub fn problem_response(
        mut self,
        registry: &dyn OpenApiRegistry,
        status: http::StatusCode,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S, A> {
        // Ensure `Problem` schema is registered in components
        let problem_name = ensure_schema::<crate::api::problem::Problem>(registry);
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type: problem::APPLICATION_PROBLEM_JSON,
            description: description.into(),
            schema_name: Some(problem_name),
        });
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }

    /// First response: SSE stream of JSON events (`text/event-stream`).
    pub fn sse_json<T>(
        mut self,
        openapi: &dyn OpenApiRegistry,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S, A>
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(openapi);
        self.spec.responses.push(ResponseSpec {
            status: http::StatusCode::OK.as_u16(),
            content_type: "text/event-stream",
            description: description.into(),
            schema_name: Some(name),
        });
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
            _auth_state: self._auth_state,
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Additional responses — for Present response state (additional responses)
// -------------------------------------------------------------------------------------------------
impl<H, S, A> OperationBuilder<H, Present, S, A>
where
    H: HandlerSlot<S>,
    A: AuthState,
{
    /// Add a JSON response (additional).
    pub fn json_response(
        mut self,
        status: http::StatusCode,
        description: impl Into<String>,
    ) -> Self {
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type: "application/json",
            description: description.into(),
            schema_name: None,
        });
        self
    }

    /// Add a JSON response with a registered schema (additional).
    pub fn json_response_with_schema<T>(
        mut self,
        registry: &dyn OpenApiRegistry,
        status: http::StatusCode,
        description: impl Into<String>,
    ) -> Self
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(registry);
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type: "application/json",
            description: description.into(),
            schema_name: Some(name),
        });
        self
    }

    /// Add a text response with a custom content type (additional).
    ///
    /// # Arguments
    /// * `status` - HTTP status code
    /// * `description` - Description of the response
    /// * `content_type` - **Pure media type without parameters** (e.g., `"text/plain"`, `"text/markdown"`)
    ///
    /// # Important
    /// The `content_type` must be a pure media type **without parameters** like `; charset=utf-8`.
    /// `OpenAPI` media type keys cannot include parameters. Use `"text/markdown"` instead of
    /// `"text/markdown; charset=utf-8"`. Actual HTTP response headers in handlers should still
    /// include the charset parameter.
    pub fn text_response(
        mut self,
        status: http::StatusCode,
        description: impl Into<String>,
        content_type: &'static str,
    ) -> Self {
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type,
            description: description.into(),
            schema_name: None,
        });
        self
    }

    /// Add an HTML response (additional).
    pub fn html_response(
        mut self,
        status: http::StatusCode,
        description: impl Into<String>,
    ) -> Self {
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type: "text/html",
            description: description.into(),
            schema_name: None,
        });
        self
    }

    /// Add an additional RFC 9457 `application/problem+json` response.
    pub fn problem_response(
        mut self,
        registry: &dyn OpenApiRegistry,
        status: http::StatusCode,
        description: impl Into<String>,
    ) -> Self {
        let problem_name = ensure_schema::<crate::api::problem::Problem>(registry);
        self.spec.responses.push(ResponseSpec {
            status: status.as_u16(),
            content_type: problem::APPLICATION_PROBLEM_JSON,
            description: description.into(),
            schema_name: Some(problem_name),
        });
        self
    }

    /// Additional SSE response (if the operation already has a response).
    pub fn sse_json<T>(
        mut self,
        openapi: &dyn OpenApiRegistry,
        description: impl Into<String>,
    ) -> Self
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(openapi);
        self.spec.responses.push(ResponseSpec {
            status: http::StatusCode::OK.as_u16(),
            content_type: "text/event-stream",
            description: description.into(),
            schema_name: Some(name),
        });
        self
    }

    /// Add standard error responses (400, 401, 403, 404, 409, 422, 429, 500).
    ///
    /// All responses reference the shared Problem schema (RFC 9457) for consistent
    /// error handling across your API. This is the recommended way to declare
    /// common error responses without repeating boilerplate.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let op = OperationBuilder::get("/users-info/v1/users")
    ///     .handler(list_users)
    ///     .json_response(StatusCode::OK, "List of users")
    ///     .standard_errors(&registry);
    /// ```
    ///
    /// This adds the following error responses:
    /// - 400 Bad Request
    /// - 401 Unauthorized
    /// - 403 Forbidden
    /// - 404 Not Found
    /// - 409 Conflict
    /// - 422 Unprocessable Entity
    /// - 429 Too Many Requests
    /// - 500 Internal Server Error
    pub fn standard_errors(mut self, registry: &dyn OpenApiRegistry) -> Self {
        use http::StatusCode;
        let problem_name = ensure_schema::<crate::api::problem::Problem>(registry);

        let standard_errors = [
            (StatusCode::BAD_REQUEST, "Bad Request"),
            (StatusCode::UNAUTHORIZED, "Unauthorized"),
            (StatusCode::FORBIDDEN, "Forbidden"),
            (StatusCode::NOT_FOUND, "Not Found"),
            (StatusCode::CONFLICT, "Conflict"),
            (StatusCode::UNPROCESSABLE_ENTITY, "Unprocessable Entity"),
            (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests"),
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        ];

        for (status, description) in standard_errors {
            self.spec.responses.push(ResponseSpec {
                status: status.as_u16(),
                content_type: problem::APPLICATION_PROBLEM_JSON,
                description: description.to_owned(),
                schema_name: Some(problem_name.clone()),
            });
        }

        self
    }

    /// Add 422 validation error response using `ValidationError` schema.
    ///
    /// This method adds a specific 422 Unprocessable Entity response that uses
    /// the `ValidationError` schema instead of the generic Problem schema. Use this
    /// for endpoints that perform input validation and need structured error details.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let op = OperationBuilder::post("/users-info/v1/users")
    ///     .handler(create_user)
    ///     .json_request::<CreateUserRequest>(&registry, "User data")
    ///     .json_response(StatusCode::CREATED, "User created")
    ///     .with_422_validation_error(&registry);
    /// ```
    pub fn with_422_validation_error(mut self, registry: &dyn OpenApiRegistry) -> Self {
        let validation_error_name =
            ensure_schema::<crate::api::problem::ValidationErrorResponse>(registry);

        self.spec.responses.push(ResponseSpec {
            status: http::StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
            content_type: problem::APPLICATION_PROBLEM_JSON,
            description: "Validation Error".to_owned(),
            schema_name: Some(validation_error_name),
        });

        self
    }

    /// Add a 400 Bad Request error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_400(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(registry, http::StatusCode::BAD_REQUEST, "Bad Request")
    }

    /// Add a 401 Unauthorized error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_401(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(registry, http::StatusCode::UNAUTHORIZED, "Unauthorized")
    }

    /// Add a 403 Forbidden error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_403(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(registry, http::StatusCode::FORBIDDEN, "Forbidden")
    }

    /// Add a 404 Not Found error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_404(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(registry, http::StatusCode::NOT_FOUND, "Not Found")
    }

    /// Add a 409 Conflict error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_409(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(registry, http::StatusCode::CONFLICT, "Conflict")
    }

    /// Add a 415 Unsupported Media Type error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_415(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(
            registry,
            http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported Media Type",
        )
    }

    /// Add a 422 Unprocessable Entity error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_422(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(
            registry,
            http::StatusCode::UNPROCESSABLE_ENTITY,
            "Unprocessable Entity",
        )
    }

    /// Add a 429 Too Many Requests error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_429(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(
            registry,
            http::StatusCode::TOO_MANY_REQUESTS,
            "Too Many Requests",
        )
    }

    /// Add a 500 Internal Server Error response.
    ///
    /// This is a convenience wrapper around `problem_response`.
    pub fn error_500(self, registry: &dyn OpenApiRegistry) -> Self {
        self.problem_response(
            registry,
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error",
        )
    }
}

// -------------------------------------------------------------------------------------------------
// Registration — only available when handler, response, AND auth are all set
// -------------------------------------------------------------------------------------------------
impl<S> OperationBuilder<Present, Present, S, AuthSet>
where
    S: Clone + Send + Sync + 'static,
{
    /// Register the operation with the router and `OpenAPI` registry.
    ///
    /// This method is only available when:
    /// - Handler is present
    /// - Response is present
    /// - Auth requirement is set (either `require_auth` or `public`)
    ///
    /// All conditions are enforced at compile time by the type system.
    pub fn register(self, router: Router<S>, openapi: &dyn OpenApiRegistry) -> Router<S> {
        // Inform the OpenAPI registry (the implementation will translate OperationSpec
        // into an OpenAPI Operation + RequestBody + Responses with component refs).
        openapi.register_operation(&self.spec);

        // In Present state the method_router is guaranteed to be a real MethodRouter<S>.
        router.route(&self.spec.path, self.method_router)
    }
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use axum::Json;

    // Mock registry for testing: stores operations; records schema names
    struct MockRegistry {
        operations: std::sync::Mutex<Vec<OperationSpec>>,
        schemas: std::sync::Mutex<Vec<String>>,
    }

    impl MockRegistry {
        fn new() -> Self {
            Self {
                operations: std::sync::Mutex::new(Vec::new()),
                schemas: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl OpenApiRegistry for MockRegistry {
        fn register_operation(&self, spec: &OperationSpec) {
            if let Ok(mut ops) = self.operations.lock() {
                ops.push(spec.clone());
            }
        }

        fn ensure_schema_raw(
            &self,
            name: &str,
            _schemas: Vec<(
                String,
                utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
            )>,
        ) -> String {
            let name = name.to_owned();
            if let Ok(mut s) = self.schemas.lock() {
                s.push(name.clone());
            }
            name
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    async fn test_handler() -> Json<serde_json::Value> {
        Json(serde_json::json!({"status": "ok"}))
    }

    #[test]
    fn test_builder_descriptive_methods() {
        let builder = OperationBuilder::<Missing, Missing, (), AuthNotSet>::get("/tests/v1/test")
            .operation_id("test.get")
            .summary("Test endpoint")
            .description("A test endpoint for validation")
            .tag("test")
            .path_param("id", "Test ID");

        assert_eq!(builder.spec.method, Method::GET);
        assert_eq!(builder.spec.path, "/tests/v1/test");
        assert_eq!(builder.spec.operation_id, Some("test.get".to_owned()));
        assert_eq!(builder.spec.summary, Some("Test endpoint".to_owned()));
        assert_eq!(
            builder.spec.description,
            Some("A test endpoint for validation".to_owned())
        );
        assert_eq!(builder.spec.tags, vec!["test"]);
        assert_eq!(builder.spec.params.len(), 1);
    }

    #[tokio::test]
    async fn test_builder_with_request_response_and_handler() {
        let registry = MockRegistry::new();
        let router = Router::new();

        let _router = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/test")
            .summary("Test endpoint")
            .json_request::<serde_json::Value>(&registry, "optional body") // registers schema
            .public()
            .handler(test_handler)
            .json_response_with_schema::<serde_json::Value>(
                &registry,
                http::StatusCode::OK,
                "Success response",
            ) // registers schema
            .register(router, &registry);

        // Verify that the operation was registered
        let ops = registry.operations.lock().unwrap();
        assert_eq!(ops.len(), 1);
        let op = &ops[0];
        assert_eq!(op.method, Method::POST);
        assert_eq!(op.path, "/tests/v1/test");
        assert!(op.request_body.is_some());
        assert!(op.request_body.as_ref().unwrap().required);
        assert_eq!(op.responses.len(), 1);
        assert_eq!(op.responses[0].status, 200);

        // Verify schemas recorded
        let schemas = registry.schemas.lock().unwrap();
        assert!(!schemas.is_empty());
    }

    #[test]
    fn test_convenience_constructors() {
        let get_builder =
            OperationBuilder::<Missing, Missing, (), AuthNotSet>::get("/tests/v1/get");
        assert_eq!(get_builder.spec.method, Method::GET);
        assert_eq!(get_builder.spec.path, "/tests/v1/get");

        let post_builder =
            OperationBuilder::<Missing, Missing, (), AuthNotSet>::post("/tests/v1/post");
        assert_eq!(post_builder.spec.method, Method::POST);
        assert_eq!(post_builder.spec.path, "/tests/v1/post");

        let put_builder =
            OperationBuilder::<Missing, Missing, (), AuthNotSet>::put("/tests/v1/put");
        assert_eq!(put_builder.spec.method, Method::PUT);
        assert_eq!(put_builder.spec.path, "/tests/v1/put");

        let delete_builder =
            OperationBuilder::<Missing, Missing, (), AuthNotSet>::delete("/tests/v1/delete");
        assert_eq!(delete_builder.spec.method, Method::DELETE);
        assert_eq!(delete_builder.spec.path, "/tests/v1/delete");

        let patch_builder =
            OperationBuilder::<Missing, Missing, (), AuthNotSet>::patch("/tests/v1/patch");
        assert_eq!(patch_builder.spec.method, Method::PATCH);
        assert_eq!(patch_builder.spec.path, "/tests/v1/patch");
    }

    #[test]
    fn test_normalize_to_axum_path() {
        // Axum 0.8+ uses {param} syntax, same as OpenAPI
        assert_eq!(
            normalize_to_axum_path("/tests/v1/users/{id}"),
            "/tests/v1/users/{id}"
        );
        assert_eq!(
            normalize_to_axum_path("/tests/v1/projects/{project_id}/items/{item_id}"),
            "/tests/v1/projects/{project_id}/items/{item_id}"
        );
        assert_eq!(
            normalize_to_axum_path("/tests/v1/simple"),
            "/tests/v1/simple"
        );
        assert_eq!(
            normalize_to_axum_path("/tests/v1/users/{id}/edit"),
            "/tests/v1/users/{id}/edit"
        );
    }

    #[test]
    fn test_axum_to_openapi_path() {
        // Regular parameters stay the same
        assert_eq!(
            axum_to_openapi_path("/tests/v1/users/{id}"),
            "/tests/v1/users/{id}"
        );
        assert_eq!(
            axum_to_openapi_path("/tests/v1/projects/{project_id}/items/{item_id}"),
            "/tests/v1/projects/{project_id}/items/{item_id}"
        );
        assert_eq!(axum_to_openapi_path("/tests/v1/simple"), "/tests/v1/simple");
        // Wildcards: Axum uses {*path}, OpenAPI uses {path}
        assert_eq!(
            axum_to_openapi_path("/tests/v1/static/{*path}"),
            "/tests/v1/static/{path}"
        );
        assert_eq!(
            axum_to_openapi_path("/tests/v1/files/{*filepath}"),
            "/tests/v1/files/{filepath}"
        );
    }

    #[test]
    fn test_path_normalization_in_constructors() {
        // Test that paths are kept as-is (Axum 0.8+ uses same {param} syntax)
        let builder = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/users/{id}");
        assert_eq!(builder.spec.path, "/tests/v1/users/{id}");

        let builder = OperationBuilder::<Missing, Missing, ()>::post(
            "/tests/v1/projects/{project_id}/items/{item_id}",
        );
        assert_eq!(
            builder.spec.path,
            "/tests/v1/projects/{project_id}/items/{item_id}"
        );

        // Simple paths remain unchanged
        let builder = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/simple");
        assert_eq!(builder.spec.path, "/tests/v1/simple");
    }

    #[test]
    fn test_standard_errors() {
        let registry = MockRegistry::new();
        let builder = OperationBuilder::<Missing, Missing, ()>::get("/tests/v1/test")
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success")
            .standard_errors(&registry);

        // Should have 1 success response + 8 standard error responses
        assert_eq!(builder.spec.responses.len(), 9);

        // Check that all standard error status codes are present
        let statuses: Vec<u16> = builder.spec.responses.iter().map(|r| r.status).collect();
        assert!(statuses.contains(&200)); // success response
        assert!(statuses.contains(&400));
        assert!(statuses.contains(&401));
        assert!(statuses.contains(&403));
        assert!(statuses.contains(&404));
        assert!(statuses.contains(&409));
        assert!(statuses.contains(&422));
        assert!(statuses.contains(&429));
        assert!(statuses.contains(&500));

        // All error responses should use Problem content type
        let error_responses: Vec<_> = builder
            .spec
            .responses
            .iter()
            .filter(|r| r.status >= 400)
            .collect();

        for resp in error_responses {
            assert_eq!(
                resp.content_type,
                crate::api::problem::APPLICATION_PROBLEM_JSON
            );
            assert!(resp.schema_name.is_some());
        }
    }

    #[test]
    fn test_with_422_validation_error() {
        let registry = MockRegistry::new();
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/test")
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::CREATED, "Created")
            .with_422_validation_error(&registry);

        // Should have success response + validation error response
        assert_eq!(builder.spec.responses.len(), 2);

        let validation_response = builder
            .spec
            .responses
            .iter()
            .find(|r| r.status == 422)
            .expect("Should have 422 response");

        assert_eq!(validation_response.description, "Validation Error");
        assert_eq!(
            validation_response.content_type,
            crate::api::problem::APPLICATION_PROBLEM_JSON
        );
        assert!(validation_response.schema_name.is_some());
    }

    #[test]
    fn test_allow_content_types_with_existing_request_body() {
        let registry = MockRegistry::new();
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/test")
            .json_request::<serde_json::Value>(&registry, "Test request")
            .allow_content_types(&["application/json", "application/xml"])
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success");

        // allowed_content_types should be on OperationSpec, not RequestBodySpec
        assert!(builder.spec.request_body.is_some());
        assert!(builder.spec.allowed_request_content_types.is_some());
        let allowed = builder.spec.allowed_request_content_types.as_ref().unwrap();
        assert_eq!(allowed.len(), 2);
        assert!(allowed.contains(&"application/json"));
        assert!(allowed.contains(&"application/xml"));
    }

    #[test]
    fn test_allow_content_types_without_existing_request_body() {
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/test")
            .allow_content_types(&["multipart/form-data"])
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success");

        // Should NOT create synthetic request body, only set allowed_request_content_types
        assert!(builder.spec.request_body.is_none());
        assert!(builder.spec.allowed_request_content_types.is_some());
        let allowed = builder.spec.allowed_request_content_types.as_ref().unwrap();
        assert_eq!(allowed.len(), 1);
        assert!(allowed.contains(&"multipart/form-data"));
    }

    #[test]
    fn test_allow_content_types_can_be_chained() {
        let registry = MockRegistry::new();
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/test")
            .operation_id("test.post")
            .summary("Test endpoint")
            .json_request::<serde_json::Value>(&registry, "Test request")
            .allow_content_types(&["application/json"])
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success")
            .problem_response(
                &registry,
                http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Unsupported Media Type",
            );

        assert_eq!(builder.spec.operation_id, Some("test.post".to_owned()));
        assert!(builder.spec.request_body.is_some());
        assert!(builder.spec.allowed_request_content_types.is_some());
        assert_eq!(builder.spec.responses.len(), 2);
    }

    #[test]
    fn test_multipart_file_request() {
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/upload")
            .operation_id("test.upload")
            .summary("Upload file")
            .multipart_file_request("file", Some("Upload a file"))
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success");

        // Should set request body with multipart/form-data
        assert!(builder.spec.request_body.is_some());
        let rb = builder.spec.request_body.as_ref().unwrap();
        assert_eq!(rb.content_type, "multipart/form-data");
        assert!(rb.description.is_some());
        assert!(rb.description.as_ref().unwrap().contains("file"));
        assert!(rb.required);

        // Should use MultipartFile schema variant
        assert_eq!(
            rb.schema,
            RequestBodySchema::MultipartFile {
                field_name: "file".to_owned()
            }
        );

        // Should also set allowed_request_content_types
        assert!(builder.spec.allowed_request_content_types.is_some());
        let allowed = builder.spec.allowed_request_content_types.as_ref().unwrap();
        assert_eq!(allowed.len(), 1);
        assert!(allowed.contains(&"multipart/form-data"));
    }

    #[test]
    fn test_multipart_file_request_without_description() {
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/upload")
            .multipart_file_request("file", None)
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success");

        assert!(builder.spec.request_body.is_some());
        let rb = builder.spec.request_body.as_ref().unwrap();
        assert_eq!(rb.content_type, "multipart/form-data");
        assert!(rb.description.is_none());
        assert_eq!(
            rb.schema,
            RequestBodySchema::MultipartFile {
                field_name: "file".to_owned()
            }
        );
    }

    #[test]
    fn test_octet_stream_request() {
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/upload")
            .operation_id("test.upload")
            .summary("Upload raw file")
            .octet_stream_request(Some("Raw file bytes"))
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success");

        // Should set request body with application/octet-stream
        assert!(builder.spec.request_body.is_some());
        let rb = builder.spec.request_body.as_ref().unwrap();
        assert_eq!(rb.content_type, "application/octet-stream");
        assert_eq!(rb.description, Some("Raw file bytes".to_owned()));
        assert!(rb.required);

        // Should use Binary schema variant
        assert_eq!(rb.schema, RequestBodySchema::Binary);

        // Should also set allowed_request_content_types
        assert!(builder.spec.allowed_request_content_types.is_some());
        let allowed = builder.spec.allowed_request_content_types.as_ref().unwrap();
        assert_eq!(allowed.len(), 1);
        assert!(allowed.contains(&"application/octet-stream"));
    }

    #[test]
    fn test_octet_stream_request_without_description() {
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/upload")
            .octet_stream_request(None)
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success");

        assert!(builder.spec.request_body.is_some());
        let rb = builder.spec.request_body.as_ref().unwrap();
        assert_eq!(rb.content_type, "application/octet-stream");
        assert!(rb.description.is_none());
        assert_eq!(rb.schema, RequestBodySchema::Binary);
    }

    #[test]
    fn test_json_request_uses_ref_schema() {
        let registry = MockRegistry::new();
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/test")
            .json_request::<serde_json::Value>(&registry, "Test request body")
            .public()
            .handler(test_handler)
            .json_response(http::StatusCode::OK, "Success");

        assert!(builder.spec.request_body.is_some());
        let rb = builder.spec.request_body.as_ref().unwrap();
        assert_eq!(rb.content_type, "application/json");

        // Should use Ref schema variant with the registered schema name
        match &rb.schema {
            RequestBodySchema::Ref { schema_name } => {
                assert!(!schema_name.is_empty());
            }
            _ => panic!("Expected RequestBodySchema::Ref for JSON request"),
        }
    }

    #[test]
    fn test_response_content_types_must_not_contain_parameters() {
        // This test ensures OpenAPI correctness: media type keys cannot include
        // parameters like "; charset=utf-8"
        let registry = MockRegistry::new();
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/tests/v1/test")
            .operation_id("test.content_type_purity")
            .summary("Test response content types")
            .json_request::<serde_json::Value>(&registry, "Test")
            .public()
            .handler(test_handler)
            .text_response(http::StatusCode::OK, "Text", "text/plain")
            .text_response(http::StatusCode::OK, "Markdown", "text/markdown")
            .html_response(http::StatusCode::OK, "HTML")
            .json_response(http::StatusCode::OK, "JSON")
            .problem_response(&registry, http::StatusCode::BAD_REQUEST, "Error");

        // Verify no response content_type contains semicolon (parameter separator)
        for response in &builder.spec.responses {
            assert!(
                !response.content_type.contains(';'),
                "Response content_type '{}' must not contain parameters. \
                 Use pure media type without charset or other parameters. \
                 OpenAPI media type keys cannot include parameters.",
                response.content_type
            );
        }
    }
}
