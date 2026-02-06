#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_span;

use rustc_ast::{visit, AttrKind, Attribute, ExprKind, Item, ItemKind, MacCall, VisibilityKind, visit::Visitor};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_session::config::CrateType;
use rustc_span::{FileName, sym};

const FORBIDDEN_MACROS: &[&str] = &["println", "eprintln", "print", "eprint", "dbg"];

dylint_linting::declare_pre_expansion_lint! {
    /// DE1301: Forbid print/debug macros in production code
    ///
    /// Disallows using the following macros:
    /// - println!
    /// - eprintln!
    /// - print!
    /// - eprint!
    /// - dbg!
    pub DE1301_NO_PRINT_MACROS,
    Deny,
    "print/debug macros are forbidden in production code (DE1301)"
}

impl EarlyLintPass for De1301NoPrintMacros {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // In pre-expansion lints, rustc does not reliably walk into bodies for us.
        // Walk the item ourselves and look for `MacCall` nodes.
        let mut v = ForbiddenMacroVisitor {
            cx,
            in_proc_macro_crate: is_proc_macro_crate(cx),
            allow_stack: Vec::new(),
        };
        v.visit_item(item);
    }
}

fn is_allowed_location(cx: &EarlyContext<'_>, span: rustc_span::Span) -> bool {
    let source_map = cx.sess().source_map();
    let file_name = source_map.span_to_filename(span);

    let Some(path_str) = (match file_name {
        FileName::Real(real_name) => real_name
            .local_path()
            .map(|p| p.to_string_lossy().to_string()),
        _ => None,
    }) else {
        return false;
    };

    // Support UI tests by allowing a first-line override of the logical path.
    let effective_path = extract_simulated_path(&path_str).unwrap_or(path_str);
    let effective_path = effective_path.replace('\\', "/");

    // Exception 1: any build.rs
    if effective_path.ends_with("/build.rs") {
        return true;
    }

    // Exception 2: anything under apps/*
    // Accept both absolute paths ("/.../apps/..."), and repo-relative paths ("apps/...").
    if effective_path.starts_with("apps/") || effective_path.contains("/apps/") {
        return true;
    }

    false
}

fn is_proc_macro_crate(cx: &EarlyContext<'_>) -> bool {
    cx.sess()
        .opts
        .crate_types
        .iter()
        .any(|t| *t == CrateType::ProcMacro)
}

fn extract_simulated_path(path_str: &str) -> Option<String> {
    // Only check for simulated_dir in temporary paths (UI tests run in temp directories)
    let is_temp = path_str.contains("/tmp/")
        || path_str.contains("/var/folders/")
        || path_str.contains("\\Temp\\")
        || path_str.contains(".tmp");

    if !is_temp {
        return None;
    }

    let contents = std::fs::read_to_string(std::path::PathBuf::from(path_str)).ok()?;
    for line in contents.lines().take(1) {
        let trimmed = line.trim();
        if trimmed.starts_with("// simulated_dir=") {
            return Some(trimmed.trim_start_matches("// simulated_dir=").to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("#!") {
            break;
        }
    }

    None
}

struct ForbiddenMacroVisitor<'a, 'cx> {
    cx: &'a EarlyContext<'cx>,
    in_proc_macro_crate: bool,
    allow_stack: Vec<bool>,
}

impl<'a, 'cx> ForbiddenMacroVisitor<'a, 'cx> {
    fn lint_mac_call(&self, mac_call: &MacCall) {
        let allowed_here = self.allow_stack.last().copied().unwrap_or(false);
        if allowed_here {
            return;
        }

        if is_allowed_location(self.cx, mac_call.span()) {
            return;
        }

        let Some(last) = mac_call.path.segments.last() else {
            return;
        };

        let name = last.ident.name.as_str();
        if !FORBIDDEN_MACROS.contains(&name) {
            return;
        }

        self.cx
            .span_lint(DE1301_NO_PRINT_MACROS, mac_call.span(), |diag| {
                diag.primary_message(format!(
                    "macro `{name}!` is forbidden in production code (DE1301)"
                ));
                diag.help(
                    "use `tracing`/`log` for observability, or return the value and handle it at the boundary",
                );
            });
    }
}

impl<'ast, 'a, 'cx> visit::Visitor<'ast> for ForbiddenMacroVisitor<'a, 'cx> {
    fn visit_item(&mut self, item: &'ast Item) {
        let parent_allow = self.allow_stack.last().copied().unwrap_or(false);

        match &item.kind {
            ItemKind::Fn(..) => {
                let is_private = matches!(item.vis.kind, VisibilityKind::Inherited);
                let allow_here = parent_allow
                    || is_test_item(&item.attrs)
                    || (self.in_proc_macro_crate
                        && (is_private || has_proc_macro_attr(&item.attrs)));

                self.allow_stack.push(allow_here);
                visit::walk_item(self, item);
                self.allow_stack.pop();
            }
            ItemKind::Mod(..) => {
                let allow_here = parent_allow || is_test_item(&item.attrs);
                self.allow_stack.push(allow_here);
                visit::walk_item(self, item);
                self.allow_stack.pop();
            }
            _ => visit::walk_item(self, item),
        }
    }

    fn visit_expr(&mut self, expr: &'ast rustc_ast::Expr) {
        if let ExprKind::MacCall(mac_call) = &expr.kind {
            self.lint_mac_call(mac_call);
        }
        visit::walk_expr(self, expr);
    }

    fn visit_mac_call(&mut self, mac_call: &'ast MacCall) {
        self.lint_mac_call(mac_call);
    }
}

fn has_proc_macro_attr(attrs: &[rustc_ast::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let AttrKind::Normal(normal) = &attr.kind else {
            return false;
        };

        let Some(last) = normal.item.path.segments.last() else {
            return false;
        };

        matches!(
            last.ident.name.as_str(),
            "proc_macro" | "proc_macro_attribute" | "proc_macro_derive"
        )
    })
}

fn is_test_item(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.has_name(sym::test) {
            return true;
        }

        if let Some(ident) = attr.path().last() {
            if *ident == sym::test {
                return true;
            }
        }

        if attr.has_name(sym::cfg) {
            if let Some(list) = attr.meta_item_list() {
                return list.iter().any(|item| item.has_name(sym::test));
            }
        }

        false
    })
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
        lint_utils::test_comment_annotations_match_stderr(&ui_dir, "DE1301", "Print macros");
    }
}
