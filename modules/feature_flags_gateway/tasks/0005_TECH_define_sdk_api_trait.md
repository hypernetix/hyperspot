# 0005 TECH — Define SDK API trait

## Title
Define `FeatureFlagsApi` trait in the SDK crate

## Description
Define the public module API as a transport-agnostic async interface intended to be resolved from ClientHub.

This task does not implement evaluation behavior.

## Acceptance Criteria
- The SDK crate exposes `FeatureFlagsApi`.
- All API methods accept `SecurityCtx` as the first parameter.
- The API surface includes:
  - a single-flag evaluation method
  - a batch evaluation method
- The trait methods return `Result` using the SDK error type.
