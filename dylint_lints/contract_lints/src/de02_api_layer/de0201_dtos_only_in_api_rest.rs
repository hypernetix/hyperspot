//! DE0201: DTOs Only in API Rest Folder
//!
//! Types with `*Dto` suffixes MUST be defined only in `*/api/rest/*.rs` files.
//! DTOs are transport-specific and should not leak into domain or contract.
//!
//! NOTE: This lint only applies to module crates in the `modules/` folder.
//! Library crates (libs/) are exempt as they may legitimately define shared response types.
//!
//! ## Example: Bad
//!
//! ```rust,ignore
//! // modules/users/src/domain/user.rs - WRONG
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! pub struct UserDto {  // ❌ DTO in domain layer
//!     pub id: String,
//!     pub name: String,
//! }
//!
//! // modules/users/src/contract/responses.rs - WRONG
//! pub struct UserResponseDto {  // ❌ DTO in contract layer
//!     pub user: User,
//! }
//! ```
//!
//! ## Example: Good
//!
//! ```rust,ignore
//! // modules/users/src/api/rest/dto.rs - CORRECT
//! use serde::{Deserialize, Serialize};
//! use utoipa::ToSchema;
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
//! pub struct UserDto {  // ✅ DTO in api/rest folder
//!     pub id: String,
//!     pub name: String,
//!     pub email: String,
//! }
//!
//! #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
//! pub struct CreateUserDto {  // ✅ DTO in api/rest folder
//!     pub name: String,
//!     pub email: String,
//! }
//! ```

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LintContext};

use crate::utils::{get_item_name, is_in_api_rest_folder, is_in_module_crate};

rustc_session::declare_lint! {
    /// DE0201: DTOs only in API rest folder
    ///
    /// Types with DTO suffixes (Dto, Request, Response, Query) must be defined
    /// only in */api/rest/* files within module crates.
    pub DE0201_DTOS_ONLY_IN_API_REST,
    Deny,
    "DTO types should only be defined in */api/rest/* files (DE0201)"
}

/// DTO type name suffixes to check (case-insensitive)
const DTO_SUFFIXES: &[&str] = &["dto"];

/// Check if a DTO type is defined outside of api/rest folder
pub fn check<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
    // Only check structs and enums
    if !matches!(item.kind, ItemKind::Struct(..) | ItemKind::Enum(..)) {
        return;
    }

    let item_name = get_item_name(cx, item);
    let item_name_lower = item_name.to_lowercase();

    // Check if the type name ends with a DTO suffix (case-insensitive)
    let is_dto = DTO_SUFFIXES
        .iter()
        .any(|suffix| item_name_lower.ends_with(suffix));

    if !is_dto {
        return;
    }

    // Skip if not in a module crate (exempt library crates)
    if !is_in_module_crate(cx, item.owner_id.def_id) {
        return;
    }

    // If it's a DTO but NOT in api/rest folder, flag it
    if !is_in_api_rest_folder(cx, item.owner_id.def_id) {
        cx.span_lint(DE0201_DTOS_ONLY_IN_API_REST, item.span, |diag| {
            diag.primary_message(format!(
                "DTO type `{}` is defined outside of api/rest folder (DE0201)",
                item_name
            ));
            diag.help("move DTO types to src/api/rest/dto.rs");
        });
    }
}
