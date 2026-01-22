# CORS credentials + wildcard is rejected

## Scenario A: reject at configuration time

Attempt to configure:

```json
{
  "cors": {
    "enabled": true,
    "allowed_origins": ["*"],
    "allow_credentials": true
  }
}
```

Expected:
- `400 Bad Request`
- `Content-Type: application/problem+json`
- `detail` indicates that credentials cannot be used with wildcard origins.

## Scenario B: reject at request time (if config validation allows storing)

If configuration is accepted, preflight or actual request must be rejected.

Expected:
- `403 Forbidden`
- `Content-Type: application/problem+json`
