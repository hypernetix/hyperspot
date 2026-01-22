# Built-in CORS handling (preflight)

## Setup

Enable CORS on upstream or route:

```json
{
  "cors": {
    "enabled": true,
    "allowed_origins": ["https://app.example.com"],
    "allowed_methods": ["GET", "POST"],
    "allowed_headers": ["Content-Type", "Authorization"],
    "max_age": 3600,
    "allow_credentials": true
  }
}
```

## Preflight request

```http
OPTIONS /api/oagw/v1/proxy/<alias>/resource HTTP/1.1
Host: oagw.example.com
Origin: https://app.example.com
Access-Control-Request-Method: POST
Access-Control-Request-Headers: Content-Type, Authorization
```

## Expected response

- `204 No Content`
- Includes CORS headers:
  - `Access-Control-Allow-Origin: https://app.example.com`
  - `Access-Control-Allow-Methods: GET, POST`
  - `Access-Control-Allow-Headers: Content-Type, Authorization`
  - `Access-Control-Max-Age: 3600`
  - `Access-Control-Allow-Credentials: true`
  - `Vary: Origin`

## What to check

- Preflight is handled locally (no upstream call).
