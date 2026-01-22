# Transform plugin: response JSON redaction

## Setup

Custom transform plugin (`on_response`):

```starlark
def on_response(ctx):
    data = ctx.response.json()
    for field in ctx.config.get("fields", []):
        if field in data:
            data[field] = "[REDACTED]"
    ctx.response.set_json(data)
    return ctx.next()
```

Config:

```json
{ "fields": ["email", "ssn"] }
```

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/user HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- Same status code.
- JSON fields `email` and `ssn` are replaced with `[REDACTED]`.

## Non-JSON variant

If upstream returns non-JSON:
- Either plugin no-ops, or gateway returns an error.
- Lock expected behavior.
