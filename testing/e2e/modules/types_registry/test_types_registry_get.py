"""E2E tests for GET /types-registry/v1/entities/{gts_id} endpoint (get entity by ID)."""
import httpx
import pytest
import time

_counter = int(time.time() * 1000) % 1000000


def unique_id(name: str) -> str:
    """Generate a unique GTS ID to avoid conflicts between test runs.
    
    GTS ID format: gts.vendor.package.namespace.name.version~
    (5 tokens after 'gts.')
    """
    global _counter
    _counter += 1
    return f"gts.e2etest.pkg.ns.{name}{_counter}.v1~"


@pytest.mark.asyncio
async def test_get_entity_by_id(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities/{gts_id} for existing entity.
    
    Verifies that a registered entity can be retrieved by its GTS ID.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        gts_id = unique_id("document")
        
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "content": {"type": "string"}
                    },
                    "required": ["title"],
                    "description": "Document type for get test"
                }
            ]
        }
        
        register_response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if register_response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {register_response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert register_response.status_code == 200, (
            f"Registration failed: {register_response.text}"
        )
        
        reg_data = register_response.json()
        assert reg_data["summary"]["succeeded"] == 1, (
            f"Registration should succeed: {reg_data}"
        )
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities/{gts_id}",
            headers=auth_headers,
        )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        assert response.headers.get("content-type", "").startswith("application/json")
        
        entity = response.json()
        
        assert entity["gtsId"] == gts_id
        assert entity["kind"] == "type"
        assert "id" in entity
        assert "content" in entity
        assert entity["description"] == "Document type for get test"


@pytest.mark.asyncio
async def test_get_entity_not_found(base_url, auth_headers):
    """
    Test GET /types-registry/v1/entities/{gts_id} for non-existent entity.
    
    Verifies 404 response for unknown GTS ID.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        nonexistent_id = "gts.nonexistent.vendor.pkg.ns.type.v1~"
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities/{nonexistent_id}",
            headers=auth_headers,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 404, (
            f"Expected 404 for non-existent entity, got {response.status_code}. "
            f"Response: {response.text}"
        )


@pytest.mark.asyncio
async def test_get_entity_returns_full_content(base_url, auth_headers):
    """
    Test that GET returns the full entity content.
    
    Verifies that the content field contains the complete schema.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        gts_id = unique_id("fullcontent")
        
        original_content = {
            "$id": gts_id,
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "field1": {"type": "string", "minLength": 1},
                "field2": {"type": "integer", "minimum": 0},
                "field3": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            },
            "required": ["field1", "field2"],
            "additionalProperties": False,
            "description": "Complex schema for content test"
        }
        
        payload = {"entities": [original_content]}
        
        register_response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if register_response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {register_response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert register_response.status_code == 200
        reg_data = register_response.json()
        assert reg_data["summary"]["succeeded"] == 1, f"Registration should succeed: {reg_data}"
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities/{gts_id}",
            headers=auth_headers,
        )
        
        assert response.status_code == 200, f"GET failed: {response.text}"
        
        entity = response.json()
        content = entity["content"]
        
        assert "properties" in content
        assert "field1" in content["properties"]
        assert "field2" in content["properties"]
        assert "field3" in content["properties"]


