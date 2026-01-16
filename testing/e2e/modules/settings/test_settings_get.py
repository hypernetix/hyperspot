"""E2E tests for settings GET endpoint."""
import httpx
import pytest


@pytest.mark.asyncio
async def test_get_settings_returns_defaults(base_url, auth_headers):
    """
    Test GET /simple-user-settings/v1/settings endpoint returns defaults when settings don't exist.

    This test verifies that the endpoint returns empty strings for theme and language
    when no settings have been created yet (lazy creation behavior).
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
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

        # Validate structure
        assert "user_id" in settings
        assert "tenant_id" in settings
        assert "theme" in settings
        assert "language" in settings

        # Values should be strings (may be empty on first GET, or null if no record)
        assert settings["theme"] is None or isinstance(settings["theme"], str)
        assert settings["language"] is None or isinstance(settings["language"], str)

        # Default values are empty strings when no record exists (or after reset)
        # Note: If a record exists with empty strings, those are returned
        assert settings["theme"] == "" or settings["theme"] is None
        assert settings["language"] == "" or settings["language"] is None


@pytest.mark.asyncio
async def test_get_settings_multiple_times(base_url, auth_headers):
    """
    Test GET /simple-user-settings/v1/settings can be called multiple times consistently.

    This test verifies idempotency of the GET endpoint.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # First GET
        response1 = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
            headers=auth_headers,
        )

        if response1.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert response1.status_code == 200
        settings1 = response1.json()

        # Second GET
        response2 = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
            headers=auth_headers,
        )

        assert response2.status_code == 200
        settings2 = response2.json()

        # Should return the same data
        assert settings1["user_id"] == settings2["user_id"]
        assert settings1["tenant_id"] == settings2["tenant_id"]
        assert settings1["theme"] == settings2["theme"]
        assert settings1["language"] == settings2["language"]


@pytest.mark.asyncio
async def test_get_settings_without_auth(base_url):
    """
    Test GET /simple-user-settings/v1/settings without authentication.

    This test verifies proper error handling when no auth is provided.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
        )

        # Should return 401 Unauthorized or work with default context
        assert response.status_code in (200, 401, 403), (
            f"Expected 200, 401, or 403, got {response.status_code}"
        )
