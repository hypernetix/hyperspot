# Plugin usage tracking and GC eligibility

## Setup

1. Create a custom plugin `gts.x.core.oagw.plugin.transform.v1~<uuid>`.
2. Attach it to a route.

## Step 1: exercise plugin (update usage)

Invoke proxy for that route.

Expected:
- Plugin is executed.
- Plugin record has:
  - `last_used_at` updated
  - `gc_eligible_at = null`

## Step 2: unlink plugin

Remove plugin reference from all upstreams/routes.

Expected:
- Plugin record sets `gc_eligible_at = now + TTL`.

## Step 3: GC deletion

After TTL and GC job:

Expected:
- Plugin row is deleted.
- `GET /api/oagw/v1/plugins/{id}` returns `404`.
