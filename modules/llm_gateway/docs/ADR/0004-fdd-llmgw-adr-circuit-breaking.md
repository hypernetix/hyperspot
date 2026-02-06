# ADR-0004: Circuit Breaking vs Health-Based Routing

**Date**: 2026-02-03

**Status**: Accepted

**ID**: `fdd-llmgw-adr-circuit-breaking`

## Context and Problem Statement

LLM Gateway needs to handle provider failures gracefully. There are two potential mechanisms: circuit breaking and health-based routing. Where should each be implemented?

## Decision Drivers

* Clear separation of infrastructure and business concerns
* Fast failure detection and recovery
* Proactive avoidance of unhealthy providers

## Considered Options

* Circuit breaking only in LLM Gateway
* Circuit breaking only in Outbound API Gateway
* Both mechanisms at different layers

## Decision Outcome

Chosen option: "Both mechanisms at different layers", because they serve different purposes and complement each other.

### Circuit Breaking (Outbound API Gateway)

Infrastructure-level protection. Prevents cascading failures by stopping requests to an endpoint that's currently failing (fast-fail). This is a standard pattern for HTTP clients and operates on real-time request/response data.

Characteristics:
* Reactive — responds to failures as they happen
* Short time window (seconds)
* Per-endpoint granularity
* Automatic recovery with half-open state

### Health-Based Routing (LLM Gateway)

Business-level decisions. Uses health metrics from Model Registry (latency percentiles, error rates over time) to select the best provider *before* making a request.

Characteristics:
* Proactive — avoids unhealthy providers before request
* Longer time window (minutes)
* Per-provider/model granularity
* Based on aggregated metrics

### How They Work Together

1. Gateway checks Model Registry health → selects healthy provider
2. Request goes to Outbound API Gateway
3. OAGW circuit breaker → fast-fails if provider becomes unhealthy mid-traffic
4. Gateway receives failure → may trigger fallback to another provider

### Consequences

* Good, because infrastructure concerns (OAGW) are separate from business logic (Gateway)
* Good, because proactive routing reduces load on degraded providers
* Good, because reactive circuit breaking provides last-line defense
* Bad, because two systems need coordination on health thresholds

## Related Design Elements

**Requirements**:
* `fdd-llmgw-fr-provider-fallback-v1` - Fallback on provider failure
* `fdd-model-registry-fr-health-monitoring-v1` - Provider health metrics
