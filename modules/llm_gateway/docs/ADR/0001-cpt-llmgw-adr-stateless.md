# ADR-0001: Stateless Gateway Design

**Date**: 2026-01-29

**Status**: Accepted

**ID**: `cpt-cf-llm-gateway-adr-stateless`

## Context and Problem Statement

LLM Gateway needs to handle high request volume while remaining scalable. Should it maintain conversation state or require consumers to provide full context?

## Decision Drivers

* Horizontal scalability without state synchronization
* Simplified deployment and failover
* Consumer flexibility in context management

## Considered Options

* Stateful: Gateway stores conversation history
* Stateless: Consumer provides full context per request

## Decision Outcome

Chosen option: "Stateless", because it enables horizontal scaling without coordination overhead.

### Consequences

* Good, because instances are interchangeable and can scale independently
* Good, because no state synchronization between nodes required
* Bad, because consumers must manage and provide conversation context
* Exception: temporary async job state stored in distributed cache

## Related Design Elements

**Requirements**:
* `cpt-cf-llm-gateway-nfr-scalability-v1` - Horizontal scaling requirement
* `cpt-cf-llm-gateway-fr-async-jobs-v1` - Exception for temporary job state
