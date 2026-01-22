# Update upstream

## Step 1: Create upstream

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "httpbin.org",
  "headers": {
    "request": {
      "set": {
        "User-Agent": "OAGW/1.0"
      }
    }
  }
}
```

Expected: `201 Created` with upstream id `gts.x.core.oagw.upstream.v1~<uuid>`.

## Step 2: Update upstream headers + enabled flag

```http
PUT /api/oagw/v1/upstreams/gts.x.core.oagw.upstream.v1~<uuid> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "httpbin.org",
  "enabled": true,
  "headers": {
    "request": {
      "set": {
        "User-Agent": "OAGW/2.0"
      }
    }
  }
}
```

## Expected response

- `200 OK`
- Returned upstream reflects changed header config.
- If plugins are referenced, only the references change; plugin resources remain immutable.
