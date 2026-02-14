use uuid::Uuid;

/// Well-known authorization property names.
///
/// These constants are shared between the PEP compiler and the ORM condition
/// builder (`ScopableEntity::resolve_property()`), ensuring a single source of
/// truth for property names.
pub mod pep_properties {
    /// Tenant-ownership property. Typically maps to the `tenant_id` column.
    pub const OWNER_TENANT_ID: &str = "owner_tenant_id";

    /// Resource identity property. Typically maps to the primary key column.
    pub const RESOURCE_ID: &str = "id";

    /// Owner (user) identity property. Typically maps to an `owner_id` column.
    pub const OWNER_ID: &str = "owner_id";
}

/// Predicate operation type for scope filters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterOp {
    /// `property = value` — exact equality (single value).
    Eq,
    /// `property IN (values)` — flat set membership.
    In,
    // Future: InSubtree, InGroup, InGroupSubtree, ...
}

/// A single scope filter — a condition on a named resource property.
///
/// The property name (e.g., `"owner_tenant_id"`, `"id"`) is an authorization
/// concept. Mapping to DB columns is done by `ScopableEntity::resolve_property()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeFilter {
    property: String,
    op: FilterOp,
    values: Vec<Uuid>,
}

impl ScopeFilter {
    /// Create a new scope filter.
    #[must_use]
    pub fn new(property: impl Into<String>, op: FilterOp, values: Vec<Uuid>) -> Self {
        Self {
            property: property.into(),
            op,
            values,
        }
    }

    /// Create an equality filter (`property = value`).
    #[must_use]
    pub fn eq(property: impl Into<String>, value: Uuid) -> Self {
        Self {
            property: property.into(),
            op: FilterOp::Eq,
            values: vec![value],
        }
    }

    /// The authorization property name.
    #[inline]
    #[must_use]
    pub fn property(&self) -> &str {
        &self.property
    }

    /// The filter operation.
    #[inline]
    #[must_use]
    pub fn op(&self) -> &FilterOp {
        &self.op
    }

    /// The filter values.
    #[inline]
    #[must_use]
    pub fn values(&self) -> &[Uuid] {
        &self.values
    }
}

/// A conjunction (AND) of scope filters — one access path.
///
/// All filters within a constraint must match simultaneously for a row
/// to be accessible via this path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeConstraint {
    filters: Vec<ScopeFilter>,
}

impl ScopeConstraint {
    /// Create a new scope constraint from a list of filters.
    #[must_use]
    pub fn new(filters: Vec<ScopeFilter>) -> Self {
        Self { filters }
    }

    /// The filters in this constraint (AND-ed together).
    #[inline]
    #[must_use]
    pub fn filters(&self) -> &[ScopeFilter] {
        &self.filters
    }

    /// Returns `true` if this constraint has no filters.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

/// A disjunction (OR) of scope constraints defining what data is accessible.
///
/// Each constraint is an independent access path (OR-ed). Filters within a
/// constraint are AND-ed. An unconstrained scope bypasses row-level filtering.
///
/// # Examples
///
/// ```
/// use modkit_security::access_scope::{AccessScope, ScopeConstraint, ScopeFilter, FilterOp, pep_properties};
/// use uuid::Uuid;
///
/// // deny-all (default)
/// let scope = AccessScope::deny_all();
/// assert!(scope.is_deny_all());
///
/// // single tenant
/// let tid = Uuid::new_v4();
/// let scope = AccessScope::for_tenant(tid);
/// assert!(!scope.is_deny_all());
/// assert!(scope.contains_value(pep_properties::OWNER_TENANT_ID, tid));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessScope {
    constraints: Vec<ScopeConstraint>,
    unconstrained: bool,
}

impl Default for AccessScope {
    /// Default is deny-all: no constraints and not unconstrained.
    fn default() -> Self {
        Self::deny_all()
    }
}

impl AccessScope {
    // ── Constructors ────────────────────────────────────────────────

    /// Create an access scope from a list of constraints (OR-ed).
    #[must_use]
    pub fn from_constraints(constraints: Vec<ScopeConstraint>) -> Self {
        Self {
            constraints,
            unconstrained: false,
        }
    }

    /// Create an access scope with a single constraint.
    #[must_use]
    pub fn single(constraint: ScopeConstraint) -> Self {
        Self::from_constraints(vec![constraint])
    }

    /// Create an "allow all" (unconstrained) scope.
    ///
    /// This represents a legitimate PDP decision with no row-level filtering.
    /// Not a bypass — it's a valid authorization outcome.
    #[must_use]
    pub fn allow_all() -> Self {
        Self {
            constraints: Vec::new(),
            unconstrained: true,
        }
    }

    /// Create a "deny all" scope (no access).
    #[must_use]
    pub fn deny_all() -> Self {
        Self {
            constraints: Vec::new(),
            unconstrained: false,
        }
    }

    // ── Convenience constructors ────────────────────────────────────

