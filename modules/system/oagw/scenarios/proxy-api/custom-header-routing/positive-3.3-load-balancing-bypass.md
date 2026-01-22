# X-OAGW-Target-Host bypasses round-robin load balancing

## Setup

- Upstream `my-service` with multiple endpoints:
  - `endpoints`: `[{"scheme": "https", "host": "server-a.example.com", "port": 443}, {"scheme": "https", "host": "server-b.example.com", "port": 443}, {"scheme": "https", "host": "server-c.example.com", "port": 443}]`
  - `alias`: `my-service`

## Request Sequence

Request 1:
```http
GET /api/oagw/v1/proxy/my-service/v1/status HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: server-c.example.com
```

Request 2:
```http
GET /api/oagw/v1/proxy/my-service/v1/status HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: server-c.example.com
```

Request 3:
```http
GET /api/oagw/v1/proxy/my-service/v1/status HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
```

## Expected behavior

- Request 1 routes to `server-c.example.com` (explicit)
- Request 2 routes to `server-c.example.com` (explicit, bypasses round-robin state)
- Request 3 uses round-robin (routes to next in sequence, e.g., `server-a.example.com`)
- All responses: `200 OK`

## Validation

- `X-OAGW-Target-Host` header overrides load balancing
- Explicit routing allows targeting specific endpoints for debugging/testing
- Load balancing state is preserved for requests without the header
