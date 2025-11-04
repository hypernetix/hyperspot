use sea_orm::{sea_query::Expr, ColumnTrait, Condition, EntityTrait};

use crate::secure::error::ScopeError;
use crate::secure::provider::{SimpleTenantFilter, TenantFilterProvider};
use crate::secure::{AccessScope, ScopableEntity};

/// Builds a SeaORM `Condition` based on the implicit security policy.
///
/// # Policy Rules
/// 1. **Empty scope** (no tenants, no resources) → deny all (`false`)
/// 2. **Tenants only** → filter by `tenant_col IN tenant_ids` (via provider)
///    - If entity has no tenant_col but tenant_ids provided → deny all
/// 3. **Resources only** → filter by `resource_col IN resource_ids`
/// 4. **Both present** → AND them: `(tenant_col IN ...) AND (resource_col IN ...)`
///
/// # Provider Pattern
///
/// Uses `SimpleTenantFilter` by default for tenant filtering. This can be
/// replaced with a hierarchical provider in the future without changing
/// calling code.
///
/// This ensures that:
/// - No query can bypass tenant isolation when tenants are specified
/// - Explicit resource IDs provide fine-grained access
/// - Empty scopes are explicitly denied rather than returning all data
pub fn build_scope_condition<E>(scope: &AccessScope) -> Result<Condition, ScopeError>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    let deny_all = || Condition::all().add(Expr::value(false));

    // Rule 1: Nothing supplied → deny all
    if scope.is_empty() {
        return Ok(deny_all());
    }

    let mut parts: Vec<Condition> = Vec::new();

    // Build tenant filter using provider pattern
    if let Some(tenant_cond) = SimpleTenantFilter::tenant_condition::<E>(scope) {
        parts.push(tenant_cond);
    }

    // Build resource ID filter
    if !scope.resource_ids().is_empty() {
        if let Some(resource_col) = E::resource_col() {
            let id_filter =
                Condition::all().add(Expr::col(resource_col).is_in(scope.resource_ids().to_vec()));
            parts.push(id_filter);
        } else {
            // Entity has no resource_col but scope requires resource filtering → deny all
            return Ok(deny_all());
        }
    }

    // Combine parts
    let cond = match parts.as_slice() {
        // Only one constraint
        [only] => only.clone(),
        // Both constraints → AND them
        [a, b] => Condition::all().add(a.clone()).add(b.clone()),
        // Neither (shouldn't happen due to earlier checks)
        [] => deny_all(),
        _ => unreachable!(),
    };

    Ok(cond)
}

#[cfg(test)]
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
