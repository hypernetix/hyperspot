//! DE0102: No ToSchema in Contract Models
//!
//! Contract models MUST NOT have `utoipa::ToSchema` derive.
//! ToSchema is for OpenAPI documentation, which is an API layer concern.
//!
//! ## Example: Bad
//!
//! ```rust,ignore
//! // src/contract/product.rs - WRONG
//! use utoipa::ToSchema;
//!
//! #[derive(Debug, Clone, ToSchema)]  // ❌ ToSchema in contract
//! pub struct Product {
//!     pub id: ProductId,
//!     pub name: String,
//!     pub price: Money,
//! }
//! ```
//!
//! ## Example: Good
//!
//! ```rust,ignore
//! // src/contract/product.rs - CORRECT
//! #[derive(Debug, Clone)]  // ✅ No ToSchema
//! pub struct Product {
//!     pub id: ProductId,
//!     pub name: String,
//!     pub price: Money,
//! }
//!
//! // src/api/rest/dto.rs - CORRECT
//! use utoipa::ToSchema;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]  // ✅ ToSchema in DTO
//! pub struct ProductDto {
//!     pub id: String,
//!     pub name: String,
//!     pub price: f64,
//! }
//! ```

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LintContext};
use rustc_span::symbol::Symbol;

use crate::utils::{get_item_name, is_in_contract_module};

rustc_session::declare_lint! {
    /// DE0102: Contract models should not have ToSchema derive
    ///
    /// ToSchema is for OpenAPI documentation, which is an API layer concern.
    /// Contract models should remain transport-agnostic.
    pub DE0102_NO_TOSCHEMA_IN_CONTRACT,
    Deny,
    "contract models should not have ToSchema derive (DE0102)"
}

/// Check for ToSchema derive on a struct or enum in contract module
pub fn check<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    // Check if we're in a contract module
    if !is_in_contract_module(cx, item.owner_id.def_id) {
        return;
    }

    let item_name = get_item_name(cx, item);
    let attrs = cx.tcx.hir_attrs(item.hir_id());

    for attr in attrs {
        if attr.has_name(Symbol::intern("derive")) {
            if let Some(list) = attr.meta_item_list() {
                for meta in list {
                    if let Some(ident) = meta.ident() {
                        let name = ident.name.as_str();
                        if name == "ToSchema" {
                            cx.span_lint(
                                DE0102_NO_TOSCHEMA_IN_CONTRACT,
                                attr.span(),
                                |diag| {
                                    diag.primary_message(format!(
                                        "contract type `{}` should not derive `ToSchema` (DE0102)",
                                        item_name
                                    ));
                                    diag.help("ToSchema is an OpenAPI concern; use DTOs in api/rest/dto.rs instead");
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}
