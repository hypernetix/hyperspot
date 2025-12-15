//! # Hyperspot Dylint Linters
//!
//! This crate provides dylint lints for validating Hyperspot architectural guidelines.
//!
//! ## Linter Categories
//!
//! - **DE01**: Contract Layer - Transport-agnostic contract design
//! - **DE02**: API Layer - REST API implementation patterns
//! - **DE08**: REST API Conventions - API versioning and best practices
//!
//! ## Usage
//!
//! Run with cargo-dylint:
//! ```bash
//! cargo +nightly dylint --lib-path <path-to-lib> -p <module>
//! ```

#![feature(rustc_private)]
#![warn(unused_extern_crates)]

// Required for dylint
#[allow(unused_extern_crates)]
extern crate rustc_driver;

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

use rustc_hir::{Expr, Item};
use rustc_lint::{LateContext, LateLintPass, LintStore};
use rustc_session::Session;

// Linter category modules
pub mod de01_contract_layer;
pub mod de02_api_layer;
pub mod de08_rest_api_conventions;
pub mod utils;

// Re-export all lints
pub use de01_contract_layer::{
    DE0103_NO_HTTP_TYPES_IN_CONTRACT, DE0101_NO_SERDE_IN_CONTRACT, DE0102_NO_TOSCHEMA_IN_CONTRACT,
};
pub use de02_api_layer::{
    DE0203_DTOS_MUST_HAVE_SERDE_DERIVES, DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE, DE0202_DTOS_NOT_REFERENCED_OUTSIDE_API,
    DE0201_DTOS_ONLY_IN_API_REST,
};
pub use de08_rest_api_conventions::DE0801_API_ENDPOINT_MUST_HAVE_VERSION;

// Export dylint version
#[doc(hidden)]
#[unsafe(no_mangle)]
pub extern "C" fn dylint_version() -> *mut std::os::raw::c_char {
    std::ffi::CString::new("0.1.0").unwrap().into_raw()
}

// =============================================================================
// Register all lints
// =============================================================================

#[unsafe(no_mangle)]
pub fn register_lints(_sess: &Session, lint_store: &mut LintStore) {
    // Register all lints
    lint_store.register_lints(&[
        // DE01: Contract Layer
        DE0101_NO_SERDE_IN_CONTRACT,
        DE0102_NO_TOSCHEMA_IN_CONTRACT,
        DE0103_NO_HTTP_TYPES_IN_CONTRACT,
        // DE02: API Layer
        DE0201_DTOS_ONLY_IN_API_REST,
        DE0202_DTOS_NOT_REFERENCED_OUTSIDE_API,
        DE0203_DTOS_MUST_HAVE_SERDE_DERIVES,
        DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE,
        // DE08: REST API Conventions
        DE0801_API_ENDPOINT_MUST_HAVE_VERSION,
    ]);

    // Register the lint passes
    lint_store.register_late_pass(|_| Box::new(HyperspotLints));
}

// =============================================================================
// Combined Lint Pass (Late)
// =============================================================================

/// Combined late lint pass for Hyperspot lints
struct HyperspotLints;

rustc_session::impl_lint_pass!(HyperspotLints => [
    // DE01: Contract Layer
    DE0101_NO_SERDE_IN_CONTRACT,
    DE0102_NO_TOSCHEMA_IN_CONTRACT,
    DE0103_NO_HTTP_TYPES_IN_CONTRACT,
    // DE02: API Layer
    DE0201_DTOS_ONLY_IN_API_REST,
    DE0202_DTOS_NOT_REFERENCED_OUTSIDE_API,
    DE0203_DTOS_MUST_HAVE_SERDE_DERIVES,
    DE0204_DTOS_MUST_HAVE_TOSCHEMA_DERIVE,
    // DE08: REST API Conventions
    DE0801_API_ENDPOINT_MUST_HAVE_VERSION
]);

impl<'tcx> LateLintPass<'tcx> for HyperspotLints {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // DE01: Contract Layer checks
        de01_contract_layer::de0101_no_serde_in_contract::check(cx, item);
        de01_contract_layer::de0102_no_toschema_in_contract::check(cx, item);
        de01_contract_layer::de0103_no_http_types_in_contract::check(cx, item);

        // DE02: API Layer checks
        de02_api_layer::de0201_dtos_only_in_api_rest::check(cx, item);
        de02_api_layer::de0202_dtos_not_referenced_outside_api::check(cx, item);
        de02_api_layer::de0203_dtos_must_have_serde_derives::check(cx, item);
        de02_api_layer::de0204_dtos_must_have_toschema_derive::check(cx, item);
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        // DE08: REST API Conventions checks
        de08_rest_api_conventions::de0801_api_endpoint_version::check_expr(cx, expr);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
    }
}
