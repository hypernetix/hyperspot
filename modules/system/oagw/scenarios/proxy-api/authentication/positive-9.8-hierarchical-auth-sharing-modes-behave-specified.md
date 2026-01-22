# Hierarchical auth sharing modes

## Scenario A: inherit + child override

Parent upstream:

```json
{
  "alias": "api.openai.com",
  "auth": {
    "sharing": "inherit",
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": { "secret_ref": "cred://partner/openai" }
  }
}
```

Child binding provides its own auth:

```json
{
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": { "secret_ref": "cred://customer/openai" }
  }
}
```

Expected:
- Effective auth uses child secret if override is allowed.

## Scenario B: enforce

Parent sets `auth.sharing=enforce`.

Expected:
- Child cannot override auth.

## Scenario C: private

Parent sets `auth.sharing=private`.

Expected:
- Child must provide its own auth.
