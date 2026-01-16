# Change: Add Resource Group Module

## Why

HyperSpot needs a Resource Group module that provides hierarchical resource organization with type-based access control. This module enables applications to organize resources in a tree structure with strict type validation, permission management, and efficient hierarchy operations using the Closure Table pattern.

The current `resource-group` directory is a symlink to an old repository with a different architecture. We need to rebuild it as a proper HyperSpot/ModKit module following the SDK pattern and DDD-light architecture.

## What Changes

**New Crates: `resource-group-sdk`  and `resource-group`**

**SDK Crate (`resource-group-sdk`):**
- `ResourceGroupApi` trait with methods for types, entities, references, and hierarchy operations
- Transport-agnostic models (`ResourceGroupType`, `ResourceGroupEntity`, `ResourceGroupReference`)
- `ResourceGroupError` enum for error handling
- All API methods accept `&SecurityCtx` for authorization and tenant isolation

**Module Crate (`resource-group`):**
- Module declaration with `#[modkit::module(name = "resource_group", capabilities = [db, rest])]`
- Domain service with business logic for type management, entity CRUD, hierarchy operations, and reference management
- Local client implementing `ResourceGroupApi` trait
- REST API handlers, routes, and DTOs following HyperSpot conventions
- SeaORM entities with Secure ORM for tenant isolation
- Closure Table pattern for efficient hierarchy queries (ancestors/descendants)
- ClientHub registration for inter-module access

**Core Features:**
1. **Type Management**: Define resource group types with allowed parent types and application permissions
2. **Hierarchical Organization**: Create, update, move, and delete entities in tree structures
3. **Access Control**: Application-based authorization with owner vs. allowed application permissions
4. **Reference Management**: Link resource groups to external resources with reference counting

## Impact

- **Affected specs**: `resource-group` (new capability)
- **Affected code**: `modules/resource-group/` (new crates)
- **Dependencies**: modkit, modkit-db, modkit-security, sea-orm, axum
- **Database**: Requires PostgreSQL/SQLite with closure table pattern
- **Migration**: Old `resource-group` symlink will be replaced with new ModKit module
