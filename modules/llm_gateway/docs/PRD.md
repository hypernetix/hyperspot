# PRD

## 1. Overview

**Purpose**: LLM Gateway provides unified access to multiple LLM providers. Consumers interact with a single interface regardless of underlying provider. Gateway normalizes requests and responses but does not execute tools or interpret content — this is consumer responsibility.

LLM Gateway is the central integration point between platform consumers and external AI providers. It abstracts provider differences — request formats, authentication, error handling, rate limits — behind a unified API. Consumers send requests in a normalized format; Gateway translates them to provider-specific calls and normalizes responses back.

The Gateway supports diverse modalities: text generation, embeddings, vision, audio, video, and document processing. It handles both synchronous and asynchronous operations, including streaming responses and long-running jobs. All interactions go through the Outbound API Gateway for reliability and credential management.

Gateway is stateless by design. It does not store conversation history or execute tools — these are consumer responsibilities. The only exception is temporary state for async job tracking.

**Target Users**:
- **Platform Developers** — build AI-powered features using Gateway API
- **External API Consumers** — third-party developers accessing AI capabilities via public API

**Key Problems Solved**:
- **Provider fragmentation**: single API abstracts differences between OpenAI, Anthropic, Google, and other providers
- **Governance**: budget enforcement, rate limiting, usage tracking, and audit logging at tenant level
- **Security**: pre-call and post-response interceptors for content moderation and PII filtering

**Success Criteria**:
- Gateway overhead < 50ms P99 (excluding provider latency)
- Availability ≥ 99.9%

**Capabilities**:
- Text generation (chat completion)
- Multimodal input/output (images, audio, video, documents)
- Embeddings generation
- Tool/function calling
- Structured output with schema validation

## 2. Actors

### 2.1 Human Actors

#### API User

**ID**: `cpt-cf-llm-gateway-actor-api-user`

**Role**: End user who interacts with LLM Gateway directly via API. Sends chat completion requests, manages async jobs, uses streaming responses.

### 2.2 System Actors

#### Consumer

**ID**: `cpt-cf-llm-gateway-actor-consumer`

**Role**: Sends requests to the Gateway.

#### Provider

**ID**: `cpt-cf-llm-gateway-actor-provider`

**Role**: External AI service that processes requests. Accessed via Outbound API Gateway.

#### Hook Plugin

**ID**: `cpt-cf-llm-gateway-actor-hook-plugin`

**Role**: Pre-call and post-response interception (moderation, PII, transformation).

#### Usage Tracker

**ID**: `cpt-cf-llm-gateway-actor-usage-tracker`

**Role**: Budget checks and usage reporting.

#### Audit Module

**ID**: `cpt-cf-llm-gateway-actor-audit-module`

**Role**: Compliance event logging.

## 3. Functional Requirements

### P1 — Core

#### Chat Completion

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-chat-completion-v1`

Consumer sends messages, Gateway routes to provider based on model, returns response with usage.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Streaming Chat Completion

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-streaming-v1`

Same as chat completion, but response is streamed. Gateway normalizes provider events to unified format.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Embeddings Generation

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-embeddings-v1`

Consumer sends text(s), Gateway returns vector embeddings.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Vision (Image Analysis)

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-vision-v1`

Consumer sends message with image URLs. Gateway fetches media from FileStorage (direct) or external URLs (via OAGW), routes to vision-capable model via OAGW.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Image Generation

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-image-generation-v1`

Consumer sends text prompt. Gateway sends request to provider via OAGW, stores generated image in FileStorage (direct), returns URL.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Speech-to-Text

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-speech-to-text-v1`

Consumer sends message with audio URL. Gateway fetches audio from FileStorage (direct) or external URLs (via OAGW), sends to provider via OAGW, returns transcription.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Text-to-Speech

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-text-to-speech-v1`

Consumer sends text. Gateway sends request to provider via OAGW, stores synthesized audio in FileStorage (direct), returns URL. Supports streaming mode.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Video Understanding

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-video-understanding-v1`

Consumer sends message with video URL. Gateway fetches video from FileStorage (direct) or external URLs (via OAGW), sends to provider via OAGW, returns analysis.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Video Generation

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-video-generation-v1`

Consumer sends text prompt. Gateway sends request to provider via OAGW, stores generated video in FileStorage (direct), returns URL. Typically requires async mode due to long processing.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Tool/Function Calling

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-tool-calling-v1`

Consumer sends request with tool definitions. Gateway resolves schema references, converts to provider format. Model returns tool calls for consumer to execute. Gateway does not execute tools — this is consumer responsibility.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Structured Output

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-structured-output-v1`

Consumer requests response matching JSON schema. Gateway validates response against schema.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Document Understanding

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-document-understanding-v1`

