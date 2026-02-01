# Technical Design — Todo App

## 1. Architecture Overview

### 1.1 Architectural Vision

The Todo App follows a clean architecture approach with clear separation between presentation, business logic, and data layers. The frontend is built as a single-page application (SPA) communicating with a RESTful backend API.

The system prioritizes offline-first capabilities using local storage with background synchronization. This ensures users can work without interruption regardless of network conditions.

Event-driven architecture is employed for real-time updates and cross-device synchronization via WebSockets.

### 1.2 Architecture Drivers

#### Functional Drivers

| Requirement | Design Response |
|-------------|-----------------|
| `fdd-todo-app-req-create-task` | REST API endpoint POST /tasks with validation |
| `fdd-todo-app-req-complete-task` | PATCH /tasks/:id with status toggle |
| `fdd-todo-app-req-filter-tasks` | Query parameters on GET /tasks |
| `fdd-todo-app-req-offline-support` | IndexedDB local storage with sync queue |

#### NFR Allocation

This table maps non-functional requirements from PRD to specific design/architecture responses, demonstrating how quality attributes are realized.

| NFR ID | NFR Summary | Allocated To | Design Response | Verification Approach |
|--------|-------------|--------------|-----------------|----------------------|
| `fdd-todo-app-req-response-time` | UI interactions <200ms p95 | TaskService + IndexedDB | Local-first architecture: all reads from IndexedDB (sub-10ms), writes optimistic with background sync | Performance benchmarks measure p95 latency |
| `fdd-todo-app-req-data-persistence` | Local persist <50ms, cloud sync <5s | SyncService + IndexedDB + REST API | IndexedDB for immediate local persistence; background WebSocket sync with retry queue | Integration tests verify timing + recovery scenarios |

### 1.3 Architecture Layers

| Layer | Responsibility | Technology |
|-------|---------------|------------|
| Presentation | User interface, user input handling | React, TailwindCSS |
| Application | Use case orchestration, DTOs | TypeScript services |
| Domain | Business logic, entities, validation | TypeScript classes |
| Infrastructure | Data persistence, external APIs | PostgreSQL, Redis |

## 2. Principles & Constraints

### 2.1 Design Principles

#### Offline-First

- [ ] `p2` - **ID**: `fdd-todo-app-design-principle-offline-first`

**ADRs**: `fdd-todo-app-adr-local-storage`

All operations must work without network connectivity. Data is persisted locally first, then synchronized to the server when connection is available.

#### Optimistic Updates

- [ ] `p2` - **ID**: `fdd-todo-app-design-principle-optimistic-updates`

**ADRs**: `fdd-todo-app-adr-optimistic-ui`

UI updates immediately on user action without waiting for server confirmation. Rollback occurs only on server rejection.

### 2.2 Constraints

#### Browser Compatibility

- [ ] `p2` - **ID**: `fdd-todo-app-design-constraint-browser-compat`

**ADRs**: `fdd-todo-app-adr-browser-support`

Application must support latest 2 versions of Chrome, Firefox, Safari, and Edge.

## 3. Technical Architecture

### 3.1 Domain Model

**Technology**: TypeScript

**Location**: [src/domain/entities](../src/domain/entities)

**Core Entities**:
- [Task](../src/domain/entities/task.ts) - Core task entity with title, status, priority
- [Category](../src/domain/entities/category.ts) - Task grouping entity
- [User](../src/domain/entities/user.ts) - User account entity

**Relationships**:
- Task → Category: Many-to-one (task belongs to optional category)
- Task → User: Many-to-one (task belongs to user)
- Category → User: Many-to-one (category belongs to user)

### 3.2 Component Model

```mermaid
flowchart LR
    subgraph Frontend["React Frontend"]
        App[App]
        TaskList[TaskList]
        TaskForm[TaskForm]
        FilterBar[FilterBar]
        SyncIndicator[SyncIndicator]
    end

    subgraph Services["Services"]
        TaskService[TaskService]
        SyncService[SyncService]
    end

    subgraph Storage["Storage"]
        IDB[(IndexedDB)]
        API[REST API]
    end

    App --> TaskList
    App --> TaskForm
    App --> FilterBar
    App --> SyncIndicator

    TaskList --> TaskService
    TaskForm --> TaskService
    FilterBar --> TaskService
    SyncIndicator --> SyncService

    TaskService --> IDB
    TaskService --> API
    SyncService --> IDB
    SyncService --> API
```

**Components**:
- **TaskList**: Displays filtered list of tasks with pagination
- **TaskForm**: Form for creating and editing tasks
- **FilterBar**: Controls for filtering and sorting tasks
- **SyncIndicator**: Shows synchronization status

