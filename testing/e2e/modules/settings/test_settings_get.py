"""E2E tests for settings GET endpoint."""
import httpx
import pytest


@pytest.mark.asyncio
async def test_get_settings_returns_defaults(base_url, auth_headers):
    """
    Test GET /settings/v1/settings endpoint returns defaults when settings don't exist.

    This test verifies that the endpoint returns empty strings for theme and language
    when no settings have been created yet (lazy creation behavior).
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/settings/v1/settings",
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
        assert "userId" in settings
        assert "tenantId" in settings
        assert "theme" in settings
        assert "language" in settings

        # Values should be strings (may be empty on first GET)
        assert isinstance(settings["theme"], str)
        assert isinstance(settings["language"], str)


@pytest.mark.asyncio
async def test_get_settings_multiple_times(base_url, auth_headers):
    """
    Test GET /settings/v1/settings can be called multiple times consistently.

    This test verifies idempotency of the GET endpoint.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # First GET
        response1 = await client.get(
            f"{base_url}/settings/v1/settings",
            headers=auth_headers,
        )

        if response1.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert response1.status_code == 200
        settings1 = response1.json()

        # Second GET
        response2 = await client.get(
            f"{base_url}/settings/v1/settings",
            headers=auth_headers,
        )

        assert response2.status_code == 200
        settings2 = response2.json()

        # Should return the same data
        assert settings1["userId"] == settings2["userId"]
        assert settings1["tenantId"] == settings2["tenantId"]
        assert settings1["theme"] == settings2["theme"]
        assert settings1["language"] == settings2["language"]


@pytest.mark.asyncio
async def test_get_settings_without_auth(base_url):
    """
    Test GET /settings/v1/settings without authentication.

    This test verifies proper error handling when no auth is provided.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/settings/v1/settings",
        )

        # Should return 401 Unauthorized or work with default context
        assert response.status_code in (200, 401, 403), (
            f"Expected 200, 401, or 403, got {response.status_code}"
        )
