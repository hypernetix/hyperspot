# Feature: Task Filtering and Logic

## 1. Feature Context

- [ ] `p2` - **ID**: `cpt-examples-todo-app-featurecontext-logic`

### 1.1 Overview

Advanced filtering, sorting, and search capabilities for tasks including status filtering, priority sorting, and full-text search.

### 1.2 Purpose

Enables users to efficiently navigate and manage large numbers of tasks by applying filters and organizing tasks by various criteria.

### 1.3 Actors

- `cpt-examples-todo-app-actor-user` - Applies filters and searches tasks

### 1.4 References

- Overall Design: [DESIGN.md](../DESIGN.md)
- PRD: [PRD.md](../PRD.md)
- Requirements: `cpt-examples-todo-app-fr-filter-tasks`, `cpt-examples-todo-app-nfr-response-time`, `cpt-examples-todo-app-interface-rest-api`, `cpt-examples-todo-app-interface-task-model`
- Dependencies: `cpt-examples-todo-app-featurecontext-core`

## 2. Actor Flows (CDSL)

### Filter Tasks Flow

- [ ] `p1` - **ID**: `cpt-examples-todo-app-flow-logic-filter-tasks`

**Actor**: `cpt-examples-todo-app-actor-user`

**Success Scenarios**:
- Task list updates to show only matching tasks
- Filter state is preserved in URL

**Error Scenarios**:
- Invalid filter parameters ignored

**Steps**:
1. [ ] - `p1` - User selects status filter (all/active/completed) - `inst-filter-1`
2. [ ] - `p1` - User optionally selects category filter - `inst-filter-2`
3. [ ] - `p1` - User optionally selects priority filter - `inst-filter-3`
4. [ ] - `p1` - UI: Update URL query parameters - `inst-filter-4`
5. [ ] - `p1` - API: GET /tasks?status={status}&category={id}&priority={level} - `inst-filter-5`
6. [ ] - `p1` - DB: SELECT * FROM tasks WHERE user_id = :userId AND status = :status AND category_id = :categoryId AND priority = :priority - `inst-filter-6`
7. [ ] - `p1` - **RETURN** filtered task list - `inst-filter-7`

### Search Tasks Flow

- [ ] `p1` - **ID**: `cpt-examples-todo-app-flow-logic-search-tasks`

**Actor**: `cpt-examples-todo-app-actor-user`

**Success Scenarios**:
- Tasks matching search query are displayed
- Search is performed across title and description

**Error Scenarios**:
- Empty search returns all tasks
- Special characters are escaped

**Steps**:
1. [ ] - `p1` - User types in search input - `inst-search-1`
2. [ ] - `p1` - UI: Debounce input (300ms) - `inst-search-2`
3. [ ] - `p1` - API: GET /tasks?q={searchQuery} - `inst-search-3`
4. [ ] - `p1` - DB: SELECT * FROM tasks WHERE user_id = :userId AND (title ILIKE :query OR description ILIKE :query) - `inst-search-4`
5. [ ] - `p1` - **RETURN** matching tasks with highlighted matches - `inst-search-5`

## 3. Algorithms (CDSL)

### Task Sorting Algorithm

- [ ] `p2` - **ID**: `cpt-examples-todo-app-algo-logic-sort-tasks`

**Input**: Task list, sort field, sort direction

**Output**: Sorted task list

**Steps**:
1. [ ] - `p1` - Parse sort parameters (field: due_date|priority|created_at, direction: asc|desc) - `inst-sort-1`
2. [ ] - `p1` - **IF** sort by priority - `inst-sort-2`
   1. [ ] - `p1` - Map priority to numeric: high=3, medium=2, low=1 - `inst-sort-2a`
   2. [ ] - `p1` - Sort by numeric priority value - `inst-sort-2b`
3. [ ] - `p1` - **IF** sort by due_date - `inst-sort-3`
   1. [ ] - `p1` - Tasks without due date go to end - `inst-sort-3a`
   2. [ ] - `p1` - Sort remaining by date timestamp - `inst-sort-3b`
4. [ ] - `p1` - **IF** sort by created_at - `inst-sort-4`
   1. [ ] - `p1` - Sort by creation timestamp - `inst-sort-4a`
