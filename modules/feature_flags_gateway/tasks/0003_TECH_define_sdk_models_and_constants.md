# 0003 TECH — Define SDK models and constants

## Title
Define the SDK-level constants for feature flags

## Description
In the SDK crate, define the minimal transport-agnostic constants needed by consumers.

The SDK contract uses raw string identifiers (`&str` / `String`) and does not require a dedicated `FeatureFlagId` model.

This task does not implement evaluation logic.

## Acceptance Criteria
- The SDK crate exposes well-known feature flag constants.
- The SDK crate exposes a constant for the global base feature flag identifier:
  - `gts.x.core.ff.flag.v1~x.core.global.base.v1`
- The SDK types remain transport-agnostic (no REST/gRPC specifics).
