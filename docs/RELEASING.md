## Releasing (automated)

This repository uses **release-plz** to automate:
- version bumps
- changelog updates
- crates.io publishing
- GitHub releases

### How the flow works

- **On every push to `main`** the workflow runs `release-plz release-pr`.
- If there are releasable changes, release-plz opens a **Release PR** updating:
  - crate versions (per-crate, based on each crate’s `Cargo.toml`)
  - the root [`CHANGELOG.md`](../CHANGELOG.md)
- The workflow automatically applies the label **`release-plz`** to that PR.
- **After the Release PR (labeled `release-plz`) is merged**, the workflow runs `release-plz release` which:
  - publishes crates to crates.io (only crates that are publishable and have a new version)
  - creates GitHub Releases

Workflows:
- Root workspace: [`.github/workflows/release-plz.yml`](../.github/workflows/release-plz.yml)

### What gets published

Publishing is controlled by Cargo manifests:
- crates with `publish = false` are **never published**
- crates without `publish = false` are **publishable** (subject to crates.io rules)

This repo is configured so that:
- `apps/**` and `examples/**` are **not** publishable (we set `publish = false`)
- `libs/**` and `modules/**` are publishable as intended

### Versioning policy (as implemented)

- **Framework (`libs/modkit-*`)**: share a single version via `version.workspace = true` and the root workspace version (`Cargo.toml` → `[workspace.package] version`).
- **System SDKs (`libs/system-sdks/**`)**: each crate has its own explicit version.
- **Modules (`modules/**`)**: each module and each `*-sdk` has its own explicit version.

### Dependency ordering

release-plz publishes crates in the correct order for intra-workspace dependencies.

### Safety checks

Before publishing, the root release workflow runs:

```bash
cargo test --workspace --no-fail-fast --exclude cf-modkit-macros-tests --exclude cf-modkit-db-macros
```

### Emergency / manual release

If you need a hotfix / manual release, prefer triggering the GitHub Actions workflow instead of publishing locally:

1. Ensure versions are bumped (edit the relevant `Cargo.toml` version fields) and the change is on the target branch.
2. Go to GitHub → **Actions** → **Release (release-plz)** → **Run workflow**.
3. Select `mode = release` (publishes crates + creates GitHub Releases).

Note: the workflow already runs tests before publishing. Running tests locally is optional and just gives faster feedback.

```bash
cargo test --workspace --no-fail-fast --exclude cf-modkit-macros-tests --exclude cf-modkit-db-macros
```

Fallback if CI is unavailable: publish locally from a clean checkout (you must have `CARGO_REGISTRY_TOKEN` set):

```bash
export CARGO_REGISTRY_TOKEN=***   # your crates.io token
cargo publish -p <crate_name>
```

### Notes for the very first publish (bootstrap)

- **crates.io rate limiting (HTTP 429)** can happen when publishing many crates for the first time.
  If the publish job fails with 429, just re-run the same workflow after the timestamp shown in the error.
  The process is idempotent: already-published crates will be skipped on retry.

