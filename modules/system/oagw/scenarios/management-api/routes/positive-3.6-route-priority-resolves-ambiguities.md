# Route priority resolves ambiguities

## Setup

Create two routes that can both match the same request:

- Route A: `path=/v1/items`, `priority=0`
- Route B: `path=/v1/items`, `priority=10`

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/v1/items HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- Route B is selected.

## What to check

Verify selection using one of:
- Distinct upstream header transforms per route (route plugin sets `X-Route: B`).
- Audit log field containing route id.
