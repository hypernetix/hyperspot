# gRPC server streaming proxied

## Setup

- Route match:
  - service `example.v1.UserService`
  - method `ListUsers`

## Inbound request

- Native gRPC client makes a server-streaming call.

## Expected behavior

- Gateway forwards stream without buffering.
- HTTP/2 flow control is respected (no uncontrolled buffering).
- Stream completes with trailers.
