# Performance Specification

**Source**: Cargo.toml, README.md, ARCHITECTURE_MANIFEST.md, libs/modkit-db/

## Overview

HyperSpot is designed as a **high-performance platform** built in Rust with explicit focus on optimization at multiple levels: compile-time, runtime, and architectural. Performance is not an afterthought but a core design principle enabling "whole subsystem locally" workflows and efficient resource utilization.

**Performance Philosophy**:
- **Compile-time optimization** via Rust's zero-cost abstractions and profile-guided settings
- **Async-first architecture** with tokio runtime for high concurrency
- **Resource efficiency** enabling local development and testing of large subsystems
- **Measured optimization** - profile before optimizing, benchmark after changes
- **Quality enables performance** - 90%+ test coverage target prevents performance regressions

---

## 1. Compilation Performance

### Profile Configuration

**Release Profile** (production):
```toml
[profile.release]
codegen-units = 1        # Maximum optimization, slower builds
lto = false              # Can enable for smaller binaries (much slower builds)
opt-level = 3            # Maximum optimization (default)
```

**Development Profile** (fast iteration):
```toml
[profile.dev]
incremental = true       # Incremental compilation
codegen-units = 16       # Parallel codegen for faster builds
opt-level = 0            # No optimization (default)
```

**Test Profile** (test performance):
```toml
[profile.test]
incremental = true
codegen-units = 16
opt-level = 0            # Fast compilation for tests
```

### Optimization Strategies

**For production builds**:
- Use `cargo build --release` (applies release profile)
- Consider `lto = "thin"` for moderate binary size reduction
- Consider `strip = true` to remove debug symbols (smaller binaries)

**For development**:
- Use `cargo check` for fast feedback (no codegen)
- Use `cargo build` with dev profile (incremental, parallel)
- Profile in release mode, not debug mode

### Clippy Performance Lints

Performance-related clippy lints are **denied** at workspace level:

```toml
[workspace.lints.clippy]
perf = "deny"                    # Performance lint group
clone_on_copy = "deny"           # Avoid unnecessary clones
cast_precision_loss = "deny"     # Catch lossy casts
```

**Command**:
```bash
cargo clippy --workspace --all-targets --all-features -- -D clippy::perf
```

---

## 2. Runtime Performance

### Async Runtime

**Framework**: Tokio (multi-threaded runtime)

**Patterns**:
- All I/O operations are `async`
- Use `tokio::spawn` for concurrent tasks
- Use `CancellationToken` for graceful shutdown
- Avoid blocking operations in async context

**Example**:
```rust
use tokio_util::sync::CancellationToken;

async fn handler(ctx: &ModuleCtx) -> Result<Response> {
    // Non-blocking async operations
    let data = fetch_data().await?;
    process_async(data).await
}
```

**Anti-patterns**:
```rust
// ❌ BAD: Blocking operation in async context
async fn bad_handler() {
    std::thread::sleep(Duration::from_secs(1));  // Blocks entire thread
}

// ✅ GOOD: Async sleep
async fn good_handler() {
    tokio::time::sleep(Duration::from_secs(1)).await;  // Yields to runtime
}
```

### Database Optimizations

**Connection Pooling**:
- DbManager caches database handles per module
- Uses `DashMap` for concurrent cache access
- Connection pool configuration per module

**Configuration**:
```yaml
modules:
  my_module:
    database:
      server: "postgres_main"
      pool:
        max_conns: 10              # Max connections per pool
        acquire_timeout: "30s"     # Connection acquisition timeout
```

**SQLite Optimizations**:
```yaml
database:
  servers:
    sqlite_db:
      params:
        WAL: "true"                # Write-Ahead Logging (better concurrency)
        synchronous: "NORMAL"      # Balance safety/performance
        busy_timeout: "5000"       # Retry on lock contention
      pool:
        max_conns: 5               # SQLite benefits from fewer connections
```

**Best Practices**:
- Use prepared statements (automatic with sqlx)
- Batch operations where possible
- Use transactions for multiple writes
- Index frequently queried columns
- Use advisory locks for distributed coordination

### Caching Strategies

**Static Data**:
```rust
use std::sync::LazyLock;

static CACHED_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    load_expensive_config()
});
```

**Module-level Caching**:
```rust
// DbManager caches database handles
let handle = db_manager.get("my_module").await?;  // Cached after first call
```

**OData Query Caching**:
- Query parsing results can be cached
- Filter node trees are reusable
- Consider caching for expensive calculations

---

## 3. Architectural Performance

### Modular Design Benefits

**Selective Compilation**:
- Modules are separate crates
- Only modified modules need recompilation
- Feature flags enable conditional compilation

