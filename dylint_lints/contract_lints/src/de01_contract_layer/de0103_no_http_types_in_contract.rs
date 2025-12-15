//! DE0103: No HTTP Types in Contract
//!
//! Contract modules MUST NOT reference HTTP-specific types (StatusCode, Headers, etc.).
//! Contract is transport-agnostic; HTTP is one possible transport.
//!
//! ## Example: Bad
//!
//! ```rust,ignore
//! // src/contract/order.rs - WRONG
//! use http::StatusCode;  // ❌ HTTP type in contract
//! use axum::http::HeaderMap;  // ❌ HTTP type in contract
//!
//! pub struct OrderResult {
//!     pub order: Order,
//!     pub status: StatusCode,  // ❌ HTTP-specific
//! }
//! ```
//!
//! ## Example: Good
//!
//! ```rust,ignore
//! // src/contract/order.rs - CORRECT
//! #[derive(Debug, Clone)]  // ✅ Transport-agnostic
//! pub struct OrderResult {
//!     pub order: Order,
//!     pub status: OrderStatus,  // ✅ Domain type
//! }
//!
//! #[derive(Debug, Clone)]
//! pub enum OrderStatus {
//!     Pending,
//!     Confirmed,
//!     Shipped,
//!     Delivered,
//! }
//!
//! // src/api/rest/handlers.rs - CORRECT
//! use http::StatusCode;  // ✅ HTTP types allowed in API layer
//!
//! pub async fn create_order(body: Json<CreateOrderDto>) -> (StatusCode, Json<OrderDto>) {
//!     // HTTP layer converts between HTTP and domain types
//!     (StatusCode::CREATED, Json(order_dto))
//! }
//! ```

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LintContext};

use crate::utils::{is_in_contract_module, path_to_string};

rustc_session::declare_lint! {
    /// DE0103: Contract modules should not reference HTTP types
    ///
    /// Contract modules should be transport-agnostic. HTTP is just one possible
    /// transport layer.
    pub DE0103_NO_HTTP_TYPES_IN_CONTRACT,
    Deny,
    "contract modules should not reference HTTP-specific types (DE0103)"
}

/// List of HTTP-related modules/types to flag
const HTTP_TYPE_PATTERNS: &[&str] = &[
    "axum::http",
    "http::StatusCode",
    "http::Method",
    "http::HeaderMap",
    "http::HeaderName",
    "http::HeaderValue",
    "http::Request",
    "http::Response",
    "hyper::StatusCode",
    "hyper::Method",
];

/// Check for HTTP type imports in contract module
pub fn check<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
    // Only check use statements
    let ItemKind::Use(path, _) = &item.kind else {
        return;
    };

    // Check if we're in a contract module
    if !is_in_contract_module(cx, item.owner_id.def_id) {
        return;
    }

    let path_str = path_to_string(path);
    for pattern in HTTP_TYPE_PATTERNS {
        if path_str.contains(pattern) {
            cx.span_lint(DE0103_NO_HTTP_TYPES_IN_CONTRACT, item.span, |diag| {
                diag.primary_message(format!(
                    "contract module imports HTTP type `{}` (DE0103)",
                    path_str
                ));
                diag.help(
                    "contract modules should be transport-agnostic; move HTTP types to api/rest/",
                );
            });
            break;
        }
    }
}
