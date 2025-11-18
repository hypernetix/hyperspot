// Proc-macro crate for modkit-db secure ORM derives
//
//! # modkit-db-macros
//!
//! Procedural macros for the `modkit-db` secure ORM layer.
//!
//! ## `#[derive(Scopable)]`
//!
//! Automatically implements `ScopableEntity` for a SeaORM entity based on attributes.
//!
//! **IMPORTANT**: All four scope dimensions must be explicitly specified. No implicit defaults.
//!
//! ### Example
//!
//! ```ignore
//! use sea_orm::entity::prelude::*;
//! use modkit_db::secure::Scopable;
//!
//! #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
//! #[sea_orm(table_name = "users")]
//! #[secure(
//!     tenant_col = "tenant_id",
//!     resource_col = "id",
//!     no_owner,
//!     no_type
//! )]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: Uuid,
//!     pub tenant_id: Uuid,
//!     pub email: String,
//! }
//! ```
//!
//! ### Attributes
//!
//! Each scope dimension requires exactly one declaration:
//! - **Tenant**: `tenant_col = "column_name"` OR `no_tenant`
//! - **Resource**: `resource_col = "column_name"` OR `no_resource`
//! - **Owner**: `owner_col = "column_name"` OR `no_owner`
//! - **Type**: `type_col = "column_name"` OR `no_type`
//! - **Unrestricted**: `unrestricted` (forbids all other attributes)

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod odata_filterable;
mod scopable;

/// Derive macro for implementing `ScopableEntity`.
///
/// Place this on your SeaORM Model struct along with `#[secure(...)]` attributes.
///
/// # Attributes
///
/// **All four scope dimensions must be explicitly specified:**
///
/// - `tenant_col = "column_name"` OR `no_tenant` - Tenant isolation column
/// - `resource_col = "column_name"` OR `no_resource` - Primary resource ID column
/// - `owner_col = "column_name"` OR `no_owner` - Owner-based filtering column
/// - `type_col = "column_name"` OR `no_type` - Type-based filtering column
/// - `unrestricted` - Mark as global entity (forbids all other attributes)
///
/// # Example
///
/// ```ignore
/// #[derive(DeriveEntityModel, Scopable)]
/// #[sea_orm(table_name = "users")]
/// #[secure(
///     tenant_col = "tenant_id",
///     resource_col = "id",
///     no_owner,
///     no_type
/// )]
/// pub struct Model {
///     #[sea_orm(primary_key)]
///     pub id: Uuid,
///     pub tenant_id: Uuid,
///     pub email: String,
/// }
/// ```
///
/// # Global Entities
///
/// For entities that are not tenant-scoped (global lookup tables, system config, etc.),
/// use the `unrestricted` flag:
///
/// ```ignore
/// #[derive(DeriveEntityModel, Scopable)]
/// #[sea_orm(table_name = "system_config")]
/// #[secure(unrestricted)]
/// pub struct Model {
///     #[sea_orm(primary_key)]
///     pub id: Uuid,
///     pub key: String,
///     pub value: String,
/// }
/// ```
#[proc_macro_derive(Scopable, attributes(secure))]
#[proc_macro_error]
pub fn derive_scopable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    scopable::expand_derive_scopable(input).into()
}

/// Derive macro for implementing type-safe OData filtering on DTOs.
///
/// This macro generates a FilterField enum and implementation for a DTO struct,
/// allowing type-safe field references in OData filter expressions.
///
/// # Attributes
///
/// Fields can be marked as filterable using `#[odata(filter(kind = "..."))]`:
///
/// - `kind`: The logical field type (String, I64, F64, Bool, Uuid, DateTimeUtc, Date, Time, Decimal)
///
/// # Example
///
/// ```ignore
/// use modkit_db_macros::ODataFilterable;
///
/// #[derive(ODataFilterable)]
/// pub struct UserDto {
///     #[odata(filter(kind = "Uuid"))]
///     pub id: uuid::Uuid,
///     
///     #[odata(filter(kind = "String"))]
///     pub email: String,
///     
///     #[odata(filter(kind = "DateTimeUtc"))]
///     pub created_at: chrono::DateTime<chrono::Utc>,
///     
///     // This field is not filterable (no attribute)
///     pub internal_data: String,
/// }
/// ```
///
/// This generates:
///
/// ```ignore
/// #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
/// pub enum UserDtoFilterField {
///     Id,
///     Email,
///     CreatedAt,
/// }
///
/// impl FilterField for UserDtoFilterField {
///     const FIELDS: &'static [Self] = &[
///         UserDtoFilterField::Id,
///         UserDtoFilterField::Email,
///         UserDtoFilterField::CreatedAt,
///     ];
///
///     fn name(&self) -> &'static str {
///         match self {
///             UserDtoFilterField::Id => "id",
///             UserDtoFilterField::Email => "email",
///             UserDtoFilterField::CreatedAt => "created_at",
///         }
///     }
///
///     fn kind(&self) -> FieldKind {
///         match self {
///             UserDtoFilterField::Id => FieldKind::Uuid,
///             UserDtoFilterField::Email => FieldKind::String,
///             UserDtoFilterField::CreatedAt => FieldKind::DateTimeUtc,
///         }
///     }
/// }
/// ```
#[proc_macro_derive(ODataFilterable, attributes(odata))]
#[proc_macro_error]
pub fn derive_odata_filterable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    odata_filterable::expand_derive_odata_filterable(input).into()
}
