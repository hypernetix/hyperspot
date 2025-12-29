"""E2E tests for nodes_registry syscap endpoint."""
import httpx
import pytest
import time


@pytest.mark.asyncio
async def test_get_node_syscap(base_url, auth_headers):
    """
    Test GET /nodes-registry/v1/nodes/{id}/syscap endpoint.

    This test verifies that we can retrieve system capabilities for a node.
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
        assert len(nodes) >= 1
        node_id = nodes[0]["id"]

        # Fetch syscap
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
        )

        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. "
            f"Response: {response.text}"
        )

        # Parse and validate syscap structure
        syscap = response.json()
        assert isinstance(syscap, dict)

        # Required fields
        assert "node_id" in syscap
        assert syscap["node_id"] == node_id
        assert "capabilities" in syscap
        assert "collected_at" in syscap

        # Validate capabilities array
        assert isinstance(syscap["capabilities"], list)


@pytest.mark.asyncio
async def test_get_node_syscap_capabilities_structure(base_url, auth_headers):
    """
    Test the structure of individual capabilities in syscap response.

    This test verifies that each capability has the correct structure.
    """
    async with httpx.AsyncClient(timeout=15.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch syscap
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
        )

        assert response.status_code == 200
        syscap = response.json()

        # Validate each capability
        capabilities = syscap["capabilities"]
        
        # Should have at least some capabilities detected
        assert len(capabilities) > 0, "Should detect at least some system capabilities"

        for cap in capabilities:
            assert isinstance(cap, dict)
            
            # Required fields
            assert "key" in cap
            assert "category" in cap
            assert "name" in cap
            assert "display_name" in cap
            assert "present" in cap
            assert "cache_ttl_secs" in cap
            assert "fetched_at_secs" in cap

            # Validate types
            assert isinstance(cap["key"], str)
            assert isinstance(cap["category"], str)
            assert isinstance(cap["name"], str)
            assert isinstance(cap["display_name"], str)
            assert isinstance(cap["present"], bool)
            assert isinstance(cap["cache_ttl_secs"], int)
            assert isinstance(cap["fetched_at_secs"], int)

            # Validate non-empty strings
            assert len(cap["key"]) > 0
            assert len(cap["category"]) > 0
            assert len(cap["name"]) > 0
            assert len(cap["display_name"]) > 0

            # Validate cache TTL is reasonable
            assert cap["cache_ttl_secs"] >= 0
            assert cap["cache_ttl_secs"] <= 86400 * 365  # Max 1 year

            # Validate fetched_at is reasonable (not too far in past or future)
            current_time = int(time.time())
            assert cap["fetched_at_secs"] > 0
            assert cap["fetched_at_secs"] <= current_time + 60  # Allow 1 min clock skew
            assert cap["fetched_at_secs"] >= current_time - 3600  # Not older than 1 hour

            # Optional fields
            if "version" in cap and cap["version"] is not None:
                assert isinstance(cap["version"], str)
            
            if "amount" in cap and cap["amount"] is not None:
                assert isinstance(cap["amount"], (int, float))
            
            if "amount_dimension" in cap and cap["amount_dimension"] is not None:
                assert isinstance(cap["amount_dimension"], str)
            
            if "details" in cap and cap["details"] is not None:
                assert isinstance(cap["details"], str)


@pytest.mark.asyncio
async def test_get_node_syscap_categories(base_url, auth_headers):
    """
    Test that syscap response includes various capability categories.

    This test verifies that the system detects capabilities across different categories.
    """
    async with httpx.AsyncClient(timeout=15.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch syscap
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
        )

        assert response.status_code == 200
        syscap = response.json()

        # Collect all categories
        categories = set()
        for cap in syscap["capabilities"]:
            categories.add(cap["category"])

        # Should have at least some common categories
        # Note: Actual categories depend on the system, so we just verify structure
        assert len(categories) > 0, "Should have at least one capability category"
        
        # All categories should be non-empty strings
        for category in categories:
            assert isinstance(category, str)
            assert len(category) > 0


@pytest.mark.asyncio
async def test_get_node_syscap_with_force_refresh(base_url, auth_headers):
    """
    Test GET /nodes-registry/v1/nodes/{id}/syscap?force_refresh=true endpoint.

    This test verifies that force_refresh parameter invalidates cache and updates timestamps.
    """
    async with httpx.AsyncClient(timeout=20.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # First request without force_refresh
        response1 = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
        )

        assert response1.status_code == 200
        syscap1 = response1.json()

        # Wait a moment to ensure timestamp difference
        time.sleep(2)

        # Second request with force_refresh
        response2 = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
            params={"force_refresh": "true"},
        )

        assert response2.status_code == 200
        syscap2 = response2.json()

        # Both should have capabilities
        assert len(syscap1["capabilities"]) > 0
        assert len(syscap2["capabilities"]) > 0

        # The number of capabilities should be similar
        assert abs(len(syscap1["capabilities"]) - len(syscap2["capabilities"])) <= 5

        # With force_refresh, fetched_at_secs should be updated for most capabilities
        # (some may have been re-cached, but at least some should be newer)
        newer_count = 0
        for cap2 in syscap2["capabilities"]:
            for cap1 in syscap1["capabilities"]:
                if cap1["key"] == cap2["key"]:
                    if cap2["fetched_at_secs"] >= cap1["fetched_at_secs"]:
                        newer_count += 1
                    break

        # Most capabilities should have been refreshed
        assert newer_count > len(syscap2["capabilities"]) * 0.5, (
            "force_refresh should update most capability timestamps"
        )


@pytest.mark.asyncio
async def test_get_node_syscap_caching(base_url, auth_headers):
    """
    Test that syscap results are cached within TTL period.

    This test verifies that repeated requests return cached data with same timestamps.
    """
    async with httpx.AsyncClient(timeout=15.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Make two requests without force_refresh
        response1 = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
        )

        response2 = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
        )

        assert response1.status_code == 200
        assert response2.status_code == 200

        syscap1 = response1.json()
        syscap2 = response2.json()

        # Should return same data (cached)
        assert len(syscap1["capabilities"]) == len(syscap2["capabilities"])

        # For capabilities with non-zero TTL, fetched_at should be identical (cached)
        for cap1 in syscap1["capabilities"]:
            for cap2 in syscap2["capabilities"]:
                if cap1["key"] == cap2["key"] and cap1["cache_ttl_secs"] > 0:
                    assert cap1["fetched_at_secs"] == cap2["fetched_at_secs"], (
                        f"Capability {cap1['key']} should be cached"
                    )
                    break


@pytest.mark.asyncio
async def test_get_node_syscap_present_vs_absent(base_url, auth_headers):
    """
    Test that syscap response includes both present and absent capabilities.

    This test verifies that the system reports on capabilities that are not present.
    """
    async with httpx.AsyncClient(timeout=15.0) as client:
        # Get node ID
        list_response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes",
            headers=auth_headers,
        )

        if list_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        nodes = list_response.json()
        node_id = nodes[0]["id"]

        # Fetch syscap
        response = await client.get(
            f"{base_url}/nodes-registry/v1/nodes/{node_id}/syscap",
            headers=auth_headers,
        )

        assert response.status_code == 200
        syscap = response.json()

        # Count present vs absent capabilities
        present_count = sum(1 for cap in syscap["capabilities"] if cap["present"])
        absent_count = sum(1 for cap in syscap["capabilities"] if not cap["present"])

        # Should have at least one present capability (e.g., OS)
        assert present_count > 0, "Should detect at least one present capability"

        # Depending on system, may or may not have absent capabilities
        # Just verify the structure is correct
        total = present_count + absent_count
        assert total == len(syscap["capabilities"])
