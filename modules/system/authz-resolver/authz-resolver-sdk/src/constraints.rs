//! Constraint types for authorization decisions.
//!
//! Constraints represent row-level filtering conditions returned by the PDP.
//! They are compiled into `AccessScope` by the PEP compiler.
//!
//! ## First iteration
//!
//! Only `Eq` and `In` predicates are supported. Complex predicates
//! (`in_tenant_subtree`, `in_group`, etc.) are deferred.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A constraint on a specific resource property.
///
/// Multiple constraints within a response are `ORed`:
/// a resource matches if it satisfies ANY constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// The predicates within this constraint. All predicates are `ANDed`:
    /// a resource matches this constraint only if ALL predicates are satisfied.
    pub predicates: Vec<Predicate>,
}

/// A predicate comparing a resource property to a value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Predicate {
    /// Equality: `resource_property = value`
    Eq(EqPredicate),
    /// Set membership: `resource_property IN (values)`
    In(InPredicate),
}

/// Equality predicate: `property = value`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqPredicate {
    /// Resource property name (e.g., `pep_properties::OWNER_TENANT_ID`, `pep_properties::RESOURCE_ID`).
    pub property: String,
    /// The value to match.
    pub value: Uuid,
}

/// Set membership predicate: `property IN (values)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InPredicate {
    /// Resource property name (e.g., `pep_properties::OWNER_TENANT_ID`, `pep_properties::RESOURCE_ID`).
    pub property: String,
    /// The set of values to match against.
    pub values: Vec<Uuid>,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use modkit_security::pep_properties;

    #[test]
    fn constraint_serialization_roundtrip() {
        let constraint = Constraint {
            predicates: vec![
                Predicate::In(InPredicate {
                    property: pep_properties::OWNER_TENANT_ID.to_owned(),
                    values: vec![
                        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
                        Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
                    ],
                }),
                Predicate::Eq(EqPredicate {
                    property: pep_properties::RESOURCE_ID.to_owned(),
                    value: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
                }),
            ],
        };

        let json = serde_json::to_string(&constraint).unwrap();
        let deserialized: Constraint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.predicates.len(), 2);
    }

    #[test]
    fn predicate_tag_serialization() {
        let eq = Predicate::Eq(EqPredicate {
            property: pep_properties::RESOURCE_ID.to_owned(),
            value: Uuid::nil(),
        });

        let json = serde_json::to_string(&eq).unwrap();
        assert!(json.contains(r#""op":"eq""#));

        let in_pred = Predicate::In(InPredicate {
            property: pep_properties::OWNER_TENANT_ID.to_owned(),
            values: vec![Uuid::nil()],
        });

        let json = serde_json::to_string(&in_pred).unwrap();
        assert!(json.contains(r#""op":"in""#));
    }
}
