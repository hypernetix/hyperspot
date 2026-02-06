# Hop-by-hop headers are stripped

## Inbound request

```http
GET /api/oagw/v1/proxy/httpbin.org/get HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Connection: keep-alive, Upgrade
Upgrade: websocket
TE: trailers
Transfer-Encoding: chunked

```

## Expected behavior

- Gateway strips hop-by-hop headers when building outbound request:
  - `Connection`
  - `Upgrade`
  - `TE`
  - `Transfer-Encoding`
- Outbound request uses correct upstream scheme and does not attempt protocol upgrades.

## What to check

- Upstream echo endpoint does not receive the stripped headers.
- Response is successful (or a gateway validation error if request is malformed; lock expected behavior).
