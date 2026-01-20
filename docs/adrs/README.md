# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for the HyperSpot platform, following the [MADR 4.0.0](https://adr.github.io/madr/) format.

## What is an ADR?

An Architecture Decision Record (ADR) captures an architecturally significant decision along with its context and consequences. ADRs help teams:

- Document the reasoning behind important decisions
- Provide historical context for future maintainers
- Enable asynchronous review and approval across distributed teams
- Create a searchable knowledge base of architectural choices

## MADR Format

We use Markdown Architectural Decision Records (MADR) version 4.0.0. For detailed information:

- [MADR Documentation](https://adr.github.io/madr/)
- [General ADR Information](https://adr.github.io/)
- [Templates](./templates/) - Local copy of MADR templates

## Directory Structure

ADRs are organized by scope and category:

### Platform-Wide ADRs (`docs/adrs/`)

ADRs that affect multiple areas of the platform or are cross-cutting concerns live in this directory, organized by category:

```text
docs/adrs/
├── README.md                    # This file
├── templates/                   # MADR templates
│   ├── adr-template.md          # Full template with explanations
│   ├── adr-template-minimal.md  # Minimal template
│   └── ...
├── 0001-*.md                    # Root-level ADRs (cross-cutting)
├── api-design/                  # API design decisions
│   └── 0001-*.md
├── authorization/               # Authorization & access control
│   └── 0001-*.md
└── storage/                     # Storage & database decisions
    └── 0001-*.md
```

ADRs can live directly in the `docs/adrs/` root if they don't fit a specific category or affect multiple areas of the platform.

### Module-Specific ADRs

ADRs that are specific to a single module should live in the module's own `docs/adrs/` directory:

```text
modules/<module-name>/docs/adrs/
└── 0001-*.md
```

This keeps module-specific decisions close to the code they affect and allows module maintainers to manage their own architectural decisions.

Each category/module maintains its own sequential numbering (0001, 0002, etc.).

### File Naming Convention

```text
NNNN-short-title-with-dashes.md
```

- `NNNN`: Sequential number within the category (0001, 0002, ...)
- `short-title`: Lowercase, dash-separated description
- Example: `0001-use-jwt-for-authentication.md`

## Creating a New ADR

1. **Copy the template**: `cp templates/adr-template.md <category>/NNNN-short-title.md`
2. **Fill in the template** with your decision (use `accepted` status)
3. **Open a PR** — discussion and approval happen in PR comments

## Approval Process

Discussion and approval happen in the Pull Request. Merge when key contributors approve.

### ADR Status Lifecycle

Track the status in the ADR's YAML frontmatter:

| Status | Description |
|--------|-------------|
| `accepted` | Approved by key contributors and committed to the repository |
| `deprecated` | No longer applicable |
| `superseded by ADR-NNNN` | Replaced by a newer decision |

## ADR Categories

### `api-design/`

Decisions about REST API conventions, versioning, error handling, and protocol choices.

### `authorization/`

Decisions about authentication, authorization, access control, multi-tenancy security, and identity management.

### `storage/`

Decisions about databases, caching, data models, and persistence strategies.

## CI Integration

ADRs are automatically linted on push and pull requests via GitHub Actions. The workflow uses [markdownlint](https://github.com/DavidAnson/markdownlint) to check for consistent formatting.

The linter checks ADRs in both:

- Platform-wide ADRs: `docs/adrs/**/*.md`
- Module-specific ADRs: `modules/**/docs/adrs/**/*.md`

Configuration: [`.markdownlint.yml`](./.markdownlint.yml)

To lint locally:

```bash
# Install markdownlint-cli2
npm install -g markdownlint-cli2

# Lint platform-wide ADRs
markdownlint-cli2 "docs/adrs/**/*.md"

# Lint all ADRs (including modules)
markdownlint-cli2 "docs/adrs/**/*.md" "modules/**/docs/adrs/**/*.md"
```

## Tips for Writing Good ADRs

1. **Be concise but complete** - Include enough context for future readers
2. **Focus on the "why"** - The reasoning is more valuable than the decision itself
3. **List alternatives** - Show what options were considered and why they were rejected
4. **Document consequences** - Both positive and negative impacts
5. **Link related ADRs** - Reference other decisions that influenced or are influenced by this one
6. **Keep it updated** - Mark ADRs as deprecated or superseded when appropriate

## Questions?

For questions about the ADR process, reach out to the platform architecture team or open a discussion in the repository.
