# HyperSpot Server - Quickstart Guide

Copy-paste commands to explore HyperSpot's API. For project overview, see [README.md](../README.md).

---

## Start the Server

```bash
# With example modules (tenant-resolver, users-info)
make example

# Or minimal (no example modules)
make quickstart
```

Server runs on `http://127.0.0.1:8087`. Open a **new terminal** to test.

---

## Health & OpenAPI

```bash
# Health check (JSON)
curl -s http://127.0.0.1:8087/health | python3 -m json.tool
# {"status": "healthy", "timestamp": "2026-01-15T15:01:02.000Z"}

# Kubernetes liveness probe
curl -s http://127.0.0.1:8087/healthz
# ok

# OpenAPI 3.1 spec
curl -s http://127.0.0.1:8087/openapi.json | python3 -m json.tool | head -50
```

**Interactive docs:** <http://127.0.0.1:8087/docs>

---

## Module APIs

Each module has its own QUICKSTART.md with detailed examples:

| Module | Description | Quickstart |
|--------|-------------|------------|
| File Parser | Parse PDF, DOCX, HTML, Markdown, images into structured blocks | [QUICKSTART.md](../modules/file_parser/QUICKSTART.md) |
| Nodes Registry | Hardware and system information for HyperSpot nodes | [QUICKSTART.md](../modules/system/nodes_registry/QUICKSTART.md) |
| Tenant Resolver | Multi-tenant hierarchy management | [QUICKSTART.md](../modules/system/tenant_resolver/QUICKSTART.md) |
| Types Registry | GTS (Global Type System) schema registry | [QUICKSTART.md](../modules/system/types-registry/QUICKSTART.md) |

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
| Empty tenant-resolver | Use `make example` |
| Connection refused | Server not running |

---

## Further Reading

- [ARCHITECTURE_MANIFEST.md](ARCHITECTURE_MANIFEST.md) - Architecture principles
- [MODKIT_UNIFIED_SYSTEM.md](MODKIT_UNIFIED_SYSTEM.md) - Module system
- [../guidelines/NEW_MODULE.md](../guidelines/NEW_MODULE.md) - Create modules
