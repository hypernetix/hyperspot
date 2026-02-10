# PRD - Todo App

## 1. Overview

### 1.1 Purpose

A web application for managing personal tasks with support for categories, priorities, and filtering.

### 1.2 Background / Problem Statement

Todo App is a simple and intuitive task management application. Users can create, edit, and delete tasks, mark them as completed, and organize them by categories and priorities.

The application is designed for individual use with cross-device synchronization. The main focus is on minimalist interface and fast performance.

### 1.3 Goals (Business Outcomes)

- Task creation time < 3 seconds
- 95% of users successfully complete onboarding
- NPS > 40

### 1.4 Glossary

| Term | Definition |
|------|------------|
| Task | A single actionable item with title, optional description, due date, priority, and category |
| Category | A user-defined grouping for tasks |

## 2. Actors

> **Note**: Stakeholder needs are managed at the project/task level by the steering committee and are not duplicated in module specs. Focus on **actors** (users, systems) that directly interact with this module.

### 2.1 Human Actors

#### User

**ID**: `cpt-examples-todo-app-actor-user`
**Role**: Primary user who creates, manages, and completes tasks in the application.
**Needs**: Simple task management, cross-device access, quick task entry.

### 2.2 System Actors

#### Sync Service

**ID**: `cpt-examples-todo-app-actor-sync-service`
**Role**: Background service that synchronizes tasks across user devices in real-time.

#### Notification Service

**ID**: `cpt-examples-todo-app-actor-notification-service`
**Role**: Sends reminders and notifications to users about upcoming or overdue tasks.

## 3. Operational Concept & Environment

> **Note**: Project-wide runtime, OS, architecture, lifecycle policy, and module integration patterns (Rust native + auto-generated gRPC/REST) are defined in root [PRD.md](../../PRD.md). Only document module-specific deviations or additional constraints here. **If this module has no special environment constraints, delete this entire section.**

### 3.1 Module-Specific Environment Constraints

- Requires IndexedDB support for offline functionality (browser-only constraint)
- WebSocket support required for real-time sync (fallback to polling if unavailable)

## 4. Scope

### 4.1 In Scope

- CRUD operations for tasks
- Categorization and prioritization
- Filtering and search
- Cross-device synchronization
- Offline support

### 4.2 Out of Scope

- Team collaboration features (future phase)
- Calendar integration
- File attachments

## 5. Functional Requirements

> **Testing strategy**: Unless otherwise specified, all requirements are verified via automated tests (unit, integration, e2e) targeting 95% code coverage. Only document verification method explicitly for non-test approaches (analysis, inspection, demonstration) or special testing needs.

### 5.1 Core Task Management

#### Create Task

- [x] `p1` - **ID**: `cpt-examples-todo-app-fr-create-task`

The system **MUST** allow users to create a new task with a title, optional description, due date, priority level, and category.

**Rationale**: Core functionality — users need to capture tasks quickly.
**Actors**: `cpt-examples-todo-app-actor-user`

#### Complete Task

- [x] `p1` - **ID**: `cpt-examples-todo-app-fr-complete-task`

The system **MUST** allow users to mark a task as completed or revert it to incomplete status.

**Rationale**: Essential for task lifecycle management.
**Actors**: `cpt-examples-todo-app-actor-user`

#### Delete Task

- [x] `p1` - **ID**: `cpt-examples-todo-app-fr-delete-task`

The system **MUST** allow users to delete a task permanently.

**Rationale**: Users need to remove irrelevant or mistaken tasks.
**Actors**: `cpt-examples-todo-app-actor-user`

### 5.2 Organization

#### Filter Tasks

- [x] `p2` - **ID**: `cpt-examples-todo-app-fr-filter-tasks`

The system **MUST** allow users to filter tasks by status (all, active, completed), category, and priority.

**Rationale**: Helps users focus on relevant tasks.
**Actors**: `cpt-examples-todo-app-actor-user`

## 6. Non-Functional Requirements

> **Default guidelines**: Project-wide NFR baselines (performance, security, reliability, scalability) are defined in root [PRD.md](../../PRD.md) and [docs/guidelines/](../../guidelines/). Only document module-specific NFRs here — either **exclusions** from defaults or **standalone** requirements unique to this module.
>
> **Testing strategy**: NFRs are verified via automated benchmarks, security scans, and monitoring unless otherwise specified.

