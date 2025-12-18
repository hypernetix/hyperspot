# 0006 FEAT — Register stub gateway in ClientHub

## Title
Register a stub `FeatureFlagsApi` implementation into ClientHub during module init

## Description
Make the module provide a usable in-process gateway by registering a `FeatureFlagsApi` implementation into ClientHub when the module is initialized.

This is the first end-to-end behavior slice for the module.

## Behavior (BDD)
- Given the `feature_flags_gateway` module is part of the running process
- When the module initialization completes successfully
- Then a consumer can resolve `FeatureFlagsApi` from ClientHub

## Acceptance Criteria
- The module registers exactly one `FeatureFlagsApi` implementation into ClientHub.
- Consumers can obtain it by type from ClientHub.
- No other modules are modified.
