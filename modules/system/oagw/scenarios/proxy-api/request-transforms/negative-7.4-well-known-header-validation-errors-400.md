# Well-known header validation: Content-Length

## Scenario A: invalid Content-Length

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json
Content-Length: not-a-number

{"a":1}
```

Expected:
- `400 Bad Request`
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`

## Scenario B: Content-Length mismatch

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json
Content-Length: 999

{"a":1}
```

Expected:
- `400 Bad Request`
- `Content-Type: application/problem+json`
- `detail` mentions mismatch between declared and actual size.
