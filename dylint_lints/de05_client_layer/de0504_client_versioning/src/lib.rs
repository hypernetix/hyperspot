#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::Span;

dylint_linting::declare_early_lint! {
    /// ### What it does
    ///
    /// Enforces that Client and PluginClient traits in non-system modules have version suffixes (V1, V2, etc.).
    ///
    /// # Why is this bad?
    ///
    /// Non-system modules require explicit versioning for their public API contracts to enable
    /// parallel versions and clear upgrade paths. System modules are exempt because they follow
    /// different versioning rules managed at the platform level.
    ///
    /// # Scope
    /// - **Applies to**: All SDK crates in `modules/*` (except `modules/system/*`) and `examples/*`
    /// - **Does NOT apply to**: System modules only (`modules/system/*`)
    ///
    /// # Example
    /// ```rust,ignore
    /// // Bad (in modules/simple_user_settings or examples/*)
    /// pub trait UsersInfoClient: Send + Sync {
    ///     async fn get_user(&self) -> Result<User, Error>;
    /// }
    ///
    /// // Good (in modules/simple_user_settings or examples/*)
    /// pub trait UsersInfoClientV1: Send + Sync {
    ///     async fn get_user(&self) -> Result<User, Error>;
    /// }
    ///
    /// // OK (in modules/system/* - exempt from versioning)
    /// pub trait TypesRegistryClient: Send + Sync {
    ///     async fn register(&self) -> Result<(), Error>;
    /// }
    /// ```
    pub DE0504_CLIENT_VERSIONING,
    Deny,
    "Client and PluginClient traits in non-system modules must have version suffixes (V1, V2, etc.) (DE0504)"
}

impl EarlyLintPass for De0504ClientVersioning {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Only check trait definitions
        let ItemKind::Trait(trait_data) = &item.kind else {
            return;
        };

        // Only apply this lint to *-sdk crates or UI test examples
        if !lint_utils::is_in_sdk_crate(cx, item.span) {
            return;
        }

        // EXEMPTION: Skip system modules (modules/system/*) from versioning requirements.
        // UI tests always run for testing purposes even if they simulate system modules.
        if is_system_module(cx, item.span) && !is_ui_test(cx, item.span) {
            return;
        }

        let trait_name = trait_data.ident.name.as_str();
        if trait_name.is_empty() {
            return;
        }

        let version = lint_utils::parse_version_suffix(trait_name);

        // Only match traits whose base name ends with "Client" to avoid false positives
        // on helper traits like ClientEventHandler, ClientConfiguration, etc.
        if !version.base.ends_with("Client") {
            return;
        }

        // If it has a valid version suffix (V1, V2, etc.), it's fine
        if version.has_valid_version() {
            return;
        }

        emit_lint(cx, item.span, trait_name, &version);
    }
}

fn is_ui_test(cx: &EarlyContext<'_>, span: Span) -> bool {
    let Some(file_path) = lint_utils::filename_str(cx.sess().source_map(), span) else {
        return false;
    };
    lint_utils::is_temp_path(&file_path)
}

/// Checks if the file is part of a system module (modules/system/*).
fn is_system_module(cx: &EarlyContext<'_>, span: Span) -> bool {
    let Some(file_path) = lint_utils::filename_str(cx.sess().source_map(), span) else {
        return false;
    };
    file_path.contains("modules/system/") || file_path.contains("modules\\system\\")
}

