# ADR-0006: Rust Stable Only (No Nightly Features)

**Date**: 2024-01-17

**Status**: Accepted

**ID**: `spd-hyperspot-adr-rust-version`

## Context and Problem Statement

Rust offers a nightly channel with experimental features that often provide ergonomic improvements or performance benefits. However, nightly features can break between compiler versions and may never stabilize. For a platform targeting production deployments and LLM-assisted development, we need predictable, reproducible builds.

## Decision Drivers

* Must ensure reproducible builds across development and CI environments
* Must avoid breaking changes from nightly compiler updates
* Must work with standard tooling and IDE support
* Must be accessible to developers without nightly Rust knowledge
* Must support long-term stability for enterprise deployments

## Considered Options

* Rust stable only (no nightly features)
* Rust nightly with pinned toolchain version
* Rust stable with occasional nightly features behind feature flags
* Rust beta as compromise between stable and nightly

## Decision Outcome

Chosen option: "Rust stable only", because stable Rust provides all necessary features for the platform (async/await, trait system, macros, const generics) with guaranteed backwards compatibility. Pinning to stable ensures builds work identically across developer machines, CI, and production environments without toolchain version management.

### Consequences

* Good, because builds are reproducible across all environments
* Good, because no breaking changes from nightly compiler updates
* Good, because standard IDE support and tooling work reliably
* Good, because documentation and Stack Overflow answers target stable
* Good, because enterprise deployments prefer stable, well-tested compilers
* Bad, because some ergonomic nightly features are unavailable (e.g., try blocks, type alias impl trait)
* Bad, because performance optimizations from nightly features cannot be used
* Bad, because new stable features arrive only every 6 weeks (slower iteration)

## Related Design Elements

**Actors**:
* `spd-hyperspot-actor-saas-developer` - Benefits from stable, predictable builds
* `spd-hyperspot-actor-platform-operator` - Benefits from reproducible deployments

**Requirements**:
* `spd-hyperspot-nfr-build-performance` - Stable rust impact on build times
* `spd-hyperspot-nfr-compilation-safety` - Stable features are well-tested
