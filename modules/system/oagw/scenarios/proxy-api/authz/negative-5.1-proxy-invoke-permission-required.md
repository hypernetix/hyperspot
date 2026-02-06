# Proxy invoke permission required

## Inbound request

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token-without-proxy-invoke>
Content-Type: application/json

{"name":"test"}
```

## Expected response

- `403 Forbidden`
- If body is present:
  - `Content-Type: application/problem+json`
  - `X-OAGW-Error-Source: gateway`
- `detail` indicates missing proxy invoke permission.