**Interactions**:
- TaskForm → TaskService: Submits task data for creation/update
- FilterBar → TaskList: Passes filter criteria for rendering

### 3.3 API Contracts

**Technology**: REST/OpenAPI

**Location**: [api/openapi.yaml](../api/openapi.yaml)

**Endpoints Overview**:

| Method | Path | Description | Stability |
|--------|------|-------------|-----------|
| `GET` | `/tasks` | List tasks with optional filters | stable |
| `POST` | `/tasks` | Create a new task | stable |
| `GET` | `/tasks/:id` | Get task by ID | stable |
| `PATCH` | `/tasks/:id` | Update task fields | stable |
| `DELETE` | `/tasks/:id` | Delete a task | stable |

### 3.4 External Interfaces & Protocols

#### WebSocket Sync Protocol

- [x] `p1` - **ID**: `fdd-todo-app-design-interface-websocket`

**Type**: Protocol (WebSocket + JSON)
**Direction**: bidirectional
**Specification**: Custom sync protocol over WebSocket; messages follow format: `{ type: "sync" | "update" | "delete", payload: Task }`
**Data Format**: JSON (follows Task model from `fdd-todo-app-interface-task-model`)
**Compatibility**: Protocol version negotiated on connection; supports fallback to HTTP polling
**References**: Links to PRD § Public Library Interfaces (`fdd-todo-app-contract-sync`)

#### IndexedDB Storage Schema

- [x] `p1` - **ID**: `fdd-todo-app-design-interface-indexeddb`

**Type**: Data Format (IndexedDB schema)
**Direction**: internal (library storage)
**Specification**: Dexie.js schema with indexes on userId, status, categoryId, dueDate
**Data Format**: Task objects stored as-is with additional metadata (syncState, lastModified)
**Compatibility**: Schema migrations handled by Dexie.js version upgrade hooks

### 3.5 Interactions & Sequences

```mermaid
sequenceDiagram
    actor User
    participant UI as React UI
    participant TS as TaskService
    participant IDB as IndexedDB
    participant API as REST API
    participant DB as PostgreSQL

    User->>UI: Click "Add Task"
    UI->>UI: Show TaskForm
    User->>UI: Enter task data & Save
    UI->>TS: createTask(data)
    TS->>IDB: store(task)
    IDB-->>TS: stored
    TS-->>UI: task (optimistic)
    UI-->>User: Show new task

    TS->>API: POST /tasks
    API->>DB: INSERT task
    DB-->>API: created
    API-->>TS: 201 Created
    TS->>IDB: markSynced(task.id)
```

**Use cases**: `fdd-todo-app-req-uc-create-task`

**Actors**: `fdd-todo-app-actor-user`, `fdd-todo-app-actor-sync-service`

### 3.6 Database schemas & tables

#### Table tasks

**ID**: `fdd-todo-app-db-table-tasks`

**Schema**

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| user_id | UUID | Foreign key to users |
| title | VARCHAR(255) | Task title |
| description | TEXT | Optional description |
| status | ENUM | 'active', 'completed' |
| priority | ENUM | 'low', 'medium', 'high' |
| category_id | UUID | Optional foreign key |
| due_date | TIMESTAMP | Optional due date |
| created_at | TIMESTAMP | Creation timestamp |
| updated_at | TIMESTAMP | Last update timestamp |

**PK**: id

**Constraints**: title NOT NULL, status NOT NULL DEFAULT 'active'

**Additional info**: Indexed on user_id, status, due_date

**Example**

| id | user_id | title | status | priority |
|----|---------|-------|--------|----------|
| abc-123 | user-1 | Buy groceries | active | medium |

### 3.7 Topology (optional)

**ID**: `fdd-todo-app-topology-cloud`

- Frontend: Static files on CDN
- Backend: Containerized Node.js on Kubernetes
- Database: Managed PostgreSQL
- Cache: Redis cluster

### 3.8 Tech stack (optional)

**ID**: `fdd-todo-app-tech-stack`

- Frontend: React 18, TypeScript, TailwindCSS, Zustand
- Backend: Node.js, Express, TypeScript
- Database: PostgreSQL 15, Redis 7
- Infrastructure: Docker, Kubernetes, GitHub Actions

## 4. Additional Context

**ID**: `fdd-todo-app-design-context-decisions`

The choice of React over other frameworks was driven by team expertise and ecosystem maturity. PostgreSQL was selected for its reliability and JSON support for flexible task metadata.

## 5. Traceability

- **PRD**: [PRD.md](./PRD.md)
- **ADRs**: [ADR/](./ADR/)
- **Features**: [features/](./features/)
