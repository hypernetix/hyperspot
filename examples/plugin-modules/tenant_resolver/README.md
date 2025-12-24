# Tenant Resolver (Gateway + Plugins) — example

This example demonstrates a **gateway module** that routes calls to one of many **plugin modules** using:

- **GTS** (Global Type System) for typed schemas and instance IDs
- **types-registry** as the runtime storage for GTS types and instances
- **ClientHub scoped clients** to bind a runtime instance (`gts_id`) to a concrete implementation

The goal is to show a clean pattern for “modules with plugins” in HyperSpot/ModKit.

## What you get in this folder

- **`tenant_resolver-sdk/`**
  - Defines the public contract (API traits + models)
  - Defines the GTS types used in this example (e.g. `ThrPluginSpec`)

- **`tenant_resolver-gw/`** (gateway module)
  - Exposes REST endpoint(s) (example: `GET /tenant-resolver/v1/root`, `GET /tenant-resolver/v1/tenants`)
  - At startup selects which plugin instance should be used (based on config + registry content)
  - At runtime simply calls the selected plugin via `ClientHub`
  - Registers a typed client `dyn TenantResolverClient` in `ClientHub` for other modules

- **`plugins/contoso_tr_plugin/`**, **`plugins/fabrikam_tr_plugin/`** (plugin modules)
  - Each plugin:
    - registers a **GTS instance** in `types-registry`
    - registers a **scoped client** in `ClientHub` bound to the same `gts_id`

## How the wiring works (high-level)

### Public client vs plugin API (two different contracts)

- **Public client (for other modules)**: `dyn TenantResolverClient`
  - registered by the gateway **without a scope**
  - consumers call it via `ctx.client_hub().get::<dyn TenantResolverClient>()?`

- **Plugin API (implemented by plugins, called by gateway)**: `dyn ThrPluginApi`
  - registered by each plugin **with a scope**
  - scope key: `ClientScope::gts_id(&instance_id)`
  - gateway resolves the selected plugin via `get_scoped::<dyn ThrPluginApi>(&scope)`

### 1) A plugin defines an instance ID as a GTS ID

Plugins generate their instance ID using the GTS type helper:

- GTS type id: `gts.x.core.plugins.thr_plugin.v1~`
- Instance ids (examples):
  - `gts.x.core.plugins.thr_plugin.v1~contoso.plugins._.thr_plugin.v1`
  - `gts.x.core.plugins.thr_plugin.v1~fabrikam.plugins._.thr_plugin.v1`

In code this is done via:

- `ThrPluginSpec::make_gts_instance_id("contoso.plugins._.thr_plugin.v1")`

### 2) A plugin registers its instance in `types-registry`

At `init()` the plugin registers a JSON document (validated by schema) in the registry:

- `gts_id`: the full instance id (string)
- `content`: serialized `ThrPluginSpec` (must match the `gts_id`)

This makes instances discoverable at runtime.

### 3) A plugin registers a scoped client in `ClientHub`

The same plugin registers its implementation under:

- interface type: `dyn ThrPluginApi`
- scope: `ClientScope::gts_id(&instance_id)`

So the gateway can later resolve *the correct implementation for a particular instance id*.

### 4) The gateway selects a plugin instance once, in `init()`

`tenant_resolver_gateway` reads config:

```yaml
modules:
  tenant_resolver_gateway:
    config:
      vendor: "Fabrikam"
```

Then it:

- lists `ThrPluginSpec` instances in `types-registry`
- filters by `vendor`
- chooses the best by `priority` (lower = higher priority)
- stores the selected `gts_id` as `active_plugin_id`

After that, gateway requests are just:

1) resolve scoped client from `ClientHub`
2) call the plugin API method

### 5) The gateway registers its public client into `ClientHub`

The gateway registers:

- interface type: `dyn TenantResolverClient`
- scope: **none** (single active gateway instance)

So other modules can call tenant resolver via:

- `ctx.client_hub().get::<dyn TenantResolverClient>()?`

## Run it (server)

The example is gated by the feature `tenant-resolver-example`.

Build/check:

```bash
cargo check -p hyperspot-server --features tenant-resolver-example
```

Run:

```bash
cargo run -p hyperspot-server --features tenant-resolver-example -- --config config/quickstart.yaml run
```

Then call:

- `GET /tenant-resolver/v1/root`
- `GET /tenant-resolver/v1/tenants?limit=50`
- `GET /tenant-resolver/v1/tenants?limit=2&cursor=<cursor-token>`

`/tenant-resolver/v1/tenants` uses ModKit OData pagination parameters:

- `limit` (required by OData rules when provided; must be > 0)
- `cursor` (opaque cursor token)
- `$select` (field projection)

## How to add a new plugin (copy/paste workflow)

1) Create a new crate under `plugins/<your_plugin>/`
2) Add `#[modkit::module(name = "<your_plugin>", deps = ["types_registry"])]`
3) Generate a stable instance id:
   - `ThrPluginSpec::make_gts_instance_id("<your.instance.name>")`
4) Register the instance in `types-registry` using that `gts_id`
5) Register your `Arc<dyn ThrPluginApi>` into `ClientHub` with scope `ClientScope::gts_id(&gts_id)`
6) Add the plugin module name into the gateway’s `deps = [...]` list so the plugin initializes before the gateway
7) Update `apps/hyperspot-server/Cargo.toml` feature `tenant-resolver-example` deps if you want it included in the example build

## Why GTS is used here

GTS gives you:

- **stable, versioned IDs** for types and instances
- **schema-driven validation** (instances are structured, not arbitrary JSON blobs)
- a clean bridge between “registry discovery” and “runtime implementation” via `gts_id`