**In-Process vs Out-of-Process**:
- **In-process**: Direct function calls, zero serialization overhead
- **Out-of-process (OoP)**: gRPC communication, process isolation
- Choose based on requirements (isolation vs performance)

**Example Decision Matrix**:
| Module Type | Deployment | Rationale |
|-------------|-----------|-----------|
| Core business logic | In-process | Maximum performance |
| Third-party integrations | OoP | Fault isolation |
| Compute-intensive tasks | OoP | Resource isolation |
| High-frequency APIs | In-process | Low latency |

### Resource Management

**Memory Efficiency**:
- Rust's ownership prevents memory leaks
- Zero-copy patterns where possible
- Use `Arc<T>` for shared ownership
- Use `&str` instead of `String` when borrowing

**Concurrency**:
- Lock-free structures (`DashMap`) for shared state
- Minimize lock contention
- Use async channels for message passing
- Prefer `tokio::sync` primitives

---

## 4. Profiling & Benchmarking

### Profiling Tools

**CPU Profiling**:
```bash
# Install flamegraph
cargo install flamegraph

# Profile release build
cargo flamegraph --release -- <args>

# Output: flamegraph.svg (interactive visualization)
```

**Memory Profiling**:
```bash
# Using valgrind (Linux/macOS)
valgrind --tool=massif target/release/hyperspot-server

# Using heaptrack (Linux)
heaptrack target/release/hyperspot-server
```

**tokio-console** (async runtime inspection):
```bash
# Enable console in Cargo.toml
tokio = { version = "1", features = ["full", "tracing"] }
console-subscriber = "0.1"

# Run tokio-console
tokio-console
```

### Benchmarking

**Criterion** (statistical benchmarking):
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parse(c: &mut Criterion) {
    c.bench_function("parse_query", |b| {
        b.iter(|| parse_query(black_box("name eq 'test'")))
    });
}

criterion_group!(benches, benchmark_parse);
criterion_main!(benches);
```

**Running Benchmarks**:
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench my_benchmark

# Save baseline for comparison
cargo bench -- --save-baseline before
# After changes
cargo bench -- --baseline before
```

### Load Testing

**E2E Performance Tests**:
```bash
# Using pytest with performance markers
pytest testing/e2e/ -m performance

# Using Apache Bench
ab -n 1000 -c 10 http://127.0.0.1:8087/health

# Using wrk (HTTP benchmarking)
wrk -t12 -c400 -d30s http://127.0.0.1:8087/api/v1/endpoint
```

**Metrics to Track**:
- Request latency (p50, p95, p99)
- Throughput (requests per second)
- Memory usage (resident set size)
- Database connection pool utilization
- Error rate

---

## 5. Performance Testing Strategy

### Test Coverage Target

**90%+ coverage** includes performance tests:
- Unit tests: Fast, isolated component tests
- Integration tests: Module interaction performance
- E2E tests: Full system performance under load
- Performance tests: Specific performance characteristics

### Performance Test Categories

**1. Latency Tests**:
- API endpoint response times
- Database query execution times
- gRPC call latencies

**2. Throughput Tests**:
- Concurrent request handling
- Batch operation efficiency
- Connection pool saturation

**3. Resource Tests**:
- Memory usage under load
- CPU utilization patterns
- Database connection limits

**4. Scalability Tests**:
- Performance with increasing data volume
- Concurrent user simulation
- Module count impact

### Performance Regression Prevention

**CI Integration**:
```bash
# Run performance checks in CI
python scripts/ci.py all          # Includes performance lints

# Performance-specific checks
cargo bench --no-run              # Verify benchmarks compile
cargo clippy -- -D clippy::perf   # Deny performance anti-patterns
```

**Baseline Comparison**:
```bash
# Establish baseline before changes
cargo bench -- --save-baseline main

# After changes, compare
cargo bench -- --baseline main
```

---

## 6. Optimization Guidelines

### When to Optimize

**Don't optimize prematurely**:
1. Profile first - identify actual bottlenecks
2. Measure impact - benchmark before/after
3. Document rationale - explain non-obvious optimizations

**Optimization Priority**:
1. **Algorithmic complexity** - O(n²) → O(n log n) wins big
2. **I/O efficiency** - Reduce round trips, batch operations
3. **Memory allocation** - Reuse buffers, avoid clones
4. **CPU-bound work** - Optimize hot paths only

### Common Optimizations

**String Operations**:
```rust
// ❌ BAD: Unnecessary allocation
let s = format!("Hello, {}", name.to_string());

// ✅ GOOD: Direct formatting
let s = format!("Hello, {name}");

// ✅ GOOD: Use &str when possible
fn process(data: &str) { /* ... */ }
```

