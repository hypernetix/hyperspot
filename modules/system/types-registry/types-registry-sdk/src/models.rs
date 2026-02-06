//! Public models for the `types-registry` module.
//!
//! These are transport-agnostic data structures that define the contract
//! between the `types-registry` module and its consumers.

use gts::GtsIdSegment;
use uuid::Uuid;

/// A registered GTS entity.
///
/// This represents either a type definition or an instance that has been
/// registered in the Types Registry.
///
/// # Type Parameter
///
/// - `C`: The content type. Use `serde_json::Value` for dynamic content,
///   or a concrete struct for type-safe access.
///
/// # Example
///
/// ```ignore
/// // Dynamic entity (default)
/// let entity: GtsEntity = registry.get(&ctx, "gts.acme.core.events.user_created.v1~").await?;
///
/// // Type-safe entity
/// let entity: GtsEntity<MySchema> = registry.get(&ctx, gts_id).await?;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct GtsEntity<C = serde_json::Value> {
    /// Deterministic UUID generated from the GTS ID.
    ///
    /// This UUID is generated using UUID v5 with a GTS-specific namespace.
    /// The namespace is derived as `Uuid::new_v5(&Uuid::NAMESPACE_URL, b"gts")`,
    /// which is a GTS specification-defined constant ensuring consistent
    /// UUID generation across all implementations.
    pub id: Uuid,

    /// The full GTS identifier string.
    ///
    /// For types: `gts.vendor.package.namespace.name.version~`
    /// For instances: `gts.vendor.package.namespace.name.version~instance.id`
    pub gts_id: String,

    /// All parsed segments from the GTS ID.
    ///
    /// For simple IDs, this contains one segment.
    /// For chained IDs (instances), this contains multiple segments.
    pub segments: Vec<GtsIdSegment>,

    /// Whether this entity is a schema (type definition).
    ///
    /// - `true`: This is a type definition (GTS ID ends with `~`)
    /// - `false`: This is an instance (GTS ID does not end with `~`)
    pub is_schema: bool,

    /// The entity content (schema for types, object for instances).
    pub content: C,

    /// Optional description of the entity.
    pub description: Option<String>,
}

/// Type alias for dynamic GTS entities using `serde_json::Value` as content.
pub type DynGtsEntity = GtsEntity<serde_json::Value>;

/// Wrapper for JSON Schema content in type definitions.
///
/// This newtype provides semantic clarity when working with GTS type entities,
/// indicating that the content represents a JSON Schema definition.
///
/// # Examples
///
/// ## Creating a schema
///
/// ```
/// use types_registry_sdk::TypeSchema;
///
/// // Direct construction
/// let schema = TypeSchema::new(serde_json::json!({
///     "type": "object",
///     "properties": {
///         "name": { "type": "string" },
///         "age": { "type": "integer" }
///     },
///     "required": ["name"]
/// }));
///
/// // From conversion
/// let schema: TypeSchema = serde_json::json!({"type": "string"}).into();
/// ```
///
/// ## Accessing the inner value
///
/// ```
/// use types_registry_sdk::TypeSchema;
///
/// let schema = TypeSchema::new(serde_json::json!({"type": "object"}));
///
/// // Via Deref (most idiomatic)
/// assert!(schema.is_object());
///
/// // Via AsRef
/// let value: &serde_json::Value = schema.as_ref();
///
/// // Consuming
/// let value: serde_json::Value = schema.into_inner();
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TypeSchema(pub serde_json::Value);

impl TypeSchema {
    /// Creates a new `TypeSchema` from a JSON value.
    #[must_use]
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }

    /// Consumes self and returns the inner JSON value.
    #[must_use]
    pub fn into_inner(self) -> serde_json::Value {
        self.0
    }
}

impl std::ops::Deref for TypeSchema {
    type Target = serde_json::Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<serde_json::Value> for TypeSchema {
    fn as_ref(&self) -> &serde_json::Value {
        &self.0
    }
}

impl From<serde_json::Value> for TypeSchema {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl From<TypeSchema> for serde_json::Value {
    fn from(schema: TypeSchema) -> Self {
        schema.0
    }
}

/// Wrapper for instance object content.
///
/// This newtype provides semantic clarity when working with GTS instance entities,
/// indicating that the content represents an instance object (data conforming to a schema).
///
/// # Examples
///
/// ## Creating an instance
///
/// ```
/// use types_registry_sdk::InstanceObject;
///
/// // Direct construction
/// let instance = InstanceObject::new(serde_json::json!({
///     "name": "John Doe",
///     "email": "john@example.com",
///     "age": 30
/// }));
///
/// // From conversion
/// let instance: InstanceObject = serde_json::json!({"id": 123}).into();
/// ```
///
/// ## Accessing the inner value
///
/// ```
/// use types_registry_sdk::InstanceObject;
///
/// let instance = InstanceObject::new(serde_json::json!({"name": "Alice"}));
///
/// // Via Deref (most idiomatic) - access serde_json::Value methods directly
/// assert!(instance.is_object());
/// assert_eq!(instance["name"], "Alice");
///
/// // Via AsRef
/// let value: &serde_json::Value = instance.as_ref();
///
/// // Consuming
/// let value: serde_json::Value = instance.into_inner();
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InstanceObject(pub serde_json::Value);

impl InstanceObject {
    /// Creates a new `InstanceObject` from a JSON value.
    #[must_use]
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }

