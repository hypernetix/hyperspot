//! DE0203: DTOs Must Have Serde Derives
//!
//! All DTO types MUST derive `Serialize` and `Deserialize`.
//! DTOs are for serialization; missing derives cause runtime errors.
//!
//! ## Example: Bad
//!
//! ```rust,ignore
//! // src/api/rest/dto.rs - WRONG
//! #[derive(Debug, Clone)]  // ❌ Missing serde derives
//! pub struct UserDto {
//!     pub id: Uuid,
//!     pub name: String,
//! }
//! ```
//!
//! ## Example: Good
//!
//! ```rust,ignore
//! // src/api/rest/dto.rs - CORRECT
//! #[derive(Debug, Clone, Serialize, Deserialize)]  // ✅ Has serde derives
//! pub struct UserDto {
//!     pub id: Uuid,
//!     pub name: String,
//! }
//! ```

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LintContext};

use crate::utils::{get_item_name, is_in_api_rest_folder};

rustc_session::declare_lint! {
    /// DE0203: DTOs must have serde derives
    ///
    /// DTO types must derive both Serialize and Deserialize from serde.
    /// Missing derives will cause runtime serialization errors.
    pub DE0203_DTOS_MUST_HAVE_SERDE_DERIVES,
    Deny,
    "DTO types must derive Serialize and Deserialize (DE0203)"
}

/// Check if a DTO type has the required serde derives
pub fn check<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
    // Only check structs and enums in api/rest folder
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    // Only check items in api/rest folder
    if !is_in_api_rest_folder(cx, item.owner_id.def_id) {
        return;
    }

    let item_name = get_item_name(cx, item);
    let item_name_lower = item_name.to_lowercase();

    // Check if the type name ends with a DTO suffix (case-insensitive)
    let dto_suffixes = ["dto"];
    let is_dto = dto_suffixes
        .iter()
        .any(|suffix| item_name_lower.ends_with(suffix));

    if !is_dto {
        return;
    }

    // Workaround: Since dylint seems to strip derive attributes from HIR,
    // we'll check the source text directly
    let mut has_serialize = false;
    let mut has_deserialize = false;

    // Get the source map and try to read the entire file containing the item
    let source_map = cx.sess().source_map();
    let item_span = item.span;

    // Get the source file and read the content
    let source_file = source_map.lookup_source_file(item_span.lo());
    if let Some(src) = source_file.src.as_ref() {
        // Get the item's position in the file relative to the file start
        let file_start_pos = source_file.start_pos;
        let item_byte_pos = item_span.lo();
        let item_offset = (item_byte_pos - file_start_pos).0 as usize;

        // Look back up to 1000 characters or to the start of file, whichever comes first
        let lookback_start = item_offset.saturating_sub(1000);
        // Get the text snippet as a string slice
        let src_str: &str = src.as_ref();
        if let Some(text) = src_str.get(lookback_start..item_offset.min(src_str.len())) {
            let text_lower = text.to_lowercase();
            if text_lower.contains("derive") {
                if text.contains("Serialize") {
                    has_serialize = true;
                }
                if text.contains("Deserialize") {
                    has_deserialize = true;
                }
            }
        }
    }

    // Check for missing derives
    if !has_serialize || !has_deserialize {
        cx.span_lint(DE0203_DTOS_MUST_HAVE_SERDE_DERIVES, item.span, |diag| {
            diag.primary_message(format!(
                "DTO type `{}` is missing required serde derives (DE0203)",
                item_name
            ));

            let mut missing = Vec::new();
            if !has_serialize {
                missing.push("Serialize");
            }
            if !has_deserialize {
                missing.push("Deserialize");
            }

            diag.help(format!(
                "add the following derive: #[derive({})]",
                missing.join(", ")
            ));
        });
    }
}
