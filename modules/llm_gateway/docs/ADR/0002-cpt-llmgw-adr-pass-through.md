# ADR-0002: Pass-through Content Processing

**Date**: 2026-01-29

**Status**: Accepted

**ID**: `cpt-cf-llm-gateway-adr-pass-through`

## Context and Problem Statement

LLM Gateway receives tool calls and structured content from providers. Should it execute tools and interpret responses, or pass them to consumers?

## Decision Drivers

* Clear separation of responsibilities
* Security: Gateway should not execute arbitrary code
* Flexibility: consumers control tool implementation

## Considered Options

* Gateway executes tools and interprets content
* Gateway passes tool calls to consumer for execution

## Decision Outcome

Chosen option: "Pass-through", because tool execution is consumer responsibility and Gateway should not interpret content.

### Consequences

* Good, because Gateway has no execution security risks
* Good, because consumers have full control over tool implementation
* Good, because Gateway remains simple and focused on routing
* Bad, because consumers must implement tool execution logic

## Related Design Elements

**Requirements**:
* `cpt-cf-llm-gateway-fr-tool-calling-v1` - Tool calls returned to consumer
