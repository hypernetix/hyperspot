"""E2E tests for file-parser API module."""
import httpx
import pytest


@pytest.mark.asyncio
async def test_file_parser_info_basic(base_url, auth_headers):
    """
    Test GET /file-parser/v1/info endpoint.

    This test verifies that the file parser info endpoint returns
    information about available file parsers.
    """
    async with httpx.AsyncClient(timeout=10.0) as client:
        response = await client.get(
            f"{base_url}/file-parser/v1/info",
            headers=auth_headers,
        )

        # If no auth token is set and we get 401/403, skip the test
        if response.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {response.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )

        # Assert successful response
        assert response.status_code == 200, (
            f"Expected 200, got {response.status_code}. "
            f"Response: {response.text}"
        )

        # Assert response is JSON
        assert response.headers.get("content-type", "").startswith(
            "application/json"
        ), "Response should be JSON"

        # Parse JSON response
        data = response.json()
        assert isinstance(data, dict), "Response should be a JSON object"

        # Assert expected structure from FileParserInfoDto
        # FileParserInfoDto has: supported_extensions: HashMap<String, Vec<String>>
        assert "supported_extensions" in data, (
            "Response should contain 'supported_extensions' field"
        )

        supported_extensions = data["supported_extensions"]
        assert isinstance(supported_extensions, dict), (
            "'supported_extensions' should be a dictionary/object"
        )

        # Assert that the structure is non-empty and stable
        # Each key should map to a list of strings (file extensions)
        for parser_name, extensions in supported_extensions.items():
            assert isinstance(parser_name, str), (
                f"Parser name '{parser_name}' should be a string"
            )
            assert isinstance(extensions, list), (
                f"Extensions for parser '{parser_name}' should be a list"
            )
            # Verify extensions are strings
            for ext in extensions:
                assert isinstance(ext, str), (
                    f"Extension '{ext}' for parser '{parser_name}' should be a string"
                )
