# Feature: Task Filtering and Logic

## 1. Feature Context

**ID**: `spd-todo-app-feature-logic`
**Status**: NOT_STARTED

### 1.1 Overview

Advanced filtering, sorting, and search capabilities for tasks including status filtering, priority sorting, and full-text search.

### 1.2 Purpose

Enables users to efficiently navigate and manage large numbers of tasks by applying filters and organizing tasks by various criteria.

### 1.3 Actors

- `spd-todo-app-actor-user` - Applies filters and searches tasks

### 1.4 References

- Overall Design: [DESIGN.md](../DESIGN.md)
- Dependencies: `spd-todo-app-feature-core`

## 2. Actor Flows (FDL)

### Filter Tasks Flow

- [ ] **ID**: `spd-todo-app-flow-logic-filter-tasks`


**Actor**: `spd-todo-app-actor-user`

**Success Scenarios**:
- Task list updates to show only matching tasks
- Filter state is preserved in URL

**Error Scenarios**:
- Invalid filter parameters ignored

**Steps**:
1. [ ] - `ph-1` - User selects status filter (all/active/completed) - `inst-filter-1`
2. [ ] - `ph-1` - User optionally selects category filter - `inst-filter-2`
3. [ ] - `ph-1` - User optionally selects priority filter - `inst-filter-3`
4. [ ] - `ph-1` - UI: Update URL query parameters - `inst-filter-4`
5. [ ] - `ph-1` - API: GET /tasks?status={status}&category={id}&priority={level} - `inst-filter-5`
6. [ ] - `ph-1` - DB: SELECT * FROM tasks WHERE user_id = :userId AND status = :status AND category_id = :categoryId AND priority = :priority - `inst-filter-6`
7. [ ] - `ph-1` - **RETURN** filtered task list - `inst-filter-7`


### Search Tasks Flow

- [ ] **ID**: `spd-todo-app-flow-logic-search-tasks`


**Actor**: `spd-todo-app-actor-user`

**Success Scenarios**:
- Tasks matching search query are displayed
- Search is performed across title and description

**Error Scenarios**:
- Empty search returns all tasks
- Special characters are escaped

**Steps**:
1. [ ] - `ph-1` - User types in search input - `inst-search-1`
2. [ ] - `ph-1` - UI: Debounce input (300ms) - `inst-search-2`
3. [ ] - `ph-1` - API: GET /tasks?q={searchQuery} - `inst-search-3`
4. [ ] - `ph-1` - DB: SELECT * FROM tasks WHERE user_id = :userId AND (title ILIKE :query OR description ILIKE :query) - `inst-search-4`
5. [ ] - `ph-1` - **RETURN** matching tasks with highlighted matches - `inst-search-5`


## 3. Algorithms (FDL)

### Task Sorting Algorithm

- [ ] **ID**: `spd-todo-app-algo-logic-sort-tasks`


**Input**: Task list, sort field, sort direction

**Output**: Sorted task list

**Steps**:
1. [ ] - `ph-1` - Parse sort parameters (field: due_date|priority|created_at, direction: asc|desc) - `inst-sort-1`
2. [ ] - `ph-1` - **IF** sort by priority - `inst-sort-2`
   1. [ ] - `ph-1` - Map priority to numeric: high=3, medium=2, low=1 - `inst-sort-2a`
   2. [ ] - `ph-1` - Sort by numeric priority value - `inst-sort-2b`
3. [ ] - `ph-1` - **IF** sort by due_date - `inst-sort-3`
   1. [ ] - `ph-1` - Tasks without due date go to end - `inst-sort-3a`
   2. [ ] - `ph-1` - Sort remaining by date timestamp - `inst-sort-3b`
4. [ ] - `ph-1` - **IF** sort by created_at - `inst-sort-4`
   1. [ ] - `ph-1` - Sort by creation timestamp - `inst-sort-4a`
5. [ ] - `ph-1` - Apply direction (reverse if desc) - `inst-sort-5`
6. [ ] - `ph-1` - **RETURN** sorted task list - `inst-sort-6`


### Overdue Detection Algorithm

- [ ] **ID**: `spd-todo-app-algo-logic-overdue-detection`


**Input**: Task with due_date

**Output**: Overdue status and urgency level

