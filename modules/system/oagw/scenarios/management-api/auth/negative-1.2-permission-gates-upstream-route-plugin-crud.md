# Management API permission gates

## Scenario A: missing permission

### Request

```http
POST /api/oagw/v1/routes HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token-without-route-create>
Content-Type: application/json

{
  "upstream_id": "gts.x.core.oagw.upstream.v1~7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "match": {
    "http": {
      "methods": ["POST"],
      "path": "/post",
      "query_allowlist": ["tag"],
      "path_suffix_mode": "disabled"
    }
  }
}
```

### Expected response

- `403 Forbidden`
- `Content-Type: application/problem+json`

```json
{
  "type": "<stable-error-type>",
  "title": "Forbidden",
  "status": 403,
  "detail": "<missing permission or access denied>",
  "instance": "/api/oagw/v1/routes"
}
```

## Scenario B: permission present

- Repeat the same request with `Authorization: Bearer <token-with-route-create>`.
- Expect `201 Created` and a returned Route resource.
