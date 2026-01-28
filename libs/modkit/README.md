# ModKit

Declarative module system and common runtime utilities used across CyberFabric.

## Overview

The `cf-modkit` crate provides:

- Module registration and lifecycle (inventory-based discovery)
- `ClientHub` for typed in-process clients
- REST/OpenAPI helpers (`OperationBuilder`, `OpenApiRegistry`, RFC-9457 `Problem`)
- Runtime helpers (module registry/manager, lifecycle helpers)

## Features

- **`db` (default)**: Enables DB integration (depends on `cf-modkit-db`), including:
  - `DatabaseCapability` (migrations contract)
  - `DbOptions::Manager` (runtime DB manager support)
  - DB handle resolution in `ModuleCtx` / `ModuleContextBuilder`

### Build without DB

To build `cf-modkit` without pulling in `cf-modkit-db` and its transitive dependencies:

```bash
cargo build -p cf-modkit --no-default-features
```

## License

Licensed under Apache-2.0.
