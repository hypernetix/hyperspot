"""E2E tests for settings PATCH (partial update) endpoint."""
import httpx
import pytest


@pytest.mark.asyncio
async def test_patch_settings_theme_only(base_url, auth_headers):
    """
    Test PATCH /simple-user-settings/v1/settings endpoint updating only theme.

    This test verifies partial update behavior - only provided fields are updated.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # First, set both fields
        initial_data = {
            "theme": "dark",
            "language": "en"
        }

        post_response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=initial_data,
            headers=auth_headers,
        )

        if post_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert post_response.status_code == 200

        # Now patch only theme
        patch_data = {
            "theme": "light"
        }

        patch_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json=patch_data,
            headers=auth_headers,
        )

        assert patch_response.status_code == 200, (
            f"Expected 200, got {patch_response.status_code}. "
            f"Response: {patch_response.text}"
        )

        settings = patch_response.json()

        # Theme should be updated, language should remain unchanged
        assert settings["theme"] == "light"
        assert settings["language"] == "en"


@pytest.mark.asyncio
async def test_patch_settings_language_only(base_url, auth_headers):
    """
    Test PATCH /simple-user-settings/v1/settings endpoint updating only language.

    This test verifies partial update behavior for the language field.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Set initial values
        initial_data = {
            "theme": "dark",
            "language": "en"
        }

        post_response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=initial_data,
            headers=auth_headers,
        )

        if post_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert post_response.status_code == 200

        # Patch only language
        patch_data = {
            "language": "fr"
        }

        patch_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json=patch_data,
            headers=auth_headers,
        )

        assert patch_response.status_code == 200
        settings = patch_response.json()

        # Language should be updated, theme should remain unchanged
        assert settings["theme"] == "dark"
        assert settings["language"] == "fr"


@pytest.mark.asyncio
async def test_patch_settings_both_fields(base_url, auth_headers):
    """
    Test PATCH /simple-user-settings/v1/settings endpoint updating both fields.

    This test verifies that PATCH can update multiple fields at once.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Set initial values
        initial_data = {
            "theme": "dark",
            "language": "en"
        }

        post_response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=initial_data,
            headers=auth_headers,
        )

        if post_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert post_response.status_code == 200

        # Patch both fields
        patch_data = {
            "theme": "light",
            "language": "es"
        }

        patch_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json=patch_data,
            headers=auth_headers,
        )

        assert patch_response.status_code == 200
        settings = patch_response.json()

        # Both should be updated
        assert settings["theme"] == "light"
        assert settings["language"] == "es"


@pytest.mark.asyncio
async def test_patch_settings_empty_patch(base_url, auth_headers):
    """
    Test PATCH /simple-user-settings/v1/settings with empty patch (no fields).

    This test verifies behavior when no fields are provided in the patch.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Set initial values
        initial_data = {
            "theme": "dark",
            "language": "en"
        }

        post_response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=initial_data,
            headers=auth_headers,
        )

        if post_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert post_response.status_code == 200

        # Patch with empty object
        patch_data = {}

        patch_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json=patch_data,
            headers=auth_headers,
        )

        assert patch_response.status_code == 200
        settings = patch_response.json()

        # Nothing should change
        assert settings["theme"] == "dark"
        assert settings["language"] == "en"


@pytest.mark.asyncio
async def test_patch_settings_creates_if_not_exists(base_url, auth_headers):
    """
    Test PATCH /simple-user-settings/v1/settings creates settings if they don't exist.

    This test verifies that PATCH also does upsert (creates on first call).
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Patch without prior POST (may create with defaults)
        patch_data = {
            "theme": "light"
        }

        patch_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json=patch_data,
            headers=auth_headers,
        )

        if patch_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert patch_response.status_code == 200
        settings = patch_response.json()

        # Theme should be set, language should be default (empty or set value)
        assert settings["theme"] == "light"
        assert isinstance(settings["language"], str)


@pytest.mark.asyncio
async def test_patch_settings_validation_max_length(base_url, auth_headers):
    """
    Test PATCH /simple-user-settings/v1/settings validates field length.

    This test verifies that PATCH also enforces validation rules.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Try to patch with a very long value
        patch_data = {
            "language": "x" * 200  # Way too long
        }

        patch_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json=patch_data,
            headers=auth_headers,
        )

        if patch_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        # Should return 400 Bad Request for validation error
        assert patch_response.status_code == 422, (
            f"Expected 422 for validation error, got {patch_response.status_code}"
        )


@pytest.mark.asyncio
async def test_patch_settings_sequential_updates(base_url, auth_headers):
    """
    Test PATCH /simple-user-settings/v1/settings with multiple sequential partial updates.

    This test verifies that multiple PATCH calls work correctly.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Initial full update
        initial_data = {
            "theme": "dark",
            "language": "en"
        }

        post_response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=initial_data,
            headers=auth_headers,
        )

        if post_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert post_response.status_code == 200

        # First patch: update theme
        patch1_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json={"theme": "light"},
            headers=auth_headers,
        )

        assert patch1_response.status_code == 200
        settings1 = patch1_response.json()
        assert settings1["theme"] == "light"
        assert settings1["language"] == "en"

        # Second patch: update language
        patch2_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json={"language": "fr"},
            headers=auth_headers,
        )

        assert patch2_response.status_code == 200
        settings2 = patch2_response.json()
        assert settings2["theme"] == "light"  # Should still be light
        assert settings2["language"] == "fr"  # Should be updated

        # Third patch: update both
        patch3_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json={"theme": "dark", "language": "es"},
            headers=auth_headers,
        )

        assert patch3_response.status_code == 200
        settings3 = patch3_response.json()
        assert settings3["theme"] == "dark"
        assert settings3["language"] == "es"
