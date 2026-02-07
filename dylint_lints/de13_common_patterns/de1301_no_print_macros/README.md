# DE1301 — No Print/Debug Macros in Production Code

## Rule

This lint forbids using the following macros in production Rust modules code:

- `println!`
- `eprintln!`
- `print!`
- `eprint!`
- `dbg!`

These macros bypass the project’s structured logging/observability approach and are easy to leave behind accidentally.

## Rationale

- **Observability consistency**: prefer `tracing` (or the project’s logging facade) so logs are structured, filterable, and routable.
- **Noise control**: ad-hoc stdout/stderr prints introduce noisy output in services, CI, and integration tests.
- **Accidental leakage**: `dbg!` and print macros often ship unintentionally.

## Allowed Exceptions

This lint intentionally allows these macros in the following cases:

### 1) `proc-macro` crates

Procedural macro crates may emit warnings or diagnostics during compilation.

This lint allows these macros:

- Inside `#[proc_macro]` / `#[proc_macro_attribute]` / `#[proc_macro_derive]` entrypoints
- Inside private helper functions (`fn helper() { ... }`)

But it still forbids these macros inside other public helper functions (`pub fn helper() { ... }`).

### 2) Any `build.rs`

`build.rs` scripts often need to print instructions to Cargo (e.g. `cargo:rerun-if-changed=...`) or debug build-time behavior.

### 3) Anything under `apps/*`

Application binaries may have legitimate reasons to print directly:

- CLI-style UX output
- Early bootstrap diagnostics before logging is initialized
- Very small tools where stdout is the primary interface

### 4) Tests (`#[test]` / `#[tokio::test]` / `#[cfg(test)]`)

Unit tests and test-only modules may use these macros for debug output and quick feedback.

## Examples

### Forbidden

```rust
fn main() {
    println!("hello");
    dbg!(42);
}
```

### Allowed in `build.rs`

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=src/schema.json");
    dbg!("build-time debug");
}
```

### Allowed in `apps/*`

```rust
// apps/my-tool/src/main.rs
fn main() {
    println!("Usage: my-tool <args>");
}
```

## Guidance

- Prefer `tracing::{info, warn, error, debug}` for runtime output.
- If you need temporary debugging in library/module code, use a proper logger at `debug` level.
- For code that runs **before tracing is initialized** (e.g. logging bootstrap), suppress the lint with a targeted allow and a comment explaining why:

```rust
#[allow(unknown_lints, de1301_no_print_macros)] // runs before tracing subscriber is installed
fn init_logging(...) {
    eprintln!("error during logging init");
}
```

## UI Tests

This lint includes UI tests covering:

- Forbidden usage in normal code
- Allowed usage in `apps/*`
- Allowed usage in `build.rs`
- Allowed usage in `proc-macro` crates
- Allowed usage in tests (`#[test]`, `#[tokio::test]`, `#[cfg(test)]`)
