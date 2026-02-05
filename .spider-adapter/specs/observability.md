# Observability (Logging, Tracing, OpenTelemetry)

**Version**: 1.0  
**Purpose**: Standardize how we instrument code for debugging and production visibility.  
**Scope**: All runtime code (apps + modules), especially request-handling and background tasks.  

## Tracing

- Use `tracing` for structured logs and spans.
- Prefer span-based instrumentation for cross-cutting flows (HTTP request, gRPC call, background job).

## OpenTelemetry

- The workspace includes OpenTelemetry crates for exporting traces (OTLP).
- Keep instrumentation lightweight in hot paths; prefer sampling and structured context.

## Validation Criteria

- [ ] New external calls (DB, network, gRPC) are instrumented with spans where useful.
- [ ] Log messages avoid secrets/PII.
- [ ] Errors include enough context to diagnose without dumping sensitive data.

## Examples

✅ Valid:
- Wrap a gRPC call with a `tracing` span and include an operation name.

❌ Invalid:
- Log full request payloads containing credentials or secrets.

---

**Source**: `Cargo.toml` (tracing + OpenTelemetry deps), `docs/MODKIT_UNIFIED_SYSTEM.md`, `libs/modkit-transport-grpc/src/rpc_retry.rs` (span patterns).  
**Last Updated**: 2026-02-05

