"""E2E tests for error scenarios in nodes_registry API."""
import httpx
import pytest
import uuid


@pytest.mark.asyncio
async def test_error_invalid_uuid_format_in_get(base_url, auth_headers):
    """
    Test that invalid UUID format returns proper error.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        invalid_ids = [
            "not-a-uuid",
            "12345",
            "xyz-abc-def",
            "00000000",
            "",
        ]
        
        for invalid_id in invalid_ids:
            response = await client.get(
                f"{base_url}/nodes-registry/v1/nodes/{invalid_id}",
                headers=auth_headers,
            )
            
            if response.status_code in (401, 403) and not auth_headers:
                pytest.skip("Endpoint requires authentication")
            
            # Should return 400 or 404 for invalid UUID
            assert response.status_code in [400, 404], (
                f"Expected 400 or 404 for invalid UUID '{invalid_id}', "
                f"got {response.status_code}"
            )


@pytest.mark.asyncio
async def test_error_invalid_uuid_in_sysinfo(base_url, auth_headers):
    """
    Test that invalid UUID in sysinfo endpoint returns error.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/invalid-uuid/sysinfo",
            headers=auth_headers,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")
        
        assert response.status_code in [400, 404]


@pytest.mark.asyncio
async def test_error_invalid_uuid_in_syscap(base_url, auth_headers):
    """
    Test that invalid UUID in syscap endpoint returns error.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/invalid-uuid/syscap",
            headers=auth_headers,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")
        
        assert response.status_code in [400, 404]


@pytest.mark.asyncio
async def test_error_response_format(base_url, auth_headers):
    """
    Test that error responses follow RFC 7807 Problem Details format.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Use a valid UUID format but nonexistent node
        fake_uuid = str(uuid.uuid4())
        
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{fake_uuid}",
            headers=auth_headers,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")
        
        assert response.status_code == 404
        
        # Check content type
        content_type = response.headers.get("content-type", "")
        assert "application/problem+json" in content_type or "application/json" in content_type, (
            f"Expected problem+json content type, got {content_type}"
        )
        
        # Parse error response
        error_data = response.json()
        
        # RFC 7807 Problem Details should have these fields
        assert "title" in error_data or "error" in error_data, (
            "Error response should have 'title' or 'error' field"
        )


@pytest.mark.asyncio
async def test_error_all_nonexistent_endpoints(base_url, auth_headers):
    """
    Test that all endpoints return 404 for nonexistent node.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        fake_uuid = str(uuid.uuid4())
        
        endpoints = [
            f"/nodes-registry/v1/nodes/{fake_uuid}",
            f"/nodes-registry/v1/nodes/{fake_uuid}/sysinfo",
            f"/nodes-registry/v1/nodes/{fake_uuid}/syscap",
        ]
        
        for endpoint in endpoints:
            response = await client.get(
                f"{base_url}{endpoint}",
                headers=auth_headers,
            )
            
            if response.status_code in (401, 403) and not auth_headers:
                continue
            
            assert response.status_code == 404, (
                f"Endpoint {endpoint} should return 404 for nonexistent node, "
                f"got {response.status_code}"
            )


@pytest.mark.asyncio
async def test_error_invalid_query_parameters(base_url, auth_headers):
    """
    Test that invalid query parameters are handled gracefully.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Get a valid node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )
        
        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")
        
        nodes = list_response.json()
        node_id = nodes[0]["id"]
        
        # Test with invalid boolean values
        invalid_params = [
            {"details": "not-a-boolean"},
            {"force_refresh": "maybe"},
            {"details": "yes"},
            {"force_refresh": "no"},
        ]
        
        for params in invalid_params:
            response = await client.get(
                f"{base_url}/nodes-registry/v1/nodes/{node_id}",
                headers=auth_headers,
                params=params,
            )
            
            # Server might treat invalid booleans as false or return 400
            # Just ensure it doesn't crash
            assert response.status_code in [200, 400], (
                f"Should handle invalid params gracefully, got {response.status_code}"
            )


@pytest.mark.asyncio
async def test_error_malformed_url(base_url, auth_headers):
    """
    Test handling of malformed URLs.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        malformed_endpoints = [
            "/nodes-registry/v1/nodes//",  # Double slash
            "/nodes-registry/v1/nodes/",   # Trailing slash with no ID
        ]
        
        for endpoint in malformed_endpoints:
            response = await client.get(
                f"{base_url}{endpoint}",
                headers=auth_headers,
            )
            
            if response.status_code in (401, 403) and not auth_headers:
                continue
            
            # Should return 404 or 400, not crash
            assert response.status_code in [200, 400, 404, 405], (
                f"Should handle malformed URL gracefully, got {response.status_code}"
            )


@pytest.mark.asyncio
async def test_error_unsupported_http_methods(base_url, auth_headers):
    """
    Test that unsupported HTTP methods return 405 Method Not Allowed.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Get a valid node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )
        
        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")
        
        nodes = list_response.json()
        node_id = nodes[0]["id"]
        
        # Try unsupported methods
        response = await client.post(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}",
            headers=auth_headers,
        )
        
        # Should return 405 Method Not Allowed or 404
        assert response.status_code in [404, 405], (
            f"POST should not be allowed, got {response.status_code}"
        )
        
        response = await client.delete(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}",
            headers=auth_headers,
        )
        
        assert response.status_code in [404, 405], (
            f"DELETE should not be allowed, got {response.status_code}"
        )


@pytest.mark.asyncio
async def test_error_case_sensitive_endpoints(base_url, auth_headers):
    """
    Test that endpoint paths are case-sensitive.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Try with wrong case
        response = await client.get(
            f"{base_url}/nodes-registry/v1/Nodes",  # Capital N
            headers=auth_headers,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")
        
        # Should return 404 (case-sensitive)
        assert response.status_code == 404, (
            "Endpoints should be case-sensitive"
        )


@pytest.mark.asyncio
async def test_error_special_uuids(base_url, auth_headers):
    """
    Test handling of special UUID values.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        special_uuids = [
            "00000000-0000-0000-0000-000000000000",  # Nil UUID
            "ffffffff-ffff-ffff-ffff-ffffffffffff",  # Max UUID
        ]
        
        for test_uuid in special_uuids:
            response = await client.get(
                f"{base_url}/nodes-registry/v1/nodes/{test_uuid}",
                headers=auth_headers,
            )
            
            if response.status_code in (401, 403) and not auth_headers:
                continue
            
            # Should return 404 (these UUIDs likely don't exist)
            assert response.status_code == 404, (
                f"Special UUID {test_uuid} should return 404"
            )


@pytest.mark.asyncio
async def test_error_concurrent_nonexistent_requests(base_url, auth_headers):
    """
    Test that multiple concurrent requests for nonexistent nodes are handled.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        fake_uuids = [str(uuid.uuid4()) for _ in range(10)]
        
        # Make concurrent requests
        tasks = []
        for fake_uuid in fake_uuids:
            tasks.append(
                client.get(
                    f"{base_url}/nodes-registry/v1/nodes/{fake_uuid}",
                    headers=auth_headers,
                )
            )
        
        import asyncio
        responses = await asyncio.gather(*tasks)
        
        # All should return 404
        for i, response in enumerate(responses):
            if response.status_code in (401, 403) and not auth_headers:
                continue
            
            assert response.status_code == 404, (
                f"Request {i} should return 404"
            )
