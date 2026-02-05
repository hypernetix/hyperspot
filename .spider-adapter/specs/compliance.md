# Compliance & Governance (Licenses, Supply Chain, PR/Release)

**Version**: 1.0  
**Purpose**: Document the guardrails around licensing, supply-chain checks, and PR/release governance.  
**Scope**: Repository policies and CI workflows.  

## License & advisory policy

- `cargo-deny` enforces a license allowlist and RustSec advisory checks.
- Exceptions must be explicit and justified (e.g., an advisory ignore entry).

## Supply-chain security

- OpenSSF Scorecard workflow is enabled for the default branch.

## PR & release governance

- PR governance workflow ensures human reviewers and monitors unanswered review comments.
- Release automation uses release-plz and updates a single repo changelog.

## Validation Criteria

- [ ] Dependency changes keep `cargo-deny` passing (or exceptions are justified).
- [ ] CI workflows remain enabled and functional.
- [ ] Release changes follow release-plz conventions.

## Examples

✅ Valid:
- Add a dependency and confirm its license is allowlisted; document any advisory exceptions.

❌ Invalid:
- Add a dependency with a non-allowlisted license without updating policy and rationale.

---

**Source**: `deny.toml`, `.github/workflows/ci.yml`, `.github/workflows/scorecard.yml`, `.github/workflows/pr-governance.yml`, `release-plz.toml`.  
**Last Updated**: 2026-02-05

