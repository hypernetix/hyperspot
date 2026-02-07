# Path suffix mode = append

## Setup

Route config:
- `path`: `/api`
- `path_suffix_mode`: `append`

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/api/v1/users/123 HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected outbound request (to upstream)

- Outbound path equals `/api` + `/v1/users/123`.

```http
GET /api/v1/users/123 HTTP/1.1
Host: <upstream-host>
```

## Expected response

- Upstream response is returned.
- No `//` introduced in path.
