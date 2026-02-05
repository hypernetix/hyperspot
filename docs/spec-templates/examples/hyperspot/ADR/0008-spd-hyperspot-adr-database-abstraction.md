# ADR-0008: Database-Agnostic Core with sqlx

**Date**: 2024-01-18

**Status**: Accepted

**ID**: `spd-hyperspot-adr-database-abstraction`

## Context and Problem Statement

Different deployment environments have different database requirements: SQLite for local development and edge deployments, PostgreSQL for cloud production, MariaDB for compatibility with existing enterprise infrastructure. We need a database abstraction that supports multiple backends without sacrificing type safety or performance.

## Decision Drivers

* Must support SQLite, PostgreSQL, and MariaDB from same codebase
* Must provide compile-time SQL query checking (not just runtime)
* Must avoid N+1 queries through explicit query design
* Must work with async Rust and connection pooling
* Must allow escape hatch for database-specific optimizations when needed

## Considered Options

* sqlx with compile-time query checking (chosen)
* Diesel ORM with full type-safe query builder
* SeaORM with async support and migrations
* Raw SQL with runtime parameter binding

## Decision Outcome

Chosen option: "sqlx with compile-time query checking", because sqlx verifies SQL syntax and result types against a real database at compile time (via DATABASE_URL environment variable), catching errors before deployment. Unlike ORMs, sqlx allows writing SQL directly, making complex queries readable and allowing developers to optimize per database dialect when needed.

### Consequences

* Good, because SQL queries are verified against schema at compile time
* Good, because raw SQL allows complex queries and database-specific optimizations
* Good, because async/await works natively (no blocking query execution)
* Good, because connection pooling is built-in and configurable
* Good, because migrations are standard SQL files (no ORM-specific DSL)
* Bad, because compile-time checking requires DATABASE_URL connection during build
* Bad, because cross-database compatibility must be tested manually (no automatic abstraction)
* Bad, because developers must know SQL (cannot rely on ORM query builder)

## Related Design Elements

**Actors**:
* `spd-hyperspot-actor-database-manager` - Abstracts multiple database backends
* `spd-hyperspot-actor-platform-operator` - Chooses database per deployment environment

**Requirements**:
* `spd-hyperspot-fr-database-agnostic` - Core requirement for multi-DB support
* `spd-hyperspot-nfr-database-compatibility` - SQLite, PostgreSQL, MariaDB support
* `spd-hyperspot-nfr-response-time` - Query performance critical for API latency
