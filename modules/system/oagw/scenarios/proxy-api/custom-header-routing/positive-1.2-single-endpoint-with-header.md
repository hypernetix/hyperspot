# Single endpoint upstream with valid X-OAGW-Target-Host header

## Setup

- Upstream `api.openai.com` with single endpoint:
  - `endpoints`: `[{"scheme": "https", "host": "api.openai.com", "port": 443}]`
  - `alias`: `api.openai.com`

## Request

```http
POST /api/oagw/v1/proxy/api.openai.com/v1/chat/completions HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: api.openai.com
Content-Type: application/json

{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}
```

## Expected behavior

- Header is validated (must match `api.openai.com`)
- Request routes to `api.openai.com:443`
- `X-OAGW-Target-Host` header is stripped before forwarding to upstream
- Response: `200 OK` with upstream response body

## Validation

- Single-endpoint upstreams accept optional `X-OAGW-Target-Host` header
- Header is validated against the single endpoint
- Header is not forwarded to upstream
