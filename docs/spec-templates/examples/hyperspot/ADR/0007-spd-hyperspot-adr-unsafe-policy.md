# ADR-0007: No Unsafe Code Without Audit and Justification

**Date**: 2024-01-18

**Status**: Accepted

**ID**: `spd-hyperspot-adr-unsafe-policy`

## Context and Problem Statement

Rust's `unsafe` keyword allows bypassing compiler safety checks for performance-critical code or FFI boundaries. While sometimes necessary, unsafe code introduces the same vulnerability classes that Rust's type system prevents (memory unsafety, data races). For a multi-tenant platform handling sensitive data, we need clear guidelines on when and how unsafe code is acceptable.

## Decision Drivers

* Must maintain Rust's safety guarantees in 99%+ of the codebase
* Must allow unsafe when truly necessary (FFI, performance-critical sections)
* Must ensure unsafe code is reviewed and documented
* Must make unsafe code blocks auditable for security reviews
* Must prevent casual use of unsafe for convenience

## Considered Options

* Ban unsafe completely (no exceptions)
* Allow unsafe with mandatory code review and safety documentation
* Allow unsafe freely (trust developers to use correctly)
* Allow unsafe only in designated low-level crates

## Decision Outcome

Chosen option: "Allow unsafe with mandatory code review and safety documentation", because some use cases (FFI to system libraries, performance-critical data structures) legitimately require unsafe. Each unsafe block must have a comment explaining why it's needed and what invariants guarantee safety. Code review must verify these invariants hold.

### Consequences

* Good, because unsafe is used only when necessary and justified
* Good, because unsafe blocks are documented with safety invariants
* Good, because security audits can focus on small surface area of unsafe code
* Good, because default remains safe Rust (99%+ of codebase)
* Good, because performance-critical sections can use unsafe when profiling proves necessity
* Bad, because requires discipline and code review to enforce policy
* Bad, because some valid unsafe code may be rejected if reviewers are conservative
* Bad, because safety invariants in comments may drift from actual code

## Related Design Elements

**Actors**:
* `spd-hyperspot-actor-saas-developer` - Must justify unsafe code in reviews
* `spd-hyperspot-actor-platform-operator` - Benefits from minimized unsafe surface

**Requirements**:
* `spd-hyperspot-nfr-compilation-safety` - Minimize unsafe to preserve guarantees
* `spd-hyperspot-nfr-memory` - Unsafe may be needed for zero-copy optimizations
