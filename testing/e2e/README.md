# E2E Testing Guide

This directory contains end-to-end tests for the Hyperspot server.

## Prerequisites

Install Python dependencies:

```bash
pip install -r e2e/requirements.txt
```

## Running E2E Tests

The `scripts/ci.py` Python script supports two modes: **local** (default) and **Docker**.

### Option 1: Local Mode (Default - Faster for Development)

This approach runs tests against a locally running hyperspot-server:

```bash
# First, start the server in a separate terminal
make example
# OR
cargo run --bin hyperspot-server --features users-info-example -- --config config/quickstart.yaml

# Then, in another terminal, run the tests
make e2e          # or make e2e-local
# Or directly
python3 scripts/ci.py e2e
```

### Option 2: Docker Mode (Recommended for CI)

This approach builds a Docker image and runs tests in an isolated environment:

```bash
# Using make
make e2e-docker

# Or directly
python3 scripts/ci.py e2e --docker
```

## Configuration

### Environment Variables

- **`E2E_BASE_URL`**: Base URL for the API (default: `http://localhost:8087`) - only used in local mode
- **`E2E_AUTH_TOKEN`**: Optional authentication token for protected endpoints

Examples:

```bash
# Test against a different port (local mode)
E2E_BASE_URL=http://localhost:9000 make e2e

# Test with authentication
E2E_AUTH_TOKEN=your-token-here make e2e

# Combine both
E2E_BASE_URL=http://localhost:9000 E2E_AUTH_TOKEN=your-token python3 scripts/ci.py e2e

# Run in Docker mode with different base URL
python3 scripts/ci.py e2e --docker
```

### Command Line Options

The `scripts/ci.py` Python script accepts the following options:

- `--docker`: Run tests in Docker environment (default is local mode)
- `--help`: Show help message

## Types Registry Test Suite

The types-registry module has comprehensive E2E test coverage for GTS entity management:

### Test Files

- **`modules/types_registry/test_types_registry_register.py`**: Tests the `POST /types-registry/v1/entities` endpoint
  - Single and batch entity registration
  - Type and instance registration
  - Invalid entity handling
  - Mixed valid/invalid batch processing
  
- **`modules/types_registry/test_types_registry_list.py`**: Tests the `GET /types-registry/v1/entities` endpoint
  - List all entities
  - Filter by kind (type/instance)
  - Filter by vendor, package, namespace
  - Wildcard pattern matching
  - Combined filters
  
- **`modules/types_registry/test_types_registry_get.py`**: Tests the `GET /types-registry/v1/entities/{gts_id}` endpoint
  - Get entity by GTS ID
  - 404 for non-existent entities
  - Response structure validation
  - UUID format verification
  
- **`modules/types_registry/test_types_registry_validation.py`**: Tests schema validation behavior
  - Instance validation against type schemas
  - Missing required fields
  - Wrong field types
  - Nested schema validation
  
- **`modules/types_registry/test_types_registry_error_handling.py`**: Tests error handling and edge cases
  - RFC-9457 error response format
  - Malformed requests
  - Large batch handling
  - Unicode content
  - Deeply nested schemas

## File Parser Test Suite

The file-parser module has comprehensive E2E test coverage including golden-reference Markdown comparison:

### Test Files

- **`modules/file_parser/test_file_parser_info.py`**: Tests the `/file-parser/v1/info` endpoint
- **`modules/file_parser/test_file_parser_upload.py`**: Tests the `/file-parser/v1/upload` endpoint (binary upload)
- **`modules/file_parser/test_file_parser_upload_markdown.py`**: Tests the `/file-parser/v1/upload/markdown` endpoint (
  multipart)
- **`modules/file_parser/test_file_parser_parse_local.py`**: Tests the `/file-parser/v1/parse-local*` endpoints
- **`modules/file_parser/test_file_parser_parse_url.py`**: Tests the `/file-parser/v1/parse-url*` endpoints

### Golden Markdown Generation

Before running file-parser tests, you need to generate golden Markdown reference files:

```bash
# Make sure the server is running first
make example

# Generate golden markdown files
python -m e2e.modules.file_parser.generate_file_parser_golden
```

This will:

1. Scan `e2e/testdata/` for input files (PDFs, DOCX, etc.)
2. Upload each file to the API with `render_markdown=true`
3. Save the markdown responses to `e2e/testdata/md/<filename>.md`

The tests will then compare API responses against these golden files.

## Writing Tests

Tests are written using pytest and httpx. See `modules/file_parser/test_file_parser_info.py` for an example.

Key fixtures available:

- `base_url`: Returns the base URL from `E2E_BASE_URL` environment variable
- `auth_headers`: Returns authorization headers if `E2E_AUTH_TOKEN` is set
- `local_files_root`: Returns the root directory for local file parsing tests
- `file_http_server`: Starts a local HTTP server serving files from `e2e/testdata`

Example:

```python
import httpx
import pytest

@pytest.mark.asyncio
async def test_my_endpoint(base_url, auth_headers):
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/my-endpoint",
            headers=auth_headers,
        )
        assert response.status_code == 200
```

## Quick Reference

| Command                              | Mode   | Description                              |
|--------------------------------------|--------|------------------------------------------|
| `make e2e`                           | Local  | Default: Run tests against local server  |
| `make e2e-local`                     | Local  | Explicit local mode (same as `make e2e`) |
| `make e2e-docker`                    | Docker | Run tests in Docker environment          |
| `python3 scripts/ci.py e2e`          | Local  | Direct script execution (local)          |
| `python3 scripts/ci.py e2e --docker` | Docker | Direct script execution (Docker)         |

## Troubleshooting

### Server not responding (Local Mode)

If you see "Server not responding" when running local tests:

1. Make sure hyperspot-server is running
2. Check that it's listening on the correct port (default: 8087)
3. Verify the health endpoint: `curl http://localhost:8087/healthz`
4. Or use Docker mode: `make e2e-docker`

### pytest not found

Install the required dependencies:

```bash
pip install -r e2e/requirements.txt
```

### Docker build fails

Make sure Docker is running and you have sufficient disk space:

```bash
docker system df
docker system prune  # if needed
```
