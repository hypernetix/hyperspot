# Request correlation headers propagate end-to-end

## Scenario A: client provides X-Request-ID

```http
GET /api/oagw/v1/proxy/httpbin.org/get HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
X-Request-ID: req-abc-123
```

Expected:
- Upstream receives `X-Request-ID: req-abc-123`.
- Client response includes the same `X-Request-ID`.
- Audit log uses this request id.

## Scenario B: client does not provide X-Request-ID

Same request without `X-Request-ID`.

Expected:
- Gateway generates a request id (if implemented).
- Generated id is consistent across response headers and audit log record.
