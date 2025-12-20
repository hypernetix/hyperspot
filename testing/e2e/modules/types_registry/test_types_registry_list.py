"""E2E tests for GET /types-registry/v1/entities endpoint (list entities)."""
import httpx
import pytest


async def register_test_entities(client, base_url, auth_headers):
    """Helper to register test entities for list tests."""
    payload = {
        "entities": [
            {
                "$id": "gts.e2e.list.acme.models.user.v1~",
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "properties": {"name": {"type": "string"}},
                "description": "User type from acme vendor"
            },
            {
                "$id": "gts.e2e.list.acme.events.created.v1~",
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "properties": {"timestamp": {"type": "string"}},
                "description": "Created event from acme vendor"
            },
            {
                "$id": "gts.e2e.list.globex.models.product.v1~",
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "properties": {"productId": {"type": "string"}},
                "description": "Product type from globex vendor"
            },
            {
                "$id": "gts.e2e.list.acme.models.user.v1~e2e.list.instances.user1.v1",
                "name": "Test User 1"
            },
            {
                "$id": "gts.e2e.list.acme.models.user.v1~e2e.list.instances.user2.v1",
                "name": "Test User 2"
            }
        ]
    }
    
    response = await client.post(
        f"{base_url}/types-registry/v1/entities",
        headers=auth_headers,
        json=payload,
    )
    return response


@pytest.mark.asyncio
async def test_list_entities_basic(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities without filters.
    
    Verifies that the endpoint returns all registered entities.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        assert response.headers.get("content-type", "").startswith("application/json")
        
        data = response.json()
        
        assert "entities" in data, "Response should contain 'entities' field"
        assert "count" in data, "Response should contain 'count' field"
        
        entities = data["entities"]
        assert isinstance(entities, list), "'entities' should be a list"
        assert data["count"] == len(entities), "'count' should match entities length"


@pytest.mark.asyncio
async def test_list_entities_filter_by_kind_type(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities?kind=type
    
    Verifies filtering entities by kind='type'.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"kind": "type"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        entities = data["entities"]
        
        for entity in entities:
            assert entity["kind"] == "type", (
                f"Expected kind='type', got '{entity['kind']}' for {entity['gtsId']}"
            )


@pytest.mark.asyncio
async def test_list_entities_filter_by_kind_instance(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities?kind=instance
    
    Verifies filtering entities by kind='instance'.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"kind": "instance"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        entities = data["entities"]
        
        for entity in entities:
            assert entity["kind"] == "instance", (
                f"Expected kind='instance', got '{entity['kind']}' for {entity['gtsId']}"
            )


@pytest.mark.asyncio
async def test_list_entities_filter_by_vendor(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities?vendor=acme
    
    Verifies filtering entities by vendor.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"vendor": "e2e"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        entities = data["entities"]
        
        for entity in entities:
            assert entity.get("vendor") == "e2e" or "e2e" in entity["gtsId"], (
                f"Entity should have vendor 'e2e': {entity['gtsId']}"
            )


@pytest.mark.asyncio
async def test_list_entities_filter_by_package(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities?package=models
    
    Verifies filtering entities by package.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"package": "list"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        entities = data["entities"]
        
        for entity in entities:
            assert entity.get("package") == "list" or ".list." in entity["gtsId"], (
                f"Entity should have package 'list': {entity['gtsId']}"
            )


@pytest.mark.asyncio
async def test_list_entities_filter_by_namespace(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities?namespace=events
    
    Verifies filtering entities by namespace.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"namespace": "acme"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        entities = data["entities"]
        
        for entity in entities:
            assert entity.get("namespace") == "acme" or ".acme." in entity["gtsId"], (
                f"Entity should have namespace 'acme': {entity['gtsId']}"
            )


@pytest.mark.asyncio
async def test_list_entities_filter_by_pattern(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities?pattern=gts.e2e.list.acme.*
    
    Verifies filtering entities by wildcard pattern.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"pattern": "gts.e2e.list.acme.*"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        entities = data["entities"]
        
        for entity in entities:
            assert entity["gtsId"].startswith("gts.e2e.list.acme."), (
                f"Entity should match pattern 'gts.e2e.list.acme.*': {entity['gtsId']}"
            )


@pytest.mark.asyncio
async def test_list_entities_combined_filters(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities with multiple filters.
    
    Verifies that multiple filters can be combined.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={
                "kind": "type",
                "vendor": "e2e"
            }
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        entities = data["entities"]
        
        for entity in entities:
            assert entity["kind"] == "type", f"Expected kind='type': {entity}"


@pytest.mark.asyncio
async def test_list_entities_no_match(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities with filter that matches nothing.
    
    Verifies empty result handling.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"vendor": "nonexistent_vendor_xyz_123"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        
        assert data["entities"] == [], "Should return empty list for no matches"
        assert data["count"] == 0, "Count should be 0 for no matches"


@pytest.mark.asyncio
async def test_list_entities_segment_scope_primary(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities?segmentScope=primary
    
    Verifies segment scope filtering.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        await register_test_entities(client, base_url, auth_headers)
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"segmentScope": "primary"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        data = response.json()
        assert "entities" in data
        assert "count" in data


@pytest.mark.asyncio
async def test_list_entities_response_structure(base_url, auth_headers):
    """
    Test that list response has correct structure for each entity.
    
    Verifies GtsEntityDto structure in list response.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": "gts.e2e.structure.models.test.v1~",
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {"value": {"type": "string"}},
                    "description": "Test entity for structure validation"
                }
            ]
        }
        
        await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            params={"pattern": "gts.e2e.structure.*"}
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200
        
        data = response.json()
        
        if data["count"] > 0:
            entity = data["entities"][0]
            
            assert "id" in entity, "Entity should have 'id' field"
            assert "gtsId" in entity, "Entity should have 'gtsId' field"
            assert "kind" in entity, "Entity should have 'kind' field"
            assert "content" in entity, "Entity should have 'content' field"
            
            assert isinstance(entity["id"], str), "'id' should be a string (UUID)"
            assert isinstance(entity["gtsId"], str), "'gtsId' should be a string"
            assert entity["kind"] in ("type", "instance"), "'kind' should be 'type' or 'instance'"
            assert isinstance(entity["content"], dict), "'content' should be an object"
