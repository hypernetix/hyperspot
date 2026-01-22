# Outbound auth: OAuth2 client credentials

## Upstream configuration

```json
{
  "alias": "vendor.example.com",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "vendor.example.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.oauth2.client_cred.v1",
    "config": {
      "token_url": "https://<oauth-host>/oauth/token",
      "client_id_ref": "cred://vendor/oauth2/client_id",
      "client_secret_ref": "cred://vendor/oauth2/client_secret",
      "scope": "read write",
      "audience": "https://<api-host>/api"
    }
  }
}
```

## Invocation

```http
GET /api/oagw/v1/proxy/vendor.example.com/v1/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- Gateway obtains an access token via client credentials.
- Gateway injects `Authorization: Bearer <access_token>`.
- Token is cached.
- If upstream returns `401` due to expired token:
  - Gateway refreshes token and retries auth handling as defined (do not duplicate the request beyond auth refresh policy).