### 6.1 Module-Specific NFRs

#### Response Time

- [x] `p1` - **ID**: `cpt-examples-todo-app-nfr-response-time`

All user interactions **MUST** complete within 200ms at p95 under normal load (stricter than project default of 500ms).

**Threshold**: 200ms p95 latency for UI interactions
**Rationale**: Todo app is a productivity tool where perceived speed directly impacts user satisfaction; willing to accept increased complexity (local-first architecture) to achieve this
**Architecture Allocation**: See DESIGN.md § NFR Allocation for how this is realized

#### Data Persistence

- [x] `p1` - **ID**: `cpt-examples-todo-app-nfr-data-persistence`

User data **MUST** be persisted locally immediately and synced to cloud storage within 5 seconds of any change when online.

**Threshold**: Local persistence: <50ms; cloud sync: <5s when online
**Rationale**: Module-specific requirement (project default doesn't cover offline-first + sync pattern)
**Architecture Allocation**: See DESIGN.md § NFR Allocation for how this is realized

#### Offline Support

- [x] `p1` - **ID**: `cpt-examples-todo-app-nfr-offline-support`

The system **MUST** support offline mode where task creation, completion, filtering, and deletion operate without network connectivity.

**Threshold**: Offline operations succeed with no errors; synchronization begins automatically when connectivity resumes
**Rationale**: Offline-first is a core product requirement for intermittent connectivity scenarios
**Architecture Allocation**: See DESIGN.md § Architecture Drivers and § NFR Allocation for how this is realized

## 7. Public Library Interfaces

### 7.1 Public API Surface

#### REST API

- [x] `p1` - **ID**: `cpt-examples-todo-app-interface-rest-api`

**Type**: REST API (OpenAPI 3.0)
**Stability**: stable
**Description**: HTTP REST API for task management (CRUD operations, filtering, search)
**Breaking Change Policy**: Major version bump required for endpoint removal or request/response schema incompatible changes

#### Task Data Model

- [x] `p1` - **ID**: `cpt-examples-todo-app-interface-task-model`

**Type**: JSON Schema
**Stability**: stable
**Description**: Task entity structure exposed via API and stored in IndexedDB
**Breaking Change Policy**: Field removals require major version; new optional fields are minor changes

### 7.2 External Integration Contracts

#### Sync Service Contract

- [x] `p1` - **ID**: `cpt-examples-todo-app-contract-sync`

**Direction**: required from client (external sync backend)
**Protocol/Format**: WebSocket + JSON for real-time task updates
**Compatibility**: Protocol versioned independently; supports graceful degradation to polling

## 8. Use Cases

#### Create a New Task

- [ ] `p1` - **ID**: `cpt-examples-todo-app-usecase-create-task`

**Actor**: `cpt-examples-todo-app-actor-user`

**Preconditions**:
- User is authenticated and on the main task list view

**Main Flow**:
1. User clicks the "Add Task" button
2. System displays the task creation form
3. User enters task title (required), description, due date, priority, and category
4. User clicks "Save"
5. System validates input and creates the task
6. System displays the updated task list with the new task

**Postconditions**:
- New task is persisted and visible in the task list

**Alternative Flows**:
- **Validation fails**: System displays error messages, user corrects input

## 9. Acceptance Criteria

- [x] User can create tasks with all required fields
- [x] User can mark tasks as complete/incomplete
- [x] User can delete tasks
- [x] User can filter tasks by status, category, priority
- [ ] Offline mode works without errors
- [ ] Sync completes within 5 seconds

## 10. Dependencies

| Dependency | Description | Criticality |
|------------|-------------|-------------|
| Auth Service | User authentication | p1 |
| Cloud Storage | Task persistence | p1 |

## 11. Assumptions

- Users have modern browsers (Chrome, Firefox, Safari, Edge — latest 2 versions)
- Users have intermittent internet connectivity

## 12. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Sync conflicts | Data loss | Implement conflict resolution with last-write-wins + user notification |
| Offline storage limits | Cannot add tasks | Implement storage quota warnings |

## 13. Open Questions

- How long should completed tasks be retained before archival?
- Should we support task sharing between users in future phases?
- What is the maximum number of categories per user?
