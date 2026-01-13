#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::sym as clippy_sym;
use rustc_hir::{FieldDef, Item, ItemKind, VariantData};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::{self, Ty};
use rustc_span::Symbol;

dylint_linting::declare_late_lint! {
    /// DE0301: Duration fields in *Config must use humantime serde mapping
    ///
    /// Checks that any `std::time::Duration` (or `Option<Duration>`) field in a struct whose
    /// name ends with `Config` has the appropriate `#[serde(with = "...")]` attribute.
    pub DE0301_CONFIG_DURATION_HUMANTIME_SERDE,
    Deny,
    "Duration fields in *Config structs must use humantime serde mapping (DE0301)"
}

const SERDE_WITH_HUMANTIME: &str = "modkit_utils::humantime_serde";
const SERDE_WITH_HUMANTIME_OPTION: &str = "modkit_utils::humantime_serde::option";

fn normalize_serde_with_value(s: &str) -> &str {
    s.trim().strip_prefix("::").unwrap_or(s.trim())
}

fn is_duration_ty<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> bool {
    let ty::Adt(adt_def, _) = ty.kind() else {
        return false;
    };

    let path = cx.tcx.def_path_str(adt_def.did());
    path == "std::time::Duration" || path == "core::time::Duration"
}

fn option_inner_ty<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let ty::Adt(adt_def, subst) = ty.kind() else {
        return None;
    };

    let path = cx.tcx.def_path_str(adt_def.did());
    if path != "core::option::Option" && path != "std::option::Option" {
        return None;
    }

    // Option<T> has a single type argument.
    subst.types().next()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DurationFieldKind {
    Duration,
    OptionDuration,
}

fn duration_field_kind<'tcx>(cx: &LateContext<'tcx>, ty: Ty<'tcx>) -> Option<DurationFieldKind> {
    if is_duration_ty(cx, ty) {
        return Some(DurationFieldKind::Duration);
    }

    let inner = option_inner_ty(cx, ty)?;
    if is_duration_ty(cx, inner) {
        return Some(DurationFieldKind::OptionDuration);
    }

    None
}

fn field_has_expected_serde_with<'tcx>(cx: &LateContext<'tcx>, field: &FieldDef<'tcx>, expected: &str) -> bool {
    let expected = expected;
    let with_sym = Symbol::intern("with");

    for attr in cx.tcx.hir_attrs(field.hir_id) {
        if !attr.has_name(clippy_sym::serde) {
            continue;
        }

        let Some(items) = attr.meta_item_list() else {
            continue;
        };

        for nested in items {
            let Some(mi) = nested.meta_item() else {
                continue;
            };

            // We only care about `with = "..."`
            if mi.path.segments.len() != 1 || mi.path.segments[0].ident.name != with_sym {
                continue;
            }

            let Some(val) = mi.value_str() else {
                continue;
            };

            let val = normalize_serde_with_value(val.as_str());
            if val == expected {
                return true;
            }
        }
    }

    false
}

fn field_label<'tcx>(field: &FieldDef<'tcx>, idx: usize) -> String {
    // Tuple struct fields may have no meaningful identifier.
    let name = field.ident.name.as_str();
    if name.is_empty() {
        format!("#{}", idx)
    } else {
        name.to_string()
    }
}

impl<'tcx> LateLintPass<'tcx> for De0301ConfigDurationHumantimeSerde {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let ItemKind::Struct(ident, _generics, variant) = &item.kind else {
            return;
        };

        let struct_name = ident.name.as_str();
        if !struct_name.ends_with("Config") {
            return;
        }

        check_variant_fields(cx, struct_name, variant);
    }
}

fn check_variant_fields<'tcx>(cx: &LateContext<'tcx>, struct_name: &str, variant: &'tcx VariantData<'tcx>) {
    for (idx, field) in variant.fields().iter().enumerate() {
        // NOTE: We intentionally do not use `cx.typeck_results()` here because this lint runs
        // outside of a body context in UI tests. Instead we use the field's `type_of`, which is
        // available for struct fields.
        let ty = cx.tcx.type_of(field.def_id).instantiate_identity();
        let Some(kind) = duration_field_kind(cx, ty) else {
            continue;
        };

        let expected = match kind {
            DurationFieldKind::Duration => SERDE_WITH_HUMANTIME,
            DurationFieldKind::OptionDuration => SERDE_WITH_HUMANTIME_OPTION,
        };

        if field_has_expected_serde_with(cx, field, expected) {
            continue;
        }

        let field_name = field_label(field, idx);
        cx.span_lint(DE0301_CONFIG_DURATION_HUMANTIME_SERDE, field.span, |diag| {
            diag.primary_message(format!(
                "`{}` field `{}` must use `#[serde(with = \"{}\")]` for proper serialization/deserialization (DE0301)",
                struct_name, field_name, expected
            ));
            diag.help(format!(
                "add `#[serde(with = \"{}\")]` to this field",
                expected
            ));
        });
    }
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
            "DE0301",
            "Config duration fields must use humantime serde",
        );
    }
}
