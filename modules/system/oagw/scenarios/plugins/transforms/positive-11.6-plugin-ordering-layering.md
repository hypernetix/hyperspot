# Plugin ordering and layering

## Setup

Configure plugins at both levels:

- Upstream plugins: `[U1, U2]`
- Route plugins: `[R1, R2]`

Where each plugin emits a distinct observable side-effect:
- Sets a header `X-Chain-Order` by appending its name.

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- Execution order:
  - Auth plugin
  - Upstream guards
  - Upstream transforms
  - Route guards
  - Route transforms

- Observable order string equals `U1,U2,R1,R2`.
