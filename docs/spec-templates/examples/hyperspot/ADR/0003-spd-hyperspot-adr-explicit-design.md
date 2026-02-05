# ADR-0003: Explicit Over Implicit Design

**Date**: 2024-01-16

**Status**: Accepted

**ID**: `spd-hyperspot-adr-explicit-design`

## Context and Problem Statement

Frameworks often use "magic" conventions (auto-discovery by naming patterns, global singletons, reflection-based injection) to reduce boilerplate. While convenient, these implicit mechanisms make code harder to understand, debug, and statically analyze. For a platform targeting LLM-assisted development and long-term maintainability, we need predictable, traceable behavior.

## Decision Drivers

* Must be understandable by reading code linearly (no "action at a distance")
* Must support static analysis tools and IDE navigation (go-to-definition works)
* Must be explainable to LLMs without deep framework knowledge
* Must make dependencies and data flow explicit for security audits
* Must avoid runtime surprises from convention-over-configuration

## Considered Options

* Convention-based framework (Rails-style magic, auto-discovery by naming)
* Explicit dependency injection with compile-time registration (Rust inventory)
* Reflection-based runtime discovery (Java annotations, Python decorators)
* Configuration-driven wiring (Spring XML, dependency injection containers)

## Decision Outcome

Chosen option: "Explicit dependency injection with compile-time registration", because Rust's inventory crate provides automatic registration without runtime reflection or naming conventions. Dependencies are declared explicitly in code, making them traceable via IDE and grep, while compile-time registration eliminates manual wiring boilerplate.

### Consequences

* Good, because all dependencies are visible in function signatures (easy to trace)
* Good, because IDE "find usages" and "go to definition" work correctly
* Good, because no global state or singletons (except compile-time registry)
* Good, because LLMs can understand code structure from static analysis
* Good, because security audits can follow data flow explicitly
* Bad, because requires more boilerplate than convention-based frameworks
* Bad, because developers accustomed to "magic" frameworks face steeper onboarding
* Bad, because refactoring may require updating multiple explicit call sites

## Related Design Elements

**Actors**:
* `spd-hyperspot-actor-saas-developer` - Benefits from predictable, traceable code
* `spd-hyperspot-actor-platform-operator` - Benefits from explicit configuration

**Requirements**:
* `spd-hyperspot-fr-configuration` - Configuration must be explicit, not convention-based
* `spd-hyperspot-fr-module-lifecycle` - Module dependencies declared explicitly
