# WebSocket connection idle timeout enforced

## Setup

- Configure idle timeout for WebSocket connections (exact config location is implementation-defined).

## Steps

1. Establish WebSocket connection.
2. Do not send frames for longer than idle timeout.

## Expected behavior

- Gateway closes the connection.
- Close frame/code is consistent (lock expected behavior).
- No leaked in-flight metrics.
