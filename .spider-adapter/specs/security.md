# Security (Hyperspot)

**Version**: 1.0  
**Purpose**: Provide actionable security guardrails for application code, with emphasis on secure DB access and secrets handling.  
**Scope**: All modules/apps, especially anything handling tenant data or authentication.  

## Input validation

- Validate request payloads and user input (use structured validation; return `Problem`-style errors).

## Secrets management

- Never commit secrets.
- Use environment variables / config for secrets.

## Database security (Secure ORM)

- DB access should use the secure ORM layer (`modkit-db` secure wrapper).
- Avoid direct execution of SeaORM queries that bypass scoping; this is also reinforced by repository lint settings.

## Supply-chain security

- `cargo-deny` checks advisories and license policy in CI.
- OpenSSF Scorecard workflow runs on the default branch.

## Validation Criteria

- [ ] Sensitive code paths do not log secrets or PII.
- [ ] DB queries are request-scoped and tenant/resource scoped where applicable.
- [ ] Dependency changes keep `cargo-deny` passing (or justify exceptions).
- [ ] CI security workflows remain intact.

## Examples

✅ Valid:
- Use `SecurityCtx` per request and `SecureConn` for ORM access.

❌ Invalid:
- Store security context globally or bypass secure DB scoping.

---

**Source**: `guidelines/SECURITY.md`, `docs/SECURE-ORM.md`, `clippy.toml`, `.github/workflows/ci.yml`, `deny.toml`, `.github/workflows/scorecard.yml`.  
**Last Updated**: 2026-02-05

