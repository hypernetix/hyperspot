# Plain HTTP Request/Response (positive)

## Upstream Configuration

```json
{
  "alias": "httpbin.org",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": {
      "header": "X-Api-Key",
      "secret_ref": "cred://httpbin/api-key"
    }
  },
  "headers": {
    "request": {
      "set": {
        "User-Agent": "OAGW/1.0"
      }
    }
  },
  "rate_limit": {
    "algorithm": "token_bucket",
    "sustained": { "rate": 100, "window": "minute" },
    "scope": "tenant",
    "strategy": "reject"
  }
}
```

## Route Configuration

```json
{
  "upstream_id": "gts.x.core.oagw.upstream.v1~<upstream-uuid>",
  "match": {
    "http": {
      "methods": ["POST"],
      "path": "/post",
      "query_allowlist": ["tag"],
      "path_suffix_mode": "disabled"
    }
  },
  "plugins": {
    "items": [
      "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1"
    ]
  }
}
```

## Inbound Request

```http
POST /api/oagw/v1/proxy/httpbin.org/post?tag=test HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json
Content-Length: 27
X-Request-ID: req-http-001

{"name": "test", "value": 42}
```

## Outbound Request (to upstream)

```http
POST /post?tag=test HTTP/1.1
Host: httpbin.org
X-Api-Key: <secret-from-cred-store>
User-Agent: OAGW/1.0
Content-Type: application/json
Content-Length: 27
X-Request-ID: req-http-001

{"name": "test", "value": 42}
```

## Upstream Response

```http
HTTP/1.1 200 OK
Content-Type: application/json

{"ok":true}
```

## Outbound Response (to client)

```http
HTTP/1.1 200 OK
Content-Type: application/json
X-Request-ID: req-http-001
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99

{"ok":true}
```
