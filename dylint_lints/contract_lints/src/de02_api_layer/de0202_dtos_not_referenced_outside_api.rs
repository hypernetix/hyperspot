//! DE0202: DTOs Not Referenced Outside API
//!
//! DTO types MUST NOT be referenced from contract, domain, or infra modules.
//! DTOs are API layer implementation details.
//!
//! ## Example: Bad
//!
//! ```rust,ignore
//! // src/domain/service.rs - WRONG
//! use crate::api::rest::dto::UserDto;  // ❌ Importing DTO from domain
//!
//! pub struct UserService;
//!
//! impl UserService {
//!     pub fn get_user(&self, id: String) -> UserDto {  // ❌ Returning DTO
//!         // Domain logic should return domain types
//!     }
//! }
//!
//! // src/contract/user.rs - WRONG
//! use crate::api::rest::dto::CreateUserDto;  // ❌ Importing DTO from contract
//! ```
//!
//! ## Example: Good
//!
//! ```rust,ignore
//! // src/domain/service.rs - CORRECT
//! use crate::contract::User;  // ✅ Use contract types
//!
//! pub struct UserService;
//!
//! impl UserService {
//!     pub fn get_user(&self, id: UserId) -> Result<User> {  // ✅ Return domain type
//!         // Domain logic returns contract/domain types
//!     }
//! }
//!
//! // src/api/rest/handlers.rs - CORRECT
//! use crate::api::rest::dto::{UserDto, CreateUserDto};  // ✅ DTOs used in API layer
//! use crate::contract::User;
//!
//! pub async fn get_user(id: String) -> Result<Json<UserDto>> {
//!     let user = service.get_user(id)?;
//!     Ok(Json(UserDto::from(user)))  // ✅ Convert contract -> DTO at boundary
//! }
//! ```

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LintContext};

use crate::utils::{
    is_in_contract_module, is_in_domain_module, is_in_infra_module, path_to_string,
};

rustc_session::declare_lint! {
    /// DE0202: DTOs not referenced outside API
    ///
    /// DTO types must not be imported by contract, domain, or infra modules.
    /// DTOs are API layer implementation details.
    pub DE0202_DTOS_NOT_REFERENCED_OUTSIDE_API,
    Deny,
    "DTO types should not be imported outside of api layer (DE0202)"
}

/// Check if a use statement imports DTOs from outside allowed modules
pub fn check<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
    // Only check use statements
    let ItemKind::Use(path, _) = &item.kind else {
        return;
    };

    // Check if we're in a forbidden module (contract, domain, infra)
    let def_id = item.owner_id.def_id;
    let in_forbidden = is_in_contract_module(cx, def_id)
        || is_in_domain_module(cx, def_id)
        || is_in_infra_module(cx, def_id);

    if !in_forbidden {
        return;
    }

    // Check if the import path references api::rest::dto
    let path_str = path_to_string(path);
    if path_str.contains("api::rest::dto") || path_str.contains("api::rest") {
        // Check if importing a DTO type
        let segments: Vec<&str> = path_str.split("::").collect();
        if let Some(last) = segments.last() {
            let is_dto = last.ends_with("Dto")
                || last.ends_with("Request")
                || last.ends_with("Response")
                || last.ends_with("Query");

            if is_dto {
                let module_type = if is_in_contract_module(cx, def_id) {
                    "contract"
                } else if is_in_domain_module(cx, def_id) {
                    "domain"
                } else {
                    "infra"
                };

                cx.span_lint(DE0202_DTOS_NOT_REFERENCED_OUTSIDE_API, item.span, |diag| {
                    diag.primary_message(format!(
                        "{} module imports DTO type `{}` from api layer (DE0202)",
                        module_type, last
                    ));
                    diag.help(
                        "DTOs are API layer details; use contract models or domain types instead",
                    );
                });
            }
        }
    }
}
