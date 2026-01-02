#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{Item, ItemKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::{FileName, RealFileName, Span};

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
        let ItemKind::Trait(_trait_data) = &item.kind else {
            return;
        };

        // Only apply this lint to *-sdk crates
        if !is_in_sdk_crate(cx, item.span) {
            return;
        }

        // Get the trait name from the item
        let trait_name = cx.sess().source_map().span_to_snippet(item.span);
        let trait_name_str = if let Ok(snippet) = trait_name {
            // Extract trait name from snippet like "pub trait MyTrait"
            snippet
                .split_whitespace()
                .find(|word| !word.starts_with("pub") && word != &"trait")
                .and_then(|name| {
                    // Handle cases like "MyTrait:" or "MyTrait<"
                    name.split(|c: char| c == ':' || c == '<' || c == '{')
                        .next()
                        .map(|s| s.to_string())
                })
                .unwrap_or_default()
        } else {
            return;
        };

        if trait_name_str.is_empty() {
            return;
        }

        // Check if trait ends with "PluginApi" or just "Api" (but not already "Client")
        if trait_name_str.ends_with("PluginApi") {
            emit_lint(cx, item.span, &trait_name_str, "PluginApi", "PluginClient");
        } else if trait_name_str.ends_with("Api") && !trait_name_str.ends_with("Client") {
            // Only lint if it looks like a plugin/client pattern
            // Check if name contains common plugin/client indicators
            let name_lower = trait_name_str.to_lowercase();
            if name_lower.contains("plugin") || name_lower.contains("client") {
                let suggested_suffix = if name_lower.contains("plugin") {
                    "PluginClient"
                } else {
                    "Client"
                };
                emit_lint(cx, item.span, &trait_name_str, "Api", suggested_suffix);
            }
        }
    }
}

fn is_in_sdk_crate(cx: &EarlyContext<'_>, span: Span) -> bool {
    if let Some(crate_name) = cx.sess().opts.crate_name.as_deref() {
        // Cargo normalizes dashes to underscores for `--crate-name`.
        if crate_name.ends_with("-sdk") || crate_name.ends_with("_sdk") {
            return true;
        }
    }

    let file_name = cx.sess().source_map().span_to_filename(span);
    let file_name = match file_name {
        FileName::Real(real) => match real {
            RealFileName::LocalPath(path) => path.to_string_lossy().to_string(),
            RealFileName::Remapped {
                local_path,
                virtual_name,
            } => local_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| virtual_name.to_string_lossy().to_string()),
        },
        other => format!("{other:?}"),
    };

    // Also support filtering by path, so the lint applies to any file under a `*-sdk` folder.
    file_name.contains("-sdk/") || file_name.contains("-sdk\\")
}

fn emit_lint(
    cx: &EarlyContext<'_>,
    span: Span,
    trait_name: &str,
    wrong_suffix: &str,
    suggested_suffix: &str,
) {
    let suggestion = if trait_name.ends_with(wrong_suffix) {
        trait_name
            .strip_suffix(wrong_suffix)
            .map(|base| format!("{base}{suggested_suffix}"))
            .unwrap_or_else(|| format!("{trait_name}Client"))
    } else {
        format!("{trait_name}Client")
    };

    cx.span_lint(DE0503_PLUGIN_CLIENT_SUFFIX, span, |diag| {
        diag.primary_message(format!(
            "plugin client trait `{trait_name}` should use `*PluginClient` suffix, not `*{wrong_suffix}` (DE0503)"
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
