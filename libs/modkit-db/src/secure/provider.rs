use sea_orm::{sea_query::Expr, ColumnTrait, Condition, EntityTrait};

use crate::secure::{AccessScope, ScopableEntity};

/// Provides tenant filtering logic for scoped queries.
///
/// This trait abstracts the tenant filtering mechanism, allowing for future
/// enhancements like hierarchical tenant structures ("effective tenants")
/// without changing calling code.
///
/// # Current Implementation
///
/// `SimpleTenantFilter` uses direct `tenant_id IN (...)` filtering.
///
/// # Future Enhancement
///
/// A `HierarchicalTenantFilter` could query "effective tenant IDs" from
/// a tenant hierarchy service and expand the filter accordingly.
pub trait TenantFilterProvider {
    /// Build a condition for tenant filtering based on the scope.
    ///
    /// Returns:
    /// - `None` if no tenant filtering needed (empty tenant_ids)
    /// - `Some(deny_all)` if entity has no tenant column but tenants requested
    /// - `Some(filter)` with appropriate tenant IN clause
    fn tenant_condition<E>(scope: &AccessScope) -> Option<Condition>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy;
}

/// Simple tenant filter using direct IN clause.
///
/// This is the v1 implementation that filters by:
/// `tenant_id IN (scope.tenant_ids)`
///
/// # Future
///
/// Can be replaced with a hierarchical provider that expands
/// tenant_ids to include child tenants.
pub struct SimpleTenantFilter;

impl TenantFilterProvider for SimpleTenantFilter {
    fn tenant_condition<E>(scope: &AccessScope) -> Option<Condition>
    where
        E: ScopableEntity + EntityTrait,
        E::Column: ColumnTrait + Copy,
    {
        // No tenant IDs in scope → no tenant filter
        if scope.tenant_ids().is_empty() {
            return None;
        }

        // Entity has no tenant column but tenant IDs requested → deny all
        let Some(tcol) = E::tenant_col() else {
            return Some(Condition::all().add(Expr::value(false)));
        };

        // Build tenant IN filter
        Some(Condition::all().add(Expr::col(tcol).is_in(scope.tenant_ids().to_vec())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests with SeaORM entities should be written in actual
    // application code where real entities are defined. These are basic unit tests
    // for the provider trait pattern.
    //
    // See USAGE_EXAMPLE.md for complete usage examples with real SeaORM entities.

    #[test]
    fn test_provider_trait_compiles() {
        // This test verifies the provider trait compiles correctly
        // The actual tenant filtering is tested in integration tests with real entities
        let scope = AccessScope::default();
        assert!(scope.is_empty());
    }
}
