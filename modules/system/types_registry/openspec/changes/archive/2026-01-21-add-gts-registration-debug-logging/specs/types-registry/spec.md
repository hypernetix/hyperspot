## ADDED Requirements

### Requirement: Debug Logging for Failed Entity Registration

The system SHALL emit debug-level log messages when GTS entity registration fails, including the complete entity content being registered.

#### Scenario: Schema registration validation failure

- **GIVEN** a GTS schema with invalid JSON Schema syntax
- **WHEN** the schema is registered via `register`
- **AND** validation fails
- **THEN** a debug log message SHALL be emitted containing the complete schema JSON (pretty-printed)
- **AND** the log message SHALL include the GTS ID of the schema being registered

#### Scenario: Instance registration validation failure

- **GIVEN** a GTS instance that does not conform to its schema
- **WHEN** the instance is registered via `register` with validation enabled
- **AND** validation fails
- **THEN** a debug log message SHALL be emitted containing the complete instance JSON (pretty-printed)
- **AND** a debug log message SHALL be emitted containing the instance's schema JSON (if found)
- **AND** the log message SHALL include the GTS ID of the instance being registered

### Requirement: Schema Chain Logging for Instance Failures

The system SHALL emit debug-level log messages containing the complete schema inheritance chain when an instance validation fails.

#### Scenario: Instance with schema inheritance chain

- **GIVEN** a GTS instance whose schema references parent schemas via `$ref`
- **WHEN** the instance validation fails
- **THEN** debug log messages SHALL be emitted for each schema in the inheritance chain
- **AND** each schema SHALL be logged as a distinct JSON object (not inlined) to preserve structure
- **AND** each schema SHALL be labeled with its position/depth in the chain (e.g., "Depth 0 (Instance Schema):", "Depth 1 (Ref Schema):")
- **AND** schemas SHALL be pretty-printed as JSON

### Requirement: Circular Reference Protection

The system SHALL handle circular references in schema chains gracefully during diagnostic logging.

#### Scenario: Circular schema reference handling

- **GIVEN** a schema chain that contains circular references (e.g., A refs B, B refs A)
- **WHEN** logging the schema chain
- **THEN** the system SHALL detect the cycle using a visited set of IDs
- **AND** the system SHALL stop traversal for that branch
- **AND** the system SHALL log a warning indicating a cycle was detected (e.g., "Cycle detected at ID: ...")
- **AND** the system SHALL NOT enter an infinite loop

### Requirement: Debug Log Level Enforcement

Debug logging for GTS registration diagnostics SHALL only be emitted at the `debug` log level, ensuring it does not affect production log volume when debug logging is disabled.

#### Scenario: Default log level excludes diagnostic logs

- **GIVEN** the application is running with default log level (info or higher)
- **WHEN** a GTS entity registration fails
- **THEN** diagnostic dump logs SHALL NOT appear in the output
- **AND** only the standard error message SHALL be returned
