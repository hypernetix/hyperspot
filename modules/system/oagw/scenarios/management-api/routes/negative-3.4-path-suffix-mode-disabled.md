# Path suffix mode = disabled

## Setup

Route config:
- `path`: `/v1/chat/completions`
- `path_suffix_mode`: `disabled`

## Inbound request (suffix provided)

```http
POST /api/oagw/v1/proxy/<alias>/v1/chat/completions/models/gpt-4 HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{"stream":false}
```

## Expected response

- `400 Bad Request` (gateway validation)
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`
- `detail` mentions path suffix not allowed.