Consumer sends message with document URL. Gateway fetches document, routes to capable model.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Async Jobs

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-async-jobs-v1`

Consumer can request async execution for long-running operations. Gateway returns job ID, consumer polls for result.

Gateway abstracts provider behavior:
- Sync provider + async request → Gateway simulates job
- Async provider + sync request → Gateway polls internally

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Realtime Audio

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-realtime-audio-v1`

Bidirectional audio streaming via WebSocket for voice conversations.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Usage Tracking

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-fr-usage-tracking-v1`

Gateway reports usage after each request via Usage Tracker: tokens, cost estimate, latency, attribution (tenant, user, conversation, model).

Cross-cutting concern — applies to all operations, no dedicated UC.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-usage-tracker`

### P2 — Reliability & Governance

#### Provider Fallback

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-fr-provider-fallback-v1`

When primary provider fails, Gateway automatically switches to fallback provider with matching capabilities.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Timeout Enforcement

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-fr-timeout-v1`

Gateway enforces timeout types:
- Time-to-first-token (TTFT): max wait for initial response chunk
- Total generation timeout: max duration for complete response

On timeout → fallback (if configured) → error.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

#### Pre-Call Interceptor

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-fr-pre-call-interceptor-v1`

Before sending to provider, Gateway invokes Hook Plugin. Plugin can allow, block, or modify request.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-hook-plugin`

#### Post-Response Interceptor

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-fr-post-response-interceptor-v1`

After receiving response, Gateway invokes Hook Plugin. Plugin can allow, block, or modify response.

**Actors**: `cpt-cf-llm-gateway-actor-hook-plugin`, `cpt-cf-llm-gateway-actor-consumer`

#### Per-Tenant Budget Enforcement

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-fr-budget-enforcement-v1`

Gateway checks budget before execution via Usage Tracker. Rejects if exhausted, reports actual usage after completion.

Cross-cutting concern — applies to all operations, no dedicated UC.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-usage-tracker`

#### Rate Limiting

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-fr-rate-limiting-v1`

Gateway enforces rate limits at tenant and user levels. Rejects requests exceeding limits.

**Actors**: `cpt-cf-llm-gateway-actor-consumer`

### P3 — Optimization

#### Batch Processing

- [ ] `p3` - **ID**: `cpt-cf-llm-gateway-fr-batch-processing-v1`

Consumer submits batch of requests for async processing at reduced cost. Gateway abstracts provider batch APIs (OpenAI Batch API, Anthropic Message Batches).

**Actors**: `cpt-cf-llm-gateway-actor-consumer`, `cpt-cf-llm-gateway-actor-provider`

### P4 — Enterprise

#### Audit Events

- [ ] `p4` - **ID**: `cpt-cf-llm-gateway-fr-audit-events-v1`

Gateway emits audit events via Audit Module for compliance: request started, completed, failed, blocked, fallback triggered.

Cross-cutting concern — applies to all operations, no dedicated UC.

**Actors**: `cpt-cf-llm-gateway-actor-audit-module`

## 4. Use Cases

#### UC-001: Chat Completion

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-chat-completion-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant.

**Flow**:
1. Consumer sends chat_completion(model, messages)
2. Gateway resolves provider via Model Registry
3. Gateway sends request via Outbound API Gateway
4. Provider returns response
5. Gateway returns normalized response with usage

**Postconditions**: Response returned, usage reported.

**Acceptance criteria**:
- Response in normalized format regardless of provider
- Usage metrics included (tokens, cost estimate)
- Provider errors normalized to Gateway error format

#### UC-002: Streaming Chat Completion

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-streaming-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant, model supports streaming.

**Flow**:
1. Consumer sends chat_completion(stream=true)
2. Gateway resolves provider via Model Registry
3. Gateway establishes streaming connection to provider
4. Gateway normalizes each chunk
5. Gateway streams chunks to Consumer
6. Gateway sends final usage summary

**Postconditions**: Stream completed, usage reported.

**Acceptance criteria**:
- Chunks normalized from provider format
- Final message includes usage metrics
- Connection errors propagated to consumer

#### UC-003: Embeddings Generation

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-embeddings-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Embedding model available for tenant.

**Flow**:
1. Consumer sends embed(model, input[])
2. Gateway resolves provider via Model Registry
3. Gateway sends request via Outbound API Gateway
4. Provider returns vectors
5. Gateway returns vectors with usage

**Postconditions**: Vectors returned, usage reported.

**Acceptance criteria**:
- Vectors returned in normalized format
- Usage metrics included (tokens)

#### UC-004: Vision (Image Analysis)

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-vision-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant, model supports required content type.

