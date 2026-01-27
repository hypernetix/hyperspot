# Dependencies Guidelines

This document outlines the preferred dependencies and libraries to use in the CyberFabric project.

## Serialization

### YAML

**Always use `serde-saphyr` for YAML serialization/deserialization, not `serde_yaml`.**

- **Package name in Cargo.toml**: `serde-saphyr`
- **Import in Rust code**: `use serde_saphyr;`
- **Reason**: `serde_yaml` is deprecated and unmaintained. `serde-saphyr` is the actively maintained fork.

#### Example Usage

```rust
use serde_saphyr;
use std::collections::HashMap;

// Serialization
let data = HashMap::from([("key", "value")]);
let yaml_string = serde_saphyr::to_string(&data)?;

// Deserialization
let parsed: HashMap<String, String> = serde_saphyr::from_str(&yaml_string)?;
```

**Note**: `serde-saphyr` does not provide a `Value` type like `serde_yaml` did. For generic YAML parsing, use `HashMap<String, serde_json::Value>` or define a specific struct.