    /// Create a scope for a set of tenant IDs.
    #[must_use]
    pub fn for_tenants(ids: Vec<Uuid>) -> Self {
        Self::single(ScopeConstraint::new(vec![ScopeFilter::new(
            pep_properties::OWNER_TENANT_ID,
            FilterOp::In,
            ids,
        )]))
    }

    /// Create a scope for a single tenant ID.
    #[must_use]
    pub fn for_tenant(id: Uuid) -> Self {
        Self::for_tenants(vec![id])
    }

    /// Create a scope for a set of resource IDs.
    #[must_use]
    pub fn for_resources(ids: Vec<Uuid>) -> Self {
        Self::single(ScopeConstraint::new(vec![ScopeFilter::new(
            pep_properties::RESOURCE_ID,
            FilterOp::In,
            ids,
        )]))
    }

    /// Create a scope for a single resource ID.
    #[must_use]
    pub fn for_resource(id: Uuid) -> Self {
        Self::for_resources(vec![id])
    }

    // ── Accessors ───────────────────────────────────────────────────

    /// The constraints in this scope (OR-ed).
    #[inline]
    #[must_use]
    pub fn constraints(&self) -> &[ScopeConstraint] {
        &self.constraints
    }

    /// Returns `true` if this scope is unconstrained (allow-all).
    #[inline]
    #[must_use]
    pub fn is_unconstrained(&self) -> bool {
        self.unconstrained
    }

    /// Returns `true` if this scope denies all access.
    ///
    /// A scope is deny-all when it is not unconstrained and has no constraints.
    #[must_use]
    pub fn is_deny_all(&self) -> bool {
        !self.unconstrained && self.constraints.is_empty()
    }

    /// Collect all values for a given property across all constraints.
    ///
    /// Useful for extracting tenant IDs when you know the scope has
    /// only simple tenant-based constraints.
    #[must_use]
    pub fn all_values_for(&self, property: &str) -> Vec<Uuid> {
        let mut result = Vec::new();
        for constraint in &self.constraints {
            for filter in constraint.filters() {
                if filter.property() == property
                    && matches!(filter.op(), FilterOp::Eq | FilterOp::In)
                {
                    result.extend_from_slice(filter.values());
                }
            }
        }
        result
    }

    /// Check if any constraint has a filter matching the given property and value.
    #[must_use]
    pub fn contains_value(&self, property: &str, id: Uuid) -> bool {
        self.constraints.iter().any(|c| {
            c.filters().iter().any(|f| {
                f.property() == property
                    && matches!(f.op(), FilterOp::Eq | FilterOp::In)
                    && f.values().contains(&id)
            })
        })
    }

    /// Check if any constraint references the given property.
    #[must_use]
    pub fn has_property(&self, property: &str) -> bool {
        self.constraints
            .iter()
            .any(|c| c.filters().iter().any(|f| f.property() == property))
    }

    /// Extract a single value for a property from the scope.
    ///
    /// Intended for CREATE operations where the PEP needs exactly one tenant ID
    /// to assign as `owner_tenant_id`. Fail-closed: ambiguous or missing
    /// constraints return an error.
    ///
    /// # Errors
    ///
    /// - [`ExtractError::NoConstraints`] — scope is deny-all or unconstrained
    /// - [`ExtractError::AmbiguousConstraints`] — more than one constraint
    ///   contains the property
    /// - [`ExtractError::PropertyNotFound`] — no filter matches the property
    /// - [`ExtractError::AmbiguousValue`] — filter has more than one value
    pub fn extract_single_value(&self, property: &str) -> Result<Uuid, ExtractError> {
        if self.is_deny_all() || self.is_unconstrained() {
            return Err(ExtractError::NoConstraints);
        }

        // Collect matching filters across all constraints.
        let matching: Vec<&ScopeFilter> = self
            .constraints
            .iter()
            .flat_map(ScopeConstraint::filters)
            .filter(|f| f.property() == property && matches!(f.op(), FilterOp::Eq | FilterOp::In))
            .collect();

        match matching.len() {
            0 => Err(ExtractError::PropertyNotFound),
            1 => {
                let filter = matching[0];
                if filter.values().len() == 1 {
                    Ok(filter.values()[0])
                } else {
                    Err(ExtractError::AmbiguousValue)
                }
            }
            _ => Err(ExtractError::AmbiguousConstraints),
        }
    }
}