    /// Consumes self and returns the inner JSON value.
    #[must_use]
    pub fn into_inner(self) -> serde_json::Value {
        self.0
    }
}

impl std::ops::Deref for InstanceObject {
    type Target = serde_json::Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<serde_json::Value> for InstanceObject {
    fn as_ref(&self) -> &serde_json::Value {
        &self.0
    }
}

impl From<serde_json::Value> for InstanceObject {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl From<InstanceObject> for serde_json::Value {
    fn from(instance: InstanceObject) -> Self {
        instance.0
    }
}

/// Type alias for GTS type definition entities.
///
/// Use this when you specifically expect a type definition (GTS ID ends with `~`).
/// The content is a [`TypeSchema`] representing the JSON Schema.
///
/// # Example
///
/// ```ignore
/// // Retrieve a type definition
/// let type_entity: GtsTypeEntity = registry.get_type(&ctx, "gts.acme.core.events.user.v1~").await?;
///
/// // Access the schema directly via Deref
/// if type_entity.content.is_object() {
///     println!("Schema: {}", type_entity.content);
/// }
/// ```
pub type GtsTypeEntity = GtsEntity<TypeSchema>;

/// Type alias for GTS instance entities.
///
/// Use this when you specifically expect an instance (GTS ID does not end with `~`).
/// The content is an [`InstanceObject`] representing data conforming to a schema.
///
/// # Example
///
/// ```ignore
/// // Retrieve an instance
/// let instance: GtsInstanceEntity = registry.get_instance(&ctx, "gts.acme.core.events.user.v1~user123").await?;
///
/// // Access instance data directly via Deref
/// let name = &instance.content["name"];
/// ```
pub type GtsInstanceEntity = GtsEntity<InstanceObject>;

/// Result of registering a single GTS entity in a batch operation.
///
/// This type provides per-item success/error reporting for batch registration,
/// allowing partial success and detailed error information for each item.
///
/// # Example
///
/// ```ignore
/// let results = registry.register(&ctx, entities).await?;
/// for (index, result) in results.iter().enumerate() {
///     match result {
///         RegisterResult::Ok(entity) => println!("Registered: {}", entity.gts_id),
///         RegisterResult::Err { gts_id, error } => {
///             eprintln!("Failed to register {}: {}", gts_id.as_deref().unwrap_or("unknown"), error);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum RegisterResult<C = serde_json::Value> {
    /// Successfully registered entity.
    Ok(GtsEntity<C>),
    /// Failed to register entity.
    Err {
        /// The GTS ID that was attempted, if it could be extracted from the input.
        gts_id: Option<String>,
        /// The error that occurred during registration.
        error: crate::TypesRegistryError,
    },
}

impl<C> RegisterResult<C> {
    /// Returns `true` if the registration was successful.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Returns `true` if the registration failed.
    #[must_use]
    pub const fn is_err(&self) -> bool {
        matches!(self, Self::Err { .. })
    }

    /// Converts to `Result<&GtsEntity<C>, &TypesRegistryError>`.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a reference to the error if this is a failed registration.
    pub fn as_result(&self) -> Result<&GtsEntity<C>, &crate::TypesRegistryError> {
        match self {
            Self::Ok(entity) => Ok(entity),
            Self::Err { error, .. } => Err(error),
        }
    }

    /// Converts into `Result<GtsEntity<C>, TypesRegistryError>`.
    ///
    /// # Errors
    ///
    /// Returns `Err` with the error if this is a failed registration.
    pub fn into_result(self) -> Result<GtsEntity<C>, crate::TypesRegistryError> {
        match self {
            Self::Ok(entity) => Ok(entity),
            Self::Err { error, .. } => Err(error),
        }
    }

    /// Returns the entity if successful, `None` otherwise.
    #[must_use]
    pub fn ok(self) -> Option<GtsEntity<C>> {
        match self {
            Self::Ok(entity) => Some(entity),
            Self::Err { .. } => None,
        }
    }

