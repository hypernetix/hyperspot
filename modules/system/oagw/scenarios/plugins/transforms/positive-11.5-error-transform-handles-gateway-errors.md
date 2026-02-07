# Transform plugin: on_error for gateway errors

## Setup

Create transform plugin with `on_error` phase:

```starlark
def on_error(ctx):
    if ctx.error and not ctx.error.upstream:
        return ctx.respond(400, "{\"error\":\"bad_request\"}")
    return ctx.next()
```

Attach plugin.

## Trigger a gateway error

Example: send disallowed query param.

```http
GET /api/oagw/v1/proxy/<alias>/resource?debug=1 HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- Error is handled by plugin and replaced by custom response.
- No upstream call is made.
- Lock expected semantics for status and content-type.
