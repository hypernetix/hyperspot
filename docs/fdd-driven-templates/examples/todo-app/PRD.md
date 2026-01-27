# PRD

## A. Vision

**Purpose**: A web application for managing personal tasks with support for categories, priorities, and filtering.

Todo App is a simple and intuitive task management application. Users can create, edit, and delete tasks, mark them as completed, and organize them by categories and priorities.

The application is designed for individual use with cross-device synchronization. The main focus is on minimalist interface and fast performance.

**Target Users**:
- **Individual users** - People who want to organize their personal tasks
- **Small teams** - Groups collaborating on shared task lists

**Key Problems Solved**:
- **Forgetting tasks**: Centralized storage of all tasks with reminders
- **Prioritization**: Visual highlighting of important tasks and sorting by priority

**Success Criteria**:
- Task creation time < 3 seconds
- 95% of users successfully complete onboarding
- NPS > 40

**Capabilities**:
- CRUD operations for tasks
- Categorization and prioritization
- Filtering and search
- Cross-device synchronization

## B. Actors

### Human Actors

#### User

**ID**: `fdd-todo-app-actor-user`

<!-- fdd-id-content -->
**Role**: Primary user who creates, manages, and completes tasks in the application.
<!-- fdd-id-content -->

### System Actors

#### Sync Service

**ID**: `fdd-todo-app-actor-sync-service`

<!-- fdd-id-content -->
**Role**: Background service that synchronizes tasks across user devices in real-time.
<!-- fdd-id-content -->

#### Notification Service

**ID**: `fdd-todo-app-actor-notification-service`

<!-- fdd-id-content -->
**Role**: Sends reminders and notifications to users about upcoming or overdue tasks.
<!-- fdd-id-content -->

## C. Functional Requirements

#### Create Task

**ID**: `fdd-todo-app-fr-create-task`

<!-- fdd-id-content -->
**Priority**: High

The system must allow users to create a new task with a title, optional description, due date, priority level, and category.

**Actors**: `fdd-todo-app-actor-user`
<!-- fdd-id-content -->

#### Complete Task

**ID**: `fdd-todo-app-fr-complete-task`

<!-- fdd-id-content -->
**Priority**: High

The system must allow users to mark a task as completed or revert it to incomplete status.

**Actors**: `fdd-todo-app-actor-user`
<!-- fdd-id-content -->

#### Filter Tasks

**ID**: `fdd-todo-app-fr-filter-tasks`

<!-- fdd-id-content -->
**Priority**: Medium

The system must allow users to filter tasks by status (all, active, completed), category, and priority.

**Actors**: `fdd-todo-app-actor-user`
<!-- fdd-id-content -->

#### Delete Task

**ID**: `fdd-todo-app-fr-delete-task`

<!-- fdd-id-content -->
**Priority**: High

The system must allow users to delete a task permanently.

**Actors**: `fdd-todo-app-actor-user`
<!-- fdd-id-content -->

## D. Use Cases

#### UC-001: Create a New Task

**ID**: `fdd-todo-app-usecase-create-task`

<!-- fdd-id-content -->
**Actor**: `fdd-todo-app-actor-user`

**Preconditions**: User is authenticated and on the main task list view.

**Flow**:
1. User clicks the "Add Task" button
2. System displays the task creation form
3. User enters task title (required), description, due date, priority, and category
4. User clicks "Save"
5. System validates input and creates the task
6. System displays the updated task list with the new task

**Postconditions**: New task is persisted and visible in the task list.

**Acceptance criteria**:
- Task appears in the list immediately after creation
- All entered fields are correctly saved
- Validation errors are shown for invalid input

<!-- fdd-id-content -->

#### UC-002: Complete a Task

**ID**: `fdd-todo-app-usecase-complete-task`

<!-- fdd-id-content -->
**Actor**: `fdd-todo-app-actor-user`

**Preconditions**: User has at least one active task in the list.

**Flow**:
1. User clicks the checkbox next to an active task
2. System marks the task as completed
3. System updates the task's visual appearance (strikethrough, moved to completed section)

**Postconditions**: Task status is updated to completed and persisted.

**Acceptance criteria**:
- Task is visually marked as completed
- Task can be reverted to active status
- Completion timestamp is recorded

<!-- fdd-id-content -->

## E. Non-functional requirements

#### Response Time

**ID**: `fdd-todo-app-nfr-response-time`

<!-- fdd-id-content -->
All user interactions must complete within 200ms under normal load conditions.
<!-- fdd-id-content -->

#### Data Persistence

**ID**: `fdd-todo-app-nfr-data-persistence`

<!-- fdd-id-content -->
User data must be persisted locally and synced to cloud storage within 5 seconds of any change.
<!-- fdd-id-content -->

#### Offline Support

**ID**: `fdd-todo-app-nfr-offline-support`

<!-- fdd-id-content -->
The application must function fully offline with automatic sync when connectivity is restored.
<!-- fdd-id-content -->

## F. Additional context

#### Market Research

**ID**: `fdd-todo-app-prd-context-market-research`

<!-- fdd-id-content -->
Primary competitors include Todoist, Microsoft To Do, and Apple Reminders. Our differentiator is the combination of simplicity with powerful filtering capabilities.
<!-- fdd-id-content -->
