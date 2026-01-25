# ModKit Macros

This crate contains the proc-macros used by `modkit`.

In most crates you should import macros from `modkit` (it re-exports them):

```rust
use modkit::{module, lifecycle};
```

If you depend on `cf-modkit-macros` directly, the Rust crate name is `modkit_macros`:

```rust
use modkit_macros::{module, lifecycle, grpc_client};
```

## Macros

### `#[module(...)]`

Attribute macro for declaring a ModKit module and registering it via `inventory`.

Parameters:

- **`name = "..."`** (required)
- **`deps = ["..."]`** (optional)
- **`capabilities = [..]`** (optional)
  - Allowed values: `db`, `rest`, `rest_host`, `stateful`, `system`, `grpc_hub`, `grpc`
- **`ctor = <expr>`** (optional)
  - If omitted, the macro uses `Default::default()` (so your type must implement `Default`).
- **`client = <path::to::Trait>`** (optional)
  - Current behavior: compile-time checks (object-safe + `Send + Sync + 'static`) and defines `MODULE_NAME`.
  - It does not generate ClientHub registration helpers.
- **`lifecycle(...)`** (optional, used for `stateful` modules)
  - `entry = "serve"` (default: `"serve"`)
  - `stop_timeout = "30s"` (default: `"30s"`; supports `ms`, `s`, `m`, `h`)
  - `await_ready` / `await_ready = true|false` (default: `false`)

Example (stateful, no ready gating):

```rust
use modkit::module;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
#[module(
    name = "demo",
    capabilities = [stateful],
    lifecycle(entry = "serve", stop_timeout = "1s")
)]
pub struct Demo;

impl Demo {
    async fn serve(&self, _cancel: CancellationToken) -> anyhow::Result<()> {
        Ok(())
    }
}
```

Example (stateful, with ready gating):

```rust
use modkit::module;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
#[module(
    name = "demo_ready",
    capabilities = [stateful],
    lifecycle(entry = "serve", await_ready, stop_timeout = "1s")
)]
pub struct DemoReady;

impl DemoReady {
    async fn serve(
        &self,
        _cancel: CancellationToken,
        _ready: modkit::lifecycle::ReadySignal,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
```

### `#[lifecycle(...)]`

Attribute macro applied to an `impl` block. It generates a `modkit::lifecycle::Runnable` impl and an `into_module()` helper.

Parameters:

- **`method = "serve"`** (required)
- **`stop_timeout = "30s"`** (optional)
- **`await_ready` / `await_ready = true|false`** (optional)

Notes:

- If `await_ready` is enabled, the runner method must accept a `ReadySignal` as the 3rd argument.

### `#[grpc_client(...)]`

Attribute macro applied to an empty struct. It generates a wrapper struct with:

- `connect(uri)` and `connect_with_config(uri, cfg)` using `modkit_transport_grpc::client::connect_with_stack`
- `from_channel(Channel)`
- `inner_mut()`
- a compile-time check that the generated client type implements the API trait

Parameters:

- **`api = "path::to::Trait"`** (required; string literal path)
- **`tonic = "path::to::TonicClient<Channel>"`** (required; string literal type)
- **`package = "..."`** (optional; currently unused)

Minimal example:

```rust
use modkit::grpc_client;

#[grpc_client(
    api = "crate::MyApi",
    tonic = "my_proto::my_service_client::MyServiceClient<tonic::transport::Channel>",
)]
pub struct MyGrpcClient;

// You still implement `MyApi` manually for the generated client type.
```

## See also

- [ModKit unified system](../../docs/MODKIT_UNIFIED_SYSTEM.md)
- [New module guideline](../../guidelines/NEW_MODULE.md)
