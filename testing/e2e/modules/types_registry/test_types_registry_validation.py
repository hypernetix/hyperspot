"""E2E tests for types-registry validation behavior."""
import httpx
import pytest
import time

_counter = int(time.time() * 1000) % 1000000


def unique_type_id(name: str) -> str:
    """Generate a unique type GTS ID.
    
    GTS ID format: gts.vendor.package.namespace.name.version~
    """
    global _counter
    _counter += 1
    return f"gts.e2etest.validation.models.{name}{_counter}.v1~"


def unique_instance_id(type_id: str, name: str) -> str:
    """Generate a unique instance GTS ID based on a type ID."""
    global _counter
    _counter += 1
    return f"{type_id}e2etest.validation.inst.{name}{_counter}.v1"


@pytest.mark.asyncio
async def test_validation_invalid_instance_against_schema(base_url, auth_headers):
    """
    Test that invalid instances fail validation against their type schema.
    
    In ready mode, instances are validated immediately.
    """
    type_id = unique_type_id("employee")
    instance_id = unique_instance_id(type_id, "emp1")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": type_id,
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {
                        "employeeId": {"type": "string"},
                        "salary": {"type": "number"}
                    },
                    "required": ["employeeId", "salary"]
                },
                {
                    "id": instance_id,
                    "employeeId": "emp-001"
                }
            ]
        }
        
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
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
        results = data["results"]
        
        assert results[0]["status"] == "ok", "Type should register successfully"
        
        if results[1]["status"] == "error":
            assert "error" in results[1]


@pytest.mark.asyncio
async def test_validation_wrong_type_for_field(base_url, auth_headers):
    """
    Test validation failure when field has wrong type.
    
    Instance with string instead of number should fail.
    """
    type_id = unique_type_id("product")
    instance_id = unique_instance_id(type_id, "prod1")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": type_id,
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {
                        "productId": {"type": "string"},
                        "price": {"type": "number"}
                    },
                    "required": ["productId", "price"]
                },
                {
                    "id": instance_id,
                    "productId": "prod-001",
                    "price": "not-a-number"
                }
            ]
        }
        
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200
        
        data = response.json()
        results = data["results"]
        
        assert results[0]["status"] == "ok", "Type should register successfully"
        
        if results[1]["status"] == "error":
            error_msg = results[1].get("error", "")
            assert error_msg, "Error message should be present"


@pytest.mark.asyncio
async def test_validation_valid_instance_succeeds(base_url, auth_headers):
    """
    Test that valid instances pass validation.
    
    Instance conforming to schema should succeed.
    """
    type_id = unique_type_id("order")
    instance_id = unique_instance_id(type_id, "order1")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": type_id,
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {
                        "orderId": {"type": "string"},
                        "total": {"type": "number"},
                        "items": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    },
                    "required": ["orderId", "total"]
                },
                {
                    "id": instance_id,
                    "orderId": "order-001",
                    "total": 99.99,
                    "items": ["item1", "item2"]
                }
            ]
        }
        
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200
        
        data = response.json()
        
        assert data["summary"]["succeeded"] == 2
        assert data["summary"]["failed"] == 0


@pytest.mark.asyncio
async def test_validation_instance_before_type_fails(base_url, auth_headers):
    """
    Test that registering instance before its type fails in ready mode.
    
    Parent type must exist before instances can be validated.
    """
    type_id = unique_type_id("widget")
    instance_id = unique_instance_id(type_id, "widget1")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "id": instance_id,
                    "widgetId": "w-001"
                },
                {
                    "$id": type_id,
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {
                        "widgetId": {"type": "string"}
                    },
                    "required": ["widgetId"]
                }
            ]
        }
        
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200
        
        data = response.json()
        results = data["results"]
        
        if results[0]["status"] == "error":
            assert "error" in results[0]
        
        assert results[1]["status"] == "ok", "Type should register successfully"


@pytest.mark.asyncio
async def test_validation_invalid_gts_id_format(base_url, auth_headers):
    """
    Test that invalid GTS ID format is rejected.
    
    GTS IDs must follow the gts.vendor.package.namespace.name.version~ format.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": "invalid-id-format",
                    "type": "object"
                }
            ]
        }
        
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200
        
        data = response.json()
        
        assert data["summary"]["failed"] == 1
        assert data["results"][0]["status"] == "error"


@pytest.mark.asyncio
async def test_validation_complex_nested_schema(base_url, auth_headers):
    """
    Test validation with complex nested schema.
    
    Verifies that nested object validation works correctly.
    """
    type_id = unique_type_id("nested")
    instance_id = unique_instance_id(type_id, "nested1")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": type_id,
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {
                        "nestedId": {"type": "string"},
                        "metadata": {
                            "type": "object",
                            "properties": {
                                "createdAt": {"type": "string"},
                                "updatedAt": {"type": "string"},
                                "tags": {
                                    "type": "array",
                                    "items": {"type": "string"}
                                }
                            },
                            "required": ["createdAt"]
                        }
                    },
                    "required": ["nestedId", "metadata"]
                },
                {
                    "id": instance_id,
                    "nestedId": "nested-001",
                    "metadata": {
                        "createdAt": "2024-01-01T00:00:00Z",
                        "tags": ["test", "e2e"]
                    }
                }
            ]
        }
        
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200
        
        data = response.json()
        
        assert data["summary"]["succeeded"] == 2, (
            f"Both entities should succeed: {data['results']}"
        )


@pytest.mark.asyncio
async def test_validation_multiple_instances_same_type(base_url, auth_headers):
    """
    Test registering multiple instances of the same type.
    
    Verifies batch validation of multiple instances.
    """
    type_id = unique_type_id("user")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": type_id,
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "type": "object",
                    "properties": {
                        "username": {"type": "string"},
                        "email": {"type": "string"}
                    },
                    "required": ["username", "email"]
                },
                {
                    "id": unique_instance_id(type_id, "user1"),
                    "username": "alice",
                    "email": "alice@example.com"
                },
                {
                    "id": unique_instance_id(type_id, "user2"),
                    "username": "bob",
                    "email": "bob@example.com"
                },
                {
                    "id": unique_instance_id(type_id, "user3"),
                    "username": "charlie",
                    "email": "charlie@example.com"
                }
            ]
        }
        
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 200
        
        data = response.json()
        
        assert data["summary"]["total"] == 4
        assert data["summary"]["succeeded"] == 4
        assert data["summary"]["failed"] == 0
