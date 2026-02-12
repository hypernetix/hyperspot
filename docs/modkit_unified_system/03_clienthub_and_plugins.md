# Typed ClientHub and Plugin Architecture

## ClientHub Overview

The **ClientHub** provides type-safe client resolution for inter-module communication. It supports both in-process and remote clients:

- **In-process clients** — direct function calls within the same process
- **Remote clients** — gRPC clients for OoP modules (resolved via DirectoryClient)
- **Scoped clients** — multiple implementations of the same interface keyed by scope (for plugins)

### Client types

- **`*-sdk` crate** defines the trait & types exposed to other modules.
- **Module crate** implements a local adapter that implements the SDK trait for in-process communication.
- **gRPC clients** implement the same SDK trait for remote communication.
- Consumers resolve the typed client from ClientHub by interface type (+ optional scope).

## In-Process vs Remote Clients

| Aspect       | In-Process              | Remote (OoP)               |
|--------------|-------------------------|----------------------------|
| Transport    | Direct call             | gRPC                       |
| Latency      | Nanoseconds             | Milliseconds               |
| Isolation    | Shared process          | Separate process           |
| Contract     | Trait in `*-sdk/` crate | Trait in `*-sdk/` crate    |
| Registration | `ClientHub::register()` | DirectoryClient + gRPC client + `ClientHub::register()` |

## Publish in `init` (provider module)

```rust
#[async_trait::async_trait]
impl Module for MyModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        let cfg = ctx.module_config::<crate::config::Config>();
        let svc = std::sync::Arc::new(domain::service::MyService::new(ctx.db.clone(), cfg));
        self.service.store(Some(svc.clone()));

        let api: std::sync::Arc<dyn my_module_sdk::MyModuleApi> =
            std::sync::Arc::new(crate::domain::local_client::MyModuleLocalClient::new(svc));

        ctx.client_hub().register::<dyn my_module_sdk::MyModuleApi>(api);
        Ok(())
    }
}
```

## Consume (consumer module)

```rust
let api = ctx.client_hub().get::<dyn my_module_sdk::MyModuleApi>()?;
```

## Scoped Clients (for Plugins)

For plugin-like scenarios where multiple implementations of the same interface coexist, use scoped clients:

```rust
use modkit::client_hub::ClientScope;

// Plugin registers with a scope (e.g., GTS instance ID)
let scope = ClientScope::gts_id("gts.x.core.modkit.plugin.v1~vendor.pkg.my_module.plugin.v1~acme.test._.plugin.v1");
ctx.client_hub().register_scoped::<dyn MyPluginClient>(scope, plugin_impl);

// Main module resolves the selected plugin
let scope = ClientScope::gts_id(&selected_instance_id);
let plugin = ctx.client_hub().get_scoped::<dyn MyPluginClient>(&scope)?;
```

### Key points

- Scoped clients are independent from global (unscoped) clients
- Use `ClientScope::gts_id()` for GTS-based plugin IDs
- See `docs/MODKIT_PLUGINS.md` for the complete plugin architecture guide

## Plugin Architecture Overview

ModKit’s plugin system enables **module + plugins** patterns where:

- **Main module** registers plugin **schemas** (GTS type definitions)
- **Plugins** register their **instances** (metadata + scoped client)
- Consumers resolve plugins via **scoped ClientHub** using GTS instance IDs

### Flow

1. **Main module** registers plugin schema with GTS
2. **Plugin** starts, registers scoped client under `ClientScope::gts_id(instance_id)`
3. **Main module** resolves plugin by instance ID via `ClientHub::get_scoped()`
4. **Requests** flow through the scoped client to the plugin implementation

### Example: Plugin registration

```rust
// In plugin module's init()
let scope = ClientScope::gts_id(&instance_id);
let client = Arc::new(MyPluginClient::new(config));
ctx.client_hub().register_scoped::<dyn MyPluginClient>(scope, client);
```

### Example: Main module resolves plugin

```rust
// In main module handler
let scope = ClientScope::gts_id(&selected_instance_id);
let plugin = ctx.client_hub().get_scoped::<dyn MyPluginClient>(&scope)?;
let result = plugin.process(&ctx, input).await?;
```

## ClientHub API Reference

### Registration

```rust
// Global (unscoped) client
ctx.client_hub().register::<dyn MyModuleApi>(api);

// Scoped client (plugins)
ctx.client_hub().register_scoped::<dyn MyPluginClient>(scope, plugin);
```

### Resolution

```rust
// Global client
let api = ctx.client_hub().get::<dyn MyModuleApi>()?;

// Scoped client
let plugin = ctx.client_hub().get_scoped::<dyn MyPluginClient>(&scope)?;

// Try scoped client (returns None if not found)
let plugin = ctx.client_hub().try_get_scoped::<dyn MyPluginClient>(&scope);
```

### Removal

```rust
// Remove global client
let removed = ctx.client_hub().remove::<dyn MyModuleApi>();

// Remove scoped client
let removed = ctx.client_hub().remove_scoped::<dyn MyPluginClient>(&scope);
```

## Error handling

```rust
use modkit::client_hub::ClientHubError;

match ctx.client_hub().get::<dyn MyModuleApi>() {
    Ok(api) => { /* use api */ }
    Err(ClientHubError::NotFound { type_key }) => { /* handle missing client */ }
    Err(ClientHubError::TypeMismatch { type_key }) => { /* handle type mismatch */ }
}
```

## Best practices

- **SDK traits**: Define in `*-sdk` crate, require `Send + Sync + 'static`.
- **Local adapters**: Implement SDK trait in module crate, register in `init()`.
- **gRPC clients**: Use `modkit_transport_grpc::client` utilities (`connect_with_stack`, `connect_with_retry`).
- **Plugins**: Use `ClientScope::gts_id()` for instance IDs; register scoped clients.
- **Error handling**: Convert domain errors to SDK errors and to `Problem` for REST.
- **Testing**: Register mock clients in tests using the same trait.

## Quick checklist

- [ ] Define SDK trait with `async_trait` and `SecurityContext` first param.
- [ ] Implement local adapter in module crate.
- [ ] Register client in `init()`: `ctx.client_hub().register::<dyn Trait>(api)`.
- [ ] Consume client: `ctx.client_hub().get::<dyn Trait>()?`.
- [ ] For plugins: use `ClientScope::gts_id()` and `register_scoped()`.
- [ ] For OoP: use gRPC client utilities and register both local and remote clients.