**Flow**:
1. Consumer sends chat_completion with image URLs
2. Gateway resolves provider via Model Registry
3. Gateway fetches images from FileStorage
4. Gateway sends request via Outbound API Gateway
5. Provider returns analysis
6. Gateway returns response with usage

**Postconditions**: Response returned, usage reported.

**Acceptance criteria**:
- Multiple images supported per request
- Response in normalized format
- Usage metrics included

#### UC-005: Image Generation

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-image-generation-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Image generation model available for tenant.

**Flow**:
1. Consumer sends generation request with prompt
2. Gateway resolves provider via Model Registry
3. Gateway sends request via Outbound API Gateway
4. Provider returns generated image
5. Gateway stores image in FileStorage
6. Gateway returns URL with usage

**Postconditions**: Image stored, URL returned, usage reported.

**Acceptance criteria**:
- Generated image accessible via returned URL
- Response in normalized format
- Usage metrics included

#### UC-006: Speech-to-Text

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-speech-to-text-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: STT model available for tenant.

**Flow**:
1. Consumer sends message with audio URL
2. Gateway resolves provider via Model Registry
3. Gateway fetches audio from FileStorage
4. Gateway sends request via Outbound API Gateway
5. Provider returns transcription
6. Gateway returns text response with usage

**Postconditions**: Transcription returned, usage reported.

**Acceptance criteria**:
- Transcription in normalized format
- Usage metrics included

#### UC-007: Text-to-Speech

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-text-to-speech-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: TTS model available for tenant.

**Flow**:
1. Consumer sends TTS request with text
2. Gateway resolves provider via Model Registry
3. Gateway sends request via Outbound API Gateway
4. Provider returns audio
5. Gateway stores audio in FileStorage
6. Gateway returns URL with usage

**Postconditions**: Audio stored, URL returned, usage reported.

**Acceptance criteria**:
- Generated audio accessible via returned URL
- Streaming mode supported (audio chunks returned directly)
- Usage metrics included

#### UC-008: Video Understanding

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-video-understanding-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant, model supports required content type.

**Flow**:
1. Consumer sends message with video URL
2. Gateway resolves provider via Model Registry
3. Gateway fetches video from FileStorage
4. Gateway sends request via Outbound API Gateway
5. Provider returns analysis
6. Gateway returns response with usage

**Postconditions**: Response returned, usage reported.

**Acceptance criteria**:
- Response in normalized format
- Usage metrics included

#### UC-009: Video Generation

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-video-generation-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Video generation model available for tenant.

**Flow**:
1. Consumer sends generation request with prompt
2. Gateway resolves provider via Model Registry
3. Gateway sends request via Outbound API Gateway
4. Provider returns generated video
5. Gateway stores video in FileStorage
6. Gateway returns URL with usage

**Postconditions**: Video stored, URL returned, usage reported.

**Acceptance criteria**:
- Generated video accessible via returned URL
- Async mode supported (typically required due to long processing)
- Usage metrics included

#### UC-010: Tool/Function Calling

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-tool-calling-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant, model supports function calling.

**Flow**:
1. Consumer sends chat_completion with tool definitions
2. Gateway resolves provider via Model Registry
3. Gateway resolves schemas via Type Registry (for reference and inline GTS formats)
4. Gateway converts tools to provider format
5. Gateway sends request via Outbound API Gateway
6. Provider returns tool_calls
7. Gateway returns tool_calls in unified format
8. Consumer executes tools, sends results
9. Gateway forwards tool results to provider
10. Provider returns final response
11. Gateway returns response with usage

**Postconditions**: Response returned, usage reported.

**Acceptance criteria**:
- Tool definitions supported: reference, inline GTS, unified format (OpenAI-like)
- Tool calls returned in unified format
- Response in normalized format

#### UC-011: Structured Output

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-structured-output-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant.

**Flow**:
1. Consumer sends chat_completion with response_schema
2. Gateway resolves provider via Model Registry
3. Gateway sends request via Outbound API Gateway
4. Provider returns JSON response
5. Gateway validates response against schema
6. Gateway returns validated response with usage (or validation_error if invalid)

**Postconditions**: Valid JSON returned, usage reported.

**Acceptance criteria**:
- Response validated against provided schema
- Returns validation_error with details if schema validation fails
- Response in normalized format

#### UC-012: Document Understanding

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-document-understanding-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant, model supports required content type.

**Flow**:
1. Consumer sends message with document URL
2. Gateway resolves provider via Model Registry
3. Gateway fetches document from FileStorage
4. Gateway sends request via Outbound API Gateway
5. Provider returns analysis
6. Gateway returns response with usage

**Postconditions**: Response returned, usage reported.

**Acceptance criteria**:
- Response in normalized format
- Usage metrics included

#### UC-013: Async Jobs

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-async-jobs-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant.

