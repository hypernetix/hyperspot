# Upstream error passthrough (`X-OAGW-Error-Source: upstream`)

## Setup

Route points to an upstream endpoint that returns an error (example: `/fail` returns `500` with JSON body).

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/fail HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Upstream response

```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{"error":"upstream_failed"}
```

## Expected response to client

- Status `500`.
- Body forwarded unchanged.
- `X-OAGW-Error-Source: upstream`.
- Content-Type preserved from upstream (not rewritten to Problem Details).
