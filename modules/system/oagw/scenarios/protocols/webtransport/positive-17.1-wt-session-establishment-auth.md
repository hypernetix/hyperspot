# WebTransport (wt) session establishment + auth (future-facing)

## Setup

- Upstream endpoint:
  - `scheme`: `wt`
  - `protocol`: `gts.x.core.oagw.protocol.v1~x.core.http.v1` (or a dedicated protocol if introduced)

## Inbound request

WebTransport session establishment (QUIC/HTTP3) is implementation-defined. Represent it as a session open request:

- open session
- open bidirectional stream
- send datagram

## Expected behavior

If WebTransport is supported:
- Auth is injected for session establishment.
- Streams are forwarded.

If WebTransport is not supported:
- Gateway returns `502 ProtocolError` (or equivalent), `application/problem+json`, `X-OAGW-Error-Source: gateway`.
- Failure mode is stable and documented.
