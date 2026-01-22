# Plugin sharing modes across tenant hierarchy

## Scenario A: parent plugins `inherit`, child extends

1. Parent tenant creates upstream with:

```json
{
  "alias": "api.openai.com",
  "plugins": {
    "sharing": "inherit",
    "items": [
      "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1"
    ]
  }
}
```

2. Child tenant binds to the upstream and adds plugins:

```json
{
  "plugins": {
    "items": [
      "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.metrics.v1"
    ]
  }
}
```

Expected effective chain order:
- `[parent.logging, child.metrics]`

## Scenario B: parent plugins `enforce`

Parent:
- `plugins.sharing=enforce`

Child attempts to remove/replace parent plugin.

Expected:
- Child cannot remove/replace parent plugins.
- If API supports validation at update time, reject with `400`.

## Scenario C: parent plugins `private`

Parent:
- `plugins.sharing=private`

Expected:
- Child does not inherit parent plugins.
