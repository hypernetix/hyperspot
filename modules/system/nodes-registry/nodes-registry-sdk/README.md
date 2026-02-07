# Nodes Registry SDK

SDK crate for the nodes registry module.

## Overview

The `cf-nodes-registry-sdk` crate provides:

- `NodesRegistryClient` trait
- Error type `NodesRegistryError`
- Node model types (re-exported from `modkit-node-info`)

## Usage

```rust,ignore
use nodes_registry_sdk::NodesRegistryClient;

let client = hub.get::<dyn NodesRegistryClient>()?;
let nodes = client.list_nodes().await?;
```

## License

Licensed under Apache-2.0.
