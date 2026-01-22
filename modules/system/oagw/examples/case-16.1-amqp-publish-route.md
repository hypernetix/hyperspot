# AMQP publish route

## Setup

- Upstream:
  - `protocol`: `gts.x.core.oagw.protocol.v1~x.core.amqp.v1`
  - endpoint: `scheme=amqp`, `port=5672`
- Route match:

```json
{
  "match": {
    "amqp": {
      "exchange": "events",
      "routing_key": "user.created"
    }
  }
}
```

## Inbound request

The proxy surface for AMQP is implementation-defined. Represent it as a logical publish request:

- exchange: `events`
- routing_key: `user.created`
- payload: `{...}`

## Expected behavior

- Message is published to upstream broker with correct exchange/routing_key.
- If response/ack is modeled, it is returned consistently.
