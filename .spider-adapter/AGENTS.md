# Spider Adapter: hyperspot

**Extends**: `../.spider/AGENTS.md`

**Version**: 1.0  
**Last Updated**: 2026-02-05  

---

## Variables

**While Spider is enabled**, remember these variables:

| Variable | Value | Description |
|----------|-------|-------------|
| `{spider_adapter_path}` | Directory containing this AGENTS.md | Root path for Spider Adapter navigation |

Use `{spider_adapter_path}` as the base path for all relative Spider Adapter file references.

---

## Navigation Rules

ALWAYS open and follow `specs/project-structure.md` WHEN creating files, adding modules, or navigating the codebase

ALWAYS open and follow `specs/tech-stack.md` WHEN writing code, choosing technologies, or adding dependencies

ALWAYS open and follow `specs/dependencies.md` WHEN changing `Cargo.toml`, adding dependencies, or discussing dependency policy

ALWAYS open and follow `specs/conventions.md` WHEN writing code, naming files/functions/variables, or reviewing code style

ALWAYS open and follow `specs/testing.md` WHEN writing tests, changing test infrastructure, or debugging test failures

ALWAYS open and follow `specs/build-deploy.md` WHEN building, releasing, or changing CI/Makefile/scripts

ALWAYS open and follow `specs/api-contracts.md` WHEN creating or consuming REST/gRPC APIs, OpenAPI, or OData behavior

ALWAYS open and follow `specs/security.md` WHEN handling authentication/authorization, secrets, PII, or database access control

ALWAYS open and follow `specs/data-governance.md` WHEN working with persistence, migrations, schemas, or multi-tenancy boundaries

ALWAYS open and follow `specs/observability.md` WHEN adding logging/tracing/metrics or debugging runtime behavior

ALWAYS open and follow `specs/performance.md` WHEN optimizing hot paths, concurrency, caching, or memory usage

ALWAYS open and follow `specs/reliability.md` WHEN adding retries/timeouts, shutdown/lifecycle logic, or error-handling policies

ALWAYS open and follow `specs/compliance.md` WHEN changing supply-chain security, licenses, release governance, or policy tooling

ALWAYS sign commits with DCO: use `git commit -s` for all commits, or enable auto-signing with `git config --global format.signoff true`

ALWAYS open and follow `specs/patterns.md` WHEN designing modules/layers, ClientHub usage, or refactoring architecture

ALWAYS open and follow `specs/gts.md` WHEN working with GTS identifiers, types registry, or docs validation

ALWAYS open and follow `{spider_path}/schemas/artifacts.schema.json` WHEN working with artifacts.json

ALWAYS open and follow `{spider_path}/requirements/artifacts-registry.md` WHEN working with artifacts.json

ALWAYS open and follow `artifacts.json` WHEN registering Spider artifacts, updating codebase paths, changing traceability settings, or running Spider validation
