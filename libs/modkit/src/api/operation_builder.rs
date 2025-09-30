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

/// Type alias for schema collections used in API operations.
type SchemaCollection = Vec<(
    String,
    utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
)>;

/// Type-state markers for compile-time enforcement
pub mod state {
    /// Marker for missing required components
    #[derive(Debug, Clone, Copy)]
    pub struct Missing;

    /// Marker for present required components
    #[derive(Debug, Clone, Copy)]
    pub struct Present;
}

/// Internal trait mapping handler state to the concrete router slot type.
/// For `Missing` there is no router slot; for `Present` it is `MethodRouter<S>`.
/// Private sealed trait to enforce the implementation is only visible within this module.
mod sealed {
    pub trait Sealed {}
}

pub trait HandlerSlot<S>: sealed::Sealed {
    type Slot;
}

impl sealed::Sealed for Missing {}
impl sealed::Sealed for Present {}

impl<S> HandlerSlot<S> for Missing {
    type Slot = ();
}
impl<S> HandlerSlot<S> for Present {
    type Slot = MethodRouter<S>;
}

pub use state::{Missing, Present};

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

/// Request body specification for API operations
#[derive(Clone, Debug)]
pub struct RequestBodySpec {
    pub content_type: &'static str,
    pub description: Option<String>,
    /// Name of a registered component schema (if any). The OpenAPI generator
    /// will reference it by $ref. If `None`, generator may inline or skip.
    pub schema_name: Option<String>,
    /// Whether request body is required (OpenAPI default is `false`).
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
}

//
pub trait OperationBuilderODataExt<S, H, R> {
    /// Adds optional `$filter` query parameter to OpenAPI.
    fn with_odata_filter(self) -> Self;

    /// Same as above but with explicit description (e.g., allowed fields).
    fn with_odata_filter_doc(self, description: impl Into<String>) -> Self;
}

impl<S, H, R> OperationBuilderODataExt<S, H, R> for OperationBuilder<H, R, S>
where
    H: HandlerSlot<S>,
{
    fn with_odata_filter(mut self) -> Self {
        self.spec.params.push(ParamSpec {
            name: "$filter".to_string(),
            location: ParamLocation::Query,
            required: false,
            description: Some("OData v4 filter expression".to_string()),
            param_type: "string".to_string(),
        });
        self
    }

    fn with_odata_filter_doc(mut self, description: impl Into<String>) -> Self {
        self.spec.params.push(ParamSpec {
            name: "$filter".to_string(),
            location: ParamLocation::Query,
            required: false,
            description: Some(description.into()),
            param_type: "string".to_string(),
        });
        self
    }
}

/// Registry trait for OpenAPI operations and schemas
pub trait OpenApiRegistry {
    /// Register an API operation specification
    fn register_operation(&self, spec: &OperationSpec);

    /// Ensure schema for `T` (including transitive dependencies) is registered
    /// under components and return the canonical component name for `$ref`.
    /// This is a type-erased version for dyn compatibility.
    fn ensure_schema_raw(&self, name: &str, schemas: SchemaCollection) -> String;

    /// Downcast support for accessing the concrete implementation if needed.
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Helper function to call ensure_schema with proper type information
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

/// Type-safe operation builder with compile-time guarantees.
///
/// Generic parameters:
/// - `H`: Handler state (Missing | Present)
/// - `R`: Response state (Missing | Present)
/// - `S`: Router state type (what you put into `Router::with_state(S)`).
pub struct OperationBuilder<H, R, S>
where
    H: HandlerSlot<S>,
{
    spec: OperationSpec,
    method_router: <H as HandlerSlot<S>>::Slot,
    _has_handler: PhantomData<H>,
    _has_response: PhantomData<R>,
    #[allow(clippy::type_complexity)]
    _state: PhantomData<fn() -> S>, // Zero-sized marker for type-state pattern
}

// -------------------------------------------------------------------------------------------------
// Constructors — starts with both handler and response missing
// -------------------------------------------------------------------------------------------------
impl<S> OperationBuilder<Missing, Missing, S> {
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
            },
            method_router: (), // no router in Missing state
            _has_handler: PhantomData,
            _has_response: PhantomData,
            _state: PhantomData,
        }
    }

    /// Convenience constructor for GET requests
    pub fn get(path: impl Into<String>) -> Self {
        Self::new(Method::GET, path)
    }

    /// Convenience constructor for POST requests
    pub fn post(path: impl Into<String>) -> Self {
        Self::new(Method::POST, path)
    }

    /// Convenience constructor for PUT requests
    pub fn put(path: impl Into<String>) -> Self {
        Self::new(Method::PUT, path)
    }

    /// Convenience constructor for DELETE requests
    pub fn delete(path: impl Into<String>) -> Self {
        Self::new(Method::DELETE, path)
    }

    /// Convenience constructor for PATCH requests
    pub fn patch(path: impl Into<String>) -> Self {
        Self::new(Method::PATCH, path)
    }
}

