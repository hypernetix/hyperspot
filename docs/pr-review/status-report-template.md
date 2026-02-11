# PR Status Report Template

This file documents the expected format for generated status reports.
`pr.py status` generates reports in `.prs/{ID}/status.md` following this structure.

---

## Format

```markdown
# PR #{number}: {title}

| Field | Value |
|---|---|
| **URL** | {url} |
| **Author** | @{author} |
| **State** | {state_icon} {state} |
| **Created** | {created_at} |
| **Updated** | {updated_at} |
| **CI Status** | {ci_text} |
| **Merge Status** | {merge_state} |
| **Review Decision** | {review_decision} |
| **Code comments** | {N} unresolved / {N} resolved / {N} suspicious |
| **PR comments** | {N} unreplied / {N} replied / {N} suspicious |

## PR Description

> {pr_body, truncated at 2000 chars}

## Reviewers

| Reviewer | Status |
|---|---|
| @{login} | {icon} {status} |

## Action Items

1. ❌ **@{reviewer}** requested changes — needs re-review
2. ⏳ Awaiting review from **@{reviewer}**
3. 💬 Reply to **@{commenter}** on `{path}:{line}`
4. 💬 Address **@{commenter}**'s PR comment ({date})

## Unreplied Code Comments

### [{path}:{line}]({url})

- **Severity**: TBD
- **Awaiting reply from**: @{pr_author}
- **Participants**: @{p1}, @{p2}

{code context + full conversation — see _format_conversation()}

---

## Unreplied PR Comments

### [PR Comment]({url})

- **Severity**: TBD
- **Asked by**: @{commenter}
- **Awaiting reply from**: @{pr_author}
- **Date**: {date}

> {comment body, quoted}

---

## Suspicious Resolutions

None.

{Populated by the LLM during the audit step. Entries moved here from
 Resolved Comments when the concern was not actually fixed in code.}

## Resolved Comments (Audit Required)

### [{path}:{line}]({url})

- **Status**: ✅ RESOLVED — AI VERIFIED
- **Participants**: @{p1}, @{p2}

{code context + full conversation}

---
```

## Notes

- `{placeholders}` are filled by `pr.py status` from fetched data.
- Loops expand for each comment/thread.
- `Severity: TBD` is a placeholder for the LLM to fill in during the `/pr-status` workflow.
- After LLM assigns severities, `pr.py reorder` re-sorts sections by severity (CRITICAL > HIGH > MEDIUM > LOW > TBD).
- Resolved comments default to `✅ RESOLVED — AI VERIFIED`. The LLM changes to `⚠️ RESOLVED — SUSPICIOUS` and moves entries to the Suspicious Resolutions section if the concern was not actually addressed.
- PR comments are split into unreplied vs replied using a quote-match heuristic (`_detect_pr_replies`). Only unreplied comments appear as action items.
- Conversations are rendered as full chat threads with code context (`diff_hunk`) where available.
