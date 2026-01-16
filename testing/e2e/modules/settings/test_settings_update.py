"""E2E tests for settings POST (full update) endpoint."""
import httpx
import pytest


def _resolve_openapi_ref(doc: dict, ref: str):
    if not ref.startswith("#/"):
        return None
    cur = doc
    for part in ref[2:].split("/"):
        if not isinstance(cur, dict):
            return None
        cur = cur.get(part)
    return cur


def _extract_settings_theme_max_length(openapi_doc: dict):
    post_op = (
        openapi_doc.get("paths", {})
        .get("/simple-user-settings/v1/settings", {})
        .get("post")
    )
    if not isinstance(post_op, dict):
        return None

    request_body = post_op.get("requestBody", {})
    content = request_body.get("content", {})
    app_json = content.get("application/json", {})
    schema = app_json.get("schema", {})
    if "$ref" in schema:
        schema = _resolve_openapi_ref(openapi_doc, schema["$ref"]) or {}

    theme_prop = (schema.get("properties", {}) or {}).get("theme")
    if not isinstance(theme_prop, dict):
        return None
    return theme_prop.get("maxLength")


async def _get_settings_max_field_length(client: httpx.AsyncClient, base_url: str):
    response = await client.get(f"{base_url}/openapi.json")
    response.raise_for_status()
    max_length = _extract_settings_theme_max_length(response.json())
    if isinstance(max_length, int) and max_length > 0:
        return max_length
    raise ValueError("Missing or invalid maxLength for theme")


@pytest.mark.asyncio
async def test_update_settings_full(base_url, auth_headers):
    """
    Test POST /simple-user-settings/v1/settings endpoint for full update.

    This test verifies that we can do a complete update of user settings.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Update settings with specific values
        update_data = {
            "theme": "dark",
            "language": "en"
        }

        response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=update_data,
            headers=auth_headers,
        )

        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. "
            f"Response: {response.text}"
        )

        settings = response.json()
        assert isinstance(settings, dict), "Response should be a JSON object"

        # Validate updated values
        assert settings["theme"] == "dark"
        assert settings["language"] == "en"
        assert "user_id" in settings
        assert "tenant_id" in settings


@pytest.mark.asyncio
async def test_update_settings_creates_on_first_call(base_url, auth_headers):
    """
    Test POST /simple-user-settings/v1/settings creates settings if they don't exist.

    This test verifies upsert behavior (insert on first call).
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # First, try to get settings (might be empty or have old values)
        get_response = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
            headers=auth_headers,
        )

        if get_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert get_response.status_code == 200

        # Now update with new values
        update_data = {
            "theme": "light",
            "language": "es"
        }

        post_response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=update_data,
            headers=auth_headers,
        )

        assert post_response.status_code == 200
        settings = post_response.json()

        assert settings["theme"] == "light"
        assert settings["language"] == "es"

        # Verify by GET
        verify_response = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
            headers=auth_headers,
        )

        assert verify_response.status_code == 200
        verified_settings = verify_response.json()

        assert verified_settings["theme"] == "light"
        assert verified_settings["language"] == "es"


@pytest.mark.asyncio
async def test_update_settings_replaces_existing(base_url, auth_headers):
    """
    Test POST /simple-user-settings/v1/settings replaces existing settings completely.

    This test verifies upsert behavior (update on subsequent calls).
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # First update
        first_data = {
            "theme": "dark",
            "language": "en"
        }

        response1 = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=first_data,
            headers=auth_headers,
        )

        if response1.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert response1.status_code == 200

        # Second update with different values
        second_data = {
            "theme": "light",
            "language": "fr"
        }

        response2 = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=second_data,
            headers=auth_headers,
        )

        assert response2.status_code == 200
        settings = response2.json()

        # Should have new values
        assert settings["theme"] == "light"
        assert settings["language"] == "fr"


@pytest.mark.asyncio
async def test_update_settings_with_empty_strings(base_url, auth_headers):
    """
    Test POST /simple-user-settings/v1/settings accepts empty strings.

    This test verifies that empty strings are valid values.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        update_data = {
            "theme": "",
            "language": ""
        }

        response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=update_data,
            headers=auth_headers,
        )

        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert response.status_code == 200
        settings = response.json()

        assert settings["theme"] == ""
        assert settings["language"] == ""


@pytest.mark.asyncio
async def test_update_settings_validation_max_length(base_url, auth_headers):
    """
    Test POST /simple-user-settings/v1/settings validates field length.

    This test verifies that fields exceeding max length are rejected.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        max_field_length = 10000
        try:
            response = await client.get(
                f"{base_url}/openapi.json",
                headers=auth_headers,
            )
            response.raise_for_status()
            max_field_length = _extract_settings_theme_max_length(response.json()) or max_field_length
        except Exception:
            pass

        # Try to set a very long theme value (dynamically computed; fallback if unknown)
        update_data = {
            "theme": "a" * (max_field_length + 1),
            "language": "en"
        }

        response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=update_data,
            headers=auth_headers,
        )

        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        # Should return 422 Unprocessable Entity for validation error
        assert response.status_code == 422, (
            f"Expected 422 for validation error, got {response.status_code}"
        )


@pytest.mark.asyncio
async def test_update_settings_missing_fields(base_url, auth_headers):
    """
    Test POST /simple-user-settings/v1/settings with missing required fields.

    This test verifies proper error handling for incomplete data.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Missing language field
        update_data = {
            "theme": "dark"
        }

        response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=update_data,
            headers=auth_headers,
        )

        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        # Should return 400 or 422 for missing required field
        assert response.status_code in (400, 422), (
            f"Expected 400 or 422 for missing field, got {response.status_code}"
        )
