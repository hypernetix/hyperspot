# Single endpoint upstream without X-OAGW-Target-Host header

## Setup

- Upstream `api.openai.com` with single endpoint:
  - `endpoints`: `[{"scheme": "https", "host": "api.openai.com", "port": 443}]`
  - `alias`: `api.openai.com` (auto-generated)

## Request

```http
POST /api/oagw/v1/proxy/api.openai.com/v1/chat/completions HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
Content-Type: application/json

{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}
```

## Expected behavior

- Request routes to `api.openai.com:443` without requiring `X-OAGW-Target-Host` header
- Behavior unchanged from current implementation
- Response: `200 OK` with upstream response body

## Validation

- Single-endpoint upstreams do not require `X-OAGW-Target-Host` header
- Absence of header does not cause routing failure
