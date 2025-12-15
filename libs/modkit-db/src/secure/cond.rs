use sea_orm::{sea_query::Expr, ColumnTrait, Condition, EntityTrait};

use crate::secure::provider::{SimpleTenantFilter, TenantFilterProvider};
use crate::secure::{AccessScope, ScopableEntity};

/// Builds a SeaORM `Condition` based on the implicit security policy.
///
/// # Policy Rules
/// 1. **Empty scope** (no tenants, no resources, not root) → deny all (`false`)
/// 2. **Root scope** → bypass tenant filtering entirely
///    - If no resource filters either → return `Condition::all()` (no filters)
///    - If resource filters present → apply only resource filters
/// 3. **Tenants only** → filter by `tenant_col IN tenant_ids` (via provider)
///    - If entity has no tenant_col but tenant_ids provided → deny all
/// 4. **Resources only** → filter by `resource_col IN resource_ids`
/// 5. **Both present** → AND them: `(tenant_col IN ...) AND (resource_col IN ...)`
///
/// # Provider Pattern
///
/// Uses `SimpleTenantFilter` by default for tenant filtering. This can be
/// replaced with a hierarchical provider in the future without changing
/// calling code.
///
/// This ensures that:
/// - No query can bypass tenant isolation when tenants are specified
/// - Root scope explicitly bypasses all tenant filtering for system-level access
/// - Explicit resource IDs provide fine-grained access
/// - Empty scopes are explicitly denied rather than returning all data
pub fn build_scope_condition<E>(scope: &AccessScope) -> Condition
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    let deny_all = || Condition::all().add(Expr::value(false));

    // Rule 1: Nothing supplied → deny all
    if scope.is_empty() {
        return deny_all();
    }

    let mut parts: Vec<Condition> = Vec::new();

    if let Some(tenant_cond) = SimpleTenantFilter::tenant_condition::<E>(scope) {
        parts.push(tenant_cond);
    }

    if !scope.resource_ids().is_empty() {
        if let Some(resource_col) = E::resource_col() {
            let id_filter =
                Condition::all().add(Expr::col(resource_col).is_in(scope.resource_ids().to_vec()));
            parts.push(id_filter);
        } else {
            // Entity has no resource_col but scope requires resource filtering → deny all
            return deny_all();
        }
    }

    let cond = match parts.as_slice() {
        [only] => only.clone(),
        [a, b] => Condition::all().add(a.clone()).add(b.clone()),
        [] if scope.is_root() => Condition::all(), // root with no filters at all
        [] => deny_all(),                          // non-root, no filters → deny all
        _ => unreachable!(),
    };

    cond
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    // Note: Full integration tests with real SeaORM entities should be written
    // in application code where actual entities are available.
    // The condition building logic is tested via the library's compile-time checks
    // and the public API surface.
    //
    // See USAGE_EXAMPLE.md for complete usage patterns.

    #[test]
    fn test_compile_check() {
        // This test verifies the module compiles
        // Actual condition building is tested in integration tests
        let scope = AccessScope::default();
        assert!(scope.is_empty());
    }
}
