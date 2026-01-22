# Gateway error uses Problem Details (`X-OAGW-Error-Source: gateway`)

## Setup

Invoke a request that triggers a gateway error.

Example: unknown alias.

## Inbound request

```http
GET /api/oagw/v1/proxy/does-not-exist/get HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- `404 Not Found`
- `X-OAGW-Error-Source: gateway`
- `Content-Type: application/problem+json`

```json
{
  "type": "<stable-error-type>",
  "title": "Not Found",
  "status": 404,
  "detail": "<upstream not found>",
  "instance": "/api/oagw/v1/proxy/does-not-exist/get"
}
```
