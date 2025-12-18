# 0001 TECH — Create SDK and module crates

## Title
Create SDK and module crates for `feature_flags_gateway`

## Description
Create the minimal crate structure for the new `feature_flags_gateway` module following the repository module guideline (SDK pattern).

This task only establishes the project skeleton and does not implement functional behavior.

## Acceptance Criteria
- A new SDK crate exists at `modules/feature_flags_gateway/feature_flags_gateway-sdk/`.
- A new module crate exists at `modules/feature_flags_gateway/feature_flags_gateway/`.
- Each crate contains a `Cargo.toml` and a minimal `src/lib.rs`.
- The module crate depends on the SDK crate.
- No other modules are modified.
