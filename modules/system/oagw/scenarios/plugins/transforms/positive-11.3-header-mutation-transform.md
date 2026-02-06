# Transform plugin: header mutation

## Setup

Custom transform plugin (`on_request`):

```starlark
def on_request(ctx):
    ctx.request.headers.set("X-Feature-Flag", "A")
    ctx.request.headers.remove("X-Internal")
    return ctx.next()
```

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
X-Internal: 1
```

## Expected outbound request

- `X-Feature-Flag: A` is present.
- `X-Internal` is removed.

## Additional invariant

- Hop-by-hop headers remain stripped even if plugin tries to set them (lock expected behavior).