**Flow**:
1. Consumer sends request with async=true
2. Gateway resolves provider via Model Registry
3. Gateway initiates async job
4. Gateway returns job_id
5. Consumer polls get_job(job_id)
6. Gateway returns status/result
7. (Optional) Consumer cancels job via cancel_job(job_id)

**Postconditions**: Job completed, cancelled, or result returned.

**Acceptance criteria**:
- Sync provider + async request: Gateway simulates job
- Async provider + sync request: Gateway polls internally
- Job status: pending, running, completed, failed, cancelled
- Job cancellation supported

#### UC-014: Realtime Audio

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-usecase-realtime-audio-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant, model supports realtime audio.

**Flow**:
1. Consumer establishes WebSocket connection
2. Gateway resolves provider via Model Registry
3. Gateway connects to provider WebSocket
4. Bidirectional audio/text streaming
5. Consumer closes connection
6. Gateway returns usage summary

**Postconditions**: Session closed, usage reported.

**Acceptance criteria**:
- Bidirectional streaming supported
- Usage summary on close

#### UC-015: Provider Fallback

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-usecase-provider-fallback-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant.

**Flow**:
1. Consumer sends request with fallback configuration
2. Gateway resolves provider via Model Registry
3. Gateway sends request to primary provider
4. Primary provider fails
5. Gateway selects fallback from request configuration
6. Gateway sends request to fallback provider
7. Gateway returns response (fallback indicated)

**Postconditions**: Response returned via fallback.

**Acceptance criteria**:
- Fallback configuration provided in request
- Fallback selection based on capability match
- Response includes fallback indicator

#### UC-016: Timeout Enforcement

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-usecase-timeout-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant.

**Flow**:
1. Consumer sends request
2. Gateway starts timeout tracking (TTFT, total)
3. Gateway sends request to provider
4. If TTFT timeout: Gateway triggers fallback or error
5. If total timeout: Gateway triggers fallback or error
6. Gateway returns response or error

**Postconditions**: Response returned or timeout error.

**Acceptance criteria**:
- TTFT (time-to-first-token) timeout enforced
- Total generation timeout enforced
- On timeout: fallback (if configured) or error

#### UC-017: Pre-Call Interceptor

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-usecase-pre-call-interceptor-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Hook Plugin configured for tenant.

**Flow**:
1. Consumer sends request
2. Gateway invokes Hook Plugin pre_call
3. Plugin allows, blocks, or modifies request
4. If blocked: Gateway returns request_blocked error
5. If allowed/modified: Gateway proceeds with request

**Postconditions**: Request processed or blocked.

**Acceptance criteria**:
- Plugin can allow, block, or modify request
- Blocked requests return request_blocked error

#### UC-018: Post-Response Interceptor

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-usecase-post-response-interceptor-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Hook Plugin configured for tenant.

**Flow**:
1. Provider returns response
2. Gateway invokes Hook Plugin post_response
3. Plugin allows, blocks, or modifies response
4. If blocked: Gateway returns response_blocked error
5. If allowed/modified: Gateway returns response to consumer

**Postconditions**: Response returned or blocked.

**Acceptance criteria**:
- Plugin can allow, block, or modify response
- Blocked responses return response_blocked error

#### UC-019: Rate Limiting

- [ ] `p2` - **ID**: `cpt-cf-llm-gateway-usecase-rate-limiting-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Rate limits configured for tenant.

**Flow**:
1. Consumer sends request
2. Gateway checks rate limits
3. If limit exceeded: Gateway returns rate_limited error
4. If within limits: Gateway proceeds with request

**Postconditions**: Request processed or rejected.

**Acceptance criteria**:
- Rate limits enforced at tenant level
- Rate limits enforced at user level
- Exceeded requests return rate_limited error

#### UC-020: Batch Processing

- [ ] `p3` - **ID**: `cpt-cf-llm-gateway-usecase-batch-processing-v1`
**Actor**: `cpt-cf-llm-gateway-actor-consumer`

**Preconditions**: Model available for tenant, provider supports batch API.

**Flow**:
1. Consumer submits batch of requests
2. Gateway resolves provider via Model Registry
3. Gateway submits to provider batch API
4. Gateway returns batch_id
5. Consumer polls for results
6. Gateway returns status and results
7. (Optional) Consumer cancels batch

**Postconditions**: Batch completed, results returned.

**Acceptance criteria**:
- Abstracts OpenAI Batch API, Anthropic Message Batches
- Partial results available as completed
- Batch cancellation supported

## 5. Non-functional requirements

#### Scalability

- [ ] `p1` - **ID**: `cpt-cf-llm-gateway-nfr-scalability-v1`
Horizontal scaling without state coordination. Stateless design with exception for temporary async job state.

