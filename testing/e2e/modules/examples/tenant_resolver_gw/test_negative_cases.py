"""Negative-path E2E tests for tenant_resolver gateway."""

import httpx
import pytest


@pytest.mark.asyncio
async def test_get_parents_unknown_tenant_returns_404(base_url, auth_headers):
    async with httpx.AsyncClient(timeout=10.0) as client:
        resp = await client.get(
            f"{base_url}/tenant-resolver/v1/tenants/ffffffffffffffffffffffffffffffff/parents",
            headers=auth_headers,
        )

        if resp.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {resp.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )

        assert resp.status_code == 404


@pytest.mark.asyncio
async def test_get_children_unknown_tenant_returns_404(base_url, auth_headers):
    async with httpx.AsyncClient(timeout=10.0) as client:
        resp = await client.get(
            f"{base_url}/tenant-resolver/v1/tenants/ffffffffffffffffffffffffffffffff/children",
            headers=auth_headers,
        )

        if resp.status_code in (401, 403) and not auth_headers:
            pytest.skip(
                f"Endpoint requires authentication (got {resp.status_code}). "
                "Set E2E_AUTH_TOKEN environment variable to run this test."
            )

        assert resp.status_code == 404


