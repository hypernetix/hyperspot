"""E2E tests for nodes_registry get node endpoint."""
import httpx
import pytest
import uuid


@pytest.mark.asyncio
async def test_get_node_by_id(base_url, auth_headers):
    """
    Test GET /nodes-registry/v1/nodes/{id} endpoint.

    This test verifies that we can retrieve a specific node by its UUID.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # First, get the list of nodes to obtain a valid node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert list_response.status_code == 200
        nodes = list_response.json()
        assert len(nodes) >= 1, "At least one node should exist"

        # Get the first node's ID
        node_id = nodes[0]["id"]

        # Now fetch that specific node
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}",
            headers=auth_headers,
        )

        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. "
            f"Response: {response.text}"
        )

        # Parse response
        node = response.json()
        assert isinstance(node, dict), "Response should be a JSON object"

        # Validate it's the node we requested
        assert node["id"] == node_id
        assert "hostname" in node
        assert "created_at" in node
        assert "updated_at" in node

        # Without details, sysinfo and syscap should not be present
        assert "sysinfo" not in node or node["sysinfo"] is None
        assert "syscap" not in node or node["syscap"] is None


@pytest.mark.asyncio
async def test_get_node_by_id_with_details(base_url, auth_headers):
    """
    Test GET /nodes-registry/v1/nodes/{id}?details=true endpoint.

    This test verifies that we can retrieve detailed information about a specific node.
    """
    async with httpx.AsyncClient(timeout=15.0) as client:
        # Get a valid node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert list_response.status_code == 200
        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch node with details
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}",
            headers=auth_headers,
            params={"details": "true"},
        )

        assert response.status_code == 200
        node = response.json()

        # Validate detailed information is present
        assert node["id"] == node_id
        assert "sysinfo" in node
        assert "syscap" in node

        # Validate sysinfo structure (comprehensive check)
        if node["sysinfo"] is not None:
            sysinfo = node["sysinfo"]
            
            # OS information
            assert sysinfo["os"]["name"] != ""
            assert sysinfo["os"]["version"] != ""
            assert sysinfo["os"]["arch"] in ["x86_64", "aarch64", "x86", "arm"]
            
            # CPU information
            assert sysinfo["cpu"]["model"] != ""
            assert sysinfo["cpu"]["num_cpus"] > 0
            assert sysinfo["cpu"]["cores"] > 0
            assert sysinfo["cpu"]["frequency_mhz"] > 0
            
            # Memory information
            assert sysinfo["memory"]["total_bytes"] > 0
            assert sysinfo["memory"]["used_percent"] <= 100
            assert sysinfo["memory"]["available_bytes"] <= sysinfo["memory"]["total_bytes"]
            
            # Host information
            assert sysinfo["host"]["hostname"] != ""
            assert sysinfo["host"]["uptime_seconds"] >= 0
            assert len(sysinfo["host"]["ip_addresses"]) > 0

        # Validate syscap structure
        if node["syscap"] is not None:
            syscap = node["syscap"]
            assert syscap["node_id"] == node_id
            assert isinstance(syscap["capabilities"], list)
            assert "collected_at" in syscap


@pytest.mark.asyncio
async def test_get_node_with_force_refresh(base_url, auth_headers):
    """
    Test GET /nodes-registry/v1/nodes/{id}?details=true&force_refresh=true endpoint.

    This test verifies that force_refresh parameter works for individual node retrieval.
    """
    async with httpx.AsyncClient(timeout=20.0) as client:
        # Get a valid node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert list_response.status_code == 200
        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch with force_refresh
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}",
            headers=auth_headers,
            params={"details": "true", "force_refresh": "true"},
        )

        assert response.status_code == 200
        node = response.json()

        # Validate response structure
        assert node["id"] == node_id
        assert "sysinfo" in node
        assert "syscap" in node

        # Verify that syscap was refreshed
        if node["syscap"] is not None and node["syscap"]["capabilities"]:
            for cap in node["syscap"]["capabilities"]:
                assert "fetched_at_secs" in cap
                assert isinstance(cap["fetched_at_secs"], int)
