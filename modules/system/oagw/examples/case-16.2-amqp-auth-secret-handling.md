# AMQP auth and secret handling

## Setup

Configure upstream auth for broker credentials using `cred_store` references (exact auth plugin/config shape is implementation-defined for AMQP).

## Scenario A: secret allowed

Expected:
- Credentials resolved.
- Connection succeeds.

## Scenario B: secret denied

Expected:
- `401 Unauthorized` gateway error.
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`
