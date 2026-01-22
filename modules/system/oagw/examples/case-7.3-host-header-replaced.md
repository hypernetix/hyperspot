# Host header replaced by upstream host

## Inbound request

```http
GET /api/oagw/v1/proxy/httpbin.org/get HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Host: evil.example.com

```

## Expected behavior

- Gateway ignores/overrides the inbound `Host` header.
- Outbound request to upstream uses `Host: httpbin.org`.

## What to check

- Use an upstream echo endpoint to verify received `Host`.
- Ensure request is not routed based on inbound host.
