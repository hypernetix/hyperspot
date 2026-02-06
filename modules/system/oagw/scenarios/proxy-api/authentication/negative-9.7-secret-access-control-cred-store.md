# Secret access control via cred_store

## Scenario A: cred_store denies access

Upstream auth references a secret not accessible to the calling tenant:

```json
{
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": {
      "header": "Authorization",
      "prefix": "Bearer ",
      "secret_ref": "cred://partner-only/secret"
    }
  }
}
```

Invoke proxy as a tenant without secret access.

Expected:
- `401 Unauthorized`
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`

## Scenario B: secret missing

Use `secret_ref` that does not exist.

Expected:
- `500 Internal Server Error` with `type` = `...secret.not_found...`.
