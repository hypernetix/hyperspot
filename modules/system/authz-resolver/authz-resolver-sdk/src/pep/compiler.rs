//! PEP constraint compiler.
//!
//! Compiles PDP evaluation responses into `AccessScope` for the secure ORM.
//!
//! ## Compilation Matrix (decision=true assumed)
//!
//! | `require_constraints` | constraints | Result |
//! |-------------------|-------------|--------|
//! | false             | empty       | `allow_all()` |
//! | false             | present     | Compile constraints → `AccessScope` |
//! | true              | empty       | `ConstraintsRequiredButAbsent` |
//! | true              | present     | Compile constraints → `AccessScope` |
//!
//! Unknown/unsupported properties fail that constraint (fail-closed).
//!
//! When `require_constraints=false`, empty constraints are treated as
//! `allow_all()` (legitimate PDP "yes, no row-level filtering"). When
//! `require_constraints=true`, empty constraints are an error (fail-closed).
//! If the PDP returns constraints regardless of the flag, they are compiled.

use modkit_security::{AccessScope, FilterOp, ScopeConstraint, ScopeFilter};

use crate::constraints::{Constraint, Predicate};
use crate::models::EvaluationResponse;

/// Error during constraint compilation.
#[derive(Debug, thiserror::Error)]
pub enum ConstraintCompileError {
    /// Constraints were required but the PDP returned none.
    ///
    /// Per the design Decision Matrix, this is a deny: the PEP asked for
    /// row-level constraints but received an empty set. Fail-closed.
    #[error("constraints required but PDP returned none (fail-closed)")]
    ConstraintsRequiredButAbsent,

    /// All constraints contained unknown predicates (fail-closed).
    #[error("all constraints failed compilation (fail-closed): {reason}")]
    AllConstraintsFailed { reason: String },
}

/// Compile constraints from an evaluation response into an `AccessScope`.
///
/// **Precondition:** the caller has already verified `response.decision == true`.
/// This function only handles constraint compilation:
/// - `require_constraints=false, constraints=[]` → `Ok(allow_all())`
/// - `require_constraints=false, constraints=[..]` → compile predicates
/// - `require_constraints=true, constraints=[]` → `Err(ConstraintsRequiredButAbsent)`
/// - `require_constraints=true, constraints=[..]` → compile predicates
///
/// Each PDP constraint compiles to a `ScopeConstraint` (AND of filters).
/// Multiple constraints become `AccessScope::from_constraints` (OR-ed).
///
/// The compiler is property-agnostic: it validates predicates against the
/// provided `supported_properties` list and converts them structurally.
/// Unknown properties fail that constraint (fail-closed).
/// If ALL constraints fail compilation, returns `AllConstraintsFailed`.
///
/// # Errors
///
/// - `ConstraintsRequiredButAbsent` if constraints were required but empty
/// - `AllConstraintsFailed` if all constraints have unsupported predicates
pub fn compile_to_access_scope(
    response: &EvaluationResponse,
    require_constraints: bool,
    supported_properties: &[&str],
) -> Result<AccessScope, ConstraintCompileError> {
    // Step 1: Handle empty constraints based on require_constraints flag.
    if response.context.constraints.is_empty() {
        if require_constraints {
            return Err(ConstraintCompileError::ConstraintsRequiredButAbsent);
        }
        return Ok(AccessScope::allow_all());
    }

    // Step 2: Compile each constraint
    let mut constraints = Vec::new();
    let mut fail_reasons: Vec<String> = Vec::new();

    for constraint in &response.context.constraints {
        match compile_constraint(constraint, supported_properties) {
            Ok(sc) => constraints.push(sc),
            Err(reason) => {
                tracing::warn!(
                    reason = %reason,
                    "constraint compilation failed (fail-closed), possible PDP contract violation",
                );
                fail_reasons.push(reason);
            }
        }
    }

    // If no constraint compiled successfully, fail-closed
    if constraints.is_empty() {
        return Err(ConstraintCompileError::AllConstraintsFailed {
            reason: fail_reasons.join("; "),
        });
    }

    // If all compiled constraints are empty (no filters), it means allow-all
    if constraints.iter().all(ScopeConstraint::is_empty) {
        return Ok(AccessScope::allow_all());
    }

    Ok(AccessScope::from_constraints(constraints))
}

