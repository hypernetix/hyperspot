# WebSocket auth injected during handshake (not per-message)

## Setup

- Upstream uses API key/bearer auth.
- Auth plugin injects credentials into the outbound upgrade request.

## Inbound request

Same as WebSocket upgrade, without passing credentials intended for upstream (only OAGW bearer token for inbound auth).

## Expected behavior

- Outbound upgrade request includes injected auth header (e.g., `Authorization: Bearer <secret>`).
- Subsequent WebSocket frames are forwarded without per-frame auth injection.

## What to check

- Upstream handshake succeeds only when auth is injected.
- Upstream does not require auth inside frames.
