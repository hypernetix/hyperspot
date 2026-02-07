# Transform plugin: query mutation

## Setup

Custom transform plugin (`on_request`):

```starlark
def on_request(ctx):
    if ctx.config.get("add_api_version", False):
        ctx.request.add_query("api_version", "2024-01")

    q = ctx.request.query
    if "internal_debug" in q:
        del q["internal_debug"]
        ctx.request.set_query(q)

    return ctx.next()
```

Config:

```json
{ "add_api_version": true }
```

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/resource?internal_debug=1 HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- `internal_debug` is removed.
- `api_version=2024-01` is added.

## Order check

- If query allowlist is enforced before plugins, then `internal_debug` may be rejected before transform.
- Lock expected order by asserting either:
  - reject occurs pre-transform, or
  - allowlist applies to post-transform query.
