//! DE0101: No Serde in Contract Models
//!
//! Contract models MUST NOT have `serde::Serialize` or `serde::Deserialize` derives.
//! Serde is a serialization concern that belongs in the API layer (DTOs).
//!
//! ## Example: Bad
//!
//! ```rust,ignore
//! // src/contract/user.rs - WRONG
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]  // ❌ Serde in contract
//! pub struct User {
//!     pub id: UserId,
//!     pub name: String,
//!     pub email: Email,
//! }
//! ```
//!
//! ## Example: Good
//!
//! ```rust,ignore
//! // src/contract/user.rs - CORRECT
//! #[derive(Debug, Clone)]  // ✅ No serde derives
//! pub struct User {
//!     pub id: UserId,
//!     pub name: String,
//!     pub email: Email,
//! }
//!
//! // src/api/rest/dto.rs - CORRECT
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]  // ✅ Serde in DTO layer
//! pub struct UserDto {
//!     pub id: String,
//!     pub name: String,
//!     pub email: String,
//! }
//! ```

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LintContext};
use rustc_span::symbol::Symbol;

use crate::utils::{get_item_name, is_in_contract_module};

rustc_session::declare_lint! {
    /// DE0101: Contract models should not have serde derives
    ///
    /// Contract models should be transport-agnostic. Serde is a serialization
    /// concern that belongs in the API layer (DTOs).
    pub DE0101_NO_SERDE_IN_CONTRACT,
    Deny,
    "contract models should not have serde derives (DE0101)"
}

/// Check for serde derives on a struct or enum in contract module
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
                        if name == "Serialize" || name == "Deserialize" {
                            cx.span_lint(
                                DE0101_NO_SERDE_IN_CONTRACT,
                                attr.span(),
                                |diag| {
                                    diag.primary_message(format!(
                                        "contract type `{}` should not derive `{}` (DE0101)",
                                        item_name, name
                                    ));
                                    diag.help("remove serde derives from contract models; use DTOs in the API layer for serialization");
                                },
                            );
                        }
                    }
                }
            }
        }
    }
}