    /// Returns the error if failed, `None` otherwise.
    #[must_use]
    pub fn err(self) -> Option<crate::TypesRegistryError> {
        match self {
            Self::Ok(_) => None,
            Self::Err { error, .. } => Some(error),
        }
    }
}

/// Type alias for dynamic register results using `serde_json::Value` as content.
pub type DynRegisterResult = RegisterResult<serde_json::Value>;

/// Summary of a batch registration operation.
///
/// Provides aggregate counts for quick success/failure assessment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegisterSummary {
    /// Number of successfully registered entities.
    pub succeeded: usize,
    /// Number of failed registrations.
    pub failed: usize,
}

impl RegisterSummary {
    /// Creates a new summary from a slice of register results.
    #[must_use]
    pub fn from_results<C>(results: &[RegisterResult<C>]) -> Self {
        let succeeded = results.iter().filter(|r| r.is_ok()).count();
        let failed = results.len() - succeeded;
        Self { succeeded, failed }
    }

    /// Returns `true` if all registrations succeeded.
    #[must_use]
    pub const fn all_succeeded(&self) -> bool {
        self.failed == 0
    }

    /// Returns `true` if all registrations failed.
    #[must_use]
    pub const fn all_failed(&self) -> bool {
        self.succeeded == 0
    }

    /// Returns the total number of items processed.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.succeeded + self.failed
    }
}

impl<C> GtsEntity<C> {
    /// Creates a new `GtsEntity` with the given components.
    #[must_use]
    pub fn new(
        id: Uuid,
        gts_id: impl Into<String>,
        segments: Vec<GtsIdSegment>,
        is_schema: bool,
        content: C,
        description: Option<String>,
    ) -> Self {
        Self {
            id,
            gts_id: gts_id.into(),
            segments,
            is_schema,
            content,
            description,
        }
    }

    /// Returns `true` if this entity is a type definition (schema).
    #[must_use]
    pub const fn is_type(&self) -> bool {
        self.is_schema
    }

    /// Returns `true` if this entity is an instance.
    #[must_use]
    pub const fn is_instance(&self) -> bool {
        !self.is_schema
    }

    /// Returns the primary segment (first segment in the chain).
    #[must_use]
    pub fn primary_segment(&self) -> Option<&GtsIdSegment> {
        self.segments.first()
    }

    /// Returns the vendor from the primary segment.
    #[must_use]
    pub fn vendor(&self) -> Option<&str> {
        self.primary_segment().map(|s| s.vendor.as_str())
    }

    /// Returns the package from the primary segment.
    #[must_use]
    pub fn package(&self) -> Option<&str> {
        self.primary_segment().map(|s| s.package.as_str())
    }

    /// Returns the namespace from the primary segment.
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.primary_segment().map(|s| s.namespace.as_str())
    }
}

/// Specifies which segments in a chained GTS ID the filters should match against.
///
/// GTS IDs can be chained (e.g., `gts.vendor.pkg.ns.type.v1~instance.id`),
/// containing multiple segments. This enum controls whether filters like
/// `vendor`, `package`, and `namespace` apply to just the primary (first)
/// segment or to any segment in the chain.
///
/// # Example
///
/// For a chained GTS ID like `gts.acme.core.events.order.v1~acme.billing.invoices.line_item.v1`:
/// - `Primary`: Only matches against `acme.core.events.order.v1`
/// - `Any`: Matches against both segments
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SegmentMatchScope {
    /// Match filters against only the primary (first) GTS ID segment.
    Primary,
    /// Match filters against any segment in the GTS ID chain.
    #[default]
    Any,
}

impl SegmentMatchScope {
    /// Returns `true` if this scope matches only the primary segment.
    #[must_use]
    pub const fn is_primary(self) -> bool {
        matches!(self, Self::Primary)
    }

    /// Returns `true` if this scope matches any segment.
    #[must_use]
    pub const fn is_any(self) -> bool {
        matches!(self, Self::Any)
    }
}

/// Query parameters for listing GTS entities.
///
/// All fields are optional. When a field is `None`, no filtering
/// is applied for that field.
///
/// # Segment Matching
///
/// The `segment_scope` field controls how `vendor`, `package`, and `namespace`
/// filters are applied to chained GTS IDs. By default, filters match
/// any segment in the chain.
///
/// # Example
///
/// ```
/// use types_registry_sdk::{ListQuery, SegmentMatchScope};
///
/// // List all entities
/// let query = ListQuery::default();
///
/// // List only types from vendor "acme" (matches any segment by default)
/// let query = ListQuery::default()
///     .with_is_type(true)
///     .with_vendor("acme");
///
/// // List entities where only the primary segment has vendor "acme"
/// let query = ListQuery::default()
///     .with_vendor("acme")
///     .with_segment_scope(SegmentMatchScope::Primary);
///
/// // List entities matching a pattern
/// let query = ListQuery::default()
///     .with_pattern("gts.acme.core.*");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListQuery {
    /// Optional wildcard pattern for GTS ID matching.
    ///
    /// Supports `*` as a wildcard character.
    pub pattern: Option<String>,

    /// Filter for entity kind: `true` for types, `false` for instances.
    pub is_type: Option<bool>,

    /// Filter by vendor.
    ///
    /// Which segments this applies to is controlled by `segment_scope`.
    pub vendor: Option<String>,

    /// Filter by package.
    ///
    /// Which segments this applies to is controlled by `segment_scope`.
    pub package: Option<String>,

    /// Filter by namespace.
    ///
    /// Which segments this applies to is controlled by `segment_scope`.
    pub namespace: Option<String>,

    /// Controls which segments the `vendor`, `package`, and `namespace`
    /// filters are matched against.
    ///
    /// Defaults to `Any` (matches any segment in the chain).
    pub segment_scope: SegmentMatchScope,
}

