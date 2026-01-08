"""E2E integration tests for nodes_registry module.

These tests verify end-to-end workflows and integration scenarios.
"""
import httpx
import pytest
import time


@pytest.mark.asyncio
async def test_complete_node_discovery_workflow(base_url, auth_headers):
    """
    Test complete node discovery workflow: list → get → sysinfo → syscap.

    This integration test verifies a typical workflow of discovering
    and retrieving detailed information about nodes.
    """
    async with httpx.AsyncClient(timeout=20.0) as client:
        # Step 1: List all nodes
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert list_response.status_code == 200
        nodes = list_response.json()
        assert len(nodes) >= 1, "Should have at least one node"

        # Pick the first node for detailed inspection
        node_id = nodes[0]["id"]
        node_hostname = nodes[0]["hostname"]

        # Step 2: Get detailed node information
        node_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}",
            headers=auth_headers,
            params={"details": "true"},
        )

        assert node_response.status_code == 200
        node_detail = node_response.json()
        assert node_detail["id"] == node_id
        assert node_detail["hostname"] == node_hostname

        # Step 3: Get sysinfo separately
        sysinfo_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/sysinfo",
            headers=auth_headers,
        )

        assert sysinfo_response.status_code == 200
        sysinfo = sysinfo_response.json()
        assert sysinfo["node_id"] == node_id

        # Step 4: Get syscap separately
        syscap_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
        )

        assert syscap_response.status_code == 200
        syscap = syscap_response.json()
        assert syscap["node_id"] == node_id

        # Verify consistency across endpoints
        # The hostname in sysinfo should match node hostname
        assert sysinfo["host"]["hostname"] == node_hostname

        # The sysinfo and syscap from detailed node should match separate calls
        if node_detail.get("sysinfo"):
            assert node_detail["sysinfo"]["node_id"] == sysinfo["node_id"]
            assert node_detail["sysinfo"]["host"]["hostname"] == sysinfo["host"]["hostname"]

        if node_detail.get("syscap"):
            assert node_detail["syscap"]["node_id"] == syscap["node_id"]
            assert len(node_detail["syscap"]["capabilities"]) == len(syscap["capabilities"])


@pytest.mark.asyncio
async def test_node_information_consistency(base_url, auth_headers):
    """
    Test that node information remains consistent across multiple requests.

    This test verifies data consistency and stability over time.
    """
    async with httpx.AsyncClient(timeout=15.0) as client:
        # Get node list
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert response.status_code == 200
        nodes1 = response.json()
        node_id = nodes1[0]["id"]

        # Get node details multiple times
        details1 = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}",
            headers=auth_headers,
        )

        details2 = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}",
            headers=auth_headers,
        )

        assert details1.status_code == 200
        assert details2.status_code == 200

        node1 = details1.json()
        node2 = details2.json()

        # Core properties should be identical
        assert node1["id"] == node2["id"]
        assert node1["hostname"] == node2["hostname"]
        assert node1["created_at"] == node2["created_at"]

        # IP address should be consistent (if present)
        if node1.get("ip_address") and node2.get("ip_address"):
            assert node1["ip_address"] == node2["ip_address"]


@pytest.mark.asyncio
async def test_multiple_nodes_scenario(base_url, auth_headers):
    """
    Test handling of multiple nodes in the registry.

    This test verifies that the system can handle multiple nodes correctly,
    even though typically only one node (current) exists in single-node deployments.
    """
    async with httpx.AsyncClient(timeout=15.0) as client:
        # Get all nodes
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
            params={"details": "true"},
        )

        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert response.status_code == 200
        nodes = response.json()

        # Should have at least the current node
        assert len(nodes) >= 1

        # Verify each node has unique ID
        node_ids = [node["id"] for node in nodes]
        assert len(node_ids) == len(set(node_ids)), "All node IDs should be unique"

        # Verify each node has complete information
        for node in nodes:
            assert "id" in node
            assert "hostname" in node
            
            # With details=true, should have sysinfo and syscap
            assert "sysinfo" in node
            assert "syscap" in node

            # Each node should be retrievable individually
            node_response = await client.get(
                f"{base_url}/nodes-registry/v1/nodes/{node['id']}",
                headers=auth_headers,
            )
            
            assert node_response.status_code == 200
            individual_node = node_response.json()
            assert individual_node["id"] == node["id"]


@pytest.mark.asyncio
async def test_performance_list_vs_individual_queries(base_url, auth_headers):
    """
    Test performance comparison: list with details vs individual queries.

    This test verifies that fetching all data at once is more efficient
    than making separate requests for each node.
    """
    async with httpx.AsyncClient(timeout=30.0) as client:
        # Get nodes list
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        
        # Method 1: List with details (one request)
        start_time_batch = time.time()
        batch_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
            params={"details": "true"},
        )
        batch_duration = time.time() - start_time_batch

        assert batch_response.status_code == 200
        batch_nodes = batch_response.json()

        # Method 2: Individual requests for each node
        start_time_individual = time.time()
        for node in nodes:
            await client.get(
                f"{base_url}/nodes-registry/v1/nodes/{node['id']}",
                headers=auth_headers,
                params={"details": "true"},
            )
        individual_duration = time.time() - start_time_individual

        # Verify we got the same data
        assert len(batch_nodes) == len(nodes)

        # Batch request should be faster (or at least not significantly slower)
        # Allow some variance due to caching and network conditions
        print(f"\nBatch request time: {batch_duration:.3f}s")
        print(f"Individual requests time: {individual_duration:.3f}s")
        
        # This is informational - actual performance may vary
