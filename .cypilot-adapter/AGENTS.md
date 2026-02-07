# Cypilot Adapter: hyperspot

**Extends**: `../.cypilot/AGENTS.md`

**Version**: 1.0  
**Last Updated**: 2026-02-05  

---

## Variables

**While Cypilot is enabled**, remember these variables:

| Variable | Value | Description |
|----------|-------|-------------|
| `{cypilot_adapter_path}` | Directory containing this AGENTS.md | Root path for Cypilot Adapter navigation |

Use `{cypilot_adapter_path}` as the base path for all relative Cypilot Adapter file references.

---

## Project Overview

This repository is a **modular monolith** built on top of **CyberFabric**.

- **CyberFabric base**: core apps/libraries live under `apps/`, `libs/`, etc.
- **Subsystems / modules**: each subsystem is a module under `modules/<module_name>/`.
- **Cypilot registry convention**: subsystems are registered as `children[]` of the root `cyberfabric` system in `{cypilot_adapter_path}/artifacts.json`.
- **Docs convention**: each module keeps its artifacts under `modules/<module_name>/docs/`.

---

## Navigation Rules

ALWAYS sign commits with DCO: use `git commit -s` for all commits

ALWAYS open and follow `{cypilot_path}/requirements/artifacts-registry.md` WHEN working with artifacts.json

ALWAYS open and follow `artifacts.json` WHEN registering Cypilot artifacts, updating codebase paths, changing traceability settings, or running Cypilot validation

ALWAYS open and follow `CONTRIBUTING.md` WHEN setting up development environment, creating feature branches, running quality checks (make all, cargo clippy, cargo fmt), signing commits with DCO, writing commit messages, creating pull requests, or understanding the review process

---

## Module Rules

ALWAYS register new modules under `modules/<module_name>/` as a `children[]` entry of the root `cyberfabric` system in `artifacts.json` WHEN adding a new module / subsystem

ALWAYS open `guidelines/NEW_MODULE.md#table-of-contents` WHEN starting to define requirements or architecture design or implement any module — review structure before diving into specific steps

ALWAYS open `guidelines/NEW_MODULE.md#canonical-project-layout` WHEN creating new module directory structure, deciding where to place files, or understanding SDK pattern

ALWAYS open `guidelines/NEW_MODULE.md#step-1-project--cargo-setup` WHEN creating Cargo.toml for new module, setting up SDK crate, or configuring workspace dependencies

ALWAYS open `guidelines/NEW_MODULE.md#step-2-data-types-naming-matrix` WHEN naming data types, deciding between Entity/Model/DTO, or mapping DB layer to API layer types

ALWAYS open `guidelines/NEW_MODULE.md#step-3-errors-management` WHEN implementing error handling, creating DomainError, mapping errors to Problem (RFC-9457), or defining SDK errors

ALWAYS open `guidelines/NEW_MODULE.md#step-4-sdk-crate-public-api-surface` WHEN creating SDK trait, defining public API, adding models.rs/errors.rs/api.rs to SDK crate

ALWAYS open `guidelines/NEW_MODULE.md#step-5-domain-layer-business-logic` WHEN implementing domain service, creating repository traits, defining domain events, or adding business logic

ALWAYS open `guidelines/NEW_MODULE.md#step-6-module-wiring--lifecycle` WHEN using #[modkit::module] macro, implementing Module trait, registering clients in ClientHub, or configuring module lifecycle

ALWAYS open `guidelines/NEW_MODULE.md#step-7-rest-api-layer-optional` WHEN adding REST endpoints, creating DTOs, implementing handlers, using OperationBuilder, or adding OData support

ALWAYS open `guidelines/NEW_MODULE.md#step-8-infrastorage-layer-optional` WHEN adding SeaORM entities, implementing repositories, using SecureConn, or creating database migrations

ALWAYS open `guidelines/NEW_MODULE.md#step-9-sse-integration-optional` WHEN adding Server-Sent Events, implementing SseBroadcaster, or creating real-time event streams

ALWAYS open `guidelines/NEW_MODULE.md#step-10-local-client-implementation` WHEN implementing local client adapter, bridging domain service to SDK trait, or registering in ClientHub

ALWAYS open `guidelines/NEW_MODULE.md#step-11-register-module-in-hyperspot-server` WHEN registering module in hyperspot-server, adding to Cargo.toml, or importing in registered_modules.rs

ALWAYS open `guidelines/NEW_MODULE.md#step-12-testing` WHEN writing module tests, creating SecurityContext for tests, or implementing integration tests

ALWAYS open `guidelines/NEW_MODULE.md#step-13-out-of-process-oop-module-support-optional` WHEN creating out-of-process module, implementing gRPC service, or setting up OoP binary

ALWAYS open `guidelines/NEW_MODULE.md#step-14-plugin-based-modules-gateway--plugins-pattern` WHEN implementing plugin architecture, creating gateway module, or registering scoped clients via GTS
