# Feature Flags Gateway Module — Specification

## 1) Purpose

Provide a **Feature Flag Gateway** that can be used by API Ingress middleware and other modules to decide whether a feature is enabled.

This spec is authored to support issue **#115** (mandatory `.require_feature_flag(...)` for every route).

## 2) Design intent (how this fits HyperSpot)

- This module is a **Gateway Module** in HyperSpot terminology.
- The gateway exposes a **typed Rust API** via **ModKit `ClientHub`** (in-process by default).
- The gateway may later be swapped to an out-of-process implementation (gRPC) without changing callers (ClientHub abstraction).

## 2.1) Reduced scope (current implementation milestone)

- **Type Registry integration** is **not implemented** yet.
- The gateway provides a **stub** evaluator:
  - returns `true` only for `FeatureFlag::GLOBAL_BASE = "gts.x.core.ff.flag.v1~x.core.global.base.v1"`
  - returns `false` for any other **valid** feature flag id
- Cache requirements and backend availability errors are **deferred** until a real registry/backend exists.

## 3) Constraints (non-negotiable)

- The gateway **MUST** provide a **default/base implementation** whose behavior is:
  - return `true` if the requested feature flag is **registered in the Type Registry**
    - Reduced scope note: Type Registry is not implemented; stub returns `true` only for `FeatureFlag::GLOBAL_BASE`.
  - return `false` otherwise
    - Reduced scope note: stub returns `false` for every other valid feature flag id.
- The Type Registry **MUST** always contain the global base feature flag:
  - `FeatureFlag::GLOBAL_BASE = "gts.x.core.ff.flag.v1~x.core.global.base.v1"`
  - Reduced scope note: enforced by stub behavior (no registry mutation).
- The API Ingress **MUST** treat a failed check as **fail-closed** (Forbidden).

## 4) Deliverables (for the implementer)

- **Module folder**: `modules/feature_flags_gateway/`
- **API trait (SDK)**: `FeatureFlagsApi` (registered into ClientHub)
- **Base implementation**: `TypeRegistryBackedFeatureFlags` (name may vary, behavior must match Section 3)
- Reduced scope note: the base implementation is a stub (GlobalBase=true, others=false) until Type Registry exists.
- **Client-side cache**: a small in-memory cache suitable for hot-path middleware usage
  - Reduced scope note: cache is deferred for the stub milestone.

> Note: This spec defines *what* to build and how it behaves; it does not mandate the exact file layout. If you follow `guidelines/NEW_MODULE.md`, place the public API in a `*-sdk` crate.

## 5) Terminology

- **Feature Flag**: a non-empty string identifier.
  - Reduced scope note: the stub milestone validates only that the id is not empty/whitespace.
- **Type Registry**: authoritative registry that answers whether a given feature flag id is registered.
  - Reduced scope note: not implemented yet; treated as a future backend.
- **Gateway API**: the trait resolved from ClientHub and called by middleware/services.

## 6) API Surface (Rust, transport-agnostic)

### 6.1 Types

- `FeatureFlag`
  - constants:
    - `GLOBAL_BASE: &str = "gts.x.core.ff.flag.v1~x.core.global.base.v1"`

> YAGNI note: a dedicated `FeatureFlagId` newtype is intentionally omitted for the public SDK contract.
> The public API accepts raw strings (`&str` / `String`) for ergonomics and to avoid allocations in middleware hot paths.

> YAGNI note: a separate per-request evaluation payload (previously proposed as `FeatureFlagContext`) is intentionally
> omitted for now.
> `SecurityCtx` already carries tenant/user scoping information and the stub milestone does not implement targeting.
> When real targeting/segmentation is introduced (e.g. tenant hierarchy, app version, region, cohorts), we can add a
> dedicated input type then (e.g. `FeatureFlagEvalInputs` / `FeatureFlagAttributes`) without over-designing upfront.

### 6.2 Errors

- `FeatureFlagsError`
  - `InvalidFeatureFlagId { value: String }`

> YAGNI note: the error surface is intentionally minimal in the stub milestone.
> When a real backend is introduced, additional error variants (e.g. backend unavailable / internal failures) may be
> added as needed.

### 6.3 Trait

The gateway must expose this trait via `ClientHub`:

```rust
#[async_trait::async_trait]
pub trait FeatureFlagsApi: Send + Sync {
    /// Returns true if the feature is enabled.
    ///
    /// Fail-closed principle:
    /// - the gateway returns Err for invalid feature flag ids.
    async fn is_enabled(
        &self,
        sec: &modkit_security::SecurityCtx,
        flag: &str,
    ) -> Result<bool, FeatureFlagsError>;

    /// Optional optimization for middleware: batch evaluation.
    async fn are_enabled(
        &self,
        sec: &modkit_security::SecurityCtx,
        flags: &[String],
    ) -> Result<std::collections::HashMap<String, bool>, FeatureFlagsError>;
}
```

