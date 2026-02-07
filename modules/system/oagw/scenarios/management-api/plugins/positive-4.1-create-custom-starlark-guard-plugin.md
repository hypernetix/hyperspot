# Create custom Starlark guard plugin

## Step 1: Create plugin

```http
POST /api/oagw/v1/plugins HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token-with-plugin-create>
Content-Type: application/json

{
  "plugin_type": "guard",
  "name": "request_validator",
  "description": "Validates required headers and max body size",
  "config_schema": {
    "type": "object",
    "properties": {
      "max_body_size": { "type": "integer", "default": 1048576 },
      "required_headers": { "type": "array", "items": { "type": "string" } }
    }
  },
  "source_code": "def on_request(ctx):\n    for h in ctx.config.get(\"required_headers\", []):\n        if not ctx.request.headers.get(h):\n            return ctx.reject(400, \"MISSING_HEADER\", \"Required header: \" + h)\n    if len(ctx.request.body) > ctx.config.get(\"max_body_size\", 1048576):\n        return ctx.reject(413, \"BODY_TOO_LARGE\", \"Body exceeds limit\")\n    return ctx.next()\n"
}
```

## Expected response

- `201 Created`
- Response includes plugin id as anonymous GTS identifier:
  - `gts.x.core.oagw.plugin.guard.v1~<uuid>`

## Step 2: Fetch plugin source

```http
GET /api/oagw/v1/plugins/gts.x.core.oagw.plugin.guard.v1~<uuid>/source HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token-with-plugin-read>
```

Expected:
- `200 OK`
- `Content-Type: text/plain`

## Step 3: Attach plugin to a route

- Update/create a route with `plugins.items` containing the plugin id.
- Invoke proxy and verify that requests missing required headers are rejected with gateway response and `application/problem+json`.
