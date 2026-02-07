# Plugin control flow: next, reject, respond

## Scenario A: reject

Plugin:

```starlark
def on_request(ctx):
    return ctx.reject(400, "REJECTED", "blocked")
```

Expected:
- Gateway returns `400` and stops plugin chain.
- Upstream is not called.

## Scenario B: respond

Plugin:

```starlark
def on_request(ctx):
    return ctx.respond(200, "{\"ok\":true}")
```

Expected:
- Gateway returns `200` with body.
- Upstream is not called.

## Scenario C: next

Plugin:

```starlark
def on_request(ctx):
    return ctx.next()
```

Expected:
- Request continues to next plugin/upstream.
