# Create AMQP route by exchange + routing_key

## Upstream configuration

```json
{
  "alias": "amqp.example.com",
  "server": {
    "endpoints": [
      { "scheme": "amqp", "host": "amqp.example.com", "port": 5672 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.amqp.v1"
}
```

## Route configuration

```http
POST /api/oagw/v1/routes HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "upstream_id": "gts.x.core.oagw.upstream.v1~<upstream-uuid>",
  "match": {
    "amqp": {
      "exchange": "events",
      "routing_key": "user.created"
    }
  }
}
```

## Expected behavior

- Publish request matching `exchange=events` and `routing_key=user.created` is routed.
- Invalid routing key patterns are rejected at config validation (`400` `application/problem+json`).
