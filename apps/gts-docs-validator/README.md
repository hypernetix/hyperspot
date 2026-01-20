# GTS Documentation Validator (DE0903)

A Rust CLI tool that validates GTS (Global Type System) identifiers in documentation files (`.md`, `.json`, `.yaml`, `.yml`).

This complements the Dylint-based DE0901 lint that validates GTS identifiers in Rust source code.

## Usage

```bash
# Basic validation
gts-docs-validator docs modules libs examples

# With vendor validation (ensures all IDs use expected vendor)
gts-docs-validator --vendor x docs modules libs examples

# With exclusions
gts-docs-validator --exclude "target/*" --exclude "docs/api/*" .

# JSON output (for CI integration)
gts-docs-validator --json docs

# Verbose output (shows files being scanned)
gts-docs-validator --verbose docs
```

## Makefile Targets

```bash
make gts-docs         # Validate GTS IDs (structural only)
make gts-docs-vendor  # Validate with --vendor x check
make gts-docs-test    # Run unit tests
```

## Options

| Option | Description |
|--------|-------------|
| `--vendor <VENDOR>` | Expected vendor for all GTS IDs (e.g., `--vendor x`) |
| `--exclude <PATTERN>` | Glob patterns to exclude (can be repeated) |
| `--json` | Output results as JSON |
| `--verbose` | Show file scanning progress |

## Example Vendors

When using `--vendor`, the following example/placeholder vendors are always tolerated (commonly used in documentation and tutorials):

- `acme`
- `globex`
- `example`
- `demo`
- `test`
- `sample`
- `tutorial`

## Smart Context Detection

The validator automatically handles:

- **Wildcard patterns**: `gts.x.*` is allowed in filter/pattern contexts (e.g., `$filter`, `pattern`, `match`)
- **Bad examples**: GTS IDs near "invalid", "wrong", "❌" markers are skipped
- **Trailing tildes**: Schema IDs ending with `~` are properly validated

## GTS Identifier Format

```text
gts.<vendor>.<org>.<package>.<type>.<version>~
    │        │      │         │      └── Version (v1, v1.2, etc.)
    │        │      │         └── Type name (snake_case)
    │        │      └── Package name
    │        └── Organization
    └── Vendor identifier
```

## Exit Codes

- `0` - All GTS identifiers are valid
- `1` - Invalid GTS identifiers found

## Related

- **DE0901**: Dylint lint for GTS patterns in Rust source (`make dylint`)
- **DE0902**: Prevents `schema_for!` on GTS structs (`make dylint`)