Notes:
- `&modkit_security::SecurityCtx` as the first parameter is **mandatory**.
- Middleware only needs `is_enabled`.

## 7) Cache behavior

Reduced scope note: cache behavior is deferred until a real registry/backend exists.

### 7.1 Goals

- Avoid repeated backend calls for the same flag in hot paths.
- Keep behavior deterministic and safe.

### 7.2 Requirements

- Cache **MUST** be in-memory and per-process.
- Cache entries **MUST** be bounded (max entries) to avoid unbounded memory growth.
- Cache entries **MUST** have TTL (positive and negative caching are allowed).
- Cache **MUST** store both:
  - enabled = true
  - enabled = false

### 7.3 Suggested configuration (module config)

A typed config object should exist (names are suggestions):

- `cache.max_entries: u64` (default: 10_000)
- `cache.ttl_ms: u64` (default: 30_000)

## 8) Base implementation behavior (Type Registry backed)

Reduced scope note: the current implementation milestone uses a stub evaluator (GlobalBase=true, others=false).

### 8.1 Definition

The base implementation is the default plugin shipped with HyperSpot.

It evaluates a feature flag id as enabled **iff** the Type Registry reports the flag is registered.

### 8.2 Type Registry dependency

This spec requires an authoritative “Type Registry” capability. Implementers may satisfy this by:

- consuming an existing registry API if one exists in the workspace, OR
- introducing a minimal internal registry adapter and ensuring it is populated with required flags.

Reduced scope note: Type Registry integration is deferred; do not introduce a registry adapter as part of the stub milestone.

Regardless of the mechanism, the observable behavior must match BDD scenarios in Section 10.

## 9) Registration into ClientHub

During module `init()`:

- Construct the base `FeatureFlagsApi` implementation.
- Register it into `ClientHub` under `dyn FeatureFlagsApi`.

The module should be discoverable via `#[modkit::module(... client = ...)]` or explicit registration.

## 10) Behaviors (BDD Scenarios)

### Scenario: Enabled when feature flag exists in Type Registry

Reduced scope note: for the stub milestone, only `FeatureFlag::GLOBAL_BASE` returns `Ok(true)`; registry membership checks are deferred.

- **Given** the Type Registry contains "gts.x.core.ff.flag.v1~acme.some.flag.v1"
- **And** `FeatureFlagsApi` is registered in ClientHub
- **When** `is_enabled(&sec, "gts.x.core.ff.flag.v1~acme.some.flag.v1")` is called
- **Then** it returns `Ok(true)`

### Scenario: Disabled when feature flag is missing in Type Registry

Reduced scope note: for the stub milestone, every valid non-GlobalBase flag returns `Ok(false)`.

- **Given** the Type Registry does not contain "gts.x.core.ff.flag.v1~acme.missing.flag.v1"
- **When** `is_enabled(&sec, "gts.x.core.ff.flag.v1~acme.missing.flag.v1")` is called
- **Then** it returns `Ok(false)`

### Scenario: Reject invalid feature flag id

- **Given** a feature flag id " "
- **When** `is_enabled(&sec, " ")` is called
- **Then** it returns `Err(InvalidFeatureFlagId { ... })`

### Scenario: GlobalBase flag must exist

Reduced scope note: this is satisfied by stub behavior (GlobalBase is treated as enabled) without requiring registry population.

- **Given** the system has started successfully
- **Then** the Type Registry contains `"gts.x.core.ff.flag.v1~x.core.global.base.v1"`

### Scenario: Cache is used for repeated checks

Reduced scope note: cache behavior is deferred for the stub milestone.

- **Given** `is_enabled(&sec, "gts.x.core.ff.flag.v1~acme.some.flag.v1")` was called successfully
- **When** the same check is performed again within TTL
- **Then** the gateway does not perform a registry/backend lookup
- **And** it returns the cached result

> YAGNI note: backend failure behavior is deferred for the stub milestone (no backend is called).
> When a real backend is introduced, fail-closed behavior and error variants will be specified and implemented.

## 11) Integration note (API Ingress)

API Ingress middleware is expected to:

- read the per-route required flag (from `OperationSpec` metadata populated by `.require_feature_flag(...)`)
- call `FeatureFlagsApi::is_enabled(&SecurityCtx, ...)`
- return **HTTP 403 Forbidden** when:
  - the flag is disabled (`Ok(false)`), OR
  - the gateway call errors (fail-closed)

## 12) Acceptance criteria checklist

Reduced scope note: for the stub milestone, interpret “Type Registry backed” requirements as “GlobalBase=true, others=false”, and treat cache/backend-unavailable requirements as deferred.

- [ ] `FeatureFlagsApi` exists and is registered into `ClientHub`.
- [ ] Base implementation returns `true` iff flag is registered in Type Registry.
- [ ] Invalid feature flag ids (empty/whitespace) are rejected.
- [ ] `FeatureFlag::GLOBAL_BASE` exists and is always registered.
- [ ] Client-side cache is bounded + TTL-based.

