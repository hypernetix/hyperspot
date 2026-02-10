# modkit-node-info

A standalone library for collecting system information about the current node where code is executed.

## Purpose

This library provides system information collection without any transport layer dependencies. It can be used by any CyberFabric module that needs to gather information about the execution environment.

## Features

- **Hardware-Based Node UUID**: Permanent UUID derived from machine hardware identifiers with hybrid fallback
- **System Information Collection**: OS, CPU, memory, GPU, battery, host details with all IP addresses
- **System Capabilities Detection**: Hardware and OS capabilities with cache metadata
- **Cross-Platform**: Platform-specific implementations for macOS, Linux, and Windows
- **Cache-Aware Capabilities**: Each capability includes TTL and fetch timestamp for intelligent caching

## Public API

```rust
use modkit_node_info::{get_hardware_uuid, NodeInfoCollector, Node, NodeSysInfo, NodeSysCap, SysCap};

// Get permanent hardware-based UUID for this machine
let node_id = get_hardware_uuid();  // Returns Uuid directly (no Result)

// Create collector
let collector = NodeInfoCollector::new();

// Create a Node instance for the current machine
let node = collector.create_current_node();

// Collect system information
let sysinfo = collector.collect_sysinfo(node.id)?;

// Collect system capabilities (with cache metadata)
let syscap = collector.collect_syscap(node.id)?;

// Collect both sysinfo and syscap in one call
let (sysinfo, syscap) = collector.collect_all(node.id)?;
```

### Hardware UUID

The `get_hardware_uuid()` function returns a permanent UUID based on the machine's hardware identifiers:

### Platform Support
- **macOS**: Uses `IOPlatformUUID` from IOKit (already a UUID)
- **Linux**: Uses `/etc/machine-id` or `/var/lib/dbus/machine-id` (converted to UUID)
- **Windows**: Uses `MachineGuid` from registry (already a UUID)

### Fallback Behavior
If hardware detection fails, returns a hybrid UUID pattern:
- Format: `00000000-0000-0000-xxxx-xxxxxxxxxxxx`
- Left 8 bytes: All zeros (indicates fallback)
- Right 8 bytes: Random (ensures uniqueness)

## Cache TTL Values

System capabilities use different cache TTLs based on change frequency:

| Capability Type | TTL | Reason |
|-----------------|-----|--------|
| Architecture | 1 hour | Never changes |
| RAM | 5 seconds | Changes frequently |
| CPU | 10 minutes | Rarely changes |
| OS | 2 minutes | Rarely changes |
| GPU | 10 seconds | Can change (hot-plug) |
| Battery | 3 seconds | Very dynamic |

## Usage Examples

### Basic Usage
```rust
use modkit_node_info::NodeInfoCollector;

let collector = NodeInfoCollector::new();
let node = collector.create_current_node();
let sysinfo = collector.collect_sysinfo(node.id)?;
let syscap = collector.collect_syscap(node.id)?;

println!("Node: {} ({})", node.hostname, node.id);
println!("CPU: {} cores", sysinfo.cpu.cores);
println!("Memory: {} GB used", sysinfo.memory.used_bytes / 1024 / 1024 / 1024);
```

### Working with Capabilities
```rust
let syscap = collector.collect_syscap(node.id)?;

for cap in syscap.capabilities {
    if cap.present {
        println!("{}: {} ({})",
            cap.display_name,
            cap.amount.unwrap_or(0.0),
            cap.amount_dimension.unwrap_or_else(|| "N/A".to_string())
        );
    }
}
```

## Platform-Specific Features

### GPU Detection

**NVIDIA GPUs (Linux & Windows):**
- Uses **NVML** (NVIDIA Management Library) via `nvml-wrapper` crate
- Provides detailed GPU information including memory usage
- Same library used by `nvidia-smi`
- Gracefully falls back if NVIDIA drivers not present

**Other GPUs:**
- **macOS**: Uses `system_profiler SPDisplaysDataType` for all GPUs
- **Linux**: Falls back to `lspci` for AMD/Intel GPUs
- **Windows**: Falls back to `wmic` Win32_VideoController for AMD/Intel GPUs

**Detection Strategy:**
1. Try NVML first for NVIDIA GPUs (Linux/Windows)
2. If NVML unavailable or no NVIDIA GPUs found, use platform-specific fallback
3. Returns empty list if no GPUs detected

### Battery Detection
- Uses `starship-battery` crate for cross-platform battery information
- Returns `None` for desktop systems without batteries

### IP Address Detection
- Detects local IP address used for default route
- Collects all network IPs in HostInfo.ip_addresses
- First IP is the primary one (matches Node.ip_address)

## Dependencies

- `sysinfo` - System information collection
- `machine-uid` - Hardware UUID detection
- `local-ip-address` - Local IP detection
- `starship-battery` - Battery information
- `nvml-wrapper` - NVIDIA GPU detection (Linux & Windows)
- `regex` - GPU parsing (macOS)
- `chrono` - Timestamps
- `uuid` - Node IDs

## Usage in Modules

This library is designed to be used by the `nodes-registry` module and any other module that needs to collect information about the current execution environment.

```toml
[dependencies]
modkit-node-info = { path = "../../libs/modkit-node-info" }
```

