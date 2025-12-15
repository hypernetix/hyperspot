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

    // Use source text scanning for reliability with dylint
    let source_map = cx.sess().source_map();
    let item_span = item.span;
    let source_file = source_map.lookup_source_file(item_span.lo());

    if let Some(src) = source_file.src.as_ref() {
        let file_start_pos = source_file.start_pos;
        let item_byte_pos = item_span.lo();
        let item_offset = (item_byte_pos - file_start_pos).0 as usize;
        let lookback_start = item_offset.saturating_sub(500);
        let src_str: &str = src.as_ref();

        if let Some(text) = src_str.get(lookback_start..item_offset.min(src_str.len())) {
            // Find the closest derive attribute before this item
            if let Some(derive_pos) = text.rfind("#[derive(") {
                let derive_end = text[derive_pos..].find(")]").map(|p| derive_pos + p + 2).unwrap_or(text.len());
                let derive_text = &text[derive_pos..derive_end];

                if derive_text.contains("Serialize") {
                    cx.span_lint(DE0101_NO_SERDE_IN_CONTRACT, item.span, |diag| {
                        diag.primary_message(format!(
                            "contract type `{}` should not derive `Serialize` (DE0101)",
                            item_name
                        ));
                        diag.help("remove serde derives from contract models; use DTOs in the API layer");
                    });
                }
                if derive_text.contains("Deserialize") {
                    cx.span_lint(DE0101_NO_SERDE_IN_CONTRACT, item.span, |diag| {
                        diag.primary_message(format!(
                            "contract type `{}` should not derive `Deserialize` (DE0101)",
                            item_name
                        ));
                        diag.help("remove serde derives from contract models; use DTOs in the API layer");
                    });
                }
            }
        }
    }
}
