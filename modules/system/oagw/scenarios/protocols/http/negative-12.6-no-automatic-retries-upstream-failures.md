# No automatic retries for upstream failures

## Setup

- Upstream endpoint performs a side effect per request and can be configured to fail once (e.g., first request returns 502/connection close).

## Inbound request

```http
POST /api/oagw/v1/proxy/<alias>/side-effect HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{"op":"create"}
```

## Expected behavior

- Gateway does not retry the request automatically.
- Verify via upstream logs/counters:
  - Exactly one request attempt is observed.
- Response is returned as an error to the client (gateway or upstream sourced depending on failure mode).
