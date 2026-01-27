# E2E Tests for nodes_registry Module

This directory contains comprehensive end-to-end tests for the `nodes_registry` module, which manages node information in CyberFabric deployments.

## Test Files

### 1. `test_nodes_registry_list.py`
Tests for the nodes listing endpoint (`GET /nodes-registry/v1/nodes`):
- **test_list_nodes_basic**: Verify basic node listing without details
- **test_list_nodes_with_details**: Verify detailed listing including sysinfo and syscap

### 2. `test_nodes_registry_get.py`
Tests for retrieving individual nodes (`GET /nodes-registry/v1/nodes/{id}`):
- **test_get_node_by_id**: Get a specific node by UUID
- **test_get_node_by_id_with_details**: Get detailed node information
- **test_get_node_with_force_refresh**: Test force refresh for individual nodes

### 3. `test_nodes_registry_sysinfo.py`
Tests for system information endpoint (`GET /nodes-registry/v1/nodes/{id}/sysinfo`):
- **test_get_node_sysinfo**: Basic sysinfo retrieval
- **test_get_node_sysinfo_os_details**: Validate OS information structure
- **test_get_node_sysinfo_cpu_details**: Validate CPU information and constraints
- **test_get_node_sysinfo_memory_details**: Validate memory information
- **test_get_node_sysinfo_host_details**: Validate host information (hostname, uptime, IPs)
- **test_get_node_sysinfo_gpu_details**: Validate GPU information structure

### 4. `test_nodes_registry_syscap.py`
Tests for system capabilities endpoint (`GET /nodes-registry/v1/nodes/{id}/syscap`):
- **test_get_node_syscap**: Basic syscap retrieval
- **test_get_node_syscap_capabilities_structure**: Validate capability structure
- **test_get_node_syscap_categories**: Verify capability categories
- **test_get_node_syscap_with_force_refresh**: Test cache invalidation
- **test_get_node_syscap_caching**: Verify syscap caching within TTL
- **test_get_node_syscap_present_vs_absent**: Verify both present and absent capabilities

### 5. `test_nodes_registry_integration.py`
Integration tests for complete workflows:
- **test_complete_node_discovery_workflow**: Test list → get → sysinfo → syscap workflow
- **test_node_information_consistency**: Verify data consistency across requests
- **test_multiple_nodes_scenario**: Handle multiple nodes correctly
- **test_performance_list_vs_individual_queries**: Compare batch vs individual queries

### 6. `test_nodes_registry_errors.py`
Comprehensive error handling and edge case tests:
- **test_error_invalid_uuid_format_in_get**: Verify error handling for malformed UUIDs in GET endpoint
- **test_error_invalid_uuid_in_sysinfo**: Invalid UUID handling for sysinfo endpoint
- **test_error_invalid_uuid_in_syscap**: Invalid UUID handling for syscap endpoint
- **test_error_response_format**: Validate RFC 7807 Problem Details format
- **test_error_all_nonexistent_endpoints**: 404 responses for non-existent nodes across all endpoints
- **test_error_invalid_query_parameters**: Handling of invalid query parameters
- **test_error_malformed_url**: Handling of malformed URLs
- **test_error_unsupported_http_methods**: Verify 405 for unsupported HTTP methods
- **test_error_case_sensitive_endpoints**: Verify endpoints are case-sensitive
- **test_error_special_uuids**: Handling of special UUID values (nil, max)
- **test_error_concurrent_nonexistent_requests**: Concurrent requests for non-existent nodes

## Test Coverage

The e2e tests cover:

✅ **All REST API endpoints** (4 endpoints):
- `GET /nodes-registry/v1/nodes` - List all nodes
- `GET /nodes-registry/v1/nodes/{id}` - Get specific node
- `GET /nodes-registry/v1/nodes/{id}/sysinfo` - Get system information
- `GET /nodes-registry/v1/nodes/{id}/syscap` - Get system capabilities

✅ **Query parameters**:
- `details=true` - Include detailed information
- `force_refresh=true` - Invalidate cache

✅ **Data validation**:
- Response structure and types
- Required and optional fields
- Data constraints (e.g., CPU count > 0, memory percent ≤ 100)
- Timestamp validity
- UUID format validation

✅ **Error scenarios**:
- Non-existent node IDs (404 responses)
- Invalid UUID formats (malformed UUIDs)
- RFC 7807 Problem Details format validation
- Content-Type verification (application/problem+json)
- Error code consistency across endpoints
- Status code consistency (HTTP status = problem.status)
- Required error fields validation
- Descriptive error messages

✅ **Caching behavior**:
- Cache hit scenarios
- Cache invalidation with force_refresh
- TTL respect
- Timestamp consistency

✅ **Integration scenarios**:
- Complete discovery workflows
- Data consistency across endpoints
- Multi-node handling
- Performance characteristics

## Test Patterns and Conventions

### Authentication Handling
All tests check for 401/403 responses and skip if authentication is required but no token is provided:
```python
if response.status_code in (401, 403) and not auth_headers:
    pytest.skip("Endpoint requires authentication")
```
