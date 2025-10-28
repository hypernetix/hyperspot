use sea_orm::EntityTrait;

/// Defines the contract for entities that can be scoped by tenant and resource ID.
///
/// Each entity implementing this trait declares:
/// - Which column (if any) represents the tenant identifier
/// - Which column represents the primary resource identifier
/// - Which column (if any) represents the resource owner
///
/// # Multi-tenant vs Global Entities
/// - Multi-tenant entities return `Some(Column::TenantId)` from `tenant_col()`
/// - Global/system entities return `None` to indicate no tenant scoping
///
/// # Example (Manual Implementation)
/// ```rust,ignore
/// impl ScopableEntity for user::Entity {
///     fn tenant_col() -> Option<Self::Column> {
///         Some(user::Column::TenantId)
///     }
///     fn id_col() -> Self::Column {
///         user::Column::Id
///     }
///     // owner_col() uses default (returns None)
/// }
/// ```
///
/// # Example (Using Derive Macro)
/// ```rust,ignore
/// use modkit_db::secure::Scopable;
///
/// #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
/// #[sea_orm(table_name = "users")]
/// #[secure(tenant_col = "tenant_id")]
/// pub struct Model {
///     #[sea_orm(primary_key)]
///     pub id: Uuid,
///     pub tenant_id: Uuid,
///     pub email: String,
/// }
/// ```
pub trait ScopableEntity: EntityTrait {
    /// Indicates whether this entity is explicitly marked as unrestricted.
    ///
    /// This is a compile-time flag set via `#[secure(unrestricted)]` that documents
    /// the entity's global nature (e.g., system configuration, lookup tables).
    ///
    /// Default: `false` (entity participates in scoping logic)
    ///
    /// # Note
    /// This constant is **expressive only** and does not affect runtime behavior.
    /// The actual scoping is determined by `tenant_col()` returning `None`.
    const IS_UNRESTRICTED: bool = false;

    /// Returns the column that stores the tenant identifier.
    /// Return `None` for global entities that don't have tenant scoping.
    fn tenant_col() -> Option<Self::Column>;

    /// Returns the column that stores the primary resource identifier.
    /// Typically this is the primary key column (e.g., `Column::Id`).
    fn id_col() -> Self::Column;

    /// Returns the column that stores the resource owner identifier.
    /// Return `None` if owner-based scoping is not used.
    ///
    /// # Future Use
    /// This is reserved for future owner-based access control policies.
    /// Currently not used by the scoping logic.
    fn owner_col() -> Option<Self::Column> {
        None
    }
}
