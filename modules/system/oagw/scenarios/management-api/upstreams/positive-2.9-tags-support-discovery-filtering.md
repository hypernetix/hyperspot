# Tags support discovery and filtering

## Setup

Create upstream with tags.

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "api.openai.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "api.openai.com",
  "tags": ["openai", "llm", "chat"]
}
```

## List/filter

If tags are exposed via OData `$filter`, use that. Otherwise use the module’s tags filter parameter.

Example (OData-style placeholder):

```http
GET /api/oagw/v1/upstreams?$filter=tags has 'llm' HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

Expected: results include the upstream with `tags` containing `llm`.
