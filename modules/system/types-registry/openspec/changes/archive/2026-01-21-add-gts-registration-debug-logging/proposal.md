# Change: Add Debug Logging for GTS Entity Registration Failures

## Why

When GTS entity registration fails, the current error messages provide limited context, making it difficult for developers and AI models to diagnose issues while learning GTS or developing new modules. Enhanced debug logging would dump the entity being registered and its schema hierarchy, significantly improving the debugging experience.

## What Changes

- Add debug-level logging when GTS entity registration fails
- Dump the complete GTS entity content being registered on failure
- For instance registration failures, dump the instance schema (if found) and all parent schemas in the inheritance chain
- Use `tracing::debug!` to ensure logs are only visible when debug logging is enabled
- Format logged schemas as pretty-printed JSON for human/model readability

## Impact

- Affected specs: `types-registry` (adds diagnostic logging to existing registration)
- Affected code:
  - `modules/system/types-registry/types-registry/src/` (registration and validation logic)
