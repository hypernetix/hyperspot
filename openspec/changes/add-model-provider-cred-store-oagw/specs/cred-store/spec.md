## ADDED Requirements

### Requirement: cred_store module responsibility
The system SHALL provide a `cred_store` gateway module with a **single active plugin (lowest priority)** responsible for tenant-scoped secret storage and retrieval.

The `cred_store` module SHALL be generic (not GenAI-specific) and reusable by other modules.

#### Scenario: Generic usage
- **WHEN** `oagw` needs a secret for outbound auth
- **THEN** it references `cred_store` secrets by UUID and `credential_type` (GTS instance id)
- **AND** `cred_store` does not assume inference-specific semantics

### Requirement: cred_store GTS schemas and instances registration
The system SHALL register the following GTS schema in `types_registry` during gateway startup:
- `gts.x.core.cred_store.secret_type.v1~`

The base schema SHALL define:
- `id: GtsInstanceId`
- `secret_schema: string` (JSON schema for secret material for the nested secret type, defined as GTS nested schema)

Plus additional traits:
- `display_name: string`
- `description?: string`
- `versioning_enabled: boolean` (global default)
- `last_versions: int` (global default 5)
- `retention_days: int` (global default)
- `encryption_required: boolean` (global default)
- `audit_retention_days: int` (global default)

There must be also predefined secret types for the following use cases:
- `gts.x.core.cred_store.secret_type.v1~x.core.db.password.v1~`
- `gts.x.core.cred_store.secret_type.v1~x.core.db.api_key.v1~`
- `gts.x.core.cred_store.secret_type.v1~x.core.ssh.private_key.v1~`
- `gts.x.core.cred_store.secret_type.v1~x.core.ssh.public_key.v1~`
- `gts.x.core.cred_store.secret_type.v1~x.core.ssh.passphrase.v1~`
- `gts.x.core.cred_store.secret_type.v1~x.core.pci.cardholder_data.v1~`
- `gts.x.core.cred_store.secret_type.v1~x.core.phi.secret.v1~`
- `gts.x.core.cred_store.secret_type.v1~x.core.api.secret.v1~`
- `gts.x.core.cred_store.secret_type.v1~x.core.user.password.v1~`

Well-known secret types SHALL be stored under `cred_store/.../gts/*.instances.json` and registered during gateway module `init()`. THe well-known secret types should define proper defaults for all the fields and define schema of the secret material.

#### Scenario: Startup registration
- **WHEN** `cred_store` gateway module `init()` runs
- **THEN** it registers the secret type schema and all shipped instances from its `gts/` folder

### Requirement: cred_store persistence model
The system SHALL persist actual per-tenant and per-user secret records as anonymous objects in the `cred_store` database.

The persistent model SHALL include:
- `secret_config` (per tenant settings for secret types)
  - `id uuid pk`
  - `type GtsInstanceId`
  - `versioning_enabled bool`
  - `last_versions int` (default 5)
  - `retention_days int`
  - `encryption_required bool`
  - `audit_retention_days int`

