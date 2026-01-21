//! # modkit-odata-macros
//!
//! Procedural macros for `OData` protocol types and schemas.
//!
//! This crate provides derive macros for generating OData-related implementations:
//! - `ODataFilterable`: Generate `FilterField` enum for server-side type-safe filtering
//! - `ODataSchema`: Generate `Schema` trait impl for client-side query building
//!
//! These macros generate code referencing `modkit-odata` types and are independent
//! of database or HTTP framework concerns.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;
use syn::{DeriveInput, parse_macro_input};

mod odata_filterable;
mod odata_schema;

/// Derive macro for implementing type-safe `OData` filtering on DTOs.
///
/// Generates a `FilterField` enum implementing `modkit_odata::filter::FilterField`.
/// This enables type-safe field references in `OData` filter expressions.
///
/// # Example
///
/// ```ignore
/// use modkit_odata_macros::ODataFilterable;
///
/// #[derive(ODataFilterable)]
/// pub struct UserQuery {
///     #[odata(filter(kind = "Uuid"))]
///     pub id: uuid::Uuid,
///     #[odata(filter(kind = "String"))]
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(ODataFilterable, attributes(odata))]
#[proc_macro_error]
pub fn derive_odata_filterable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    odata_filterable::expand_derive_odata_filterable(input).into()
}

/// Derive macro for implementing `OData` schema for client-side query building.
///
/// Generates a `Schema` trait impl and field enum for building type-safe queries.
///
/// # Example
///
/// ```ignore
/// use modkit_odata_macros::ODataSchema;
///
/// #[derive(ODataSchema)]
/// pub struct User {
///     pub id: uuid::Uuid,
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(ODataSchema, attributes(odata))]
#[proc_macro_error]
pub fn derive_odata_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    odata_schema::expand_derive_odata_schema(&input).into()
}
