# Implementation Tasks

## 1. Core Debug Logging in Registration Logic

- [x] 1.1 Add debug logging in schema validation to dump schema content on validation failure
- [x] 1.2 Add debug logging in instance validation to dump instance content on validation failure
- [x] 1.3 Add helper method to collect and log parent schema chain for instances
- [x] 1.4 Ensure all logged JSON is pretty-printed using `serde_json::to_string_pretty`

## 2. Debug Logging in Registration Entry Points

- [x] 2.1 Add debug logging in `register` method when entity registration fails
- [x] 2.2 Log the complete entity content that was attempted to be registered
- [x] 2.3 Include GTS ID (if extractable) in all diagnostic log messages

## 3. Schema Chain Resolution

- [x] 3.1 Implement helper function to walk schema inheritance chain via `$ref` fields
      - Must collect distinct schema objects without inlining them, to preserve individual schema structure for debugging
- [x] 3.2 Log each schema in the chain with clear labels including depth/role (e.g., "Depth 0 (Root)", "Depth 1 (Ref)")
- [x] 3.3 Implement robust cycle detection (using visited IDs set) to prevent infinite recursion and log warnings if cycles are found

## 4. Unit Testing

- [x] 4.1 Add unit test for debug logging on schema validation failure
- [x] 4.2 Add unit test for debug logging on instance validation failure
- [x] 4.3 Add unit test for schema chain logging with multiple parent schemas
- [x] 4.4 Verify logs are only emitted at debug level (not info/warn/error)

## 5. E2E Testing

- [x] 5.1 Create `testing/e2e/modules/types_registry/test_registration_debug_logging.py`
- [x] 5.2 Add E2E test: register invalid schema, verify debug logs contain schema JSON dump
- [x] 5.3 Add E2E test: register instance with schema mismatch, verify debug logs contain instance + schema dumps
- [x] 5.4 Add E2E test: register instance with multi-level schema chain, verify all parent schemas logged with depth labels
- [x] 5.5 Add E2E test: verify debug logs are NOT present when running with info log level
- [x] 5.6 Add E2E test: circular schema reference scenario, verify cycle detection warning logged

## 6. Validation & Quality

- [x] 6.1 Run `cargo fmt --all`
- [x] 6.2 Run `cargo clippy --workspace --all-targets`
- [x] 6.3 Run `cargo test --workspace`
- [x] 6.4 Manually verify debug output with a failing registration scenario