/// Compile a single PDP constraint into a `ScopeConstraint`.
///
/// Each predicate becomes a `ScopeFilter`. If any predicate's property
/// is not in `supported_properties`, the entire constraint fails (fail-closed).
fn compile_constraint(
    constraint: &Constraint,
    supported_properties: &[&str],
) -> Result<ScopeConstraint, String> {
    let mut filters = Vec::new();

    for predicate in &constraint.predicates {
        let (property, op, values) = match predicate {
            Predicate::Eq(eq) => (eq.property.as_str(), FilterOp::Eq, vec![eq.value]),
            Predicate::In(p) => (p.property.as_str(), FilterOp::In, p.values.clone()),
        };

        if !supported_properties.contains(&property) {
            return Err(format!("unsupported property: {property}"));
        }

        filters.push(ScopeFilter::new(property, op, values));
    }

    Ok(ScopeConstraint::new(filters))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::constraints::{EqPredicate, InPredicate};
    use crate::models::EvaluationResponseContext;
    use modkit_security::pep_properties;
    use uuid::Uuid;

    fn uuid(s: &str) -> Uuid {
        Uuid::parse_str(s).unwrap()
    }

    const T1: &str = "11111111-1111-1111-1111-111111111111";
    const T2: &str = "22222222-2222-2222-2222-222222222222";
    const R1: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";

    const DEFAULT_PROPS: &[&str] = &[pep_properties::OWNER_TENANT_ID, pep_properties::RESOURCE_ID];

    // === Constraint Compilation Matrix Tests ===

    #[test]
    fn no_require_constraints_empty_returns_allow_all() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext::default(),
        };

        let scope = compile_to_access_scope(&response, false, DEFAULT_PROPS).unwrap();
        assert!(scope.is_unconstrained());
    }

    #[test]
    fn no_require_constraints_with_constraints_compiles_them() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![Constraint {
                    predicates: vec![Predicate::Eq(EqPredicate {
                        property: pep_properties::OWNER_TENANT_ID.to_owned(),
                        value: uuid(T1),
                    })],
                }],
                ..Default::default()
            },
        };

        let scope = compile_to_access_scope(&response, false, DEFAULT_PROPS).unwrap();
        assert!(!scope.is_unconstrained());
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID),
            &[uuid(T1)]
        );
    }

    #[test]
    fn decision_true_require_constraints_empty_returns_error() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext::default(),
        };

        let result = compile_to_access_scope(&response, true, DEFAULT_PROPS);
        assert!(matches!(
            result,
            Err(ConstraintCompileError::ConstraintsRequiredButAbsent)
        ));
    }

    // === Constraint Compilation Tests ===

    #[test]
    fn single_tenant_eq_constraint() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![Constraint {
                    predicates: vec![Predicate::Eq(EqPredicate {
                        property: pep_properties::OWNER_TENANT_ID.to_owned(),
                        value: uuid(T1),
                    })],
                }],
                ..Default::default()
            },
        };

        let scope = compile_to_access_scope(&response, true, DEFAULT_PROPS).unwrap();
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID),
            &[uuid(T1)]
        );
        assert!(scope.all_values_for(pep_properties::RESOURCE_ID).is_empty());

        // Verify Predicate::Eq produces FilterOp::Eq (not In)
        let filter = &scope.constraints()[0].filters()[0];
        assert_eq!(*filter.op(), FilterOp::Eq);
    }

    #[test]
    fn multiple_tenants_in_constraint() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![Constraint {
                    predicates: vec![Predicate::In(InPredicate {
                        property: pep_properties::OWNER_TENANT_ID.to_owned(),
                        values: vec![uuid(T1), uuid(T2)],
                    })],
                }],
                ..Default::default()
            },
        };

        let scope = compile_to_access_scope(&response, true, DEFAULT_PROPS).unwrap();
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID),
            &[uuid(T1), uuid(T2)]
        );
    }

    #[test]
    fn resource_id_eq_constraint() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![Constraint {
                    predicates: vec![Predicate::Eq(EqPredicate {
                        property: pep_properties::RESOURCE_ID.to_owned(),
                        value: uuid(R1),
                    })],
                }],
                ..Default::default()
            },
        };

        let scope = compile_to_access_scope(&response, true, DEFAULT_PROPS).unwrap();
        assert!(
            scope
                .all_values_for(pep_properties::OWNER_TENANT_ID)
                .is_empty()
        );
        assert_eq!(
            scope.all_values_for(pep_properties::RESOURCE_ID),
            &[uuid(R1)]
        );

        // Verify Predicate::Eq produces FilterOp::Eq
        let filter = &scope.constraints()[0].filters()[0];
        assert_eq!(*filter.op(), FilterOp::Eq);
    }

    #[test]
    fn multiple_constraints_produce_or_scope() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![
                    Constraint {
                        predicates: vec![Predicate::In(InPredicate {
                            property: pep_properties::OWNER_TENANT_ID.to_owned(),
                            values: vec![uuid(T1)],
                        })],
                    },
                    Constraint {
                        predicates: vec![Predicate::In(InPredicate {
                            property: pep_properties::OWNER_TENANT_ID.to_owned(),
                            values: vec![uuid(T2)],
                        })],
                    },
                ],
                ..Default::default()
            },
        };

        let scope = compile_to_access_scope(&response, true, DEFAULT_PROPS).unwrap();
        // Each constraint is a separate ScopeConstraint (ORed)
        assert_eq!(scope.constraints().len(), 2);
        // Both tenants accessible
        assert!(scope.contains_value(pep_properties::OWNER_TENANT_ID, uuid(T1)));
        assert!(scope.contains_value(pep_properties::OWNER_TENANT_ID, uuid(T2)));
    }

    #[test]
    fn unknown_predicate_fails_constraint() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![Constraint {
                    predicates: vec![Predicate::Eq(EqPredicate {
                        property: "unknown_property".to_owned(),
                        value: uuid(T1),
                    })],
                }],
                ..Default::default()
            },
        };

        let result = compile_to_access_scope(&response, true, DEFAULT_PROPS);
        assert!(matches!(
            result,
            Err(ConstraintCompileError::AllConstraintsFailed { .. })
        ));
    }

    #[test]
    fn mixed_known_and_unknown_constraints() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![
                    // This constraint has an unknown property → fails
                    Constraint {
                        predicates: vec![Predicate::Eq(EqPredicate {
                            property: "group_id".to_owned(),
                            value: uuid(T1),
                        })],
                    },
                    // This constraint is valid → succeeds
                    Constraint {
                        predicates: vec![Predicate::In(InPredicate {
                            property: pep_properties::OWNER_TENANT_ID.to_owned(),
                            values: vec![uuid(T2)],
                        })],
                    },
                ],
                ..Default::default()
            },
        };

        // Should succeed — the second constraint compiled
        let scope = compile_to_access_scope(&response, true, DEFAULT_PROPS).unwrap();
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID),
            &[uuid(T2)]
        );
    }

    #[test]
    fn both_tenant_and_resource_in_single_constraint() {
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![Constraint {
                    predicates: vec![
                        Predicate::In(InPredicate {
                            property: pep_properties::OWNER_TENANT_ID.to_owned(),
                            values: vec![uuid(T1)],
                        }),
                        Predicate::Eq(EqPredicate {
                            property: pep_properties::RESOURCE_ID.to_owned(),
                            value: uuid(R1),
                        }),
                    ],
                }],
                ..Default::default()
            },
        };

        let scope = compile_to_access_scope(&response, true, DEFAULT_PROPS).unwrap();
        // Single constraint with both properties (AND)
        assert_eq!(scope.constraints().len(), 1);
        assert_eq!(
            scope.all_values_for(pep_properties::OWNER_TENANT_ID),
            &[uuid(T1)]
        );
        assert_eq!(
            scope.all_values_for(pep_properties::RESOURCE_ID),
            &[uuid(R1)]
        );
    }

    #[test]
    fn mixed_shape_constraints_produce_or_scope() {
        // T1+R1 (AND) OR T2 — two different-shaped constraints
        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![
                    Constraint {
                        predicates: vec![
                            Predicate::In(InPredicate {
                                property: pep_properties::OWNER_TENANT_ID.to_owned(),
                                values: vec![uuid(T1)],
                            }),
                            Predicate::Eq(EqPredicate {
                                property: pep_properties::RESOURCE_ID.to_owned(),
                                value: uuid(R1),
                            }),
                        ],
                    },
                    Constraint {
                        predicates: vec![Predicate::In(InPredicate {
                            property: pep_properties::OWNER_TENANT_ID.to_owned(),
                            values: vec![uuid(T2)],
                        })],
                    },
                ],
                ..Default::default()
            },
        };

        let scope = compile_to_access_scope(&response, true, DEFAULT_PROPS).unwrap();
        assert_eq!(scope.constraints().len(), 2);
        // First constraint has 2 filters (AND), second has 1 filter
        assert_eq!(scope.constraints()[0].filters().len(), 2);
        assert_eq!(scope.constraints()[1].filters().len(), 1);
    }

    #[test]
    fn supported_properties_validation() {
        // Only owner_tenant_id is supported — id should fail
        let limited_props: &[&str] = &[pep_properties::OWNER_TENANT_ID];

        let response = EvaluationResponse {
            decision: true,
            context: EvaluationResponseContext {
                constraints: vec![Constraint {
                    predicates: vec![Predicate::Eq(EqPredicate {
                        property: pep_properties::RESOURCE_ID.to_owned(),
                        value: uuid(R1),
                    })],
                }],
                ..Default::default()
            },
        };

        let result = compile_to_access_scope(&response, true, limited_props);
        assert!(matches!(
            result,
            Err(ConstraintCompileError::AllConstraintsFailed { .. })
        ));
    }
}
