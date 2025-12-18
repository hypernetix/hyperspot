# 0010 FEAT — Reject invalid feature flag id

## Title
Return an error for feature flag identifiers containing only whitespaces.

## Description
Enforce the required feature flag identifier validation. This prevents accidental usage of invalid feature flag identifiers.

Type Registry integration is out of scope; the implementation is a stub.

## Behavior (BDD)
- Given a valid `SecurityCtx`
- When feature flag evaluation is requested for an identifier that is empty or contains only whitespaces
- Then the call returns an invalid-identifier error

## Acceptance Criteria
- Invalid identifiers (empty/whitespace) are rejected consistently for single-flag evaluation.
- Invalid identifiers (empty/whitespace) are rejected consistently for batch evaluation.
- The error returned is the SDK error type’s invalid-identifier variant.
