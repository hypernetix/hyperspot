# New Module Scaffold Generator

Automated tool to generate minimal, production-ready Hyperspot modules following `guidelines/NEW_MODULE.md`.

## Usage

```bash
python scripts/new-module-scaffold/main.py <module_name> [--force] [--validate]
```

### Arguments

- `module_name` - Module name in snake_case (e.g., `users_info`, `types_registry`)
- `--force` - Overwrite existing files (optional)
- `--validate` - Run `cargo check` and `cargo fmt` after generation (optional)

### Example

```bash
# Generate a new module
python scripts/new-module-scaffold/main.py my_new_module

# Generate and validate
python scripts/new-module-scaffold/main.py my_new_module --validate
```

## What Gets Generated

The script generates a parent directory under `modules/` containing both crates:

```
modules/<module_name>/
├── <module_name>-sdk/     # SDK crate (public API)
└── <module_name>/         # Module implementation crate
```

### 1. SDK Crate (`<name>-sdk`)

Transport-agnostic public API:
- `src/api.rs` - API trait with `health()` method
- `src/models.rs` - Model types (no serde)
- `src/errors.rs` - Error enum
- `Cargo.toml` - Minimal dependencies

### 2. Module Crate (`<name>`)

Full implementation with DDD-light structure:
- `src/module.rs` - Module struct with `#[modkit::module]` macro
- `src/local_client.rs` - Local client implementing SDK trait
- `src/config.rs` - Typed configuration
- `src/domain/` - Business logic layer
  - `service.rs` - Domain service with health check
  - `error.rs` - Domain errors + SDK error conversion
  - `ports.rs` - Output port traits (EventPublisher)
  - `repo.rs` - Repository trait placeholder
- `src/api/rest/` - REST transport layer
  - `dto.rs` - DTOs with serde + ToSchema
  - `handlers.rs` - Axum handlers
  - `routes.rs` - OperationBuilder route registration
  - `error.rs` - DomainError → Problem (RFC-9457) mapping
- `tests/smoke.rs` - Basic integration test

## Manual Wiring Steps

After generation, you must manually apply these changes:

### 1. Root `Cargo.toml`

Add to `[workspace].members`:
```toml
"modules/<name>/<name>-sdk",
"modules/<name>/<name>",
```

### 2. Server `apps/hyperspot-server/Cargo.toml`

Add to dependencies:
```toml
<name> = { path = "../../modules/<name>/<name>" }
```

### 3. Server `apps/hyperspot-server/src/registered_modules.rs`

Add import:
```rust
use <name> as _;
```

## Verification

```bash
# Check compilation
cargo check --workspace

# Format code
cargo fmt --all

# Run tests
cargo test --workspace

# Start server and test health endpoint
cargo run --bin hyperspot-server -- --config config/quickstart.yaml run

# Test health endpoint
curl http://127.0.0.1:8087/<kebab-name>/v1/health

# Check OpenAPI documentation
open http://127.0.0.1:8087/docs
```

## Generated Features

✅ **SDK Pattern** - Separate public API crate for consumers  
✅ **ClientHub Registration** - Local client auto-registered  
✅ **REST Endpoint** - Health check at `GET /<kebab>/v1/health`  
✅ **OpenAPI** - Automatic schema generation via utoipa  
✅ **SecurityCtx** - All API methods accept `&SecurityCtx`  
✅ **RFC-9457 Errors** - Problem Details error mapping  
✅ **Inventory Discovery** - Module auto-discovered via `#[modkit::module]`  
✅ **Integration Test** - Basic smoke test included  

## What's NOT Included (Minimal Template)

❌ Database layer (SeaORM)  
❌ SSE (Server-Sent Events)  
❌ gRPC / OoP (Out-of-Process)  
❌ Plugin architecture  

To add these features, edit the generated files following `guidelines/NEW_MODULE.md`.

## Naming Conventions

| Input | Derived | Example |
|-------|---------|---------|
| snake_case | `module_name` | `users_info` |
| PascalCase | Type names | `UsersInfo` |
| kebab-case | REST paths | `/users-info/v1` |
| snake_sdk | Rust imports | `users_info_sdk` |

## File Locations

- **Script**: `scripts/new-module-scaffold/main.py`
- **Spec**: `scripts/new-module-scaffold/spec.md`
- **Design**: `scripts/new-module-scaffold/design.md`
- **Tasks**: `scripts/new-module-scaffold/tasks.md`
- **Guidelines**: `guidelines/NEW_MODULE.md`

## Troubleshooting

### "ERROR: Invalid module name"
Module name must be snake_case matching `^[a-z0-9_]+$`.

### "ERROR: File already exists"
Use `--force` to overwrite existing files, or choose a different module name.

### "ERROR: modules/ directory not found"
Run the script from the workspace root directory.

### Compilation errors after generation
1. Ensure you applied all manual wiring steps
2. Run `cargo check --workspace` to see specific errors
3. Run `cargo fmt --all` to fix formatting
4. Check that SDK imports use underscores (e.g., `my_module_sdk`, not `my-module-sdk`)

## Next Steps

After generating your module:

1. ✅ Apply manual wiring instructions
2. ✅ Verify compilation with `cargo check`
3. ✅ Format code with `cargo fmt`
4. ✅ Test health endpoint
5. 📝 Implement your domain logic
6. 📝 Add more REST endpoints (following the health endpoint pattern)
7. 📝 Add database support (if needed, see NEW_MODULE.md Step 8)
8. 📝 Add SSE support (if needed, see NEW_MODULE.md Step 9)
9. 📝 Write comprehensive tests

## References

- [NEW_MODULE.md](../../guidelines/NEW_MODULE.md) - Complete module creation guide
- [MODKIT_UNIFIED_SYSTEM.md](../../docs/MODKIT_UNIFIED_SYSTEM.md) - ModKit architecture
- [SECURE-ORM.md](../../docs/SECURE-ORM.md) - Secure ORM with tenant isolation
- [examples/modkit/users_info/](../../examples/modkit/users_info/) - Reference implementation
