# PR Review with Cypilot

This project uses **Cypilot** for AI-powered PR reviews and status reports.

## Quick Start

Cyber Fabric has integrated Cypilot automation for PR review assistance. Use any supported agent
(Windsurf, Cursor, Claude, Copilot) — each has thin stubs that redirect to
the canonical workflows in `.cypilot/workflows/`.

You can use the following prompts in your IDE to review PRs or get status:

> cypilot list all open PRs
> cypilot preview PR 100
> /cypilot-pr-review 100
> cypilot preview ALL PRs
> cypilot get status for PR 300
> /cypilot-pr-status 300

See the .prs/{PR}/ folder for the review results.
```
review.md
status.md
meta.json
diff.patch
review_comments.json
review_threads.json
```

## Configuration

### Configure GitHub API token

The `pr.py` script uses the [GitHub CLI (`gh`)](https://cli.github.com/) to fetch PR data. You need `gh` installed and authenticated:

1. **Update .cypilot submodule**

   ```bash
   git submodule update --init --recursive
   ```

2. **Install `gh`**

   ```bash
   # macOS
   brew install gh

   # Linux (Debian/Ubuntu)
   sudo apt install gh

   # Other: https://github.com/cli/cli#installation
   ```

3. **Authenticate with GitHub**

   ```bash
   gh auth login
   ```

   Follow the interactive prompts. Choose:
   - **GitHub.com** (or your GitHub Enterprise host)
   - **HTTPS** as the preferred protocol
   - **Login with a web browser** (recommended) or paste a personal access token

   The token needs these scopes: `repo`, `read:org` (for private repos).

4. **Verify authentication**

   ```bash
   gh auth status
   ```

   You should see `Logged in to github.com as <your-username>`.

5. **(Optional) Use a personal access token directly**

   If you prefer not to use the browser flow:

   ```bash
   # Create a token at: https://github.com/settings/tokens
   # Required scopes: repo, read:org
   gh auth login --with-token < token.txt
   ```

   Or set the `GH_TOKEN` / `GITHUB_TOKEN` environment variable:

   ```bash
   export GH_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
   ```

### Workflow Reference

1. Fetch PR metadata using `.cypilot/skills/scripts/pr.py` CLI tool
2. Select the most appropriate review prompt (code, design, ADR, or PRD)
3. Analyze changes against the corresponding checklist
4. Write a structured review to `.prs/{ID}/review.md` or status report to `.prs/{ID}/status.md`

## Configuration

| Setting | Location | Purpose |
|---------|----------|---------|
| Review prompts, templates, data dir | `.cypilot-adapter/pr-review.json` | Main review configuration |
| PR exclude list | `.prs/config.yaml` → `exclude_prs` | Skip specific PRs during bulk operations |

## Templates

Report templates define the expected output format for reviews and status reports.

| Template | Canonical location | Docs copy |
|----------|-------------------|-----------|
| Code review | `.cypilot/templates/pr/code-review.md` | `docs/pr-review/code-review-template.md` |
| Status report | `.cypilot/templates/pr/status.md` | `docs/pr-review/status-report-template.md` |

The canonical templates live inside `.cypilot/templates/pr/`. If a project
overrides `templatesDir` in its `.cypilot-adapter`, those templates take
precedence. Otherwise, the embedded Cypilot templates are used.

## Review Prompts

Each review type has a dedicated prompt file and checklist:

| Review type | Prompt | Checklist |
|-------------|--------|-----------|
| Code Review | `.cypilot/prompts/pr/code-review.md` | `docs/checklists/CODE.md` |
| Design Review | `.cypilot/prompts/pr/design-review.md` | `docs/checklists/DESIGN.md` |
| ADR Review | `.cypilot/prompts/pr/adr-review.md` | `docs/checklists/ADR.md` |
| PRD Review | `.cypilot/prompts/pr/prd-review.md` | `docs/checklists/PRD.md` |
