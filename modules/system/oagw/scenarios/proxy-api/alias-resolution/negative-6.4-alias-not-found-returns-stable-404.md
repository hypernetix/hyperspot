# Alias not found returns stable 404

## Inbound request

```http
GET /api/oagw/v1/proxy/does-not-exist/health HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- `404 Not Found`
- `X-OAGW-Error-Source: gateway`
- `Content-Type: application/problem+json`
- `detail` indicates upstream alias not found.
