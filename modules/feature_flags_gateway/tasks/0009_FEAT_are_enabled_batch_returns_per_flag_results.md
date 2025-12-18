# 0009 FEAT — Batch evaluation

## Title
Return per-flag results from the batch evaluation method

## Description
Implement the batch evaluation method so callers can evaluate multiple feature flags in a single call.

The stub behavior must be consistent with the single-flag evaluation behavior.

## Behavior (BDD)
- Given a valid `SecurityCtx`
- When the batch evaluation method is called with a list of feature flag identifiers
- Then the response contains one result per input flag
- And each result matches what the single-flag evaluation method would return for the same flag

## Acceptance Criteria
- The batch method returns a result for every input flag.
- The batch method is consistent with the single-flag method.
- The batch method does not require Type Registry.
