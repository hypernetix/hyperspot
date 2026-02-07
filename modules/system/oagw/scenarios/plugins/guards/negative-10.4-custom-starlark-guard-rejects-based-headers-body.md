# Custom Starlark guard rejects based on headers/body

## Setup

Create and attach a custom guard plugin similar to:

```starlark
def on_request(ctx):
    for h in ctx.config.get("required_headers", []):
        if not ctx.request.headers.get(h):
            return ctx.reject(400, "MISSING_HEADER", "Required header: " + h)

    if len(ctx.request.body) > ctx.config.get("max_body_size", 1048576):
        return ctx.reject(413, "BODY_TOO_LARGE", "Body exceeds limit")

    return ctx.next()
```

Attach to route with config:

```json
{
  "required_headers": ["X-Customer-Id"],
  "max_body_size": 1024
}
```

## Inbound request (missing required header)

```http
POST /api/oagw/v1/proxy/<alias>/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{"a":1}
```

## Expected response

- `400 Bad Request`
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`
- Error code matches plugin: `MISSING_HEADER`

## Inbound request (body too large)

Send a JSON body larger than 1024 bytes.

Expected:
- `413 Payload Too Large`
