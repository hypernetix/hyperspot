# Nodes Registry Module

## Overview

The Nodes Registry module manages node information in the Hyperspot deployment. A node represents a deployment unit (host, VM, container) where Hyperspot is running.

Each node contains:
- **System Information (sysinfo)**: OS, CPU, memory, GPU, battery, host details with all IP addresses
- **System Capabilities (syscap)**: Hardware and software capabilities with cache metadata
- **Node Metadata**: Hardware-based UUID, hostname, IP address
- **Custom Capabilities**: Software capabilities reported by modules (e.g., LM Studio)

## Architecture

```
modules/nodes_registry/
├── src/
│   ├── contract/          # Public API for other modules
│   │   ├── client.rs      # NodesRegistryApi trait
│   │   ├── model.rs       # Re-exports from modkit-node-info
│   │   └── error.rs       # Transport-agnostic errors
│   ├── domain/            # Business logic
│   │   ├── service.rs     # Node management service with caching
│   │   ├── node_storage.rs # In-memory storage with cache metadata
│   │   └── error.rs       # Domain errors
│   ├── api/rest/          # REST API layer
│   │   ├── dto.rs         # DTOs with serde/ToSchema
│   │   ├── handlers.rs    # Axum handlers with cache refresh
│   │   ├── routes.rs      # Route registration with OpenAPI docs
│   │   ├── mapper.rs      # Domain to DTO mapping
│   │   └── error.rs       # Problem response mapping
│   ├── gateways/          # Client implementations
│   │   └── local.rs       # Local client for ClientHub
│   ├── config.rs          # Module configuration
│   ├── module.rs          # Module registration
│   └── lib.rs             # Public exports
└── Cargo.toml
```

## Features

- **Multi-Node Support**: Store and manage multiple nodes in memory
- **Hardware-Based UUID**: Permanent node identification using machine hardware
- **Intelligent Caching**: Per-capability TTL with automatic refresh
- **Custom Capabilities**: Modules can report software capabilities
- **Cache Invalidation**: Force refresh endpoints for fresh data
- **System + Custom Merging**: Automatic merging of system and custom capabilities
- **REST API**: Clean endpoints with OpenAPI documentation
- **Thread-Safe**: Concurrent access with RwLock protection

## API Endpoints

All endpoints are registered under `/nodes`.

### List Nodes
```bash
# Basic list (node metadata only)
curl -X GET "http://localhost:8080/nodes"

# With details (includes cached sysinfo and syscap)
curl -X GET "http://localhost:8080/nodes?details=true"

# With details + force refresh (ignores cache)
curl -X GET "http://localhost:8080/nodes?details=true&force_refresh=true"
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "hostname": "my-computer",
    "ip_address": "192.168.1.100",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "sysinfo": { ... },  // Only when details=true
    "syscap": { ... }    // Only when details=true
  }
]
```

### Get Node by ID
```bash
# Basic node info
curl -X GET "http://localhost:8080/nodes/{id}"

# With details (includes cached sysinfo and syscap)
curl -X GET "http://localhost:8080/nodes/{id}?details=true"

# With details + force refresh
curl -X GET "http://localhost:8080/nodes/{id}?details=true&force_refresh=true"
```

### Get Node System Information
```bash
curl -X GET "http://localhost:8080/nodes/{id}/sysinfo"
```

**Response:**
```json
{
  "node_id": "550e8400-e29b-41d4-a716-446655440000",
  "os": {
    "name": "macOS",
    "version": "14.5",
    "arch": "arm64"
  },
  "cpu": {
    "model": "Apple M1 Pro",
    "num_cpus": 10,
    "cores": 10,
    "frequency_mhz": 3200.0
  },
  "memory": {
    "total_bytes": 34359738368,
    "available_bytes": 17179869184,
    "used_bytes": 17179869184,
    "used_percent": 50
  },
  "host": {
    "hostname": "my-computer",
    "uptime_seconds": 86400,
    "ip_addresses": ["192.168.1.100", "10.0.0.5", "127.0.0.1"]
  },
  "gpus": [
    {
      "model": "Apple M1 Pro GPU",
      "cores": 16,
      "total_memory_mb": 16384
    }
  ],
  "battery": {
    "on_battery": false,
    "percentage": 85
  },
  "collected_at": "2024-01-01T00:00:00Z"
}
```

### Get Node System Capabilities
```bash
# Uses cached data (auto-refreshes expired capabilities)
curl -X GET "http://localhost:8080/nodes/{id}/syscap"

# Force refresh all capabilities (ignores cache)
curl -X GET "http://localhost:8080/nodes/{id}/syscap?force_refresh=true"
```

