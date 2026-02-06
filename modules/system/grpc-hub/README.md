# gRPC Hub Module

This module builds and hosts the single `tonic::Server` instance for the process.

## Overview

The `cf-grpc-hub` crate implements the `grpc_hub` module and is responsible for:

- Hosting the gRPC server
- Installing gRPC services collected from other modules

## Configuration

```yaml
modules:
  grpc_hub:
    config:
      # TCP example: "0.0.0.0:50051"
      # Unix example (unix only): "uds:///tmp/hyperspot.sock"
      # Windows named pipe example (windows only): "pipe://\\\\.\\pipe\\hyperspot"
      listen_addr: "0.0.0.0:50051"
```

## License

Licensed under Apache-2.0.
