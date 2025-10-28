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
//! ### Example
//!
//! ```ignore
//! use sea_orm::entity::prelude::*;
//! use modkit_db::secure::Scopable;
//!
//! #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
//! #[sea_orm(table_name = "users")]
//! #[secure(tenant_col = "tenant_id")]
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
//! - `tenant_col = "column_name"` - Optional. The column containing tenant ID.
//! - `resource_col = "column_name"` - Optional. The primary resource ID column (defaults to "id").
//! - `owner_col = "column_name"` - Optional. The column containing owner ID.
//! - `entity = "EntityName"` - Optional. Override the entity type name (defaults to "Entity").

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod scopable;

/// Derive macro for implementing `ScopableEntity`.
///
/// Place this on your SeaORM Model struct along with `#[secure(...)]` attributes.
///
/// # Attributes
///
/// - `#[secure(tenant_col = "tenant_id")]` - Specify tenant column name
/// - `#[secure(resource_col = "custom_id")]` - Override default resource ID column
/// - `#[secure(owner_col = "owner_id")]` - Specify owner column name
/// - `#[secure(entity = "CustomEntity")]` - Override entity type name
/// - `#[secure(unrestricted)]` - Mark entity as unrestricted (global entity)
///
/// # Example
///
/// ```ignore
/// #[derive(DeriveEntityModel, Scopable)]
/// #[sea_orm(table_name = "users")]
/// #[secure(tenant_col = "tenant_id")]
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