**Collections**:
```rust
// ✅ Pre-allocate when size is known
let mut vec = Vec::with_capacity(100);

// ✅ Use iterators (lazy evaluation)
let result: Vec<_> = items.iter()
    .filter(|x| x.is_valid())
    .map(|x| x.process())
    .collect();
```

**Async Operations**:
```rust
// ❌ BAD: Sequential async calls
let a = fetch_a().await?;
let b = fetch_b().await?;

// ✅ GOOD: Parallel async calls
let (a, b) = tokio::join!(fetch_a(), fetch_b());
```

**Database Queries**:
```rust
// ❌ BAD: N+1 queries
for user in users {
    let posts = fetch_posts(user.id).await?;
}

// ✅ GOOD: Batch query
let user_ids: Vec<_> = users.iter().map(|u| u.id).collect();
let posts = fetch_posts_batch(&user_ids).await?;
```

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Release profile optimized** (codegen-units = 1)
- [ ] **Dev profile fast** (incremental, codegen-units = 16)
- [ ] **Clippy perf lints denied** (-D clippy::perf)
- [ ] **Async runtime documented** (tokio with patterns)
- [ ] **Database pooling configured** (per-module pools)
- [ ] **Profiling tools listed** (flamegraph, valgrind, tokio-console)
- [ ] **Benchmarking framework documented** (criterion)
- [ ] **Load testing approach defined** (E2E + load tests)
- [ ] **Optimization guidelines provided** (when/how to optimize)
- [ ] **Performance testing in CI** (clippy perf, benchmarks compile)

### SHOULD Requirements (Strongly Recommended)

- [ ] Baseline benchmarks for critical paths
- [ ] Performance regression tests in CI
- [ ] Memory profiling in development
- [ ] Load testing before releases
- [ ] Performance metrics tracked over time

### MAY Requirements (Optional)

- [ ] Continuous performance monitoring
- [ ] Automated performance alerts
- [ ] Performance budget enforcement
- [ ] Custom performance visualizations

## Compliance Criteria

**Pass**: All MUST requirements met (10/10) + performance tests passing  
**Fail**: Any MUST requirement missing OR performance regressions detected

### Agent Instructions

When implementing features:
1. ✅ **ALWAYS profile before optimizing** (measure, don't guess)
2. ✅ **ALWAYS use async for I/O** (never block tokio threads)
3. ✅ **ALWAYS configure connection pools** (per-module settings)
4. ✅ **ALWAYS run clippy with perf lints** (-D clippy::perf)
5. ✅ **ALWAYS benchmark critical paths** (establish baselines)
6. ✅ **ALWAYS use release builds for profiling** (dev builds are misleading)
7. ✅ **ALWAYS document performance decisions** (rationale for optimizations)
8. ✅ **ALWAYS test under realistic load** (E2E + load tests)
9. ❌ **NEVER optimize without profiling** (premature optimization)
10. ❌ **NEVER block async runtime** (use tokio primitives)
11. ❌ **NEVER ignore performance regressions** (fix before merge)
12. ❌ **NEVER skip performance testing** (90%+ coverage includes perf)

### Performance Review Checklist

Before committing performance-sensitive code:
- [ ] Profiled with flamegraph or similar tool
- [ ] Benchmarked before/after changes
- [ ] No blocking operations in async code
- [ ] Connection pooling configured appropriately
- [ ] Clippy perf lints pass
- [ ] Memory usage measured if allocating significantly
- [ ] Load tested if changing API endpoints
- [ ] Performance impact documented in PR

---

## Reference

- **Profiling**: flamegraph, valgrind, heaptrack, tokio-console
- **Benchmarking**: criterion, cargo bench, pytest performance markers
- **Load testing**: ab, wrk, E2E tests
- **Compilation**: Cargo.toml profiles (release, dev, test)
- **Runtime**: tokio async runtime, connection pooling
- **Caching**: LazyLock, DashMap, DbManager cache
- **CI**: `python scripts/ci.py all` includes performance checks
- **Linting**: `cargo clippy -- -D clippy::perf`

---

## Summary

**Performance is a feature**:
- 🎯 **Measure first** - Profile before optimizing
- 🚀 **Optimize strategically** - Focus on hot paths
- 🔧 **Use the right tools** - flamegraph, criterion, tokio-console
- 📊 **Track metrics** - Latency, throughput, memory
- 🧪 **Test continuously** - Benchmarks in CI, load tests before release

**Golden rules**:
1. Profile → Optimize → Benchmark → Document
2. Async for I/O, never block tokio threads
3. Configure pools, cache strategically
4. Test performance, prevent regressions
