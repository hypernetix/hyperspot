# Reliability (Error Handling, Retries, Lifecycle)

**Version**: 1.0  
**Purpose**: Keep retry logic, graceful shutdown, and error semantics consistent and safe.  
**Scope**: Apps/modules, especially networked and background-task code.  

## Error semantics

- Prefer typed errors at boundaries; map to `Problem` for HTTP responses.
- Avoid swallowing errors; add context but keep sensitive details out of responses/logs.

## Retries & timeouts

- Use retry logic for transient failures (especially gRPC) with bounded backoff and clear operation naming.
- Ensure timeouts exist for external calls (HTTP/gRPC/DB) where appropriate.

## Lifecycle / shutdown

- Background tasks should support cancellation (cooperative shutdown).
- Prefer module lifecycle helpers (ModKit) for long-running tasks.

## Validation Criteria

- [ ] Retries are bounded and do not amplify load during outages.
- [ ] Timeouts are explicit for external calls.
- [ ] Background tasks terminate on shutdown signals.
- [ ] Errors are observable (tracing) and actionable.

## Examples

✅ Valid:
- Retry gRPC `Unavailable`/`DeadlineExceeded` with backoff and a tracing span.

❌ Invalid:
- Infinite retry loops without backoff or cancellation support.

---

**Source**: `docs/MODKIT_UNIFIED_SYSTEM.md` (lifecycle), `libs/modkit-transport-grpc/src/rpc_retry.rs`, `Cargo.toml` (tower-http timeout).  
**Last Updated**: 2026-02-05

