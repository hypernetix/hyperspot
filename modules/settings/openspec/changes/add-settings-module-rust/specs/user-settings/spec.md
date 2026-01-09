# User Settings Specification

## ADDED Requirements

### Requirement: User Settings Storage

The system SHALL store user-specific settings (theme and language preferences) with automatic tenant and user isolation via `SecurityContext`.

#### Scenario: Settings are tenant and user scoped
- **WHEN** a user retrieves their settings via `SecurityContext`
- **THEN** the system returns only settings for that specific user and tenant combination
- **AND** settings from other users or tenants are not accessible

#### Scenario: Default settings for new users
- **WHEN** a user retrieves settings for the first time (no record exists)
- **THEN** the system returns empty/default values for theme and language
- **AND** no database record is created until the user performs an update

### Requirement: Retrieve User Settings

The system SHALL provide a REST endpoint to retrieve the current user's settings using `SecurityContext` for authentication and authorization.

#### Scenario: Successful settings retrieval
- **WHEN** an authenticated user sends `GET /settings/v1/settings`
- **THEN** the system returns HTTP 200 with the user's settings (theme, language)
- **AND** the response uses camelCase JSON field naming

#### Scenario: First-time user retrieval
- **WHEN** a new user (no existing settings record) sends `GET /settings/v1/settings`
- **THEN** the system returns HTTP 200 with empty default values
- **AND** no database insert occurs

#### Scenario: Unauthorized access
- **WHEN** an unauthenticated request is sent to `GET /settings/v1/settings`
- **THEN** the system returns HTTP 401 Unauthorized

### Requirement: Full Update User Settings

The system SHALL provide a REST endpoint to perform a full update of user settings, replacing all fields.

#### Scenario: Successful full update
- **WHEN** an authenticated user sends `POST /settings/v1/settings` with theme and language
- **THEN** the system updates or creates the settings record
- **AND** returns HTTP 200 with the updated settings
- **AND** both theme and language are set to the provided values

#### Scenario: Create settings on first update
- **WHEN** a user without existing settings sends `POST /settings/v1/settings`
- **THEN** the system creates a new settings record
- **AND** returns HTTP 200 with the created settings

#### Scenario: Update existing settings
- **WHEN** a user with existing settings sends `POST /settings/v1/settings`
- **THEN** the system updates the existing record (not create duplicate)
- **AND** returns HTTP 200 with the updated settings

#### Scenario: Unauthorized update
- **WHEN** an unauthenticated request is sent to `POST /settings/v1/settings`
- **THEN** the system returns HTTP 401 Unauthorized

### Requirement: Partial Update User Settings

The system SHALL provide a REST endpoint to partially update user settings, modifying only the fields provided in the request.

#### Scenario: Update only theme
- **WHEN** an authenticated user sends `PATCH /settings/v1/settings` with only theme field
- **THEN** the system updates only the theme value
- **AND** the language value remains unchanged
- **AND** returns HTTP 200 with the complete updated settings

#### Scenario: Update only language
- **WHEN** an authenticated user sends `PATCH /settings/v1/settings` with only language field
- **THEN** the system updates only the language value
- **AND** the theme value remains unchanged
- **AND** returns HTTP 200 with the complete updated settings

#### Scenario: Update both fields via PATCH
- **WHEN** an authenticated user sends `PATCH /settings/v1/settings` with both theme and language
- **THEN** the system updates both values
- **AND** returns HTTP 200 with the complete updated settings

#### Scenario: Patch creates settings if not exists
- **WHEN** a user without existing settings sends `PATCH /settings/v1/settings`
- **THEN** the system creates a new settings record with provided fields
- **AND** unspecified fields remain empty/default
- **AND** returns HTTP 200 with the created settings

#### Scenario: Unauthorized partial update
- **WHEN** an unauthenticated request is sent to `PATCH /settings/v1/settings`
- **THEN** the system returns HTTP 401 Unauthorized

### Requirement: Security Context Integration

The system SHALL use `SecurityContext` to automatically extract tenant_id and user_id without requiring these values in the request payload or URL.

#### Scenario: Tenant isolation enforcement
- **WHEN** a user from tenant A attempts to access settings
- **THEN** the system only returns/modifies settings for tenant A
- **AND** settings from other tenants are never accessible regardless of user_id

#### Scenario: User isolation enforcement
- **WHEN** user X from a tenant attempts to access settings
- **THEN** the system only returns/modifies settings for user X
- **AND** settings from other users in the same tenant are not accessible

### Requirement: Data Model

The system SHALL store user settings with the following fields:
- `user_id` (UUID, part of composite primary key)
- `tenant_id` (UUID, part of composite primary key)
- `theme` (String, optional)
- `language` (String, optional)
- `created_at` (Timestamp)
- `updated_at` (Timestamp)

#### Scenario: Composite primary key uniqueness
- **WHEN** the system stores settings
- **THEN** the combination of (tenant_id, user_id) is unique
- **AND** no duplicate settings records exist for the same user-tenant pair

#### Scenario: Automatic timestamp management
- **WHEN** settings are created
- **THEN** created_at and updated_at are set to current timestamp

#### Scenario: Update timestamp tracking
- **WHEN** settings are modified
- **THEN** updated_at is set to current timestamp
- **AND** created_at remains unchanged

### Requirement: Error Handling

The system SHALL return RFC-9457 Problem Details for all error responses with appropriate HTTP status codes.

#### Scenario: Validation error
- **WHEN** invalid data is provided (e.g., excessively long strings)
- **THEN** the system returns HTTP 400 Bad Request with Problem Details
- **AND** the error includes field-level validation information

#### Scenario: Internal error
- **WHEN** a database error occurs
- **THEN** the system returns HTTP 500 Internal Server Error with Problem Details
- **AND** detailed error information is logged but not exposed to client

### Requirement: Module SDK

The system SHALL provide a separate SDK crate (`settings-sdk`) with transport-agnostic public API for inter-module communication.

#### Scenario: SDK API trait usage
- **WHEN** another module needs to access user settings
- **THEN** it obtains the client via `ClientHub.get::<dyn SettingsApi>()?`
- **AND** calls methods passing `SecurityContext` for authorization

#### Scenario: SDK models have no transport dependencies
- **WHEN** SDK models are defined
- **THEN** they do not include `serde` or other HTTP-specific derives
- **AND** they are plain Rust structs suitable for any transport (gRPC, local, HTTP)

### Requirement: Database Security

The system SHALL use Secure ORM patterns with automatic tenant and user scoping for all database queries.

#### Scenario: Scoped query execution
- **WHEN** a repository method is called with `SecurityContext`
- **THEN** all database queries automatically include tenant_id and user_id filters
- **AND** queries cannot access data outside the security scope

#### Scenario: Deny-by-default empty scope
- **WHEN** a query is attempted without proper security scope
- **THEN** the system generates `WHERE 1=0` query (returns no results)
- **AND** no unauthorized data is exposed
