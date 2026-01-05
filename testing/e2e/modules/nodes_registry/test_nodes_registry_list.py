"""E2E tests for nodes_registry list endpoints."""
import httpx
import pytest


@pytest.mark.asyncio
async def test_list_nodes_basic(base_url, auth_headers):
    """
    Test GET /nodes-registry/v1/nodes endpoint without details.

    This test verifies that the nodes listing endpoint returns
    a list of registered nodes with basic information.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        # If no auth token is set and we get 401/403, skip the test
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )

        # Assert successful response
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. "
            f"Response: {response.text}"
        )

        # Assert response is JSON
        assert response.headers.get("content-type", "").startswith(
            "application/json"
        ), "Response should be JSON"

        # Parse JSON response
        data = response.json()
        assert isinstance(data, list), "Response should be a JSON array"

        # At least the current node should be registered
        assert len(data) >= 1, "At least one node should be registered (current node)"

        # Validate structure of each node
        for node in data:
            assert isinstance(node, dict), "Each node should be a JSON object"
            
            # Required fields
            assert "id" in node, "Node should have 'id' field"
            assert "hostname" in node, "Node should have 'hostname' field"
            assert "created_at" in node, "Node should have 'created_at' field"
            assert "updated_at" in node, "Node should have 'updated_at' field"
            
            # Validate field types
            assert isinstance(node["id"], str), "id should be a string (UUID)"
            assert isinstance(node["hostname"], str), "hostname should be a string"
            assert isinstance(node["created_at"], str), "created_at should be a string (ISO datetime)"
            assert isinstance(node["updated_at"], str), "updated_at should be a string (ISO datetime)"
            
            # Optional fields
            if "ip_address" in node and node["ip_address"] is not None:
                assert isinstance(node["ip_address"], str), "ip_address should be a string"
            
            # When details is not requested, sysinfo and syscap should not be present
            assert "sysinfo" not in node or node["sysinfo"] is None, (
                "sysinfo should not be included without details=true"
            )
            assert "syscap" not in node or node["syscap"] is None, (
                "syscap should not be included without details=true"
            )


@pytest.mark.asyncio
async def test_list_nodes_with_details(base_url, auth_headers):
    """
    Test GET /nodes-registry/v1/nodes?details=true endpoint.

    This test verifies that the nodes listing endpoint returns
    detailed information including sysinfo and syscap when requested.
    """
    async with httpx.AsyncClient(timeout=15.0) as client:
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
            params={"details": "true"},
        )

        # If no auth token is set and we get 401/403, skip the test
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )

        # Assert successful response
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. "
            f"Response: {response.text}"
        )

        # Parse JSON response
        data = response.json()
        assert isinstance(data, list), "Response should be a JSON array"
        assert len(data) >= 1, "At least one node should be registered"

        # Validate detailed structure of each node
        for node in data:
            # Basic fields
            assert "id" in node
            assert "hostname" in node
            
            # When details=true, sysinfo and syscap should be present
            assert "sysinfo" in node, "sysinfo should be included with details=true"
            assert "syscap" in node, "syscap should be included with details=true"
            
            # Validate sysinfo structure
            if node["sysinfo"] is not None:
                sysinfo = node["sysinfo"]
                assert isinstance(sysinfo, dict), "sysinfo should be an object"
                
                # Required sysinfo fields
                assert "node_id" in sysinfo
                assert "os" in sysinfo
                assert "cpu" in sysinfo
                assert "memory" in sysinfo
                assert "host" in sysinfo
                assert "gpus" in sysinfo
                assert "collected_at" in sysinfo
                
                # Validate os info
                assert isinstance(sysinfo["os"], dict)
                assert "name" in sysinfo["os"]
                assert "version" in sysinfo["os"]
                assert "arch" in sysinfo["os"]
                
                # Validate cpu info
                assert isinstance(sysinfo["cpu"], dict)
                assert "model" in sysinfo["cpu"]
                assert "num_cpus" in sysinfo["cpu"]
                assert "cores" in sysinfo["cpu"]
                assert "frequency_mhz" in sysinfo["cpu"]
                assert isinstance(sysinfo["cpu"]["num_cpus"], int)
                assert sysinfo["cpu"]["num_cpus"] > 0
                
                # Validate memory info
                assert isinstance(sysinfo["memory"], dict)
                assert "total_bytes" in sysinfo["memory"]
                assert "available_bytes" in sysinfo["memory"]
                assert "used_bytes" in sysinfo["memory"]
                assert "used_percent" in sysinfo["memory"]
                assert sysinfo["memory"]["total_bytes"] > 0
                
                # Validate host info
                assert isinstance(sysinfo["host"], dict)
                assert "hostname" in sysinfo["host"]
                assert "uptime_seconds" in sysinfo["host"]
                assert "ip_addresses" in sysinfo["host"]
                assert isinstance(sysinfo["host"]["ip_addresses"], list)
                
                # Validate gpus is a list
                assert isinstance(sysinfo["gpus"], list)
            
            # Validate syscap structure
            if node["syscap"] is not None:
                syscap = node["syscap"]
                assert isinstance(syscap, dict), "syscap should be an object"
                
                # Required syscap fields
                assert "node_id" in syscap
                assert "capabilities" in syscap
                assert "collected_at" in syscap
                
                # Validate capabilities array
                assert isinstance(syscap["capabilities"], list)
                
                # If capabilities exist, validate their structure
                for cap in syscap["capabilities"]:
                    assert isinstance(cap, dict)
                    assert "key" in cap
                    assert "category" in cap
                    assert "name" in cap
                    assert "display_name" in cap
                    assert "present" in cap
                    assert isinstance(cap["present"], bool)
                    assert "cache_ttl_secs" in cap
                    assert "fetched_at_secs" in cap
