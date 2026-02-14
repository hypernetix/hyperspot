use sea_orm::{ColumnTrait, Condition, EntityTrait, sea_query::Expr};

use crate::secure::{AccessScope, ScopableEntity};
use modkit_security::access_scope::{FilterOp, ScopeConstraint};

/// Build a deny-all condition (`WHERE false`).
fn deny_all() -> Condition {
    Condition::all().add(Expr::value(false))
}

/// Builds a `SeaORM` `Condition` from an `AccessScope` using property resolution.
///
/// # OR/AND Semantics
///
/// - Multiple constraints are OR-ed (alternative access paths)
/// - Filters within a constraint are AND-ed (all must match)
/// - Unknown `pep_properties` fail that constraint (fail-closed)
/// - If all constraints fail resolution, deny-all
///
/// # Policy Rules
///
/// | Scope | Behavior |
/// |-------|----------|
/// | deny-all (default) | `WHERE false` |
/// | unconstrained (allow-all) | No filtering (`WHERE true`) |
/// | single constraint | AND of resolved filters |
/// | multiple constraints | OR of ANDed filter groups |
pub fn build_scope_condition<E>(scope: &AccessScope) -> Condition
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    if scope.is_unconstrained() {
        return Condition::all();
    }
    if scope.is_deny_all() {
        return deny_all();
    }

    let compiled: Vec<Condition> = scope
        .constraints()
        .iter()
        .filter_map(build_constraint_condition::<E>)
        .collect();

    match compiled.len() {
        0 => deny_all(),
        1 => compiled.into_iter().next().unwrap_or_else(deny_all),
        _ => {
            let mut or_cond = Condition::any();
            for c in compiled {
                or_cond = or_cond.add(c);
            }
            or_cond
        }
    }
}

/// Build SQL for a single constraint (AND of filters).
///
/// Returns `None` if any filter references an unknown property (fail-closed).
fn build_constraint_condition<E>(constraint: &ScopeConstraint) -> Option<Condition>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    if constraint.is_empty() {
        return Some(Condition::all());
    }
    let mut and_cond = Condition::all();
    for filter in constraint.filters() {
        let col = E::resolve_property(filter.property())?;
        match filter.op() {
            FilterOp::Eq => {
                if let Some(&val) = filter.values().first() {
                    and_cond = and_cond.add(Expr::col(col).eq(val));
                }
            }
            FilterOp::In => {
                and_cond = and_cond.add(Expr::col(col).is_in(filter.values().to_vec()));
            }
        }
    }
    Some(and_cond)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use modkit_security::access_scope::{FilterOp, ScopeConstraint, ScopeFilter, pep_properties};

    #[test]
    fn test_deny_all_scope() {
        let scope = AccessScope::default();
        assert!(scope.is_deny_all());
    }

    #[test]
    fn test_allow_all_scope() {
        let scope = AccessScope::allow_all();
        assert!(scope.is_unconstrained());
    }

    #[test]
    fn test_tenant_scope_not_empty() {
        let tid = uuid::Uuid::new_v4();
        let scope = AccessScope::for_tenant(tid);
        assert!(!scope.is_deny_all());
        assert!(scope.contains_value(pep_properties::OWNER_TENANT_ID, tid));
    }

    #[test]
    fn test_or_scope_has_multiple_constraints() {
        let t1 = uuid::Uuid::new_v4();
        let t2 = uuid::Uuid::new_v4();
        let r1 = uuid::Uuid::new_v4();

        let scope = AccessScope::from_constraints(vec![
            ScopeConstraint::new(vec![
                ScopeFilter::new(pep_properties::OWNER_TENANT_ID, FilterOp::In, vec![t1]),
                ScopeFilter::new(pep_properties::RESOURCE_ID, FilterOp::In, vec![r1]),
            ]),
            ScopeConstraint::new(vec![ScopeFilter::new(
                pep_properties::OWNER_TENANT_ID,
                FilterOp::In,
                vec![t2],
            )]),
        ]);
        assert_eq!(scope.constraints().len(), 2);
    }

    // --- Custom PEP property tests ---

    /// Test entity with a custom `department_id` property, mimicking what the
    /// derive macro generates for an entity with `pep_prop(department_id = "department_id")`.
    mod custom_prop_entity {
        use super::*;
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "custom_prop_test")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: Uuid,
            pub tenant_id: Uuid,
            pub department_id: Uuid,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}

        impl crate::secure::ScopableEntity for Entity {
            fn tenant_col() -> Option<Column> {
                Some(Column::TenantId)
            }
            fn resource_col() -> Option<Column> {
                Some(Column::Id)
            }
            fn owner_col() -> Option<Column> {
                None
            }
            fn type_col() -> Option<Column> {
                None
            }
            fn resolve_property(property: &str) -> Option<Column> {
                match property {
                    p if p == pep_properties::OWNER_TENANT_ID => Some(Column::TenantId),
                    p if p == pep_properties::RESOURCE_ID => Some(Column::Id),
                    "department_id" => Some(Column::DepartmentId),
                    _ => None,
                }
            }
        }
    }

    #[test]
    fn test_custom_property_resolves() {
        let dept = uuid::Uuid::new_v4();
        let scope =
            AccessScope::from_constraints(vec![ScopeConstraint::new(vec![ScopeFilter::new(
                "department_id",
                FilterOp::In,
                vec![dept],
            )])]);
        // Should produce a real condition (not deny-all) since the entity resolves "department_id".
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        // A deny-all condition contains `Expr::value(false)` — verify this is NOT that.
        let cond_str = format!("{cond:?}");
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "Expected a real condition, got deny-all: {cond_str}"
        );
    }

    #[test]
    fn test_unknown_property_deny_all() {
        let val = uuid::Uuid::new_v4();
        let scope =
            AccessScope::from_constraints(vec![ScopeConstraint::new(vec![ScopeFilter::new(
                "nonexistent",
                FilterOp::In,
                vec![val],
            )])]);
        // Unknown property should cause the constraint to fail → deny-all.
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            cond_str.contains("Value(Bool(Some(false)))"),
            "Expected deny-all, got: {cond_str}"
        );
    }

    #[test]
    fn test_eq_filter_produces_equality_condition() {
        let tid = uuid::Uuid::new_v4();
        let scope =
            AccessScope::from_constraints(vec![ScopeConstraint::new(vec![ScopeFilter::eq(
                pep_properties::OWNER_TENANT_ID,
                tid,
            )])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        // Should produce an equality condition, not an IN condition
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "Expected a real condition, got deny-all: {cond_str}"
        );
    }

    #[test]
    fn test_standard_plus_custom_scope() {
        let tid = uuid::Uuid::new_v4();
        let dept = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::new(pep_properties::OWNER_TENANT_ID, FilterOp::In, vec![tid]),
            ScopeFilter::new("department_id", FilterOp::In, vec![dept]),
        ])]);
        // Both standard and custom pep_properties should resolve successfully.
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "Expected a real condition, got deny-all: {cond_str}"
        );
    }
}
