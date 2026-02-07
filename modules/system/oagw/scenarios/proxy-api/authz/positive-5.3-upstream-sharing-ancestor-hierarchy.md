# Upstream sharing via ancestor hierarchy

## Setup

- Ancestor tenant (partner) creates upstream `alias=api.openai.com` with shareable fields:
  - `auth.sharing=inherit`
  - `rate_limit.sharing=enforce`
  - `plugins.sharing=inherit`

## Descendant binds/uses upstream

- Descendant tenant invokes proxy:

```http
POST /api/oagw/v1/proxy/api.openai.com/v1/chat/completions HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <descendant-token>
Content-Type: application/json

{"model":"gpt-4","stream":false,"messages":[]}
```

## Expected behavior

- Alias resolves to ancestor upstream.
- Descendant request is authorized only if upstream is visible (shared).
- Effective configuration merges as designed (auth/rate/plugins).
