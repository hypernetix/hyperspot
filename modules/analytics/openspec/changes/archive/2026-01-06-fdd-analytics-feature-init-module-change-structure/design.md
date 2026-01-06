# Design Reference

**MUST READ**: [Feature DESIGN.md](../../architecture/features/feature-init-module/DESIGN.md)

**Agent Instruction**: This change implements requirements from Feature DESIGN.md. Read the full design before implementation.

---

## Key Design Sections

- **Section A**: What Init IS vs IS NOT - Critical scope constraints
- **Section E**: Technical Details - Complete file structure and implementation patterns
- **Section F**: Requirements - Formal requirement with testing scenarios
- **Section G**: Implementation Plan - This change

---

## Implementation Notes

This is a **structural change** with minimal complexity. No custom design decisions required - follow Feature DESIGN.md Section E exactly.

**Critical Rules**:
- NO business logic
- NO GTS types
- NO API method definitions
- SDK must be transport-agnostic (no serde)
- All layer folders must be empty

**Reference Patterns**:
- See `@/guidelines/NEW_MODULE.md` for module structure
- See `@/docs/MODKIT_UNIFIED_SYSTEM.md` for ModKit integration
- See `@/guidelines/hyperspot-fdd-adapter/INIT_MODULE_PATTERNS.md` for SDK pattern