/// Error extracting a single value from an [`AccessScope`].
#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    /// The scope is deny-all or unconstrained — no constraints to inspect.
    #[error("scope has no constraints (deny-all or unconstrained)")]
    NoConstraints,

    /// More than one constraint contains the requested property.
    #[error("ambiguous: multiple constraints contain the property")]
    AmbiguousConstraints,

    /// No filter in any constraint matches the requested property.
    #[error("property not found in scope constraints")]
    PropertyNotFound,

    /// The filter matches but contains more than one value.
    #[error("ambiguous: filter contains multiple values")]
    AmbiguousValue,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use uuid::Uuid;

    const T1: &str = "11111111-1111-1111-1111-111111111111";
    const T2: &str = "22222222-2222-2222-2222-222222222222";

    fn uid(s: &str) -> Uuid {
        Uuid::parse_str(s).unwrap()
    }

    // --- FilterOp::Eq ---

    #[test]
    fn scope_filter_eq_constructor() {
        let f = ScopeFilter::eq(pep_properties::OWNER_TENANT_ID, uid(T1));
        assert_eq!(f.property(), pep_properties::OWNER_TENANT_ID);
        assert_eq!(*f.op(), FilterOp::Eq);
        assert_eq!(f.values(), &[uid(T1)]);
    }

    #[test]
    fn all_values_for_works_with_eq() {
        let scope = AccessScope::single(ScopeConstraint::new(vec![ScopeFilter::eq(
            pep_properties::OWNER_TENANT_ID,
            uid(T1),
        )]));
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID),
            &[uid(T1)]
        );
    }

    #[test]
    fn all_values_for_works_with_mixed_eq_and_in() {
        let scope = AccessScope::from_constraints(vec![
            ScopeConstraint::new(vec![ScopeFilter::eq(
                pep_properties::OWNER_TENANT_ID,
                uid(T1),
            )]),
            ScopeConstraint::new(vec![ScopeFilter::new(
                pep_properties::OWNER_TENANT_ID,
                FilterOp::In,
                vec![uid(T2)],
            )]),
        ]);
        let values = scope.all_values_for(pep_properties::OWNER_TENANT_ID);
        assert_eq!(values, &[uid(T1), uid(T2)]);
    }

    #[test]
    fn contains_value_works_with_eq() {
        let scope = AccessScope::single(ScopeConstraint::new(vec![ScopeFilter::eq(
            pep_properties::OWNER_TENANT_ID,
            uid(T1),
        )]));
        assert!(scope.contains_value(pep_properties::OWNER_TENANT_ID, uid(T1)));
        assert!(!scope.contains_value(pep_properties::OWNER_TENANT_ID, uid(T2)));
    }

    // --- extract_single_value ---

    #[test]
    fn extract_single_value_eq_happy_path() {
        let scope = AccessScope::single(ScopeConstraint::new(vec![ScopeFilter::eq(
            pep_properties::OWNER_TENANT_ID,
            uid(T1),
        )]));
        assert_eq!(
            scope
                .extract_single_value(pep_properties::OWNER_TENANT_ID)
                .unwrap(),
            uid(T1)
        );
    }

    #[test]
    fn extract_single_value_in_with_one_value() {
        let scope = AccessScope::single(ScopeConstraint::new(vec![ScopeFilter::new(
            pep_properties::OWNER_TENANT_ID,
            FilterOp::In,
            vec![uid(T1)],
        )]));
        assert_eq!(
            scope
                .extract_single_value(pep_properties::OWNER_TENANT_ID)
                .unwrap(),
            uid(T1)
        );
    }

    #[test]
    fn extract_single_value_deny_all() {
        let scope = AccessScope::deny_all();
        assert!(matches!(
            scope.extract_single_value(pep_properties::OWNER_TENANT_ID),
            Err(ExtractError::NoConstraints)
        ));
    }

    #[test]
    fn extract_single_value_unconstrained() {
        let scope = AccessScope::allow_all();
        assert!(matches!(
            scope.extract_single_value(pep_properties::OWNER_TENANT_ID),
            Err(ExtractError::NoConstraints)
        ));
    }

    #[test]
    fn extract_single_value_property_not_found() {
        let scope = AccessScope::single(ScopeConstraint::new(vec![ScopeFilter::eq(
            pep_properties::RESOURCE_ID,
            uid(T1),
        )]));
        assert!(matches!(
            scope.extract_single_value(pep_properties::OWNER_TENANT_ID),
            Err(ExtractError::PropertyNotFound)
        ));
    }

    #[test]
    fn extract_single_value_ambiguous_constraints() {
        // Two constraints each with the same property → ambiguous
        let scope = AccessScope::from_constraints(vec![
            ScopeConstraint::new(vec![ScopeFilter::eq(
                pep_properties::OWNER_TENANT_ID,
                uid(T1),
            )]),
            ScopeConstraint::new(vec![ScopeFilter::eq(
                pep_properties::OWNER_TENANT_ID,
                uid(T2),
            )]),
        ]);
        assert!(matches!(
            scope.extract_single_value(pep_properties::OWNER_TENANT_ID),
            Err(ExtractError::AmbiguousConstraints)
        ));
    }

    #[test]
    fn extract_single_value_in_with_multiple_values() {
        let scope = AccessScope::single(ScopeConstraint::new(vec![ScopeFilter::new(
            pep_properties::OWNER_TENANT_ID,
            FilterOp::In,
            vec![uid(T1), uid(T2)],
        )]));
        assert!(matches!(
            scope.extract_single_value(pep_properties::OWNER_TENANT_ID),
            Err(ExtractError::AmbiguousValue)
        ));
    }
}