5. [ ] - `p1` - Apply direction (reverse if desc) - `inst-sort-5`
6. [ ] - `p1` - **RETURN** sorted task list - `inst-sort-6`

### Overdue Detection Algorithm

- [ ] `p2` - **ID**: `cpt-examples-todo-app-algo-logic-overdue-detection`

**Input**: Task with due_date

**Output**: Overdue status and urgency level

**Steps**:
1. [ ] - `p1` - **IF** task.due_date is null - `inst-overdue-1`
   1. [ ] - `p1` - **RETURN** { isOverdue: false, urgency: 'none' } - `inst-overdue-1a`
2. [ ] - `p1` - Calculate days until due: daysDiff = (due_date - now) / (24*60*60*1000) - `inst-overdue-2`
3. [ ] - `p1` - **IF** daysDiff < 0 - `inst-overdue-3`
   1. [ ] - `p1` - **RETURN** { isOverdue: true, urgency: 'critical' } - `inst-overdue-3a`
4. [ ] - `p1` - **IF** daysDiff < 1 - `inst-overdue-4`
   1. [ ] - `p1` - **RETURN** { isOverdue: false, urgency: 'high' } - `inst-overdue-4a`
5. [ ] - `p1` - **IF** daysDiff < 3 - `inst-overdue-5`
   1. [ ] - `p1` - **RETURN** { isOverdue: false, urgency: 'medium' } - `inst-overdue-5a`
6. [ ] - `p1` - **RETURN** { isOverdue: false, urgency: 'low' } - `inst-overdue-6`

## 4. States (CDSL)

### Filter State Machine

- [ ] `p2` - **ID**: `cpt-examples-todo-app-state-logic-filter`

**States**: all, active, completed

**Initial State**: all

**Transitions**:
1. [ ] - `p1` - **FROM** all **TO** active **WHEN** user clicks "Active" tab - `inst-fstate-1`
2. [ ] - `p1` - **FROM** all **TO** completed **WHEN** user clicks "Completed" tab - `inst-fstate-2`
3. [ ] - `p1` - **FROM** active **TO** all **WHEN** user clicks "All" tab - `inst-fstate-3`
4. [ ] - `p1` - **FROM** active **TO** completed **WHEN** user clicks "Completed" tab - `inst-fstate-4`
5. [ ] - `p1` - **FROM** completed **TO** all **WHEN** user clicks "All" tab - `inst-fstate-5`
6. [ ] - `p1` - **FROM** completed **TO** active **WHEN** user clicks "Active" tab - `inst-fstate-6`

## 5. Definitions of Done

### Implement Task Filtering

- [ ] `p1` - **ID**: `cpt-examples-todo-app-dod-logic-filtering`

The system **MUST** allow filtering tasks by status, category, and priority. Filters **MUST** be combinable and reflected in the URL for shareability.

**Implements**:
- `cpt-examples-todo-app-flow-logic-filter-tasks`
- `cpt-examples-todo-app-state-logic-filter`

**Touches**:
- API: `GET /tasks`
- DB: `tasks`
- Entities: `FilterCriteria`

### Implement Task Search

- [ ] `p1` - **ID**: `cpt-examples-todo-app-dod-logic-search`

The system **MUST** provide full-text search across task titles and descriptions. Search **MUST** be case-insensitive and support partial matching.

**Implements**:
- `cpt-examples-todo-app-flow-logic-search-tasks`

**Touches**:
- API: `GET /tasks`
- DB: `tasks`
- Entities: `SearchQuery`

### Implement Task Sorting

- [ ] `p1` - **ID**: `cpt-examples-todo-app-dod-logic-sorting`

The system **MUST** allow sorting tasks by due date, priority, and creation date in ascending or descending order.

**Implements**:
- `cpt-examples-todo-app-algo-logic-sort-tasks`
- `cpt-examples-todo-app-algo-logic-overdue-detection`

**Touches**:
- API: `GET /tasks`
- DB: `tasks`
- Entities: `SortCriteria`

## 6. Additional Context (optional)

### UX Considerations

**ID**: `cpt-examples-todo-app-featurecontext-logic-ux`


Filter and sort preferences should persist in localStorage so users don't need to reapply them on each visit. Consider showing filter badges to indicate active filters.

