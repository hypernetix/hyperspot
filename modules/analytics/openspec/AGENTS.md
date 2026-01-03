# OpenSpec SDLC for Analytics Module

**For AI Agents (Windsurf, Cascade, etc.)**

This module follows **OpenSpec** - a spec-driven development workflow where specifications are written BEFORE implementation.

---

## 🎯 Core Principle

**Specs first, code second**. All changes start with a proposal in `openspec/changes/`, get approved, then implemented.

---

## 📋 Prerequisites

Read before starting:
- `@/AGENTS.md` - Root project instructions  
- `@/guidelines/NEW_MODULE.md` - Module implementation patterns
- OpenSpec workflows in `../.windsurf/workflows/openspec-*.md`
- **GTS Specification**: https://github.com/GlobalTypeSystem/gts-spec - Type system for data schemas

---

## 📚 GTS Quick Reference

**GTS (Global Type System)** is our type system for all data schemas and plugin communication.

### Identifier Format

```
gts.<vendor>.<package>.<namespace>.<type>.v<MAJOR>[.<MINOR>]
```

**Examples**:
- Type: `gts.hyperspot.ax.widgets.chart.v1~` (ends with `~`)
- Instance: `gts.hyperspot.ax.dashboards.main.v1.0`

### Key Concepts

**1. Schema vs Instance**
- **Schema**: JSON document with `$schema` field (type definition)
- **Instance**: JSON document without `$schema` (data object)

**2. Schema Definition**
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "gts://gts.hyperspot.ax.datasource.v1~",
  "type": "object",
  "properties": {
    "id": {"type": "string"},
    "name": {"type": "string"},
    "schema": {"type": "object"}
  }
}
```

**3. Instance with Type Reference**
```json
{
  "id": "monitoring-ds-1",
  "type": "gts.hyperspot.ax.datasource.v1~",
  "name": "Server Metrics",
  "schema": { ... }
}
```

**4. Chained Identifiers** (inheritance/compatibility)
```
gts.hyperspot.ax.widget.v1~hyperspot.charts.bar.v1.0
```
Left segment = base type, right = derived/instance

### When to Use

- ✅ Defining datasource schemas
- ✅ Widget type definitions
- ✅ Plugin registration schemas
- ✅ API request/response types
- ✅ Event message formats

### Full Specification

When in doubt, read: https://github.com/GlobalTypeSystem/gts-spec

---

## 🛠️ OpenSpec CLI

### Installation

```bash
npm install -g @fission-ai/openspec@latest
```

### Essential Commands

```bash
# List active changes
openspec list

# View interactive dashboard
openspec view

# Show change details
openspec show <change-name>

# Validate change structure
openspec validate <change-name>

# Archive completed change (non-interactive)
openspec archive <change-name> --yes
```

---

## 🔄 OpenSpec SDLC Workflow

### Phase 1: Create Proposal

**Workflow**: Run `/openspec-proposal` in Windsurf

**What happens**:
- Agent creates `openspec/changes/<change-name>/`
- Generates `proposal.md` - WHY and WHAT
- Generates `tasks.md` - implementation checklist
- Creates spec deltas in `specs/` subfolder

**User action**: Review and approve the proposal

**Checkpoint**: ✅ Proposal approved

---

### Phase 2: Implement Tasks

**Workflow**: Run `/openspec-apply` in Windsurf

**What happens**:
- Agent reads proposal, tasks, and spec deltas
- Implements each task following specs
- Marks tasks as complete in `tasks.md`
- Writes tests
- Updates documentation

**References**:
- `@/guidelines/NEW_MODULE.md` for implementation patterns
- Use `SecurityCtx` in all handlers
- Keep domain layer transport-agnostic

**Checkpoint**: ✅ All tasks complete and tested

---

### Phase 3: Archive Change

**Workflow**: Run `/openspec-archive` in Windsurf

**What happens**:
- Agent merges spec deltas into `openspec/specs/`
- Moves change to `openspec/archive/`
- Updates source of truth

**User action**: Confirm archival

**Checkpoint**: ✅ Change archived, specs updated

---

## 📁 Directory Structure

```
openspec/
├── specs/              # Source of truth
│   └── api/
│       └── spec.md     # Current specs
├── changes/            # Active proposals
│   └── add-feature/
│       ├── proposal.md
│       ├── tasks.md
│       └── specs/
│           └── api/
│               └── spec.md  # Delta
└── archive/            # Completed changes
    └── add-feature/
```

---

## 🚨 Critical Rules

### ❌ NEVER:
1. **Skip proposal** - Always create proposal first
2. **Write code without approval** - Wait for user approval
3. **Forget SecurityCtx** - Required in all handlers
4. **Skip specs** - Specs define the contract

### ✅ ALWAYS:
1. **Use Windsurf workflows** - `/openspec-proposal`, `/openspec-apply`, `/openspec-archive`
2. **Get approval** - After proposal, before implementation
3. **Follow specs** - Spec deltas define what to build
4. **Reference guidelines** - `@/guidelines/NEW_MODULE.md`
5. **Use OpenSpec CLI** - `openspec list`, `openspec view`, `openspec validate`
6. **Verify GTS types and instances** - Check `gts/types/` for type schemas and `gts/instances/` for instance definitions before using any GTS identifiers or schemas

---

## 🔧 Windsurf Workflows

Available in `../.windsurf/workflows/`:
- `/openspec-proposal` - Create change proposal
- `/openspec-apply` - Implement tasks
- `/openspec-archive` - Archive completed change

### Typical Flow:

```
User: "Add dashboard widgets feature"
   ↓
Agent: /openspec-proposal
   ↓
Agent: Creates proposal + tasks + spec deltas
   ↓
User: Reviews and approves
   ↓
Agent: /openspec-apply
   ↓
Agent: Implements all tasks
   ↓
User: Confirms completion
   ↓
Agent: /openspec-archive
   ↓
Done: Specs updated, change archived
```

---

## 📊 Quick Checklist

Per change:
- [ ] Proposal created (`/openspec-proposal`)
- [ ] User approved proposal
- [ ] Tasks implemented (`/openspec-apply`)
- [ ] Tests pass
- [ ] Change archived (`/openspec-archive`)
- [ ] Specs updated in `openspec/specs/`

---

## ❓ When Unclear

Stop and ask. Reference workflows in `../.windsurf/workflows/openspec-*.md`.
