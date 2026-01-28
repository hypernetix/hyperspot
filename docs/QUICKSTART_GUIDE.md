# HyperSpot Server - Quickstart Guide

Start HyperSpot server and verify it works. For project overview, see [README.md](../README.md).

---

## Start the Server

```bash
# With example modules (tenant-resolver, users-info)
make example

# Or minimal (no example modules)
make quickstart
```

Server runs on `http://127.0.0.1:8087`.

---

## Verify It's Running

```bash
curl -s http://127.0.0.1:8087/health
# {"status": "healthy", "timestamp": "..."}
```

---

## API Documentation

### Interactive Documentation

Open <http://127.0.0.1:8087/docs> in your browser for the full API reference with interactive testing.

### OpenAPI Spec

```bash
curl -s http://127.0.0.1:8087/openapi.json > openapi.json
```

### Module Examples

Each module has a QUICKSTART.md with minimal curl examples:

- [File Parser](../modules/file_parser/QUICKSTART.md) - Parse documents into structured blocks
- [Nodes Registry](../modules/system/nodes_registry/QUICKSTART.md) - Hardware and system info
- [Tenant Resolver](../modules/system/tenant_resolver/QUICKSTART.md) - Multi-tenant hierarchy
- [Types Registry](../modules/system/types-registry/QUICKSTART.md) - GTS schema registry

> **Note:** Module quickstarts show basic usage only. Use `/docs` for complete API documentation.

---

## Stop the Server

```bash
pkill -f hyperspot-server
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Port 8087 in use | `pkill -f hyperspot-server` |
| Empty tenant-resolver | Use `make example` instead of `make quickstart` |
| Connection refused | Server not running - check logs |

---

## Further Reading

- [/docs](http://127.0.0.1:8087/docs) - Full API reference
- [ARCHITECTURE_MANIFEST.md](ARCHITECTURE_MANIFEST.md) - Architecture principles
- [MODKIT_UNIFIED_SYSTEM.md](MODKIT_UNIFIED_SYSTEM.md) - Module system
- [../guidelines/NEW_MODULE.md](../guidelines/NEW_MODULE.md) - Create modules
