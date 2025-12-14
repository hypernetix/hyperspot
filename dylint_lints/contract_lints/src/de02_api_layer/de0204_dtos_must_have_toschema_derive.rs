//! DE0204: DTOs Must Have ToSchema Derive
//!
//! All DTO types MUST derive `utoipa::ToSchema`.
//! Required for OpenAPI documentation generation.
//!
//! ## Example: Bad
//!
//! ```rust,ignore
//! // src/api/rest/dto.rs - WRONG
//! #[derive(Debug, Clone, Serialize, Deserialize)]  // ❌ Missing ToSchema
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
//! #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]  // ✅ Has ToSchema
//! pub struct UserDto {
//!     pub id: Uuid,
//!     pub name: String,
//! }
//! ```

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LintContext};

use crate::utils::{get_item_name, is_in_api_rest_folder};

rustc_session::declare_lint! {
    /// DE0204: DTOs must have ToSchema derive
    ///
    /// DTO types must derive utoipa::ToSchema for OpenAPI documentation.
    /// Missing derive will cause documentation generation to fail.
    pub DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE,
    Deny,
    "DTO types must derive utoipa::ToSchema (DE0204)"
}

/// Check if a DTO type has the required ToSchema derive
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
    let mut has_toschema = false;

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
                if text.contains("ToSchema") {
                    has_toschema = true;
                }
            }
        }
    }

    // Check for missing ToSchema derive
    if !has_toschema {
        cx.span_lint(DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE, item.span, |diag| {
            diag.primary_message(format!(
                "DTO type `{}` is missing required ToSchema derive (DE0204)",
                item_name
            ));
            diag.help("add the following derive: #[derive(utoipa::ToSchema)]");
        });
    }
}
