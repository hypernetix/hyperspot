"""E2E tests for GET /tenant-resolver/v1/root endpoint."""

import httpx
import pytest

from .helpers import TENANT_ROOT


@pytest.mark.asyncio
async def test_get_root_tenant(base_url, auth_headers):
    async with httpx.AsyncClient(timeout=10.0) as client:
        resp = await client.get(
            f"{base_url}/tenant-resolver/v1/root",
            headers=auth_headers,
        )

        if resp.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {resp.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )

        assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
        assert resp.headers.get("content-type", "").startswith("application/json")

        data = resp.json()
        assert isinstance(data, dict)
        assert data.get("id") == TENANT_ROOT
        assert data.get("parentId") in ("", None)


