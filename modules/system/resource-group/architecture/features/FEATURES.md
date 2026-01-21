# Features: Hyperspot Resource Group

**Status Overview**: 1 features total (0 implemented, 0 in development, 0 design ready, 1 in design, 0 not started)

**Meaning**:
- ⏳ NOT_STARTED
- 📝 IN_DESIGN
- 📘 DESIGN_READY
- 🔄 IN_DEVELOPMENT
- ✅ IMPLEMENTED

**Status Summary**:
- 📝 IN_DESIGN: 1

---

### 1. [fdd-hyperspot-feature-resource-group](feature-resource-group/) 📝 CRITICAL

- **Purpose**: Core resource organization and hierarchy management
- **Status**: IN_DESIGN
- **Depends On**: None
- **Blocks**: None
- **Phases**:
  - `ph-1`: ⏳ NOT_STARTED — Basic CRUD and Type Management
  - `ph-2`: ⏳ NOT_STARTED — Hierarchy Moves and Constraints
  - `ph-3`: ⏳ NOT_STARTED — References and Advanced Features
- **Scope**:
  - Type management (creation, validation)
  - Entity management (create, update, delete)
  - Hierarchy operations (move subtree, cycle detection)
  - Closure table maintenance
- **Requirements Covered**: `fdd-hyperspot-req-resource-org`, `fdd-hyperspot-nfr-performance`
- **Principles Covered**: `fdd-hyperspot-principle-efficient-reads`
- **Constraints Affected**: `fdd-hyperspot-constraint-db-independence`
