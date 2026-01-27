# Module Orchestrator Module

System module for service discovery.

## Overview

The `cf-module-orchestrator` crate implements the `module_orchestrator` module.

It:

- Registers `DirectoryClient` in `ClientHub` for in-process modules
- Exposes the `DirectoryService` gRPC service (via `grpc_hub`)
- Uses the runtime `ModuleManager` for instance tracking and service resolution

## License

Licensed under Apache-2.0.