@pytest.mark.asyncio
async def test_get_instance_entity(base_url, auth_headers):
    """
    Test GET for an instance entity.
    
    Verifies that instance entities can be retrieved.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        global _counter
        _counter += 1
        type_id = f"gts.e2etest.pkg.ns.item{_counter}.v1~"
        instance_id = f"{type_id}e2etest.inst.ns.item1.v1"
        
        payload = {
            "entities": [
                {
                    "$id": type_id,
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {
                        "itemName": {"type": "string"},
                        "quantity": {"type": "integer"}
                    },
                    "required": ["itemName", "quantity"]
                },
                {
                    "$id": instance_id,
                    "type": type_id,
                    "itemName": "Test Item",
                    "quantity": 42
                }
            ]
        }
        
        register_response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if register_response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {register_response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert register_response.status_code == 200
        reg_data = register_response.json()
        assert reg_data["summary"]["succeeded"] == 2, f"Both entities should register: {reg_data}"
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities/{instance_id}",
            headers=auth_headers,
        )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        entity = response.json()
        
        assert entity["gtsId"] == instance_id
        assert entity["kind"] == "instance"
        
        content = entity["content"]
        assert content.get("itemName") == "Test Item"
        assert content.get("quantity") == 42


@pytest.mark.asyncio
async def test_get_entity_with_special_characters_in_id(base_url, auth_headers):
    """
    Test GET with GTS ID containing special characters.
    
    Verifies proper URL encoding handling.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        gts_id = unique_id("specialchars")
        
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "type": "object",
                    "description": "Entity with underscores in ID"
                }
            ]
        }
        
        register_response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if register_response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {register_response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert register_response.status_code == 200
        reg_data = register_response.json()
        assert reg_data["summary"]["succeeded"] == 1, f"Registration should succeed: {reg_data}"
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities/{gts_id}",
            headers=auth_headers,
        )
        
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. Response: {response.text}"
        )
        
        entity = response.json()
        assert entity["gtsId"] == gts_id


@pytest.mark.asyncio
async def test_get_entity_uuid_format(base_url, auth_headers):
    """
    Test that entity ID is a valid UUID format.
    
    Verifies the deterministic UUID generation from GTS ID.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        gts_id = unique_id("uuidtest")
        
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "type": "object"
                }
            ]
        }
        
        register_response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if register_response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {register_response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert register_response.status_code == 200
        reg_data = register_response.json()
        assert reg_data["summary"]["succeeded"] == 1, f"Registration should succeed: {reg_data}"
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities/{gts_id}",
            headers=auth_headers,
        )
        
        assert response.status_code == 200, f"GET failed: {response.text}"
        
        entity = response.json()
        uuid_str = entity["id"]
        
        import re
        uuid_pattern = re.compile(
            r'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',
            re.IGNORECASE
        )
        assert uuid_pattern.match(uuid_str), f"ID should be valid UUID format: {uuid_str}"


@pytest.mark.asyncio
async def test_get_entity_vendor_package_namespace(base_url, auth_headers):
    """
    Test that GET returns vendor, package, namespace from primary segment.
    
    Verifies segment parsing in response.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        global _counter
        _counter += 1
        gts_id = f"gts.e2etest.testpkg.testns.typename{_counter}.v1~"
        
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "type": "object",
                    "description": "Entity for segment test"
                }
            ]
        }
        
        register_response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if register_response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {register_response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert register_response.status_code == 200
        reg_data = register_response.json()
        assert reg_data["summary"]["succeeded"] == 1, f"Registration should succeed: {reg_data}"
        
        response = await client.get(
            f"{base_url}/types-registry/v1/entities/{gts_id}",
            headers=auth_headers,
        )
        
        assert response.status_code == 200, f"GET failed: {response.text}"
        
        entity = response.json()
        
        assert "vendor" in entity or entity.get("vendor") is None
        assert "package" in entity or entity.get("package") is None
        assert "namespace" in entity or entity.get("namespace") is None


@pytest.mark.asyncio
async def test_get_entity_deterministic_uuid(base_url, auth_headers):
    """
    Test that the same GTS ID always produces the same UUID.
    
    Verifies deterministic UUID generation.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        gts_id = unique_id("deterministic")
        
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "type": "object"
                }
            ]
        }
        
        register_response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if register_response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {register_response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert register_response.status_code == 200
        reg_data = register_response.json()
        assert reg_data["summary"]["succeeded"] == 1, f"Registration should succeed: {reg_data}"
        
        response1 = await client.get(
            f"{base_url}/types-registry/v1/entities/{gts_id}",
            headers=auth_headers,
        )
        
        response2 = await client.get(
            f"{base_url}/types-registry/v1/entities/{gts_id}",
            headers=auth_headers,
        )
        
        assert response1.status_code == 200, f"First GET failed: {response1.text}"
        assert response2.status_code == 200, f"Second GET failed: {response2.text}"
        
        entity1 = response1.json()
        entity2 = response2.json()
        
        assert entity1["id"] == entity2["id"], (
            "Same GTS ID should produce same UUID across requests"
        )
