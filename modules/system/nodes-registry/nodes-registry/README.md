# Nodes Registry Module

Node inventory and node system information for CyberFabric.

## Overview

The `cf-nodes-registry` crate implements the `nodes_registry` module.

The module manages node information (host/VM/container) and provides REST endpoints to:

- List nodes
- Get node by ID
- Get node sysinfo (`/nodes/{id}/sysinfo`)
- Get node syscap (`/nodes/{id}/syscap`)

## Configuration

```yaml
modules:
  nodes_registry:
    config:
      enabled: true
```

## License

Licensed under Apache-2.0.
