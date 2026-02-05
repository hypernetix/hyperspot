# Performance (Hyperspot)

**Version**: 1.0  
**Purpose**: Capture practical guidance for performance-sensitive code and how we avoid regressions.  
**Scope**: Hot paths in libs/modules and request-handling code.  

## Practical guidance

- Prefer data structures with good cache locality and avoid known slow patterns (e.g., avoid `LinkedList` per clippy config).
- Be mindful of stack usage and large allocations (clippy has stack-size thresholds).
- Use lock-free or low-contention patterns where appropriate (`arc-swap`, `dashmap`, `parking_lot` are in the stack).
- Instrument before optimizing; use tracing spans to find bottlenecks.

## Validation Criteria

- [ ] New code avoids obvious anti-patterns (unbounded clones, huge stack arrays, etc.).
- [ ] Performance-sensitive changes include measurement notes (bench, profiling, or at least tracing evidence).
- [ ] Clippy and custom lints remain passing.

## Examples

✅ Valid:
- Replace shared mutable state hotspots with `arc-swap` for read-mostly access.

❌ Invalid:
- Introduce a `LinkedList` for general-purpose collections.

---

**Source**: `clippy.toml` (disallowed types + thresholds), `Cargo.toml` (dashmap/arc-swap/parking_lot), `docs/ARCHITECTURE_MANIFEST.md`.  
**Last Updated**: 2026-02-05

