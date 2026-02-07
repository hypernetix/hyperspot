# Starlark sandbox restrictions

## Scenario A: network I/O is blocked

Plugin attempts to perform network I/O.

Expected:
- Plugin execution fails.
- Gateway returns `503` (plugin error) or `400` (validation), `application/problem+json` (lock expected behavior).

## Scenario B: file I/O is blocked

Plugin attempts to read a file.

Expected:
- Denied.

## Scenario C: infinite loop is timed out

Plugin:

```starlark
def on_request(ctx):
    while True:
        pass
```

Expected:
- Execution time limit triggers.
- Gateway returns error.

## Scenario D: large allocations are blocked

Plugin attempts to allocate large data.

Expected:
- Memory limit triggers.
- Gateway returns error.
