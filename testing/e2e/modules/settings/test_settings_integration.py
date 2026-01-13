"""E2E integration tests for settings module - full workflow scenarios."""
import httpx
import pytest


@pytest.mark.asyncio
async def test_settings_full_workflow(base_url, auth_headers):
    """
    Test complete workflow: GET (defaults) -> POST (create) -> GET (verify) -> PATCH (update) -> GET (verify).

    This test verifies the entire lifecycle of user settings.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Step 1: GET settings (should return defaults or empty)
        get1_response = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
            headers=auth_headers,
        )

        if get1_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert get1_response.status_code == 200
        initial_settings = get1_response.json()
        assert "theme" in initial_settings
        assert "language" in initial_settings

        # Step 2: POST to create/update settings
        post_data = {
            "theme": "dark",
            "language": "en"
        }

        post_response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=post_data,
            headers=auth_headers,
        )

        assert post_response.status_code == 200
        created_settings = post_response.json()
        assert created_settings["theme"] == "dark"
        assert created_settings["language"] == "en"

        # Step 3: GET to verify POST worked
        get2_response = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
            headers=auth_headers,
        )

        assert get2_response.status_code == 200
        verified_settings = get2_response.json()
        assert verified_settings["theme"] == "dark"
        assert verified_settings["language"] == "en"

        # Step 4: PATCH to partially update
        patch_data = {
            "theme": "light"
        }

        patch_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json=patch_data,
            headers=auth_headers,
        )

        assert patch_response.status_code == 200
        patched_settings = patch_response.json()
        assert patched_settings["theme"] == "light"
        assert patched_settings["language"] == "en"  # Should remain unchanged

        # Step 5: Final GET to verify PATCH worked
        get3_response = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
            headers=auth_headers,
        )

        assert get3_response.status_code == 200
        final_settings = get3_response.json()
        assert final_settings["theme"] == "light"
        assert final_settings["language"] == "en"


@pytest.mark.asyncio
async def test_settings_idempotency(base_url, auth_headers):
    """
    Test idempotency: multiple identical requests produce consistent results.

    This test verifies that settings operations are idempotent.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        test_data = {
            "theme": "dark",
            "language": "es"
        }

        if auth_headers:
            # POST same data twice
            response1 = await client.post(
                f"{base_url}/simple-user-settings/v1/settings",
                json=test_data,
                headers=auth_headers,
            )

            if response1.status_code in (401, 403):
                pytest.skip("Endpoint requires authentication")

            assert response1.status_code == 200
            settings1 = response1.json()

            response2 = await client.post(
                f"{base_url}/simple-user-settings/v1/settings",
                json=test_data,
                headers=auth_headers,
            )

            assert response2.status_code == 200
            settings2 = response2.json()

            # Should produce same result
            assert settings1["theme"] == settings2["theme"]
            assert settings1["language"] == settings2["language"]


@pytest.mark.asyncio
async def test_settings_consistency_across_methods(base_url, auth_headers):
    """
    Test consistency: POST and PATCH should result in same state when setting all fields.

    This test verifies that different update methods produce consistent results.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        # Set via POST
        post_data = {
            "theme": "light",
            "language": "fr"
        }

        post_response = await client.post(
            f"{base_url}/simple-user-settings/v1/settings",
            json=post_data,
            headers=auth_headers,
        )

        if post_response.status_code in (401, 403) and not auth_headers:
            pytest.skip("Endpoint requires authentication")

        assert post_response.status_code == 200

        # Now use PATCH with both fields to set to same values
        patch_data = {
            "theme": "light",
            "language": "fr"
        }

        patch_response = await client.patch(
            f"{base_url}/simple-user-settings/v1/settings",
            json=patch_data,
            headers=auth_headers,
        )

        assert patch_response.status_code == 200
        patched_settings = patch_response.json()

        # GET to verify final state
        get_response = await client.get(
            f"{base_url}/simple-user-settings/v1/settings",
            headers=auth_headers,
        )

        assert get_response.status_code == 200
        final_settings = get_response.json()

        # Should match what we set
        assert final_settings["theme"] == "light"
        assert final_settings["language"] == "fr"
