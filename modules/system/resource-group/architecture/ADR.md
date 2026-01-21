# Resource Group - Architecture Decision Records

**Module**: Resource Group
**Version**: 1.0
**Last Updated**: 2026-01-21

This document tracks all significant architectural decisions for the Resource Group module.

---

## ADR-0001: Use Closure Table for Hierarchy Storage

**ID**: `fdd-hyperspot-adr-closure-table`
<!-- fdd-id-content -->
**Date**: 2026-01-21
**Status**: Accepted
**Deciders**: Hyperspot Team
**Technical Story**: Need efficient way to store and query hierarchical resource groups

### Context and Problem Statement

The Resource Group module requires storing entities in a hierarchical (tree) structure.
Key requirements include:
- Efficient querying of all ancestors of a node (for permission checks).
- Efficient querying of all descendants of a node (for list operations).
- Support for moving subtrees to new parents.
- Referential integrity enforcement.
- Support for depth and width constraints.

We need a database pattern that supports these operations efficiently in a relational database (PostgreSQL/SQLite/MariaDB).

### Decision Drivers

- **Read Performance**: Querying ancestors and descendants must be fast (critical path for permissions).
- **Data Integrity**: Must ensure no cycles and valid parent references.
- **Flexibility**: Need to support moving subtrees.
- **Complexity**: Implementation complexity should be manageable.
- **Database Independence**: Should work with standard SQL (SeaORM).

### Considered Options

1. **Closure Table** (chosen)
2. **Adjacency List** (Recursive CTEs)
3. **Path Enumeration** (Materialized Path)
4. **Nested Sets**

### Decision Outcome

**Chosen option**: "Closure Table"

**Rationale**:
Closure Table stores all paths between nodes (not just direct parent-child).
- **Ancestors/Descendants**: Can be queried with a simple `JOIN` without recursion, which is very efficient and portable.
- **Depth**: Storing depth allows easy limiting (`max_depth`) and sorting.
- **Move Operations**: Moving a subtree involves deleting old paths and inserting new paths, which is straightforward to implement transactionally.
- **Referential Integrity**: Can use foreign keys on the closure table to ensure validity.

**Positive Consequences**:
- O(1) query complexity for "is descendant of".
- Simple SQL queries for retrieving subtrees.
- Separation of tree structure from node data.

**Negative Consequences**:
- Higher storage requirement (O(N^2) in worst case, but usually O(N*depth)).
- More complex write operations (need to update multiple rows in closure table).

### Related Design Elements

**Actors**:
- `fdd-hyperspot-actor-application` - Manages resource groups

**Capabilities**:
- `fdd-hyperspot-capability-resource-organization` - Hierarchical organization

**Requirements**:
- `fdd-hyperspot-req-resource-org` - Efficient hierarchy queries

**Principles**:
- `fdd-hyperspot-principle-efficient-reads` - Optimize for read performance
<!-- fdd-id-content -->
