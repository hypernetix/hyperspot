# 0007 FEAT — GlobalBase is enabled

## Title
Return enabled=true for the global base feature flag

## Description
Implement the stub feature flag evaluation behavior so that the global base feature flag is treated as enabled.

Type Registry integration is out of scope; the implementation is a stub.

## Behavior (BDD)
- Given a valid `SecurityCtx`
- When `is_enabled` is called with the global base feature flag identifier
- Then the result is enabled

## Acceptance Criteria
- `is_enabled` returns enabled=true for `gts.x.core.ff.flag.v1~x.core.global.base.v1`.
- The result does not depend on any Type Registry implementation.
- No other feature flags are enabled by this task.
