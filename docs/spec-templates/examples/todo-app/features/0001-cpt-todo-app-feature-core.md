# Feature Context: Task Management Core

## 1. Feature Context

- [ ] `p2` - **ID**: `cpt-examples-todo-app-featurecontext-core`

### 1.1 Overview

Core CRUD operations for tasks including creation, reading, updating, and deletion of tasks.

### 1.2 Purpose

Provides the fundamental task management capabilities that all other features depend on.

### 1.3 Actors

- `cpt-examples-todo-app-actor-user` - Creates and manages tasks
- `cpt-examples-todo-app-actor-sync-service` - Synchronizes task changes
- `cpt-examples-todo-app-actor-notification-service` - Sends reminders based on task state changes

### 1.4 References

- Overall Design: [DESIGN.md](../DESIGN.md)
- PRD: [PRD.md](../PRD.md)
- Decomposition: `cpt-examples-todo-app-feature-core`
- Requirements: `cpt-examples-todo-app-fr-create-task`, `cpt-examples-todo-app-fr-complete-task`, `cpt-examples-todo-app-fr-delete-task`, `cpt-examples-todo-app-nfr-offline-support`, `cpt-examples-todo-app-nfr-data-persistence`, `cpt-examples-todo-app-interface-rest-api`, `cpt-examples-todo-app-interface-task-model`
- Design elements: `cpt-examples-todo-app-interface-websocket`, `cpt-examples-todo-app-design-db-tasks`, `cpt-examples-todo-app-design-context-decisions`
- Dependencies: None

## 2. Actor Flows (CDSL)

### Create Task Flow

- [ ] `p1` - **ID**: `cpt-examples-todo-app-flow-core-create-task`

**Actor**: `cpt-examples-todo-app-actor-user`

**Success Scenarios**:
- Task is created with all provided fields
- Task appears in the task list immediately

**Error Scenarios**:
- Validation fails for required fields
- Storage quota exceeded

**Steps**:
1. [ ] - `p1` - User clicks "Add Task" button - `inst-create-1`
2. [ ] - `p1` - UI: Display task creation form - `inst-create-2`
3. [ ] - `p1` - User enters task title (required) - `inst-create-3`
4. [ ] - `p1` - User optionally sets description, due date, priority, category - `inst-create-4`
5. [ ] - `p1` - User clicks "Save" - `inst-create-5`
6. [ ] - `p1` - API: POST /tasks ({ title, description, dueDate, priority, categoryId }) - `inst-create-6`
7. [ ] - `p1` - DB: INSERT tasks (id, user_id, title, description, status, priority, category_id, due_date) - `inst-create-7`
8. [ ] - `p1` - **IF** validation passes - `inst-create-8`
   1. [ ] - `p1` - DB: COMMIT transaction - `inst-create-8a`
   2. [ ] - `p1` - **RETURN** created task with generated ID - `inst-create-8b`
9. [ ] - `p1` - **ELSE** - `inst-create-9`
   1. [ ] - `p1` - **RETURN** validation error response - `inst-create-9a`

### Delete Task Flow

- [ ] `p1` - **ID**: `cpt-examples-todo-app-flow-core-delete-task`

**Actor**: `cpt-examples-todo-app-actor-user`

**Success Scenarios**:
- Task is permanently removed from storage
- Task disappears from the list

**Error Scenarios**:
- Task not found
- Concurrent deletion conflict

**Steps**:
1. [ ] - `p1` - User clicks delete icon on a task - `inst-delete-1`
2. [ ] - `p1` - UI: Display confirmation dialog - `inst-delete-2`
3. [ ] - `p1` - User confirms deletion - `inst-delete-3`
4. [ ] - `p1` - API: DELETE /tasks/:id - `inst-delete-4`
5. [ ] - `p1` - DB: DELETE FROM tasks WHERE id = :id AND user_id = :userId - `inst-delete-5`
6. [ ] - `p1` - **IF** task exists - `inst-delete-6`
   1. [ ] - `p1` - **RETURN** success (204 No Content) - `inst-delete-6a`
7. [ ] - `p1` - **ELSE** - `inst-delete-7`
   1. [ ] - `p1` - **RETURN** not found error (404) - `inst-delete-7a`

## 3. Algorithms (CDSL)

### Task Validation Algorithm

- [ ] `p2` - **ID**: `cpt-examples-todo-app-algo-core-validate-task`

**Input**: Task creation/update payload

**Output**: Validation result with errors array

**Steps**:
1. [ ] - `p1` - Parse and normalize input fields - `inst-val-1`
2. [ ] - `p1` - **IF** title is empty or > 255 chars - `inst-val-2`
   1. [ ] - `p1` - Add error: "Title is required and must be under 255 characters" - `inst-val-2a`
3. [ ] - `p1` - **IF** description > 5000 chars - `inst-val-3`
   1. [ ] - `p1` - Add error: "Description must be under 5000 characters" - `inst-val-3a`
4. [ ] - `p1` - **IF** dueDate is in the past - `inst-val-4`
   1. [ ] - `p1` - Add warning: "Due date is in the past" - `inst-val-4a`
5. [ ] - `p1` - **IF** priority not in ['low', 'medium', 'high'] - `inst-val-5`
   1. [ ] - `p1` - Add error: "Invalid priority value" - `inst-val-5a`
6. [ ] - `p1` - **IF** categoryId provided - `inst-val-6`
   1. [ ] - `p1` - DB: SELECT id FROM categories WHERE id = :categoryId AND user_id = :userId - `inst-val-6a`
   2. [ ] - `p1` - **IF** category not found, add error - `inst-val-6b`
7. [ ] - `p1` - **RETURN** { valid: errors.length === 0, errors, warnings } - `inst-val-7`

## 4. States (CDSL)

### Task State Machine

- [ ] `p2` - **ID**: `cpt-examples-todo-app-state-core-task`

**States**: draft, active, completed, deleted

**Initial State**: active

**Transitions**:
1. [ ] - `p1` - **FROM** active **TO** completed **WHEN** user marks task as done - `inst-state-1`
2. [ ] - `p1` - **FROM** completed **TO** active **WHEN** user unchecks completed task - `inst-state-2`
3. [ ] - `p1` - **FROM** active **TO** deleted **WHEN** user deletes task - `inst-state-3`
4. [ ] - `p1` - **FROM** completed **TO** deleted **WHEN** user deletes completed task - `inst-state-4`

## 5. Definitions of Done

### Implement Task CRUD Operations

- [ ] `p1` - **ID**: `cpt-examples-todo-app-dod-core-crud`

The system **MUST** provide full Create, Read, Update, Delete operations for tasks. All operations **MUST** validate input and return appropriate error responses.

**Implements**:
- `cpt-examples-todo-app-flow-core-create-task`
- `cpt-examples-todo-app-flow-core-delete-task`
- `cpt-examples-todo-app-algo-core-validate-task`
- `cpt-examples-todo-app-state-core-task`

## 6. Additional Context (optional)

### Performance Considerations

**ID**: `cpt-examples-todo-app-featurecontext-core-performance`

Task list queries should use cursor-based pagination for lists > 100 items. Consider implementing virtual scrolling on the frontend for smooth UX with large datasets.
