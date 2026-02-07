# Transform plugin: request path rewrite

## Setup

Create custom transform plugin with phase `on_request`:

```starlark
def on_request(ctx):
    prefix = ctx.config.get("path_prefix", "")
    if prefix:
        ctx.request.set_path(prefix + ctx.request.path)
    return ctx.next()
```

Plugin config:

```json
{ "path_prefix": "/v2" }
```

Attach plugin to route.

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/users HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected outbound request

- Path is rewritten to `/v2/users`.

## What to check

- Upstream receives rewritten path.
- Logs do not contain secrets.
