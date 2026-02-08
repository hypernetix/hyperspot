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
    /// Checks that plugin client traits use the `*Client` suffix instead of `*Api` or `*PluginApi`.
    ///
    /// # Why is this bad?
    ///
    /// Inconsistent naming makes it harder to identify client traits
    /// and violates the project's architectural conventions.
    ///
    /// # Scope
    /// This lint only applies to `*-sdk` crates where plugin client traits are defined.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Bad (in a *-sdk crate)
    /// pub trait ThrPluginApi: Send + Sync {
    ///     async fn get_root_tenant(&self) -> Result<Tenant, Error>;
    /// }
    ///
    /// // Good
    /// pub trait ThrPluginClient: Send + Sync {
    ///     async fn get_root_tenant(&self) -> Result<Tenant, Error>;
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// // Good - uses Client suffix
    /// #[async_trait]
    /// pub trait ThrPluginClient: Send + Sync {
    ///     async fn get_data(&self) -> Result<Data, Error>;
    /// }
    /// ```
    pub DE0503_PLUGIN_CLIENT_SUFFIX,
    Deny,
    "plugin client traits should use *PluginClient suffix, not *Api or *PluginApi (DE0503)"
}

impl EarlyLintPass for De0503PluginClientSuffix {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Only check trait definitions
        let ItemKind::Trait(trait_data) = &item.kind else {
            return;
        };

        // Only apply this lint to *-sdk crates
        if !lint_utils::is_in_sdk_crate(cx, item.span) {
            return;
        }

        let trait_name = trait_data.ident.name.as_str();
        if trait_name.is_empty() {
            return;
        }

        // Strip version suffix (valid or malformed) to check the base name
        let version = lint_utils::parse_version_suffix(trait_name);
        let base_name = version.base;

        // Check if base name ends with "PluginApi" or just "Api"
        if base_name.ends_with("PluginApi") {
            emit_lint(cx, item.span, trait_name, "PluginApi", "PluginClient");
        } else if base_name.ends_with("Api") {
            let base_without_api = base_name.strip_suffix("Api").unwrap_or(base_name);
            let name_lower = base_name.to_lowercase();

            let is_plugin_api = base_without_api.ends_with("Plugin")
                || name_lower.contains("plugin");

            if is_plugin_api && !base_without_api.ends_with("Client") {
                emit_lint(cx, item.span, trait_name, "Api", "PluginClient");
            } else if base_without_api.ends_with("Client") {
                // Trait like SomeClientApi — already has Client suffix, just drop Api
                emit_lint(cx, item.span, trait_name, "ClientApi", "Client");
            }
        }
    }
}

fn emit_lint(
    cx: &EarlyContext<'_>,
    span: Span,
    trait_name: &str,
    wrong_suffix: &str,
    suggested_suffix: &str,
) {
    let version = lint_utils::parse_version_suffix(trait_name);

    let suggestion = if version.base.ends_with(wrong_suffix) {
        let base = version.base.strip_suffix(wrong_suffix).unwrap();
        if version.has_valid_version() {
            format!("{base}{suggested_suffix}{}", version.version_suffix)
        } else if version.has_malformed_version() {
            // Only suggest Vn if digits don't start with 0 (e.g., ThrPluginApi2 -> ThrPluginClientV2)
            if !version.malformed_digits.starts_with('0') {
                format!("{base}{suggested_suffix}V{}", version.malformed_digits)
            } else {
                format!("{base}{suggested_suffix}")
            }
        } else {
            format!("{base}{suggested_suffix}")
        }
    } else {
        format!("{trait_name}Client")
    };

    cx.span_lint(DE0503_PLUGIN_CLIENT_SUFFIX, span, |diag| {
        diag.primary_message(format!(
            "plugin client trait `{trait_name}` should use `*{suggested_suffix}` suffix, not `*{wrong_suffix}` (DE0503)"
        ));
        diag.help(format!(
            "rename trait to `{suggestion}` to follow plugin client naming conventions"
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
            "DE0503",
            "plugin client traits should use",
        );
    }
}
