
<b>Pattern 1: Avoid direct stdout/stderr printing (println!/eprintln!/dbg!/print!) in library and module code; use structured logging (e.g., tracing) or return the data to the caller, and only allow direct printing in clearly-scoped CLI/one-shot “dump output then exit” paths with explicit justification.
</b>

Example code before:
```
pub fn do_work() -> Result<()> {
    println!("starting...");
    // ...
    eprintln!("failed: {err}");
    Ok(())
}
```

Example code after:
```
pub fn do_work() -> Result<()> {
    tracing::info!("starting...");
    // ...
    tracing::warn!(error = %err, "work failed");
    Ok(())
}

// If a CLI must print machine-readable output:
pub fn render_config_yaml() -> Result<String> {
    Ok(serde_yaml::to_string(&config)?)
}
```

<details><summary>Examples for relevant past discussions:</summary>

- https://github.com/cyberfabric/cyberfabric-core/pull/475#discussion_r2771740365
- https://github.com/cyberfabric/cyberfabric-core/pull/478#discussion_r2772918171
</details>


___

<b>Pattern 2: In CI/workflows, avoid time-wasting patterns (e.g., sleeping in jobs) and ensure notification logic is explicitly rate-limited/deduplicated so the same condition cannot spam reviewers or external channels on every cron run.
</b>

Example code before:
```
on:
  schedule:
    - cron: "*/10 * * * *"

steps:
  - run: |
      # notify every run if condition is true
      curl -XPOST "$WEBHOOK" -d "no reviewers!"
```

Example code after:
```
on:
  schedule:
    - cron: "*/10 * * * *"

steps:
  - run: |
      if already_notified "$PR_NUMBER"; then
        echo "skip duplicate notification"
        exit 0
      fi
      notify "$PR_NUMBER"
      mark_notified "$PR_NUMBER"
```

<details><summary>Examples for relevant past discussions:</summary>

- https://github.com/cyberfabric/cyberfabric-core/pull/469#discussion_r2771252165
</details>


___

<b>Pattern 3: When introducing policy/linting/contract checks, keep exceptions narrowly scoped and correct (e.g., tests, proc-macros), and avoid “temporarily disabling” enforcement without an explicit replacement plan (tracking issue/task) to prevent permanent erosion of architectural rules.
</b>

Example code before:
```
# Cargo.toml
[workspace]
members = [
  # "lints/contract_layer/no_serde_in_contract", # temporarily disabled
]
```

Example code after:
```
# Cargo.toml
[workspace]
members = [
  "lints/contract_layer/no_serde_in_contract",
]

# If disabling is unavoidable, add an explicit TODO + issue link
# and keep CI failing/alerting elsewhere.
# TODO(#1234): Re-enable once rustc/nightly issue is fixed.
```

<details><summary>Examples for relevant past discussions:</summary>

- https://github.com/cyberfabric/cyberfabric-core/pull/469#discussion_r2771269040
- https://github.com/cyberfabric/cyberfabric-core/pull/469#discussion_r2771277461
</details>


___

<b>Pattern 4: Keep docs and templates single-sourced and consistent: avoid duplicating section outlines or “restating” information across multiple pages when it can drift out of sync; prefer a canonical README (or one primary template) and link to it from other documents.
</b>

Example code before:
```
# README.md
## Document Structure
- PRD has sections: A, B, C
- DESIGN has sections: D, E, F

# PRD.md
## Document Structure
- PRD has sections: A, B, C   # duplicated, can diverge
```

Example code after:
```
# README.md
## Document Structure
See: ./PRD.md, ./DESIGN.md, and ./ADR/ for canonical templates.

# PRD.md
<!-- Keep this file canonical; other docs link here instead of copying -->
```

<details><summary>Examples for relevant past discussions:</summary>

- https://github.com/cyberfabric/cyberfabric-core/pull/400#discussion_r2752999605
- https://github.com/cyberfabric/cyberfabric-core/pull/400#discussion_r2753165397
</details>


___
