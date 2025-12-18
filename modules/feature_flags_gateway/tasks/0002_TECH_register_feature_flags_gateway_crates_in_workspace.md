# 0002 TECH — Register crates in workspace

## Title
Register `feature_flags_gateway` crates in the root workspace

## Description
Add the new `feature_flags_gateway` crates to the root workspace configuration so they are built and linted as part of the repository.

This task is only about workspace wiring.

## Acceptance Criteria
- The root workspace member list includes:
  - `modules/feature_flags_gateway/feature_flags_gateway-sdk`
  - `modules/feature_flags_gateway/feature_flags_gateway`
- Workspace build/lint tooling can discover these crates.
- No code or behavior changes are introduced outside of workspace membership.