// -------------------------------------------------------------------------------------------------
// Descriptive methods — available at any stage
// -------------------------------------------------------------------------------------------------
impl<H, R, S> OperationBuilder<H, R, S>
where
    H: HandlerSlot<S>,
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
            param_type: "string".to_string(),
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
            param_type: "string".to_string(),
        });
        self
    }

    /// Add a typed query parameter with explicit OpenAPI type
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
            schema_name: Some(schema_name.into()),
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
            schema_name: Some(schema_name.into()),
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
            schema_name: Some(name),
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
            schema_name: Some(name),
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
}

// -------------------------------------------------------------------------------------------------
// Handler setting — transitions Missing -> Present for handler
// -------------------------------------------------------------------------------------------------
impl<R, S> OperationBuilder<Missing, R, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Set the handler for this operation (function handlers are recommended).
    ///
    /// This transitions the builder from `Missing` to `Present` handler state.
    pub fn handler<F, T>(self, h: F) -> OperationBuilder<Present, R, S>
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
        }
    }

    /// Alternative path: provide a pre-composed `MethodRouter<S>` yourself
    /// (useful to attach per-route middleware/layers).
    pub fn method_router(self, mr: MethodRouter<S>) -> OperationBuilder<Present, R, S> {
        OperationBuilder {
            spec: self.spec,
            method_router: mr, // concrete MethodRouter<S> in Present state
            _has_handler: PhantomData::<Present>,
            _has_response: self._has_response,
            _state: self._state,
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Response setting — transitions Missing -> Present for response (first response)
// -------------------------------------------------------------------------------------------------
impl<H, S> OperationBuilder<H, Missing, S>
where
    H: HandlerSlot<S>,
{
    /// Add a raw response spec (transitions from Missing to Present).
    pub fn response(mut self, resp: ResponseSpec) -> OperationBuilder<H, Present, S> {
        self.spec.responses.push(resp);
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
        }
    }

    /// Add a JSON response (transitions from Missing to Present).
    pub fn json_response(
        mut self,
        status: u16,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S> {
        self.spec.responses.push(ResponseSpec {
            status,
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
        }
    }

    /// Add a JSON response with a registered schema (transitions from Missing to Present).
    pub fn json_response_with_schema<T>(
        mut self,
        registry: &dyn OpenApiRegistry,
        status: u16,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S>
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(registry);
        self.spec.responses.push(ResponseSpec {
            status,
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
        }
    }

    /// Add a text response (transitions from Missing to Present).
    pub fn text_response(
        mut self,
        status: u16,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S> {
        self.spec.responses.push(ResponseSpec {
            status,
            content_type: "text/plain",
            description: description.into(),
            schema_name: None,
        });
        OperationBuilder {
            spec: self.spec,
            method_router: self.method_router,
            _has_handler: self._has_handler,
            _has_response: PhantomData::<Present>,
            _state: self._state,
        }
    }

    /// Add an HTML response (transitions from Missing to Present).
    pub fn html_response(
        mut self,
        status: u16,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S> {
        self.spec.responses.push(ResponseSpec {
            status,
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
        }
    }

    /// Add an RFC 9457 `application/problem+json` response (transitions from Missing to Present).
    pub fn problem_response(
        mut self,
        registry: &dyn OpenApiRegistry,
        status: u16,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S> {
        // Ensure `Problem` schema is registered in components
        let problem_name = ensure_schema::<crate::api::problem::Problem>(registry);
        self.spec.responses.push(ResponseSpec {
            status,
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
        }
    }

    /// First response: SSE stream of JSON events (`text/event-stream`).
    pub fn sse_json<T>(
        mut self,
        openapi: &dyn OpenApiRegistry,
        description: impl Into<String>,
    ) -> OperationBuilder<H, Present, S>
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(openapi);
        self.spec.responses.push(ResponseSpec {
            status: 200,
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
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Additional responses — for Present response state (additional responses)
// -------------------------------------------------------------------------------------------------
impl<H, S> OperationBuilder<H, Present, S>
where
    H: HandlerSlot<S>,
{
    /// Add a JSON response (additional).
    pub fn json_response(mut self, status: u16, description: impl Into<String>) -> Self {
        self.spec.responses.push(ResponseSpec {
            status,
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
        status: u16,
        description: impl Into<String>,
    ) -> Self
    where
        T: utoipa::ToSchema + utoipa::PartialSchema + 'static,
    {
        let name = ensure_schema::<T>(registry);
        self.spec.responses.push(ResponseSpec {
            status,
            content_type: "application/json",
            description: description.into(),
            schema_name: Some(name),
        });
        self
    }

    /// Add a text response (additional).
    pub fn text_response(mut self, status: u16, description: impl Into<String>) -> Self {
        self.spec.responses.push(ResponseSpec {
            status,
            content_type: "text/plain",
            description: description.into(),
            schema_name: None,
        });
        self
    }

    /// Add an HTML response (additional).
    pub fn html_response(mut self, status: u16, description: impl Into<String>) -> Self {
        self.spec.responses.push(ResponseSpec {
            status,
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
        status: u16,
        description: impl Into<String>,
    ) -> Self {
        let problem_name = ensure_schema::<crate::api::problem::Problem>(registry);
        self.spec.responses.push(ResponseSpec {
            status,
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
            status: 200,
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
    /// let op = OperationBuilder::get("/users")
    ///     .handler(list_users)
    ///     .json_response(200, "List of users")
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
        let problem_name = ensure_schema::<crate::api::problem::Problem>(registry);

        let standard_errors = [
            (400, "Bad Request"),
            (401, "Unauthorized"),
            (403, "Forbidden"),
            (404, "Not Found"),
            (409, "Conflict"),
            (422, "Unprocessable Entity"),
            (429, "Too Many Requests"),
            (500, "Internal Server Error"),
        ];

        for (status, description) in standard_errors {
            self.spec.responses.push(ResponseSpec {
                status,
                content_type: problem::APPLICATION_PROBLEM_JSON,
                description: description.to_string(),
                schema_name: Some(problem_name.clone()),
            });
        }

        self
    }

    /// Add 422 validation error response using ValidationError schema.
    ///
    /// This method adds a specific 422 Unprocessable Entity response that uses
    /// the ValidationError schema instead of the generic Problem schema. Use this
    /// for endpoints that perform input validation and need structured error details.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let op = OperationBuilder::post("/users")
    ///     .handler(create_user)
    ///     .json_request::<CreateUserRequest>(&registry, "User data")
    ///     .json_response(201, "User created")
    ///     .with_422_validation_error(&registry);
    /// ```
    pub fn with_422_validation_error(mut self, registry: &dyn OpenApiRegistry) -> Self {
        let validation_error_name =
            ensure_schema::<crate::api::problem::ValidationErrorResponse>(registry);

        self.spec.responses.push(ResponseSpec {
            status: 422,
            content_type: problem::APPLICATION_PROBLEM_JSON,
            description: "Validation Error".to_string(),
            schema_name: Some(validation_error_name),
        });

        self
    }
}

// -------------------------------------------------------------------------------------------------
// Registration — only available when both handler AND response are present
// -------------------------------------------------------------------------------------------------
impl<S> OperationBuilder<Present, Present, S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Register the operation with the router and OpenAPI registry.
    ///
    /// This method is only available when both handler and response are present,
    /// enforced at compile time by the type system.
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
            let name = name.to_string();
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
        let builder = OperationBuilder::<Missing, Missing, ()>::get("/test")
            .operation_id("test.get")
            .summary("Test endpoint")
            .description("A test endpoint for validation")
            .tag("test")
            .path_param("id", "Test ID");

        assert_eq!(builder.spec.method, Method::GET);
        assert_eq!(builder.spec.path, "/test");
        assert_eq!(builder.spec.operation_id, Some("test.get".to_string()));
        assert_eq!(builder.spec.summary, Some("Test endpoint".to_string()));
        assert_eq!(
            builder.spec.description,
            Some("A test endpoint for validation".to_string())
        );
        assert_eq!(builder.spec.tags, vec!["test"]);
        assert_eq!(builder.spec.params.len(), 1);
    }

    #[tokio::test]
    async fn test_builder_with_request_response_and_handler() {
        let registry = MockRegistry::new();
        let router = Router::new();

        let _router = OperationBuilder::<Missing, Missing, ()>::post("/test")
            .summary("Test endpoint")
            .json_request::<serde_json::Value>(&registry, "optional body") // registers schema
            .handler(test_handler)
            .json_response_with_schema::<serde_json::Value>(&registry, 200, "Success response") // registers schema
            .register(router, &registry);

        // Verify that the operation was registered
        let ops = registry.operations.lock().unwrap();
        assert_eq!(ops.len(), 1);
        let op = &ops[0];
        assert_eq!(op.method, Method::POST);
        assert_eq!(op.path, "/test");
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
        let get_builder = OperationBuilder::<Missing, Missing, ()>::get("/get");
        assert_eq!(get_builder.spec.method, Method::GET);

        let post_builder = OperationBuilder::<Missing, Missing, ()>::post("/post");
        assert_eq!(post_builder.spec.method, Method::POST);

        let put_builder = OperationBuilder::<Missing, Missing, ()>::put("/put");
        assert_eq!(put_builder.spec.method, Method::PUT);

        let delete_builder = OperationBuilder::<Missing, Missing, ()>::delete("/delete");
        assert_eq!(delete_builder.spec.method, Method::DELETE);

        let patch_builder = OperationBuilder::<Missing, Missing, ()>::patch("/patch");
        assert_eq!(patch_builder.spec.method, Method::PATCH);
    }

    #[test]
    fn test_standard_errors() {
        let registry = MockRegistry::new();
        let builder = OperationBuilder::<Missing, Missing, ()>::get("/test")
            .handler(test_handler)
            .json_response(200, "Success")
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
        let builder = OperationBuilder::<Missing, Missing, ()>::post("/test")
            .handler(test_handler)
            .json_response(201, "Created")
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
}
