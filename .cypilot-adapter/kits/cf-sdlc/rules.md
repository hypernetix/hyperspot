# Common rules

## Navigation rules

ALWAYS open and follow `{cypilot_path}/requirements/artifacts-registry.md` WHEN creating/registering artifacts in `artifacts.json`

## ID Format (REQUIRED)

All Cypilot IDs MUST:

- Use format: `cpt-{hierarchy-prefix}-{kind}-{slug}`
- Match regex: `^cpt-[a-z0-9][a-z0-9-]+$`
- Be wrapped in backticks: `` `cpt-...` ``
- Use only lowercase `a-z`, digits `0-9`, and `-` (kebab-case)

**ID definition**:

```text
**ID**: `cpt-...`
- [ ] `p1` - **ID**: `cpt-...`
```

**ID reference**:

```text
`cpt-...`
[x] `p1` - `cpt-...`
```

Any inline `` `cpt-...` `` in text is treated as an ID reference.