fn emit_lint(cx: &EarlyContext<'_>, span: Span, trait_name: &str, version: &lint_utils::VersionParts<'_>) {
    let suggestion = if version.has_malformed_version() && !version.malformed_digits.starts_with('0') {
        // Trailing digits without V prefix: suggest inserting V
        // e.g., UsersInfoClient2 -> UsersInfoClientV2
        format!("{}V{}", version.base, version.malformed_digits)
    } else {
        // No version, bare V, V0, or leading-zero digits: suggest appending V1 to base
        format!("{}V1", version.base)
    };

    cx.span_lint(DE0504_CLIENT_VERSIONING, span, |diag| {
        diag.primary_message(format!(
            "Client trait `{trait_name}` in non-system module must have a version suffix (DE0504)"
        ));
        diag.help(format!(
            "rename trait to `{suggestion}` to indicate API version"
        ));
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui_examples() {
        dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
    }

    #[test]
    fn test_comment_annotations_match_stderr() {
        let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui");
        lint_utils::test_comment_annotations_match_stderr(
            &ui_dir,
            "DE0504",
            "Client trait",
        );
    }

    // NOTE: Positive-case testing (lint fires on bad code) is covered by UI tests in ui/
    // (non_system_missing_version.rs, invalid_version_suffix.rs, generic_parameters.rs).
    // Integration tests in tests/system_module_exemption.rs verify the system module
    // exemption works with real crate paths, which cannot be tested through UI tests.

    // --- Unit tests for lint_utils::parse_version_suffix ---
    // Placed here because lint_utils can't run unit tests directly (rustc_private linking).

    fn assert_version(
        name: &str,
        expected_base: &str,
        expected_suffix: &str,
        expected_malformed: &str,
    ) {
        let v = lint_utils::parse_version_suffix(name);
        assert_eq!(
            v.base, expected_base,
            "parse_version_suffix({name:?}): base mismatch"
        );
        assert_eq!(
            v.version_suffix, expected_suffix,
            "parse_version_suffix({name:?}): version_suffix mismatch"
        );
        assert_eq!(
            v.malformed_digits, expected_malformed,
            "parse_version_suffix({name:?}): malformed_digits mismatch"
        );
    }

    #[test]
    fn test_parse_version_suffix_empty_and_single_char() {
        assert_version("", "", "", "");
        // Single "V" is just a name, not a bare-V suffix (requires len > 1)
        assert_version("V", "V", "", "");
        assert_version("A", "A", "", "");
        assert_version("1", "", "", "1");
    }

    #[test]
    fn test_parse_version_suffix_valid_versions() {
        assert_version("FooClientV1", "FooClient", "V1", "");
        assert_version("FooClientV2", "FooClient", "V2", "");
        assert_version("FooClientV10", "FooClient", "V10", "");
        assert_version("FooClientV99", "FooClient", "V99", "");
        assert_version("V1", "", "V1", "");
    }

    #[test]
    fn test_parse_version_suffix_rejected_versions() {
        // V0: version zero is invalid
        assert_version("FooClientV0", "FooClient", "", "");
        // V00: leading zero
        assert_version("FooClientV00", "FooClient", "", "");
        // V01: leading zero
        assert_version("FooClientV01", "FooClient", "", "");
        // V0 standalone
        assert_version("V0", "", "", "");
    }

    #[test]
    fn test_parse_version_suffix_bare_v() {
        assert_version("FooClientV", "FooClient", "", "");
        assert_version("VV", "V", "", "");
    }

    #[test]
    fn test_parse_version_suffix_malformed_digits() {
        assert_version("FooClient2", "FooClient", "", "2");
        assert_version("FooClient123", "FooClient", "", "123");
        assert_version("Client1", "Client", "", "1");
    }

    #[test]
    fn test_parse_version_suffix_no_suffix() {
        assert_version("FooClient", "FooClient", "", "");
        assert_version("ThrPluginApi", "ThrPluginApi", "", "");
        assert_version("SomeTraitName", "SomeTraitName", "", "");
    }

    #[test]
    fn test_version_parts_helpers() {
        let v = lint_utils::parse_version_suffix("FooClientV1");
        assert!(v.has_valid_version());
        assert!(!v.has_malformed_version());

        let v = lint_utils::parse_version_suffix("FooClient2");
        assert!(!v.has_valid_version());
        assert!(v.has_malformed_version());

        let v = lint_utils::parse_version_suffix("FooClient");
        assert!(!v.has_valid_version());
        assert!(!v.has_malformed_version());

        let v = lint_utils::parse_version_suffix("FooClientV0");
        assert!(!v.has_valid_version());
        assert!(!v.has_malformed_version());

        let v = lint_utils::parse_version_suffix("FooClientV");
        assert!(!v.has_valid_version());
        assert!(!v.has_malformed_version());
    }
}
