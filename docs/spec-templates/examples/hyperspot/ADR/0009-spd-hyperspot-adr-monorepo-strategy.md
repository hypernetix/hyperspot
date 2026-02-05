# ADR-0009: Monorepo Until Scale Forces Split

**Date**: 2024-01-19

**Status**: Accepted

**ID**: `spd-hyperspot-adr-monorepo-strategy`

## Context and Problem Statement

Should the platform use a monorepo (all modules in one repository) or polyrepo (separate repositories per module)? Monorepos enable atomic cross-module refactoring and consistent tooling but can become unwieldy at scale. Polyrepos enable independent versioning but complicate coordinated changes across multiple modules.

## Decision Drivers

* Must support atomic refactoring across multiple modules simultaneously
* Must maintain single source of truth for tooling configuration (CI, lints, formatting)
* Must enable realistic local builds and end-to-end testing
* Must work efficiently for team sizes from 1 to 50 developers
* Must provide migration path to polyrepo if repository size becomes problematic

## Considered Options

* Monorepo with Cargo workspace (all modules together)
* Polyrepo with separate repositories per module
* Hybrid approach (core modules in monorepo, extensions in separate repos)
* Monorepo with sparse checkout for large team scalability

## Decision Outcome

Chosen option: "Monorepo with Cargo workspace", because Rust's Cargo workspace provides excellent monorepo support with shared dependencies, unified build cache, and atomic commits across modules. The platform's target team size (1-50 developers) is well within monorepo scalability limits. Atomic refactoring (renaming types, changing APIs) affects all consumers in a single commit, verified by CI before merge.

### Consequences

* Good, because atomic refactoring across modules (one commit, one PR, one review)
* Good, because single CI configuration applies to all modules consistently
* Good, because shared dependency versions avoid version conflicts
* Good, because realistic local testing (all modules built together)
* Good, because single source of truth for formatting, linting, and coding standards
* Bad, because repository size grows with module count (clone time, disk space)
* Bad, because developers must rebuild all modules when shared dependencies change
* Bad, because granular access control requires tooling (cannot restrict by repo)

## Related Design Elements

**Actors**:
* `spd-hyperspot-actor-saas-developer` - Benefits from atomic refactoring and consistent tooling
* `spd-hyperspot-actor-platform-operator` - Simplifies deployment (single repository to track)

**Requirements**:
* `spd-hyperspot-nfr-build-performance` - Monorepo affects incremental build times
* `spd-hyperspot-fr-module-lifecycle` - Modules are discoverable within workspace
