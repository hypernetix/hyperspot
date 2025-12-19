//! Public models for the `types-registry` module.
//!
//! These are transport-agnostic data structures that define the contract
//! between the `types-registry` module and its consumers.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

// Re-export GtsIdSegment from gts-rust
pub use gts::GtsIdSegment;

/// The kind of GTS entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GtsEntityKind {
    /// A type definition (GTS ID ends with `~`)
    Type,
    /// An instance of a type (GTS ID does not end with `~`)
    Instance,
}

impl GtsEntityKind {
    /// Returns `true` if this is a type definition.
    #[must_use]
    pub const fn is_type(self) -> bool {
        matches!(self, Self::Type)
    }

    /// Returns `true` if this is an instance.
    #[must_use]
    pub const fn is_instance(self) -> bool {
        matches!(self, Self::Instance)
    }
}

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
pub struct GtsEntity<C = serde_json::Value>
where
    C: Serialize + DeserializeOwned + Clone,
{
    /// Deterministic UUID generated from the GTS ID.
    ///
    /// This UUID is generated using UUID v5 with a GTS namespace,
    /// ensuring the same GTS ID always produces the same UUID.
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

    /// The kind of entity (Type or Instance).
    pub kind: GtsEntityKind,

    /// The entity content (schema for types, object for instances).
    pub content: C,

    /// Optional description of the entity.
    pub description: Option<String>,
}

/// Type alias for dynamic GTS entities using `serde_json::Value` as content.
pub type DynGtsEntity = GtsEntity<serde_json::Value>;

impl<C> GtsEntity<C>
where
    C: Serialize + DeserializeOwned + Clone,
{
    /// Creates a new `GtsEntity` with the given components.
    #[must_use]
    pub fn new(
        id: Uuid,
        gts_id: impl Into<String>,
        segments: Vec<GtsIdSegment>,
        kind: GtsEntityKind,
        content: C,
        description: Option<String>,
    ) -> Self {
        Self {
            id,
            gts_id: gts_id.into(),
            segments,
            kind,
            content,
            description,
        }
    }

    /// Returns `true` if this entity is a type definition.
    #[must_use]
    pub const fn is_type(&self) -> bool {
        self.kind.is_type()
    }

    /// Returns `true` if this entity is an instance.
    #[must_use]
    pub const fn is_instance(&self) -> bool {
        self.kind.is_instance()
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

/// Query parameters for listing GTS entities.
///
/// All fields are optional. When a field is `None`, no filtering
/// is applied for that field.
///
/// # Example
///
/// ```
/// use types_registry_sdk::ListQuery;
///
/// // List all entities
/// let query = ListQuery::default();
///
/// // List only types from vendor "acme"
/// let query = ListQuery::default()
///     .with_is_type(true)
///     .with_vendor("acme");
///
/// // List entities matching a pattern
/// let query = ListQuery::default()
///     .with_pattern("gts.acme.core.*");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListQuery {
    /// Optional wildcard pattern for GTS ID matching.
    ///
    /// Supports `*` as a wildcard character.
    pub pattern: Option<String>,

    /// Filter for entity kind: `true` for types, `false` for instances.
    pub is_type: Option<bool>,

    /// Filter by vendor (matches any segment).
    pub vendor: Option<String>,

    /// Filter by package (matches any segment).
    pub package: Option<String>,

    /// Filter by namespace (matches any segment).
    pub namespace: Option<String>,
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
    fn test_gts_entity_kind() {
        assert!(GtsEntityKind::Type.is_type());
        assert!(!GtsEntityKind::Type.is_instance());
        assert!(GtsEntityKind::Instance.is_instance());
        assert!(!GtsEntityKind::Instance.is_type());
    }

    #[test]
    fn test_gts_entity_accessors() {
        let segment = GtsIdSegment::new(0, 0, "acme.core.events.user_created.v1~").unwrap();
        let entity = GtsEntity::new(
            Uuid::nil(),
            "gts.acme.core.events.user_created.v1~",
            vec![segment],
            GtsEntityKind::Type,
            serde_json::json!({"type": "object"}),
            Some("A user created event".to_owned()),
        );

        assert!(entity.is_type());
        assert!(!entity.is_instance());
        assert_eq!(entity.vendor(), Some("acme"));
        assert_eq!(entity.package(), Some("core"));
        assert_eq!(entity.namespace(), Some("events"));
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
        assert!(!query.is_empty());
    }

    #[test]
    fn test_list_query_empty() {
        let query = ListQuery::default();
        assert!(query.is_empty());
    }
}
