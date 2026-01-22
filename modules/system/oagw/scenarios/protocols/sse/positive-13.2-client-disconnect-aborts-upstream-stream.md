# SSE client disconnect aborts upstream stream

## Setup

SSE route with continuous event stream.

## Steps

1. Start SSE request.
2. After receiving N events, disconnect the client.

## Expected behavior

- Gateway closes/cancels upstream stream.
- No resource leak:
  - in-flight metrics decrease
  - audit log records stream abort (if implemented)
- If an error is produced, it is classified as `StreamAborted`.