**Response:**
```json
{
  "node_id": "550e8400-e29b-41d4-a716-446655440000",
  "capabilities": [
    {
      "key": "hardware:arm64",
      "category": "hardware",
      "name": "arm64",
      "display_name": "ARM64",
      "present": true,
      "details": "arm64 architecture detected",
      "cache_ttl_secs": 3600,
      "fetched_at_secs": 1704067200
    },
    {
      "key": "hardware:ram",
      "category": "hardware",
      "name": "ram",
      "display_name": "RAM",
      "present": true,
      "amount": 32.0,
      "amount_dimension": "GB",
      "details": "Total: 32.00 GB, Used: 50%",
      "cache_ttl_secs": 5,
      "fetched_at_secs": 1704067200
    },
    {
      "key": "hardware:cpu",
      "category": "hardware",
      "name": "cpu",
      "display_name": "CPU",
      "present": true,
      "amount": 10.0,
      "amount_dimension": "cores",
      "details": "Apple M1 Pro with 10 cores @ 3200 MHz",
      "cache_ttl_secs": 600,
      "fetched_at_secs": 1704067200
    },
    {
      "key": "software:lm-studio",
      "category": "software",
      "name": "lm-studio",
      "display_name": "LM Studio",
      "present": true,
      "version": "0.2.9",
      "details": "Local LLM server",
      "cache_ttl_secs": 60,
      "fetched_at_secs": 1704067200
    }
  ],
  "collected_at": "2024-01-01T00:00:00Z"
}
```

## Cache Behavior

### TTL Values
| Capability | TTL | Reason |
|------------|-----|--------|
| Architecture | 1 hour | Never changes |
| RAM | 5 seconds | Changes frequently |
| CPU | 10 minutes | Rarely changes |
| OS | 2 minutes | Rarely changes |
| GPU | 10 seconds | Can change |
| Battery | 3 seconds | Very dynamic |
| Custom Software | 60 seconds | Module-defined |

### Cache Refresh
- **Automatic**: Expired capabilities refresh on request
- **Manual**: Use `?force_refresh=true` to ignore all cache
- **Merging**: System capabilities merge with custom ones (custom overrides)

## Custom Capabilities API

Modules can report custom software capabilities:

```rust
use nodes_registry::contract::client::NodesRegistryApi;

// Set custom capabilities (e.g., LM Studio presence)
let lm_studio_caps = vec![
    SysCap {
        key: "software:lm-studio".to_string(),
        category: "software".to_string(),
        name: "lm-studio".to_string(),
        display_name: "LM Studio".to_string(),
        present: true,
        version: Some("0.2.9".to_string()),
        amount: None,
        amount_dimension: None,
        details: Some("Local LLM server".to_string()),
        cache_ttl_secs: 60,
        fetched_at_secs: chrono::Utc::now().timestamp(),
    }
];

client.set_custom_syscap(node_id, lm_studio_caps).await?;

// Remove custom capabilities
client.remove_custom_syscap(node_id, vec!["software:lm-studio".to_string()]).await?;

// Clear all custom capabilities
client.clear_custom_syscap(node_id).await?;
```

## Usage from Other Modules

```rust
use modkit::ClientHub;
use nodes_registry::contract::client::NodesRegistryApi;

// Get the client from ClientHub
let client = ctx.client_hub().get::<dyn NodesRegistryApi>()?;

// List all nodes
let nodes = client.list_nodes().await?;

// Get system info for a node (cached)
let sysinfo = client.get_node_sysinfo(node_id).await?;

// Get system capabilities for a node (cached, auto-refreshes expired)
let syscap = client.get_node_syscap(node_id).await?;
```

## Configuration

Add to your `config.yaml`:

```yaml
modules:
  nodes_registry:
    enabled: true
```

## Design Decisions

1. **In-Memory Multi-Node Storage**: Uses `NodeStorage` with thread-safe `RwLock<HashMap>` for concurrent access.

2. **Intelligent Caching**: Per-capability TTL with automatic refresh when expired. Manual refresh available via `force_refresh=true`.

3. **System + Custom Separation**: System-collected capabilities (from modkit-node-info) are stored separately from custom capabilities (set by modules), then merged for API responses.

4. **Hardware-Based UUID**: Uses permanent hardware identifiers with hybrid fallback for reliable node identification.

5. **Centralized Collection**: System information collection is centralized in modkit-node-info library for reuse across modules.

6. **Cache-Aware API**: All endpoints support cache control with detailed OpenAPI documentation.

## Implementation Details

### Storage Architecture
```rust
struct CachedNodeData {
    node: Node,
    sysinfo: Option<NodeSysInfo>,
    syscap_system: Option<NodeSysCap>,    // From modkit-node-info
    syscap_custom: HashMap<String, SysCap>, // From modules
}
```

### Cache Management
- Each capability has individual TTL and fetch timestamp
- Expired capabilities auto-refresh on request
- Force refresh ignores all cache
- Custom capabilities persist until explicitly removed

### Thread Safety
- `RwLock` allows concurrent reads
- Write locks only during updates
- No blocking for read-heavy operations
