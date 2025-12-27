# Change: Add model_provider, cred_store, and oagw module specifications

## Why
HyperSpot needs standardized, reusable building blocks for (1) managing GenAI model providers and their endpoints, (2) secure tenant-scoped secret storage, and (3) outbound remote-call invocation with pluggable protocol/auth/streaming implementations.

These three modules are foundational for inference, embeddings, and future integrations, and must follow HyperSpot/ModKit conventions (DDD-light, TypesRegistry-backed GTS registration, gateway+plugin patterns).

## What Changes
- Define three new gateway modules and their plugin interfaces:
  - `model_provider` (gateway + plugins)
  - `cred_store` (gateway + single highest-priority plugin)
  - `oagw` (gateway + single highest-priority eligible plugin)
- Define complete domain model (GTS schemas/instances + DB entities) and the module startup procedure for registering:
  - All GTS schemas (`XxxV1::gts_schema_with_refs_as_string()`)
  - All well-known GTS instances from `gts/*.instances.json` payloads
- Define complete REST APIs (OpenAPI 3.1) and Rust-native (ClientHub) APIs, including DTOs, errors, and step-by-step processing scenarios.
- Define detailed database schema for anonymous (UUID-identified) objects.

## Impact
- Affected specs:
  - New: `model-provider` (gateway + plugin contracts)
  - New: `cred-store` (gateway + plugin contracts)
  - New: `oagw` (gateway + plugin contracts)
- Affected code:
  - New modules will be introduced under `modules/` (implementation is **out of scope** for this proposal stage)
  - Integrates with existing `types_registry` module lifecycle (`init()` registration + `post_init()` ready switch)

## Success Criteria

### Specification Completeness
- ✅ Complete domain models (GTS schemas, DB tables, anonymous UUID-based persistence)
- ✅ Complete API contracts (Rust-native ClientHub APIs + REST OpenAPI 3.1 endpoints)
- ✅ Complete plugin interfaces (trait definitions, priority-based selection, scoped client registration)
- ✅ Complete operational requirements (audit logging, observability, error handling, resilience)
- ✅ Complete compliance requirements (SOC2, HIPAA, PCI-DSS guidelines for `cred_store`)
- ✅ Complete security requirements (RBAC, ACL, secret versioning, KEK rotation, encryption)

### Industry Standards Alignment
- ✅ Audit logging comparable to HashiCorp Vault, AWS Secrets Manager
- ✅ Access control (RBAC + ACL) following least-privilege principle
- ✅ Circuit breaker and retry patterns per industry best practices
- ✅ Observability (Prometheus metrics, OpenTelemetry tracing, health checks)
- ✅ Rate limiting and resource quotas to prevent abuse
- ✅ Comprehensive error taxonomy using RFC 9457 Problem Details

### Production Readiness
- ✅ SLOs defined (99.9% availability, p95 latency targets)
- ✅ Disaster recovery (backup, KEK rotation, secret versioning)
- ✅ Compliance posture (GDPR right-to-erasure, data residency, retention policies)
- ✅ Operational excellence (health checks, readiness probes, graceful degradation)

### OpenSpec Validation
- Specifications MUST pass `openspec validate --strict` without errors
- All requirements MUST have at least one scenario
- All scenarios MUST use correct `#### Scenario:` formatting
