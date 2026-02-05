# ADR-0001: Use IndexedDB for Offline Storage

**Date**: 2024-01-15

**Status**: Accepted

**ID**: `spd-todo-app-adr-local-storage`

## Context and Problem Statement

The application requires offline-first functionality where users can create, edit, and complete tasks without network connectivity. We need to choose a client-side storage solution that can handle structured data with efficient querying.

## Decides For Requirements

This decision directly addresses the following requirements from PRD/DESIGN:

* `spd-todo-app-nfr-offline-support` — Enables full offline functionality by providing local storage for tasks
* `spd-todo-app-nfr-response-time` — IndexedDB's indexed queries enable <200ms response times for filtering/search operations
* `spd-todo-app-req-filter-tasks` — Indexes on status/category/priority enable efficient filtering without loading all data
* `spd-todo-app-principle-offline-first` — This is the core technical decision enabling the offline-first design principle

See:

* **PRD**: [PRD.md](../PRD.md)
* **DESIGN**: [DESIGN.md](../DESIGN.md)

## Decision Drivers

* Must support structured data with indexes for filtering
* Must handle storage of thousands of tasks efficiently
* Must work across all target browsers
* Must support asynchronous operations to avoid UI blocking

## Considered Options

* LocalStorage with JSON serialization
* IndexedDB with Dexie.js wrapper
* SQLite via WebAssembly (sql.js)

## Decision Outcome

Chosen option: "IndexedDB with Dexie.js wrapper", because it provides native browser support for structured data with indexes, handles large datasets efficiently, and Dexie.js provides a clean Promise-based API that simplifies development.

### Consequences

* Good, because IndexedDB is supported by all modern browsers natively
* Good, because Dexie.js provides TypeScript support and intuitive query syntax
* Good, because we can create indexes for efficient filtering by status, category, and due date
* Bad, because IndexedDB API complexity requires the Dexie.js abstraction layer
* Bad, because debugging IndexedDB issues requires specialized browser dev tools

### Confirmation

Implementation verified via:

* Unit tests for IndexedDB operations using fake-indexeddb
* Integration tests with Dexie.js queries
* Manual testing of offline scenarios in Chrome DevTools

## Pros and Cons of the Options

### LocalStorage with JSON serialization

Simple key-value storage with JSON.stringify/parse.

* Good, because simple API
* Good, because universal browser support
* Bad, because no indexing — filtering requires loading all data
* Bad, because 5MB storage limit
* Bad, because synchronous API blocks UI thread

### IndexedDB with Dexie.js wrapper

Native browser database with Promise-based wrapper library.

* Good, because supports indexes for efficient queries
* Good, because handles large datasets (100MB+)
* Good, because asynchronous API
* Good, because Dexie.js simplifies complex API
* Bad, because requires additional dependency
* Bad, because debugging is more complex

### SQLite via WebAssembly (sql.js)

Full SQL database compiled to WebAssembly.

* Good, because full SQL support
* Good, because familiar query language
* Bad, because large bundle size (~1MB)
* Bad, because requires manual persistence to IndexedDB anyway
* Bad, because performance overhead from WASM

## More Information

Decision aligns with offline-first architecture principle. Dexie.js chosen over raw IndexedDB for developer productivity.

## Traceability

**Actors**:
* `spd-todo-app-actor-user` - Primary beneficiary of offline functionality
* `spd-todo-app-actor-sync-service` - Syncs IndexedDB changes to server

**Requirements**:
* `spd-todo-app-nfr-offline-support` - Core requirement driving this decision
* `spd-todo-app-nfr-response-time` - IndexedDB enables fast local reads
