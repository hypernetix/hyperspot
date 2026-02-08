
# DE0503: Plugin Client Trait Suffix

### What it does

Checks that plugin client traits in `*-sdk` crates use the `*Client` suffix instead of `*Api` or `*PluginApi`.

### Why is this bad?

In the SDK pattern used by CyberFabric, `*-sdk` crates define public API traits for consumers (often wired through the ClientHub). If those traits use inconsistent suffixes like `*Api` or `*PluginApi`:

- **The role of the trait is unclear**: is it a server-side API surface or a client interface?
- **Naming becomes inconsistent across SDK crates**: harder to find and standardize clients.
- **Refactors become noisy**: multiple patterns (`Api`, `PluginApi`, `Client`) spread across the codebase.

### Example

```rust
// ❌ Bad - plugin client trait using *PluginApi suffix
use async_trait::async_trait;

#[async_trait]
pub trait TenantResolverPluginApi: Send + Sync {
    async fn get_root_tenant(&self) -> Result<(), ()>;
}
```

```rust
// ❌ Bad - plugin client trait using *Api suffix
use async_trait::async_trait;

#[async_trait]
pub trait TenantResolverApi: Send + Sync {
    async fn get_root_tenant(&self) -> Result<(), ()>;
}
```

Use instead:

```rust
// ✅ Good - uses *Client / *PluginClient suffix
use async_trait::async_trait;

#[async_trait]
pub trait TenantResolverPluginClient: Send + Sync {
    async fn get_root_tenant(&self) -> Result<(), ()>;
}
```

### Configuration

This lint is configured to **deny** by default.

It only applies to code inside `*-sdk` crates.

In practice, it enables itself when either:

- the crate name ends with `-sdk` or `_sdk`, or
- the source file path contains a `-sdk/` path segment

It reports a violation when a trait name:

- ends with `PluginApi`, or
- ends with `Api` (and looks like a plugin/client trait)

### See Also

- [Issue #181](https://github.com/cyberfabric/cyberfabric-core/issues/181)
