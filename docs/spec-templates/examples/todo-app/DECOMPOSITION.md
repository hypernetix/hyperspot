# DECOMPOSITION — Todo App

## 1. Decomposition Status

- [ ] `p1` - **ID**: `cpt-examples-todo-app-status-overall`

**Overall Progress**: 67% complete (2 of 3 features completed)

**Status**: In Progress

**Notes**: Core task management and organization features are complete. Real-time synchronization is pending implementation.

---

## 2. Feature Breakdown

### Feature 1: Task Management Core

- [x] `p1` - **ID**: `cpt-examples-todo-app-feature-core`

**Feature**: [features/0001-cpt-todo-app-feature-core.md](./features/0001-cpt-todo-app-feature-core.md)

**Purpose**: Implement core CRUD operations for tasks including creation, reading, updating, and deletion.

**Design Coverage**:
- Component: `cpt-examples-todo-app-component-react-ui` (task UI rendering)
- Component: `cpt-examples-todo-app-component-task-service` (CRUD orchestration)
- Component: `cpt-examples-todo-app-component-indexeddb` (local persistence)
- Component: `cpt-examples-todo-app-component-rest-api` (server-side CRUD)
- Component: `cpt-examples-todo-app-component-postgresql` (persistent storage)
- Database: `cpt-examples-todo-app-db-table-tasks` (task persistence)
- Principle: `cpt-examples-todo-app-principle-offline-first` (IndexedDB local storage)
- Sequence: `cpt-examples-todo-app-seq-create-task-v1` (optimistic create flow)

**Dependencies**: None

**Rationale**: Foundation for all other features — tasks must exist before they can be organized or synced.

**Status**: Completed

---

### Feature 2: Task Organization & Logic

- [x] `p2` - **ID**: `cpt-examples-todo-app-feature-logic`

**Feature**: [features/0002-cpt-todo-app-feature-logic.md](./features/0002-cpt-todo-app-feature-logic.md)

**Purpose**: Implement filtering, sorting, and display logic for task lists.

**Design Coverage**:
- Principle: `cpt-examples-todo-app-principle-optimistic-updates` (immediate UI feedback)

**Dependencies**: `cpt-examples-todo-app-feature-core` (requires tasks to exist)

**Rationale**: Task organization helps users manage growing task lists efficiently. Builds on core CRUD to provide filtering by status, category, and priority.

**Status**: Completed

---

### Feature 3: Real-time Synchronization

- [ ] `p2` - **ID**: `cpt-examples-todo-app-feature-sync`

**Feature**: [features/0003-cpt-todo-app-feature-sync.md](./features/0003-cpt-todo-app-feature-sync.md)

**Purpose**: Implement cross-device synchronization via WebSocket with fallback to polling.

**Design Coverage**:
- Component: `cpt-examples-todo-app-component-sync-service` (sync orchestration)
- Component: `cpt-examples-todo-app-component-websocket-server` (real-time notifications)
- Interface: `cpt-examples-todo-app-interface-websocket` (WebSocket sync protocol)
- Constraint: `cpt-examples-todo-app-constraint-browser-compat` (WebSocket availability check)
- Principle: `cpt-examples-todo-app-principle-offline-first` (sync queue when offline)

**Dependencies**: `cpt-examples-todo-app-feature-core` (syncs task data created/modified by core operations)

**Rationale**: Enables cross-device experience essential for users who work across multiple devices. Can be built after core functionality is stable. Implements sync queue for offline changes that get pushed when connectivity resumes.

**Status**: Pending

---

## 3. Feature Dependencies

```text
cpt-examples-todo-app-feature-core
    ↓
    ├─→ cpt-examples-todo-app-feature-logic
    └─→ cpt-examples-todo-app-feature-sync
```

**Dependency Rationale**:

- `cpt-examples-todo-app-feature-logic` requires `cpt-examples-todo-app-feature-core`: Cannot organize/filter tasks that don't exist
- `cpt-examples-todo-app-feature-sync` requires `cpt-examples-todo-app-feature-core`: Cannot sync tasks that haven't been created
- `cpt-examples-todo-app-feature-logic` and `cpt-examples-todo-app-feature-sync` are independent of each other and can be developed in parallel
