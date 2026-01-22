# Nodes Registry - Quickstart

Provides hardware and system information for all running HyperSpot nodes.

## Examples

### List All Nodes

```bash
curl -s http://127.0.0.1:8087/nodes-registry/v1/nodes | python3 -m json.tool
```

**Output:**
```json
[
    {
        "id": "35b975fc-3c13-c04e-d62a-43c7623895e5",
        "hostname": "your-hostname",
        "ip_address": "192.168.1.100",
        "created_at": "2026-01-15T15:01:02.000Z",
        "updated_at": "2026-01-15T15:01:02.000Z"
    }
]
```

### Get Node by ID

```bash
NODE_ID=$(curl -s http://127.0.0.1:8087/nodes-registry/v1/nodes | python3 -c "import sys,json; print(json.load(sys.stdin)[0]['id'])")
curl -s "http://127.0.0.1:8087/nodes-registry/v1/nodes/$NODE_ID" | python3 -m json.tool
```

### Get System Info

```bash
curl -s "http://127.0.0.1:8087/nodes-registry/v1/nodes/$NODE_ID/sysinfo" | python3 -m json.tool
```

**Output:**
```json
{
    "node_id": "35b975fc-3c13-c04e-d62a-43c7623895e5",
    "os": {"name": "Ubuntu", "version": "24.04", "arch": "x86_64"},
    "cpu": {"model": "Intel Core i7-1165G7", "num_cpus": 8, "cores": 4, "frequency_mhz": 2803.0},
    "memory": {"total_bytes": 16624349184, "used_bytes": 9171423232, "used_percent": 55},
    "host": {"hostname": "your-hostname", "uptime_seconds": 26268},
    "gpus": [],
    "collected_at": "2026-01-15T15:05:11.234Z"
}
```

### Get System Capabilities

```bash
curl -s "http://127.0.0.1:8087/nodes-registry/v1/nodes/$NODE_ID/syscap" | python3 -m json.tool
```

**Output:**
```json
{
    "node_id": "35b975fc-3c13-c04e-d62a-43c7623895e5",
    "capabilities": [
        {"key": "hardware:ram", "category": "hardware", "name": "ram", "present": true, "amount": 15.48, "amount_dimension": "GB"},
        {"key": "hardware:cpu", "category": "hardware", "name": "cpu", "present": true, "amount": 4.0, "amount_dimension": "cores"},
        {"key": "os:linux", "category": "os", "name": "linux", "present": true, "version": "24.04"}
    ]
}
```
