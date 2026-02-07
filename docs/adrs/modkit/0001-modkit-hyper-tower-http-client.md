---
status: accepted
date: 2026-02-04
decision-makers: MikeFalcon77 and team members
---

# Build a first-party HTTP client stack for ModKit instead of using reqwest directly

## Context and Problem Statement

ModKit modules and core libraries (notably `modkit-auth`) must call internal vendor services and identity endpoints (OAuth2 token endpoints, OIDC discovery, JWKS). The HTTP stack must support strict dependency hygiene (especially avoiding `reqwest` in security-critical or footprint-sensitive crates), consistent middleware composition (timeouts, retries, concurrency, OpenTelemetry), and predictable behavior across Windows/Linux/macOS and Kubernetes environments.

The key question is: should ModKit rely on a general-purpose client like `reqwest::Client`, expose a thin wrapper over `hyper::Client`, or ship a first-party HTTP client abstraction that is built on `hyper + tower` and tuned for ModKit’s security and DX requirements?

## Decision Drivers

* Hard constraint: avoid `reqwest` dependency (direct or transitive) in `modkit-auth` and security-adjacent crates.
* Composable tower middleware stack: custom layers (OAuth token injection, OTel tracing, retry, concurrency control) can be inserted into the service stack without wrapping or monkey-patching the client. With `reqwest` this requires external wrappers or middleware crates that sit outside the client's request pipeline.
* Consistent middleware model with the rest of the stack (tower layers for auth, OTel, retries, limits, load-shed).
* Clear, small, and stable API surface for internal calls (token endpoint, JWKS fetch, internal REST calls).
* Predictable and testable error taxonomy for security flows (status vs transport vs timeout vs body-too-large).
* Ability to enforce operational defaults (timeouts, retry policy, backoff+jitter, concurrency limits, body size limits).
* Cross-platform TLS behavior that can be configured (native roots vs bundled vs explicit roots).
* Avoid requiring module/business code to manage raw network clients, locks, or ad-hoc policies.
* Maintainability: keep complexity bounded, do not aim to replicate all features of general-purpose clients.

## Considered Options

* Use `reqwest::Client` everywhere
* Expose `hyper::Client` (or a minimal wrapper) and let callers build policy
* Use AWS SDK for Rust client patterns (Smithy-based stack) for all outbound HTTP
* Use `tonic`-style client approach (gRPC only; avoid HTTP REST client)
* Build a first-party `modkit-http` client based on `hyper + tower` with a small, opinionated API surface

## Decision Outcome

Chosen option: "Build a first-party `modkit-http` client based on `hyper + tower`", because it is the only option that satisfies the hard constraint of keeping `modkit-auth` free of `reqwest` while also aligning with ModKit’s tower-based middleware composition and enforcing consistent operational and security defaults.

### Consequences

* Good, because `modkit-auth` stays independent of `reqwest` and its transitive graph, while still having a production-grade HTTP stack.
* Good, because tower layer composability makes it straightforward to add cross-cutting concerns (e.g., an OAuth token injection layer, OTel tracing, custom header propagation) directly into the service stack — something that requires external workarounds with `reqwest`.
* Good, because we can standardize behavior across modules (timeouts, retries, concurrency limits, body limits, OTel) and review it once.
* Good, because the client can be made `Clone + Send + Sync` for ergonomic sharing, while internal mechanics handle tower's `&mut self` requirements.
* Good, because the implementation already ships transparent response decompression (gzip, brotli, deflate) and secure redirect following with SSRF protection, covering the most needed "batteries" without pulling in `reqwest`.
* Bad, because we assume ownership of HTTP client behavior, correctness, and long-term maintenance (even with a limited surface).
* Bad, because we must be explicit about non-goals (no attempt to match reqwest feature completeness).

### Confirmation

* CI tests validate:

  * Compile-time `HttpClient: Clone + Send + Sync`
  * Concurrent usage under load (multiple tasks issuing requests)
  * Strict body size limiting behavior
  * Timeout mapping correctness
  * Retry policy behavior (only on retryable errors and configured methods)
  * JWKS fetch path uses the shared client and surfaces correct errors
* Dependency checks confirm `modkit-auth` has no `reqwest` dependency (direct/transitive).
* Code review checklist verifies no module code introduces ad-hoc HTTP policies or raw hyper usage bypassing `modkit-http`.

## Pros and Cons of the Options

### Build a first-party `modkit-http` client (hyper + tower)

