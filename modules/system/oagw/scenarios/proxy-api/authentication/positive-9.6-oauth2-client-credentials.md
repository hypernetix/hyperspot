# Outbound auth: OAuth2 client credentials (basic-auth client)

## Upstream configuration

```json
{
  "alias": "vendor-basic.example.com",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "vendor-basic.example.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.oauth2.client_cred_basic.v1",
    "config": {
      "token_url": "https://<oauth-host>/oauth/token",
      "client_id_ref": "cred://vendor-basic/oauth2/client_id",
      "client_secret_ref": "cred://vendor-basic/oauth2/client_secret",
      "scope": "read"
    }
  }
}
```

## What to check

- Token request authenticates client via HTTP Basic (not form params).
- Access token is injected as bearer token on outbound requests.
