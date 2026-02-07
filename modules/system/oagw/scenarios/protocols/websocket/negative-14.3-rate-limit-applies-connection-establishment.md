# WebSocket rate limit applies to connection establishment

## Setup

- Upstream/route rate limit is configured low (example: 1 per minute for tenant).

## Steps

1. Establish first WebSocket connection (should succeed).
2. Establish second connection within the same window.

## Expected behavior

- Second upgrade request is rejected:
  - `429 Too Many Requests`
  - `Content-Type: application/problem+json`
  - `X-OAGW-Error-Source: gateway`
  - `Retry-After` present

## What to check

- Rate limiting is applied to the handshake request, not individual frames.
