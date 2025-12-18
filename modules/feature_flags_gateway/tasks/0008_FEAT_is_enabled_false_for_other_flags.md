# 0008 FEAT — Other flags are disabled

## Title
Return enabled=false for any other valid feature flag

## Description
Extend the stub evaluation so that any feature flag identifier other than the global base flag is treated as disabled.

Type Registry integration is out of scope; the implementation is a stub.

## Behavior (BDD)
- Given a valid `SecurityCtx`
- When `is_enabled` is called with a valid feature flag identifier that is not the global base flag
- Then the result is disabled

## Acceptance Criteria
- `is_enabled` returns enabled=false for any valid flag not equal to the global base flag.
- The behavior remains independent of any Type Registry implementation.
- The behavior does not depend on tenant/user attributes yet.