- `secret_store`
  - `id uuid pk`
  - `tenant_id uuid not null`
  - `client_id uuid null`
  - `secret_type text not null` (GTS instance id from `gts.x.core.cred_store.secret_type.v1~`)
  - `secret_ciphertext bytes not null` (recommended; plugin-dependent)
  - `parameters jsonb null` (client-dependent, tenants/users/integrations can store it's own attributes here, the schema need to be defined separately for each secret type `gts.x.core.cred_store.secret_type.v1~vendor.pkg._.secrets_extension.v1~`)
  - `created_at timestamptz not null`
  - `updated_at timestamptz not null`

The system SHOULD avoid storing plaintext secrets in the database.

#### Scenario: Store a new secret
- **WHEN** a client creates a secret
- **THEN** the stored record contains encrypted secret material
- **AND** the gateway does not log the secret value

### Requirement: cred_store gateway API (Rust-native)
The system SHALL expose a Rust-native API (ClientHub) named `CredStoreApi`.

The API SHALL:
- Accept `&SecurityCtx` for every method.
- Support create/get/list/delete operations for secrets with proper Access control (role check, DB-level tenant/user scope)

#### Scenario: Rust-native read
- **WHEN** a consumer calls `CredStoreApi::get_secret(ctx, secret_id, secret_type_id)`
- **THEN** the gateway enforces tenant and user (optional) scope via `SecurityCtx`
- **AND** the gateway enforces secret type scope via `secret_type_id`
- **AND** the gateway returns the secret material

### Requirement: cred_store REST API
The system SHALL expose a REST API:
- `GET/POST /cred-store/v1/secrets`
- `GET/DELETE /cred-store/v1/secrets/{secretId}`
The system must distinguish global per-tenant secret and per-user secret types.

REST responses SHOULD avoid returning secret plaintext except in explicitly authorized retrieval modes.

#### Scenario: REST list secrets
- **WHEN** a client calls `GET /cred-store/v1/secrets`
- **THEN** the response omits plaintext secret values

### Requirement: cred_store plugin interface and selection
The system SHALL define a plugin interface for secret storage backends.

The plugin interface SHALL be expressed as a Rust-native async trait in the `cred_store` plugin SDK.

The plugin SDK SHALL define a `CredStorePluginApi` trait where all methods accept `&SecurityCtx`.

The plugin trait SHALL support at least:
- `upsert_secret(ctx, secret_id, secret_type_id, secret_ciphertext, parameters) -> ()`
- `get_secret_material(ctx, secret_id, secret_type_id) -> SecretMaterial`
- `delete_secret(ctx, secret_id, secret_type_id) -> ()`

The plugin trait SHALL treat `secret_id` as an opaque UUID and SHALL require `secret_type_id` to prevent accidental cross-type retrieval.

The plugin SDK SHALL define a plugin spec schema type (for plugin instance registration) as a derived GTS schema:
- `CredStorePluginSpecV1` with schema id `gts.x.core.modkit.plugin.v1~x.core.cred_store.plugin.v1~`

Each plugin module SHALL register exactly one or more plugin instances in `types_registry` using:
- `BaseModkitPluginV1<CredStorePluginSpecV1>`

Each registered plugin instance SHALL include:
- `id: GtsInstanceId` (full instance id)
- `vendor: string`
- `priority: int` (lower wins)
- `properties: CredStorePluginSpecV1` can be empty

Each plugin module SHALL also register a scoped client in `ClientHub` under `ClientScope::gts_id(&instance_id)`.

Only one plugin SHALL be active at runtime: the eligible plugin with the lowest `priority` value among registered instances.

Eligibility MUST be determined by priority (lower priority wins).

#### Scenario: Single active plugin
- **GIVEN** multiple cred_store plugins are registered
- **WHEN** the gateway resolves its plugin
- **THEN** it chooses the plugin with the lowest priority

#### Scenario: Scoped client resolution
- **GIVEN** the gateway selected plugin instance id `gts.x.core.modkit.plugin.v1~x.core.cred_store.plugin.v1~vendor.pkg.plugin.v1`
- **WHEN** the gateway requests a client via `ClientHub.get_scoped::<dyn CredStorePluginApi>(ClientScope::gts_id(&plugin_instance_id))`
- **THEN** the plugin client is resolved and used to serve the request

### Requirement: cred_store plugin persistence, access control, and cryptography
The system SHALL define how `cred_store` plugins persist secrets, enforce access control, and apply cryptography.

The gateway SHALL be responsible for:
- Performing request authentication and high-level authorization (role/permission checks) before calling the plugin.
- Passing request-scoped `SecurityCtx` into the plugin.

Each plugin SHALL be responsible for:
- Enforcing tenant/user scoping when it interacts with any persistent store.
- Ensuring that secret material is never logged.

#### Access control (DB-backed plugins)
For DB-backed plugins, access control SHALL be enforced using the `modkit-db` secure ORM layer.

DB-backed plugins SHALL:
- Use a secure DB access wrapper (e.g., `SecureConn`) for all queries.
- Derive an `AccessScope` from the request `SecurityCtx` and apply it to all queries.
- Use scoped CRUD operations that automatically produce deny-by-default behavior for empty scopes.

#### Scenario: DB-backed plugin denies cross-tenant reads
- **GIVEN** a secret record exists for tenant A with secret id `S`
- **WHEN** a caller from tenant B calls `CredStorePluginApi::get_secret_material(ctx, S, secret_type_id)`
- **THEN** the plugin query is scoped to tenant B
- **AND** the plugin returns a not-found or permission-denied error

#### Cryptography (DB-backed plugins)
DB-backed plugins SHALL encrypt secret material before persisting it.

DB-backed plugins SHOULD implement envelope encryption:
- A per-record Data Encryption Key (DEK) encrypts the secret plaintext
- A Key Encryption Key (KEK) encrypts (wraps) the DEK

The system SHOULD support the following AEAD algorithms for DEK encryption:
- `AES-256-GCM`
- `XChaCha20-Poly1305`

The plugin SHOULD bind context into Additional Authenticated Data (AAD), including:
- `tenant_id`
- `client_id` (if present)
- `secret_id`
- `secret_type_id`

#### Scenario: Envelope encryption
- **WHEN** the plugin persists secret material
- **THEN** the plugin encrypts the plaintext with a per-record DEK using an AEAD cipher
- **AND** the plugin stores only ciphertext and encryption metadata
- **AND** the plugin stores the DEK only in wrapped form (encrypted by KEK)

### Requirement: cred_store SQLite-backed plugin
The system SHALL support a `cred_store` plugin implementation that persists secrets in a local database (SQLite/SeaORM).

The SQLite-backed plugin SHALL store only encrypted secret material in the database.

#### SQLite-backed plugin tables (logical)
The plugin SHALL persist at least the following tables.

- `cred_store_secret_blob`
  - `id uuid pk` (same identifier as `cred_store.secret_store.id`)
  - `tenant_id uuid not null`
  - `client_id uuid null`
  - `secret_type_id uuid not null` (GTS instance UUID5 obtained by gts-rust lib)
  - `ciphertext bytes not null`
  - `cipher_alg text not null` (e.g., `AES-256-GCM`)
  - `nonce bytes not null`
  - `aad bytes not null`
  - `wrapped_dek bytes not null`
  - `kek_id text not null` (key identifier; plugin-defined)
  - `created_at timestamptz not null`
  - `updated_at timestamptz not null`

- `cred_store_secret_kek_state`
  - `kek_id text pk`
  - `kek_provider text not null` (e.g., `env`, `os_keychain`, `vault_transit`)
  - `kek_version int not null`
  - `created_at timestamptz not null`

The SQLite-backed plugin SHOULD add indexes for hot paths:
- `(tenant_id, id)`
- `(tenant_id, id, client_id)`
- `(tenant_id, client_id, secret_type_id)`

#### Scenario: SQLite-backed plugin write
- **WHEN** the gateway calls `CredStorePluginApi::upsert_secret(ctx, secret_id, secret_type_id, secret_ciphertext, parameters)`
- **THEN** the plugin scopes the operation to the tenant/user in `SecurityCtx`
- **AND** the plugin writes encryption metadata and ciphertext to `cred_store_secret_blob`

#### Scenario: SQLite-backed plugin read
- **WHEN** the gateway calls `CredStorePluginApi::get_secret_material(ctx, secret_id, secret_type_id)`
- **THEN** the plugin loads the row from `cred_store_secret_blob` under secure ORM scope
- **AND** the plugin unwraps the DEK using the configured KEK
- **AND** the plugin decrypts ciphertext using the configured AEAD algorithm

### Requirement: cred_store Hashicorp Vault-backed plugin
The system SHALL support a `cred_store` plugin implementation that uses Hashicorp Vault as the backend.

The Vault-backed plugin SHALL avoid persisting secret plaintext in the HyperSpot database.

The Vault-backed plugin SHOULD use one of the following Vault mechanisms:
- Vault KV v2 for storing secret values (Vault handles encryption-at-rest)
- Vault Transit for encryption/decryption (HyperSpot DB stores only ciphertext)

#### Vault-backed plugin tables (logical)
The plugin SHALL persist references required to locate the secret in Vault.

- `cred_store_vault_secret_ref`
  - `id uuid pk` (same identifier as `cred_store.secret_store.id`)
  - `tenant_id uuid not null`
  - `client_id uuid null`
  - `secret_type_id text not null` (GTS instance id)
  - `vault_mount text not null` (e.g., `kv`, `secret`, `transit`)
  - `vault_path text not null` (logical secret path)
  - `vault_key_name text null` (required for transit)
  - `vault_version int null` (required for kv v2 reads)
  - `created_at timestamptz not null`
  - `updated_at timestamptz not null`

The Vault-backed plugin SHALL enforce access control by scoping the reference table queries (tenant/user) using secure ORM.

#### Scenario: Vault-backed plugin write
- **WHEN** the gateway calls `CredStorePluginApi::upsert_secret(ctx, secret_id, secret_type_id, secret_ciphertext, parameters)`
- **THEN** the plugin scopes the operation to the tenant/user in `SecurityCtx`
- **AND** the plugin writes the secret to Vault under a tenant/user scoped path
- **AND** the plugin stores only a Vault reference in `cred_store_vault_secret_ref`

#### Scenario: Vault-backed plugin read
- **WHEN** the gateway calls `CredStorePluginApi::get_secret_material(ctx, secret_id, secret_type_id)`
- **THEN** the plugin resolves the Vault reference under secure ORM scope
- **AND** the plugin reads/decrypts the secret material from Vault

### Requirement: global and per-tenant configuration

All the parameters, such as timeouts, retries, etc must be configurable globally (as serviec config) and on per-tenant basis.

### Requirement: cred_store audit logging
The system SHALL maintain an immutable audit log for all secret operations.

The system SHALL persist audit records in a separate append-only table.

#### Database table: cred_store_audit_log
- `id uuid pk`
- `tenant_id uuid not null`
- `client_id uuid null`
- `is_user_operation bool not null` (true for user-initiated operations, false for system-initiated operations)
- `operation text not null` (enum: 'create', 'read', 'update', 'delete', 'rotate')
- `secret_id uuid not null`
- `secret_type_id uuid not null`
- `status text not null` (enum: 'success', 'failure')
- `error_code text null`
- `source_ip inet null`
- `user_agent text null`
- `trace_id text null`
- `context_metadata jsonb null`
- `timestamp timestamptz not null default now()`

**Indexes:**
- `(tenant_id, timestamp desc)` for tenant audit queries
- `(secret_id, timestamp desc)` for secret-specific audit trail
- `(client_id, timestamp desc)` for user activity queries

NOTE: table parititioning and archiving and retention policies are not specified in this document. It's a subject for future improvements.

#### Scenario: Audit log write on secret read
- **WHEN** user/client U reads secret S via `get_secret()`
- **THEN** the gateway inserts a row into `cred_store_audit_log`
- **AND** captures (tenant_id, client_id, 'read', secret_id, secret_type_id, 'success', timestamp)
- **AND** includes OpenTelemetry trace_id from current span

#### Scenario: Audit log for failed access
- **WHEN** user U attempts to read secret S but is denied (403)
- **THEN** the gateway logs (client_id, 'read', secret_id, 'failure', 'access_denied')

### Requirement: cred_store audit retention
The system SHOULD support configurable audit log retention policies.

Retention SHALL be configurable per tenant with defaults:
- Default retention: 90 days
- Compliance mode: 7 years (for financial/healthcare tenants)

Audit log cleanup SHOULD run as a background job.

#### Scenario: Audit log cleanup
- **GIVEN** tenant T has retention policy = 90 days (configurable, global, per-tenant)
- **WHEN** cleanup job runs
- **THEN** audit records older than 90 days are archived or purged

### Requirement: cred_store audit export
The system SHALL expose audit log export via REST API.

REST endpoints:
- `GET /cred-store/v1/audit` - List audit logs with OData support
- apply tenant/user/client scope
- require admin role (`gts.x.core.idp.role.v1~x.cred_store.secret.admin.v1`)
- apply OData filters (paging, sorting, filtering)

Export format SHOULD support JSON and CSV.

#### Scenario: Export audit logs for compliance
- **GIVEN** admin U has `gts.x.core.idp.role.v1~x.cred_store.secret.admin.v1` role
- **AND** admin U is scoped to tenant T
- **WHEN** admin U requests `GET /cred-store/v1/audit` with date range filter
- **THEN** system returns paginated audit records with OData support (`$filter`, `$orderby`, `$skip`, `$top`)

### Requirement: cred_store RBAC model
The system SHALL enforce role-based access control for secret operations.

#### Database table: cred_store_role_assignment
- `id uuid pk`
- `tenant_id uuid not null`
- `principal_id uuid not null`
- `principal_type text not null` (enum: 'user', 'service_account')
- `role text not null` (enum: 'secret.admin', 'secret.writer', 'secret.reader', 'secret.none')
- `scope text null`
- `created_at timestamptz not null`

**Indexes:**
- `(tenant_id, principal_id)` for fast role lookup

NOTE: `cred_store_role_assignment` is currently out of implementation scope, it's assumed to be implemented by external access policy engine.

#### Role definitions:
- `gts.x.core.idp.role.v1~x.cred_store.secret.admin.v1`: full CRUD + key management + audit access
- `gts.x.core.idp.role.v1~x.cred_store.secret.writer.v1`: create, update, read (no delete, no key rotation)
- `gts.x.core.idp.role.v1~x.cred_store.secret.reader.v1`: read-only
- `gts.x.core.idp.role.v1~x.cred_store.secret.none.v1`: explicit deny (overrides other grants)

#### Scenario: Role enforcement at gateway
- **GIVEN** user U has role `gts.x.core.idp.role.v1~x.cred_store.secret.reader.v1`
- **WHEN** U attempts `DELETE /cred-store/v1/secrets/{id}`
- **THEN** gateway checks SecurityCtx roles
- **AND** returns 403 Forbidden with RFC 9457 Problem Detail

#### Scenario: Scoped role assignment
- **GIVEN** user U has role `gts.x.core.idp.role.v1~x.cred_store.secret.writer.v1` scoped to `secret_type_id = API_KEY`
- **WHEN** U attempts to create a DATABASE_PASSWORD secret
- **THEN** gateway denies with 403 Forbidden

### Requirement: cred_store secret-level ACL
The system SHALL support per-secret access control lists.

#### Database table: cred_store_secret_acl
- `id uuid pk`
- `tenant_id uuid not null`
- `secret_id uuid not null fk cred_store.secret_store(id)`
- `principal_id uuid not null`
- `principal_type text not null` (enum: 'user', 'service_account')
- `permissions text[] not null` (array of: 'read', 'update', 'delete', 'share')
- `created_at timestamptz not null`

**Indexes:**
- `(secret_id, principal_id)` for ACL lookup

NOTE: `cred_store_secret_acl` is currently out of implementation scope, it's assumed to be implemented by external access policy engine.

#### Scenario: ACL-based access grant
- **GIVEN** secret S has ACL: {user_A: ['read'], user_B: ['read', 'update']}
- **WHEN** user_C (not in ACL) attempts GET /cred-store/v1/secrets/{S}
- **THEN** gateway checks ACL after RBAC check
- **AND** returns 403 Forbidden

#### Scenario: ACL precedence over RBAC
- **GIVEN** user U has tenant-wide role `gts.x.core.idp.role.v1~x.cred_store.secret.reader.v1`
- **AND** secret S has ACL excluding U
- **WHEN** U attempts to read S
- **THEN** ACL denial takes precedence, returns 403

### Requirement: cred_store authorization enforcement
The gateway SHALL enforce authorization at two layers:
1. **API layer**: Check SecurityCtx roles/permissions before domain service calls
2. **DB layer**: Use `modkit-db` SecureConn with tenant/user scoping

#### Scenario: Double-layer authorization
- **WHEN** gateway invokes domain service `get_secret(ctx, secret_id)`
- **THEN** API layer checks RBAC roles first
- **AND** domain service uses `SecureConn::scoped_to_tenant(tenant_id)` for DB query
- **AND** DB layer ensures tenant isolation via secure ORM

### Requirement: cred_store data residency
The system SHALL support per-tenant data residency constraints.

#### Database table: cred_store_compliance_policy
- `id uuid pk`
- `tenant_id uuid not null unique`
- `secret_type_id uuid not null fk cred_store.secret_type(id)` (GTS secret type UUID)
- `data_residency text[] null` (e.g., ['EU', 'US'])
- `retention_days int not null default 90`
- `encryption_required bool not null default true`
- `audit_retention_days int not null default 365`

NOTE: `cred_store_compliance_policy` support is out of scope of the first version.

#### Scenario: EU-only data residency
- **GIVEN** tenant T has data_residency = ['EU']
- **WHEN** secret is created
- **THEN** system ensures storage in EU-region database shard
- **AND** prevents replication to non-EU regions

### Requirement: cred_store GDPR right-to-erasure
The system SHALL support complete user data deletion.

REST endpoint:
- `DELETE /cred-store/v1/users/{userId}/secrets?gdpr_erasure=true`

#### Scenario: GDPR erasure request
- **WHEN** user/client U requests GDPR erasure
- **THEN** system permanently deletes all secrets where `cliend_id = U`
- **AND** retains audit logs per legal retention requirements (anonymized)
- **AND** logs erasure operation with justification

### Requirement: cred_store SOC2 compliance
To achieve SOC2 Type II compliance, the system SHALL:
- Maintain comprehensive audit logs (all CRUD operations)
- Enforce least-privilege access control (RBAC + ACL)
- Encrypt secrets at rest and in transit
- Implement key rotation procedures
- Provide security monitoring and alerting
- Support access reviews and attestation exports

Additional requirements:
- Audit logs MUST be immutable (append-only)
- Access to encryption keys MUST be logged
- Failed access attempts MUST trigger alerts after threshold
- Security incidents MUST be trackable via audit trail

#### Scenario: SOC2 audit trail verification
- **WHEN** auditor reviews secret access for period P
- **THEN** system provides complete audit log export
- **AND** verifies log integrity via checksums or signatures

### Requirement: cred_store HIPAA compliance
To achieve HIPAA compliance, the system SHALL:
- Explicitly tag secrets as `gts.x.core.cred_store.secret_type.v1~x.core.phi.secret.v1`
- Encrypt all PHI (Protected Health Information) at rest using FIPS 140-2 validated crypto modules
- Implement automatic logoff after 15 minutes of inactivity (enforced by session management)
- Maintain audit trails for minimum 6 years
- Provide data breach notification capabilities
- Support Business Associate Agreement (BAA) requirements via tenant compliance policies

Additional requirements:
- Audit logs MUST capture all access to secrets containing PHI
- System MUST support emergency access procedures with justification
- Encryption keys MUST be rotatable without service interruption

#### Scenario: HIPAA emergency access
- **WHEN** admin requires emergency access to PHI secret
- **THEN** system logs access with justification field required
- **AND** triggers security alert for review

### Requirement: cred_store PCI-DSS compliance
To achieve PCI-DSS compliance, the system SHALL:
- Explicitly tag secrets as `gts.x.core.cred_store.secret_type.v1~x.core.pci.cardholder_data.v1`
- Never store full magnetic stripe data, card verification codes (CVV2/CVC2), or PINs
- Mask PAN (Primary Account Number) when displaying (show first 6 and last 4 digits only)
- Implement quarterly key rotation for cardholder data encryption keys
- Maintain audit trails for minimum 1 year (immediately accessible) + 3 years (archival)
- Restrict access to cardholder data on need-to-know basis

Additional requirements:
- Secrets tagged as `gts.x.core.cred_store.secret_type.v1~x.core.pci.cardholder_data.v1` MUST have additional access restrictions
- All access to PCI-scoped secrets MUST be logged and monitored
- Failed access attempts exceeding threshold MUST trigger security alerts
- Quarterly access reviews MUST be facilitated via export API

#### Scenario: PCI secret masking
- **WHEN** admin views secret metadata for PAN (tagged as `gts.x.core.cred_store.secret_type.v1~x.core.pci.cardholder_data.v1`)
- **THEN** system displays masked value: `411111******1111`
- **AND** full value only accessible via explicit `get_secret()` call (audited)

### Requirement: cred_store secret versioning
The system SHALL support optional secret versioning, configurable per secret type.

#### Extended database table: cred_store.secret_store
- Add: `versioning_enabled bool not null default false`
- Add: `current_version int not null default 1`

#### New table: cred_store_secret_version
- `id uuid pk`
- `tenant_id uuid not null`
- `secret_id uuid not null fk cred_store.secret_store(id)`
- `version int not null`
- `secret_type_id uuid not null`
- `parameters jsonb null`
- `created_at timestamptz not null`
- `created_by uuid null`
- **Unique constraint:** `(secret_id, version)`

#### New table: cred_store_secret_blob_version
- `id uuid pk`
- `secret_id uuid not null`
- `version int not null`
- `ciphertext bytes not null`
- `cipher_alg text not null`
- `nonce bytes not null`
- `aad bytes not null`
- **Unique constraint:** `(secret_id, version)`

**Indexes:**
- `(secret_id, version desc)` for version retrieval

#### Scenario: Enable versioning per secret type
- **GIVEN** secret_type `cred_store.secret_type.v1~x.core.db.password.v1` has versioning enabled
- **WHEN** admin creates a secret of this type
- **THEN** `versioning_enabled = true` is set
- **AND** initial version = 1 is created

#### Scenario: Update secret with versioning
- **WHEN** user updates a secret with `versioning_enabled = true`
- **THEN** system creates new version (e.g., v2)
- **AND** preserves previous version (v1) in `cred_store_secret_version`
- **AND** updates `current_version = 2`

#### Scenario: Retrieve specific version
- **WHEN** client requests `GET /cred-store/v1/secrets/{id}/versions/2`
- **THEN** system retrieves version 2 from `cred_store_secret_blob_version`
- **AND** decrypts using stored KEK metadata

#### Scenario: List secret versions
- **WHEN** client requests `GET /cred-store/v1/secrets/{id}/versions`
- **THEN** system returns versions ordered by version desc
- **AND** each version includes (version, created_at, created_by)

### Requirement: cred_store version retention
The system SHALL support configurable version retention policies.

Default policy:
- Keep last 5 versions (configurable) for the secrets of the same type
- Auto-prune older versions after retention period

#### Scenario: Version pruning
- **GIVEN** secret S has 10 versions and retention policy = 5
- **WHEN** version cleanup job runs
- **THEN** versions 1-5 are pruned
- **AND** versions 6-10 are retained

### Requirement: cred_store version REST API
The system SHALL expose version management endpoints.

REST endpoints:
- `GET /cred-store/v1/secrets/{secretId}/versions` - List versions
- `GET /cred-store/v1/secrets/{secretId}/versions/{version}` - Get specific version
- `POST /cred-store/v1/secrets/{secretId}/rollback` - Rollback to previous version

#### Scenario: Rollback to previous version
- **WHEN** admin posts to `/cred-store/v1/secrets/{id}/rollback` with target_version=3
- **THEN** system creates new version (e.g., v6) with content from v3
- **AND** marks v6 as current version

### Requirement: cred_store error taxonomy
The system SHALL define comprehensive error types using RFC 9457 Problem Details.

Error types SHALL include:
- `PluginUnavailable`: Plugin is unreachable
- `DecryptionFailure`: Cannot decrypt ciphertext
- `KekUnavailable`: KEK not accessible
- `QuotaExceeded`: Tenant secret limit reached
- `SecretTooLarge`: Secret exceeds size limit
- `InvalidSecretType`: Unknown secret type
- `ConcurrentModification`: Optimistic locking conflict
- `Forbidden`: Access denied
- `NotFound`: Secret does not exist

#### Scenario: Plugin unavailable error
- **WHEN** active plugin is unreachable (connection timeout)
- **THEN** gateway returns 503 Service Unavailable
- **AND** includes Problem Detail with type `urn:cred-store:error:plugin-unavailable`
- **AND** logs error with OpenTelemetry span

#### Scenario: Decryption failure error
- **WHEN** plugin cannot decrypt ciphertext (corrupted data or missing KEK)
- **THEN** gateway returns 500 Internal Server Error
- **AND** triggers alert to operations team
- **AND** logs error with secret_id and kek_id context

### Requirement: cred_store retry logic
The system SHALL implement exponential backoff retry for transient failures.

Retry policy:
- **Transient errors:** Network timeout, 502/503/504 from plugin
- **Max retries:** 3
- **Backoff:** 100ms, 200ms, 400ms (max timeout must be configurable)
- **Non-retryable:** 400 Bad Request, 403 Forbidden, 404 Not Found, 409 Conflict

Implementation SHOULD use `tower` crate retry middleware.

#### Scenario: Retry transient plugin error
- **WHEN** plugin returns 503 Service Unavailable
- **THEN** gateway retries after 100ms
- **AND** if still failing, retries after 200ms, then 400ms
- **AND** if all retries exhausted, returns 503 to client

### Requirement: cred_store circuit breaker
The system SHALL implement circuit breaker pattern to prevent cascade failures.

Circuit breaker configuration:
- **Failure threshold:** 5 consecutive failures (configurable)
- **Timeout:** 30 seconds (half-open after this duration, configurable)
- **Success threshold:** 2 consecutive successes to close circuit (configurable)

Implementation SHOULD use `failsafe` or `tower` crate.

#### Scenario: Circuit breaker opens
- **GIVEN** plugin fails 5 consecutive operations (configurable)
- **WHEN** circuit breaker opens
- **THEN** subsequent requests fail-fast with 503 (no plugin call)
- **AND** after 30 seconds, circuit enters half-open state
- **AND** next request tests plugin availability

#### Scenario: Circuit breaker recovery
- **GIVEN** circuit breaker is half-open
- **WHEN** 2 consecutive plugin calls succeed (configurable)
- **THEN** circuit breaker closes
- **AND** normal operation resumes

### Requirement: cred_store KEK rotation without downtime
The system SHALL support KEK (Key Encryption Key) rotation with zero downtime.

#### Extended table: cred_store_secret_kek_state
- Add: `kek_version int not null default 1`
- Add: `rotation_status text null` (enum: 'active', 'rotating', 'deprecated')

#### New table: cred_store_kek_metadata
- `id uuid pk`
- `tenant_id uuid null`
- `kek_version int not null`
- `kek_algorithm text not null` (e.g., 'age', 'aes-256-gcm')
- `status text not null` (enum: 'active', 'deprecated', 'revoked')
- `created_at timestamptz not null`
- `rotated_at timestamptz null`
- **Unique constraint:** `(tenant_id, kek_version)`

#### Scenario: Initiate KEK rotation
- **WHEN** admin triggers KEK rotation for tenant T
- **THEN** system generates new KEK v2
- **AND** marks KEK v1 as `status = 'deprecated'`
- **AND** marks KEK v2 as `status = 'active'`
- **AND** new secrets use KEK v2 immediately

#### Scenario: Gradual re-encryption
- **WHEN** KEK rotation is initiated
- **THEN** background job lists all secrets encrypted with old KEK v1
- **AND** decrypts each secret with KEK v1
- **AND** re-encrypts with new KEK v2
- **AND** updates `kek_version = 2` in `cred_store_secret_kek_state`
- **AND** both KEK v1 and v2 remain available during transition

#### Scenario: Dual-KEK read during rotation
- **GIVEN** secret S is still encrypted with KEK v1
- **AND** KEK rotation is in progress (v2 is active)
- **WHEN** user reads secret S
- **THEN** system checks `kek_version = 1` in metadata
- **AND** uses KEK v1 to decrypt

#### Scenario: Complete KEK rotation
- **WHEN** all secrets have been re-encrypted with KEK v2
- **THEN** admin marks KEK v1 as `status = 'revoked'`
- **AND** KEK v1 is deleted from key storage

### Requirement: cred_store tenant quotas
The system SHALL enforce per-tenant resource limits.

#### Database table: cred_store_tenant_quota
- `id uuid pk`
- `tenant_id uuid not null unique`
- `max_secrets_per_client int not null default 1000`
- `max_secret_size_bytes int not null default 65536` (64 KB)
- `max_versions_per_secret int not null default 5`
- `max_write_requests_per_minute int not null default 100`
- `max_read_requests_per_minute int not null default 1000`
- `max_admin_requests_per_minute int not null default 10`

#### Scenario: Enforce secret count quota
- **GIVEN** tenant T has max_secrets = 1000 and current count = 1000
- **WHEN** T attempts to create secret 1001
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes Problem Detail with retry guidance

#### Scenario: Enforce secret size limit
- **WHEN** client attempts to store 128 KB secret
- **AND** tenant quota max_secret_size_bytes = 65536
- **THEN** gateway returns 400 Bad Request
- **AND** error detail specifies: "Secret size 131072 bytes exceeds limit of 65536 bytes"

### Requirement: cred_store rate limiting
The system SHALL enforce rate limits per tenant.

Rate limits:
- **Write operations:** 100 req/min per tenant
- **Read operations:** 1000 req/min per tenant
- **Admin operations:** 10 req/min per tenant

Implementation SHOULD use `governor` crate.

#### Scenario: Rate limit exceeded
- **GIVEN** tenant T has made 100 write requests in current minute
- **WHEN** T attempts write request 101
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes `Retry-After: 15` header (seconds until reset)

### Requirement: cred_store quota management REST API
The system SHALL expose quota management endpoints.

REST endpoints:
- `GET /cred-store/v1/quotas` - View current quota and usage
- `PATCH /cred-store/v1/quotas` - Update quota (admin only)

#### Scenario: View quota usage
- **WHEN** tenant requests GET /cred-store/v1/quotas
- **THEN** response includes current quota configuration and usage metrics

### Requirement: cred_store metrics instrumentation
The system SHALL expose Prometheus-compatible metrics.

Required metrics (using `metrics` crate):
- `cred_store.secrets.created` (counter, labels: tenant_id, secret_type)
- `cred_store.secrets.read` (counter, labels: tenant_id)
- `cred_store.secrets.updated` (counter, labels: tenant_id)
- `cred_store.secrets.deleted` (counter, labels: tenant_id)
- `cred_store.errors` (counter, labels: error_type, plugin_id)
- `cred_store.operation.duration` (histogram, labels: operation, tenant_id)
- `cred_store.plugin.call.duration` (histogram, labels: plugin_id, method)
- `cred_store.secrets.total` (gauge, labels: tenant_id)
- `cred_store.kek.active_version` (gauge, labels: tenant_id)
- `cred_store.circuit_breaker.state` (gauge, labels: plugin_id)

#### Scenario: Metrics collection
- **WHEN** secret operation completes
- **THEN** system increments appropriate counters
- **AND** records latency histogram
- **AND** metrics are exposed at GET /metrics endpoint

### Requirement: cred_store health checks
The system SHALL expose health check endpoints.

REST endpoints:
- `GET /cred-store/v1/health` - Liveness probe
- `GET /cred-store/v1/ready` - Readiness probe (checks plugin connectivity)

#### Scenario: Readiness check
- **WHEN** GET /cred-store/v1/ready is called
- **THEN** gateway tests active plugin connectivity (lightweight ping operation)
- **AND** returns 200 OK if plugin is reachable
- **AND** returns 503 Service Unavailable if plugin is down

### Requirement: cred_store tracing integration
The system SHALL integrate with OpenTelemetry for distributed tracing.

All operations SHALL create spans with attributes:
- `tenant_id`
- `client_id` (if available)
- `secret_id`
- `secret_type`
- `plugin_id`
- `kek_version`
- `operation` (create/read/update/delete)

Implementation SHALL use `tracing` and `tracing-opentelemetry` crates.

#### Scenario: Trace secret operation
- **WHEN** secret operation is performed
- **THEN** system creates OpenTelemetry span with all relevant attributes
- **AND** span is linked to parent trace context
- **AND** errors are recorded as span events

### Requirement: cred_store SLOs (Service Level Objectives)
The system SHOULD target:
- **Availability:** 99.9% (excluding planned maintenance)
- **Latency:**
  - p50: < 50ms for `get_secret`
  - p95: < 100ms for `get_secret`
  - p99: < 500ms for `upsert_secret`
- **Error rate:** < 0.1% (excluding client errors 4xx)

#### Scenario: SLO monitoring
- **WHEN** operations team reviews SLO dashboard
- **THEN** metrics show p95 latency, error rate, and uptime
- **AND** alerts trigger if SLO thresholds are breached

### Requirement: cred_store OData support
The system SHALL implement OData query capabilities per ModKit conventions.

REST endpoints SHALL support:
- `$skip` and `$top` for pagination
- `$filter` for filtering
- `$orderby` for sorting
- `$select` for field projection

Implementation SHALL use `modkit::api::OperationBuilder` OData support.

#### Scenario: Paginated secret list
- **WHEN** `GET /cred-store/v1/secrets?$skip=20&$top=10`
- **THEN** returns secrets 21-30
- **AND** includes `@odata.nextLink` for pagination

#### Scenario: Filter by secret type
- **WHEN** `GET /cred-store/v1/secrets?$filter=secret_type_id eq '550e8400-e29b-41d4-a716-446655440000'`
- **THEN** returns only secrets matching that type

#### Scenario: Sort and project fields
- **WHEN** `GET /cred-store/v1/secrets?$orderby=created_at desc&$select=id,created_at,secret_type_id`
- **THEN** returns secrets sorted by creation date
- **AND** response includes only selected fields
