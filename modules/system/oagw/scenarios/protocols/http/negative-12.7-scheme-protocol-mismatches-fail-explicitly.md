# Scheme/protocol mismatches fail explicitly

## Scenario A: protocol=http with scheme=grpc

Attempt to create upstream:

```json
{
  "alias": "grpc.example.com",
  "server": { "endpoints": [ { "scheme": "grpc", "host": "grpc.example.com", "port": 443 } ] },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1"
}
```

Expected:
- Rejected at validation with `400` `application/problem+json`.

## Scenario B: protocol=grpc with scheme=https

Attempt to create upstream:

```json
{
  "alias": "api.example.com",
  "server": { "endpoints": [ { "scheme": "https", "host": "api.example.com", "port": 443 } ] },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.grpc.v1"
}
```

Expected:
- Rejected at validation (`400`), or invoke fails with `502 ProtocolError`.
- Lock expected behavior.
