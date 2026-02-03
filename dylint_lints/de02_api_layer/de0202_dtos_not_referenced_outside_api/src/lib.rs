#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;

use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LateLintPass, LintContext};

dylint_linting::declare_late_lint! {
    /// DE0202: DTOs not referenced outside API
    ///
    /// DTO types must not be imported by contract, domain, or infra modules.
    /// DTOs are API layer implementation details.
    pub DE0202_DTOS_NOT_REFERENCED_OUTSIDE_API,
    Deny,
    "DTO types should not be imported outside of api layer (DE0202)"
}

impl<'tcx> LateLintPass<'tcx> for De0202DtosNotReferencedOutsideApi {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // Only check use statements
        let ItemKind::Use(path, _) = &item.kind else {
            return;
        };

        // Check if we're in a forbidden module (contract, domain, infra)
        let sm = cx.sess().source_map();
        let span = cx.tcx.def_span(item.owner_id.def_id);

        let in_forbidden = lint_utils::is_in_contract_path(sm, span)
            || lint_utils::is_in_domain_path(sm, span)
            || lint_utils::is_in_infra_path(sm, span);
        if !in_forbidden {
            return;
        }

        // Check if the import path references api::rest::dto
        let path_str = path_to_string(path);

        // Only check imports from api::rest::dto or api::rest
        if !path_str.contains("api::rest::dto") && !path_str.contains("api::rest") {
            return;
        }

        // Check if importing a DTO type
        let segments: Vec<&str> = path_str.split("::").collect();
        if let Some(last) = segments.last() {
            let is_dto = last.ends_with("Dto")
                || last.ends_with("Request")
                || last.ends_with("Response")
                || last.ends_with("Query");

            if is_dto {
                let module_type = if lint_utils::is_in_contract_path(sm, span) {
                    "contract"
                } else if lint_utils::is_in_domain_path(sm, span) {
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

fn path_to_string(path: &rustc_hir::UsePath<'_>) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
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
            "DE0202",
            "DTOs not referenced outside api",
        );
    }
}
