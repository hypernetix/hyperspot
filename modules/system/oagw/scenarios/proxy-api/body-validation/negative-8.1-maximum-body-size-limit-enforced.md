# Maximum body size limit enforced (100MB)

## Inbound request

Send a request body larger than 100MB.

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/octet-stream
Content-Length: <greater-than-100mb>

<binary>
```

## Expected response

- `413 Payload Too Large`
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`
- Rejection happens before buffering the entire body (lock via memory/latency instrumentation if available).