impl ListQuery {
    /// Creates a new empty `ListQuery`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pattern filter.
    #[must_use]
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Sets the `is_type` filter.
    #[must_use]
    pub const fn with_is_type(mut self, is_type: bool) -> Self {
        self.is_type = Some(is_type);
        self
    }

    /// Sets the vendor filter.
    #[must_use]
    pub fn with_vendor(mut self, vendor: impl Into<String>) -> Self {
        self.vendor = Some(vendor.into());
        self
    }

    /// Sets the package filter.
    #[must_use]
    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    /// Sets the namespace filter.
    #[must_use]
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Sets the segment match scope.
    #[must_use]
    pub const fn with_segment_scope(mut self, scope: SegmentMatchScope) -> Self {
        self.segment_scope = scope;
        self
    }

    /// Returns `true` if no filters are set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pattern.is_none()
            && self.is_type.is_none()
            && self.vendor.is_none()
            && self.package.is_none()
            && self.namespace.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gts_id_segment_from_gts_rust() {
        // GtsIdSegment::new(num, offset, segment_str) parses a GTS segment string
        let segment = GtsIdSegment::new(0, 0, "acme.core.events.user_created.v1~").unwrap();
        assert_eq!(segment.vendor, "acme");
        assert_eq!(segment.package, "core");
        assert_eq!(segment.namespace, "events");
        assert_eq!(segment.type_name, "user_created");
        assert_eq!(segment.ver_major, 1);
        assert!(segment.is_type);
    }

    #[test]
    fn test_gts_entity_accessors() {
        let segment = GtsIdSegment::new(0, 0, "acme.core.events.user_created.v1~").unwrap();
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.events.user_created.v1~",
            vec![segment],
            true, // is_schema
            serde_json::json!({"type": "object"}),
            Some("A user created event".to_owned()),
        );

        assert!(entity.is_type());
        assert!(!entity.is_instance());
        assert_eq!(entity.vendor(), Some("acme"));
        assert_eq!(entity.package(), Some("core"));
        assert_eq!(entity.namespace(), Some("events"));

        // Test instance
        let instance = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.events.user_created.v1~acme.core.instances.instance1.v1",
            vec![],
            false, // is_schema
            serde_json::json!({"data": "value"}),
            None,
        );
        assert!(!instance.is_type());
        assert!(instance.is_instance());
    }

    #[test]
    fn test_list_query_builder() {
        let query = ListQuery::new()
            .with_pattern("gts.acme.*")
            .with_is_type(true)
            .with_vendor("acme")
            .with_package("core")
            .with_namespace("events");

        assert_eq!(query.pattern, Some("gts.acme.*".to_owned()));
        assert_eq!(query.is_type, Some(true));
        assert_eq!(query.vendor, Some("acme".to_owned()));
        assert_eq!(query.package, Some("core".to_owned()));
        assert_eq!(query.namespace, Some("events".to_owned()));
        assert_eq!(query.segment_scope, SegmentMatchScope::Any);
        assert!(!query.is_empty());
    }

    #[test]
    fn test_list_query_empty() {
        let query = ListQuery::default();
        assert!(query.is_empty());
        assert_eq!(query.segment_scope, SegmentMatchScope::Any);
    }

    #[test]
    fn test_segment_match_scope() {
        assert!(SegmentMatchScope::Primary.is_primary());
        assert!(!SegmentMatchScope::Primary.is_any());
        assert!(SegmentMatchScope::Any.is_any());
        assert!(!SegmentMatchScope::Any.is_primary());

        // Default is Any
        assert_eq!(SegmentMatchScope::default(), SegmentMatchScope::Any);
    }

    #[test]
    fn test_list_query_with_segment_scope() {
        let query = ListQuery::new()
            .with_vendor("acme")
            .with_segment_scope(SegmentMatchScope::Any);

        assert_eq!(query.vendor, Some("acme".to_owned()));
        assert_eq!(query.segment_scope, SegmentMatchScope::Any);
    }
}
