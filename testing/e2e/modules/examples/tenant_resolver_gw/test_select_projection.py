"""E2E tests for $select projection on list endpoint."""

import httpx
import pytest

from .helpers import fetch_all_tenant_ids


@pytest.mark.asyncio
async def test_list_tenants_select_id_only(base_url, auth_headers):
    async with httpx.AsyncClient(timeout=10.0) as client:
        url = f"{base_url}/tenant-resolver/v1/tenants"
        resp = await client.get(
            url,
            headers=auth_headers,
            params={"limit": "2", "$select": "id"},
        )

        if resp.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {resp.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )

        assert resp.status_code == 200, f"Expected 200, got {resp.status_code}: {resp.text}"
        data = resp.json()
        assert "items" in data and "page_info" in data

        for item in data["items"]:
            assert set(item.keys()) == {"id"}, f"Expected only id in item, got keys={item.keys()}"

        # Also ensure we can paginate under projection
        ids = await fetch_all_tenant_ids(client, base_url, auth_headers, limit=2, select="id")
        assert all(isinstance(x, str) and x for x in ids)


