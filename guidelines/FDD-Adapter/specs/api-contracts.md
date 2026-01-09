# API Contracts Specification

**Technology**: Hybrid - OpenAPI 3.x (REST) + gRPC Protocol Buffers

---

## OpenAPI Specification

**Format**: OpenAPI 3.x via utoipa macros
**Generation**: Automatic via utoipa annotations
**Location**: `docs/api/api.json`
**Runtime endpoint**: `http://localhost:8087/openapi.json`

---

## gRPC Specification

**Format**: Protocol Buffers (.proto files)
**Location**: `proto/`
**Code generation**: Via tonic-build in build.rs

---

## API Location

- REST: `modules/*/src/api/rest/`
- gRPC: `modules/*/src/api/grpc/`

---

## Validation

```bash
# Validate OpenAPI spec (design-time)
openapi-spec-validator modules/*/architecture/openapi/v1/api.yaml

# Validate OpenAPI spec (runtime)
make openapi-fetch  # Fetches from running server
openapi-spec-validator docs/api/api.json

# gRPC validation happens at compile time via tonic-build
```

---

## Installation

```bash
# Install OpenAPI validator
pip install openapi-spec-validator

# Or via requirements.txt
echo "openapi-spec-validator>=0.7.1" >> testing/requirements.txt
```
