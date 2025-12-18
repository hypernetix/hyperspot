# 0004 TECH — Define SDK error type

## Title
Define the SDK error type for feature flag evaluation

## Description
Introduce a transport-agnostic error type in the SDK crate to represent failures of feature flag evaluation.

This task does not implement evaluation logic.

## Acceptance Criteria
- The SDK crate exposes a single error type used by the API trait.
- The error type supports:
  - invalid feature flag identifier
- YAGNI note: backend unavailable / internal errors are deferred for the stub milestone (no backend integration).
- The error type is transport-agnostic (no HTTP status codes, no REST DTOs).
