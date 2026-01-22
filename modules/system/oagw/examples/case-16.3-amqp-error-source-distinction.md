# AMQP error source distinction

## Scenario A: broker unavailable

- Upstream host/port not reachable.

Expected:
- Gateway-generated failure (connect/timeout):
  - `X-OAGW-Error-Source: gateway`
  - `503 LinkUnavailable` or `502 DownstreamError` (lock expected behavior)

## Scenario B: broker-level publish failure

- Broker rejects publish (if modeled).

Expected:
- Treated as upstream error:
  - `X-OAGW-Error-Source: upstream`
  - Body/status passthrough or mapped error.
