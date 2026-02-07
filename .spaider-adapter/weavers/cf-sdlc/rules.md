# Common rules

## Navigation rules

ALWAYS open and follow `{spaider_path}/requirements/artifacts-registry.md` WHEN creating/registering artifacts in `artifacts.json`

## ID Format (REQUIRED)

All Spaider IDs MUST:

- Use format: `spd-{hierarchy-prefix}-{kind}-{slug}`
- Match regex: `^spd-[a-z0-9][a-z0-9-]+$`
- Be wrapped in backticks: `` `spd-...` ``
- Use only lowercase `a-z`, digits `0-9`, and `-` (kebab-case)

**ID definition** examples (in `id:*` blocks):

```text
**ID**: `spd-...`
- [ ] `p1` - **ID**: `spd-...`
```

**ID reference** examples (in `id-ref:*` blocks):

```text
`spd-...`
[x] `p1` - `spd-...`
```

Any inline `` `spd-...` `` in text is treated as an ID reference.
