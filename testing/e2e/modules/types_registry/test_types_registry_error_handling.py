"""E2E tests for types-registry error handling and edge cases."""
import httpx
import pytest
import time

_counter = int(time.time() * 1000) % 1000000


def unique_type_id(name: str) -> str:
    """Generate a unique type GTS ID."""
    global _counter
    _counter += 1
    return f"gts.e2etest.err.models.{name}{_counter}.v1~"


@pytest.mark.asyncio
async def test_error_response_format_rfc9457(base_url, auth_headers):
    """
    Test that error responses follow RFC-9457 Problem Details format.
    
    Verifies standardized error response structure.
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
        
        assert response.status_code == 404
        
        if response.headers.get("content-type", "").startswith("application/problem+json"):
            data = response.json()
            assert "type" in data or "title" in data or "status" in data, (
                "RFC-9457 response should have type, title, or status"
            )


@pytest.mark.asyncio
async def test_missing_content_type_header(base_url, auth_headers):
    """
    Test POST without Content-Type header.
    
    Verifies proper handling of missing headers.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            content=b'{"entities": []}',
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code in (200, 400, 415), (
            f"Expected 200, 400, or 415, got {response.status_code}"
        )


@pytest.mark.asyncio
async def test_wrong_content_type_header(base_url, auth_headers):
    """
    Test POST with wrong Content-Type header.
    
    Verifies handling of unsupported media types.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers={**auth_headers, "Content-Type": "text/plain"},
            content=b'{"entities": []}',
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code in (200, 400, 415), (
            f"Expected 200, 400, or 415, got {response.status_code}"
        )


@pytest.mark.asyncio
async def test_empty_request_body(base_url, auth_headers):
    """
    Test POST with empty request body.
    
    Verifies proper error handling for empty body.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers={**auth_headers, "Content-Type": "application/json"},
            content=b'',
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code in (400, 422), (
            f"Expected 400 or 422 for empty body, got {response.status_code}"
        )


@pytest.mark.asyncio
async def test_missing_entities_field(base_url, auth_headers):
    """
    Test POST without 'entities' field in request body.
    
    Verifies validation of required request fields.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json={"other_field": "value"},
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code in (400, 422), (
            f"Expected 400 or 422 for missing 'entities' field, got {response.status_code}"
        )


@pytest.mark.asyncio
async def test_entities_not_array(base_url, auth_headers):
    """
    Test POST with 'entities' as non-array value.
    
    Verifies type validation for entities field.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json={"entities": "not-an-array"},
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code in (400, 422), (
            f"Expected 400 or 422 for non-array entities, got {response.status_code}"
        )


@pytest.mark.asyncio
async def test_large_batch_registration(base_url, auth_headers):
    """
    Test registering a large batch of entities.
    
    Verifies handling of larger payloads.
    """
    global _counter
    _counter += 1
    batch_id = _counter
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        entities = []
        for i in range(50):
            entities.append({
                "$id": f"gts.e2etest.large.models.type{i}x{batch_id}.v1~",
                "type": "object",
                "properties": {
                    "field": {"type": "string"}
                }
            })
        
        payload = {"entities": entities}
        
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
            f"Expected 200, got {response.status_code}. Response: {response.text[:500]}"
        )
        
        data = response.json()
        assert data["summary"]["total"] == 50


@pytest.mark.asyncio
async def test_duplicate_entity_registration(base_url, auth_headers):
    """
    Test registering the same entity twice.
    
    Verifies handling of duplicate registrations.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        gts_id = unique_type_id("duplicate")
        
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "type": "object",
                    "description": "First registration"
                }
            ]
        }
        
        response1 = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        if response1.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response1.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response1.status_code == 200
        
        payload["entities"][0]["description"] = "Second registration"
        
        response2 = await client.post(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
            json=payload,
        )
        
        assert response2.status_code == 200, (
            f"Expected 200, got {response2.status_code}. Response: {response2.text}"
        )


@pytest.mark.asyncio
async def test_very_long_gts_id(base_url, auth_headers):
    """
    Test with very long GTS ID.
    
    Verifies handling of edge case ID lengths.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        long_segment = "a" * 100
        gts_id = f"gts.e2e.{long_segment}.models.test.v1~"
        
        payload = {
            "entities": [
                {
                    "$id": gts_id,
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


@pytest.mark.asyncio
async def test_unicode_in_content(base_url, auth_headers):
    """
    Test entity with unicode characters in content.
    
    Verifies proper handling of international characters.
    """
    gts_id = unique_type_id("unicode")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"}
                    },
                    "description": "Test with unicode: 日本語 中文 한국어 émojis 🎉"
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
        assert data["summary"]["succeeded"] == 1
        
        entity = data["results"][0]["entity"]
        assert "日本語" in entity["description"]
        assert "🎉" in entity["description"]


@pytest.mark.asyncio
async def test_null_values_in_entity(base_url, auth_headers):
    """
    Test entity with null values in content.
    
    Verifies handling of null JSON values.
    """
    gts_id = unique_type_id("nulltest")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "type": "object",
                    "properties": {
                        "optional_field": {"type": ["string", "null"]}
                    },
                    "description": None
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


@pytest.mark.asyncio
async def test_deeply_nested_schema(base_url, auth_headers):
    """
    Test entity with deeply nested schema structure.
    
    Verifies handling of complex nested objects.
    """
    gts_id = unique_type_id("deep")
    
    async with httpx.AsyncClient(timeout=10.0) as client:
        payload = {
            "entities": [
                {
                    "$id": gts_id,
                    "type": "object",
                    "properties": {
                        "level1": {
                            "type": "object",
                            "properties": {
                                "level2": {
                                    "type": "object",
                                    "properties": {
                                        "level3": {
                                            "type": "object",
                                            "properties": {
                                                "level4": {
                                                    "type": "object",
                                                    "properties": {
                                                        "value": {"type": "string"}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
        assert data["summary"]["succeeded"] == 1


@pytest.mark.asyncio
async def test_method_not_allowed(base_url, auth_headers):
    """
    Test unsupported HTTP methods on endpoints.
    
    Verifies proper 405 Method Not Allowed responses.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.delete(
            f"{base_url}/types-registry/v1/entities",
            headers=auth_headers,
        )
        
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )
        
        assert response.status_code == 405, (
            f"Expected 405 Method Not Allowed, got {response.status_code}"
        )