A small, opinionated internal HTTP client crate used by ModKit libraries and modules. Uses `hyper` for transport and `tower` for middleware. Exposes a constrained API (`get`, `post`, `post_form`, body readers, JSON parsing) and a builder for policy defaults. Designed to be `Clone + Send + Sync` by construction.

* Good, because it meets the reqwest exclusion constraint for `modkit-auth`.
* Good, because it composes naturally with tower layers used elsewhere (OTel, auth headers, retries, limits) — adding a new layer (e.g., OAuth token injection, request signing) is a standard `impl Layer<S>` without wrapping or forking the client.
* Good, because it centralizes security and operational defaults (timeouts, body limits, retry rules, SSRF-safe redirects, decompression with zip-bomb protection).
* Neutral, because it will not offer full "batteries included" DX like reqwest (by design).
* Bad, because we own correctness and long-term maintenance of the client behavior.
* Bad, because the implementation must carefully handle tower `Service` mutability and concurrency (typically via `Buffer` or internal async mutex).

### Use `reqwest::Client`

Use reqwest as the standard HTTP client and optionally layer custom behavior with middleware patterns (where possible) or wrappers.

* Good, because it is mature, widely used, and has strong ergonomics and feature coverage.
* Good, because it reduces our maintenance surface and leverages established TLS/proxy/redirect/compression behavior.
* Neutral, because it does not naturally align with tower-first composition — adding custom middleware (e.g., an OAuth token injection layer) requires external wrapping rather than composing into the client's service stack.
* Bad, because it violates the constraint: `modkit-auth` must not depend on `reqwest` (direct/transitive).
* Bad, because transitive dependency footprint and feature interactions are harder to control in security-adjacent crates.

### Expose `hyper::Client` directly (or a thin wrapper)

Provide hyper client access and let each caller implement policy (timeouts, retries, headers, tracing).

* Good, because it keeps dependencies minimal and gives maximum control.
* Good, because hyper is the foundational transport layer and is stable.
* Neutral, because some shared helpers can be built incrementally.
* Bad, because policy fragments across the codebase (inconsistent retries/timeouts/body limits).
* Bad, because security reviews become harder: behavior is distributed across multiple call sites.
* Bad, because caller DX degrades: repeated boilerplate and higher chance of subtle mistakes.

### Use AWS SDK for Rust client patterns (Smithy stack)

Adopt an AWS SDK-style client architecture for outbound HTTP, potentially reusing Smithy middleware concepts.

* Good, because it is designed for large-scale clients with pluggable middleware and consistent signing/retries.
* Neutral, because it is well-tested within the AWS ecosystem for AWS services.
* Bad, because it is not a generic REST client framework for arbitrary internal services; adapting it increases complexity.
* Bad, because dependency footprint and conceptual overhead are large relative to ModKit’s needed surface.
* Bad, because it does not directly solve the tower alignment goal unless we build substantial adapters.

### Use `tonic`-style client approach (gRPC only)

Prefer gRPC for inter-service calls and avoid building a REST HTTP client.

* Good, because gRPC clients in tonic are ergonomic, typed, and support interceptors/middleware.
* Good, because it can reduce the number of REST calls if services provide gRPC endpoints.
* Neutral, because some vendor/IdP endpoints (OIDC discovery, JWKS, OAuth2 token) are HTTP by definition.
* Bad, because it does not eliminate the need for an HTTP client in `modkit-auth`.
* Bad, because it assumes broader architectural changes (service APIs), which is out of scope for the immediate requirement.

## More Information

* Scope boundary (non-goals):

  * Not a full replacement for reqwest.
  * No cookie jar, proxy auth, request body compression, caching, WebSocket, or streaming uploads.
  * Minimal stable API surface intended for infrastructure calls (auth flows, internal REST).
* Already implemented beyond the original minimal scope:

  * Transparent response decompression (gzip, brotli, deflate) via tower-http `DecompressionLayer`.
  * Secure redirect following with SSRF protection (same-origin default, header stripping, HTTPS downgrade blocking) via custom `SecureRedirectPolicy`.
* Revisit criteria:

  * If requirements expand to general outbound HTTP client for external integrations, reassess whether a feature-gated reqwest-based crate is warranted (separate from `modkit-auth`), while keeping `modkit-auth` on `modkit-http`.
  * If tower-based buffering introduces unacceptable latency/queue behavior for key flows, reassess concurrency model (buffer sizing, load-shed, per-host pools) before considering a switch in underlying client.
