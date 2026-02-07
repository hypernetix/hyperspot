# Descendant override permissions

## Setup

Parent upstream shares configs with `sharing=inherit`:
- `auth.sharing=inherit`
- `rate_limit.sharing=inherit`
- `plugins.sharing=inherit`

## Scenario A: child lacks override_auth permission

Child attempts to override auth config.

Expected:
- Update/binding rejected or effective config ignores child override (lock expected behavior).

## Scenario B: child lacks override_rate permission

Child sets a custom `rate_limit`.

Expected:
- Rejected or ignored.

## Scenario C: child lacks add_plugins permission

Child attempts to append plugins.

Expected:
- Rejected or ignored.
