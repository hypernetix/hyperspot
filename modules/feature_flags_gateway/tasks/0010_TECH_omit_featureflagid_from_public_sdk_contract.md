# 0010 TECH — Omit FeatureFlagId from the public SDK contract

## Title
Omit `FeatureFlagId` from the public SDK contract and use raw string ids

## Description
Align the public SDK surface with the decision that feature flag identifiers are passed as raw strings (`&str` / `String`) across module boundaries.

If a `FeatureFlagId` newtype exists, it should not be required by (or exposed as part of) the SDK contract that consumers depend on.

This task is documentation and contract alignment only; it does not introduce Type Registry integration.

## Acceptance Criteria
- The SDK public API does not require consumers to construct or pass a `FeatureFlagId` type.
- The SDK exposes a well-known constant for the global base feature flag identifier.
- The module spec (`SPEC.md`) describes feature flag identifiers as raw strings for the stub milestone.
- Any module tasks/spec sections referencing `FeatureFlagId` as a required public type are updated to match this contract.
