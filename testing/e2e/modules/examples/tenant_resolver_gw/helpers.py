"""Helpers and constants for tenant_resolver gateway E2E tests."""

from __future__ import annotations

from typing import Any, Dict, List, Optional, Set

import httpx


# These IDs MUST match config/e2e-local.yaml (fabrikam_tr_plugin.config.tenants)
TENANT_ROOT = "00000000000000000000000000000001"
TENANT_ENG = "00000000000000000000000000000002"
TENANT_SOFT_DELETED = "00000000000000000000000000000003"
TENANT_BACKEND = "00000000000000000000000000000004"
TENANT_SALES = "00000000000000000000000000000005"
TENANT_ENTERPRISE = "00000000000000000000000000000006"


ALL_TENANTS: List[str] = [
    TENANT_ROOT,
    TENANT_ENG,
    TENANT_SOFT_DELETED,
    TENANT_BACKEND,
    TENANT_SALES,
    TENANT_ENTERPRISE,
]

ACTIVE_TENANTS: List[str] = [
    TENANT_ROOT,
    TENANT_ENG,
    TENANT_BACKEND,
    TENANT_SALES,
    TENANT_ENTERPRISE,
]


async def fetch_all_tenant_ids(
    client: httpx.AsyncClient,
    base_url: str,
    auth_headers: Dict[str, str],
    *,
    limit: int = 2,
    statuses: Optional[str] = None,
    select: Optional[str] = None,
) -> List[str]:
    """
    Fetch all tenant IDs via cursor pagination.

    Returns tenant IDs in the order returned by the API.
    """
    url = f"{base_url}/tenant-resolver/v1/tenants"
    params: Dict[str, Any] = {"limit": str(limit)}
    if statuses is not None:
        params["statuses"] = statuses
    if select is not None:
        params["$select"] = select

    seen: Set[str] = set()
    result: List[str] = []

    cursor: Optional[str] = None
    for _ in range(20):
        if cursor:
            params["cursor"] = cursor
        else:
            params.pop("cursor", None)

        resp = await client.get(url, headers=auth_headers, params=params)
        if resp.status_code in (401, 403) and not auth_headers:
            return []

        resp.raise_for_status()
        data = resp.json()

        items = data.get("items", [])
        page_info = data.get("page_info", {})
        cursor = page_info.get("next_cursor")

        for item in items:
            tid = item.get("id")
            if tid is None:
                continue
            if tid not in seen:
                seen.add(tid)
                result.append(tid)

        if not cursor:
            break

    return result