**Steps**:
1. [ ] - `ph-1` - **IF** task.due_date is null - `inst-overdue-1`
   1. [ ] - `ph-1` - **RETURN** { isOverdue: false, urgency: 'none' } - `inst-overdue-1a`
2. [ ] - `ph-1` - Calculate days until due: daysDiff = (due_date - now) / (24*60*60*1000) - `inst-overdue-2`
3. [ ] - `ph-1` - **IF** daysDiff < 0 - `inst-overdue-3`
   1. [ ] - `ph-1` - **RETURN** { isOverdue: true, urgency: 'critical' } - `inst-overdue-3a`
4. [ ] - `ph-1` - **IF** daysDiff < 1 - `inst-overdue-4`
   1. [ ] - `ph-1` - **RETURN** { isOverdue: false, urgency: 'high' } - `inst-overdue-4a`
5. [ ] - `ph-1` - **IF** daysDiff < 3 - `inst-overdue-5`
   1. [ ] - `ph-1` - **RETURN** { isOverdue: false, urgency: 'medium' } - `inst-overdue-5a`
6. [ ] - `ph-1` - **RETURN** { isOverdue: false, urgency: 'low' } - `inst-overdue-6`


## 4. States (FDL)

### Filter State Machine

- [ ] **ID**: `spd-todo-app-state-logic-filter`


**States**: all, active, completed

**Initial State**: all

**Transitions**:
1. [ ] - `ph-1` - **FROM** all **TO** active **WHEN** user clicks "Active" tab - `inst-fstate-1`
2. [ ] - `ph-1` - **FROM** all **TO** completed **WHEN** user clicks "Completed" tab - `inst-fstate-2`
3. [ ] - `ph-1` - **FROM** active **TO** all **WHEN** user clicks "All" tab - `inst-fstate-3`
4. [ ] - `ph-1` - **FROM** active **TO** completed **WHEN** user clicks "Completed" tab - `inst-fstate-4`
5. [ ] - `ph-1` - **FROM** completed **TO** all **WHEN** user clicks "All" tab - `inst-fstate-5`
6. [ ] - `ph-1` - **FROM** completed **TO** active **WHEN** user clicks "Active" tab - `inst-fstate-6`


## 5. Requirements

### Implement Task Filtering

- [ ] **ID**: `spd-todo-app-req-logic-filtering`


**Status**: NOT_STARTED

**Description**: The system SHALL allow filtering tasks by status, category, and priority. Filters MUST be combinable and reflected in the URL for shareability.

**Implementation details**:
- API: GET /tasks with query parameters status, category, priority
- DB: Compound index on (user_id, status, category_id, priority)
- Domain: FilterCriteria value object

**Implements**:
- `spd-todo-app-flow-logic-filter-tasks`
- `spd-todo-app-state-logic-filter`

**Phases**:
- [ ] `ph-1`: Status and category filtering


### Implement Task Search

- [ ] **ID**: `spd-todo-app-req-logic-search`


**Status**: NOT_STARTED

**Description**: The system SHALL provide full-text search across task titles and descriptions. Search MUST be case-insensitive and support partial matching.

**Implementation details**:
- API: GET /tasks?q={query} with debounced client requests
- DB: GIN index on title and description for full-text search
- Domain: SearchQuery value object with sanitization

**Implements**:
- `spd-todo-app-flow-logic-search-tasks`

**Phases**:
- [ ] `ph-1`: Basic ILIKE search


### Implement Task Sorting

- [ ] **ID**: `spd-todo-app-req-logic-sorting`


**Status**: NOT_STARTED

**Description**: The system SHALL allow sorting tasks by due date, priority, and creation date in ascending or descending order.

**Implementation details**:
- API: GET /tasks?sort={field}&order={asc|desc}
- DB: Indexes on due_date, priority, created_at
- Domain: SortCriteria value object

**Implements**:
- `spd-todo-app-algo-logic-sort-tasks`
- `spd-todo-app-algo-logic-overdue-detection`

**Phases**:
- [ ] `ph-1`: Basic sorting by all fields


## 6. Additional Context (optional)

### UX Considerations

**ID**: `spd-todo-app-featurecontext-logic-ux`


Filter and sort preferences should persist in localStorage so users don't need to reapply them on each visit. Consider showing filter badges to indicate active filters.

