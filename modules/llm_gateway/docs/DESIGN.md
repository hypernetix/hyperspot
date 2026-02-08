# Technical Design: LLM Gateway

## 1. Architecture Overview

### 1.1 Architectural Vision

LLM Gateway provides unified access to multiple LLM providers. Consumers interact with a single interface regardless of underlying provider.

The architecture follows a pass-through design: Gateway normalizes requests and responses but does not interpret content or execute tools. Provider-specific adapters handle translation to/from each provider's API format. All external calls route through Outbound API Gateway for credential injection and circuit breaking. Gateway also performs health-based routing using Model Registry metrics — see [ADR-0004](./ADR/0004-cpt-cyberfabric-llm-gateway-adr-circuit-breaking.md) for the distinction between infrastructure-level circuit breaking (OAGW) and business-level health routing (Gateway).

The system is horizontally scalable and stateless. No conversation history is stored; consumers provide full context with each request. The only state is temporary async job tracking, which can be stored in distributed cache.

### 1.2 Architecture Drivers

#### Product requirements

See [PRD.md](./PRD.md) section 1 "Overview" — Key Problems Solved:
- Provider fragmentation
- Governance
- Security

#### Functional requirements

| Cypilot ID | Solution short description |
|--------|----------------------------|
| `cpt-cyberfabric-llm-gateway-fr-chat-completion-v1` | Provider adapters + Outbound API GW |
| `cpt-cyberfabric-llm-gateway-fr-streaming-v1` | SSE pass-through via adapters |
| `cpt-cyberfabric-llm-gateway-fr-embeddings-v1` | Provider adapters + Outbound API GW |
| `cpt-cyberfabric-llm-gateway-fr-vision-v1` | FileStorage fetch + provider adapters |
| `cpt-cyberfabric-llm-gateway-fr-image-generation-v1` | Provider adapters + FileStorage store |
| `cpt-cyberfabric-llm-gateway-fr-speech-to-text-v1` | FileStorage fetch + provider adapters |
| `cpt-cyberfabric-llm-gateway-fr-text-to-speech-v1` | Provider adapters + FileStorage store |
| `cpt-cyberfabric-llm-gateway-fr-video-understanding-v1` | FileStorage fetch + provider adapters |
| `cpt-cyberfabric-llm-gateway-fr-video-generation-v1` | Provider adapters + FileStorage store |
| `cpt-cyberfabric-llm-gateway-fr-tool-calling-v1` | Type Registry resolution + format conversion |
| `cpt-cyberfabric-llm-gateway-fr-structured-output-v1` | Schema validation |
| `cpt-cyberfabric-llm-gateway-fr-document-understanding-v1` | FileStorage fetch + provider adapters |
| `cpt-cyberfabric-llm-gateway-fr-async-jobs-v1` | Distributed cache for job state, polling abstraction |
| `cpt-cyberfabric-llm-gateway-fr-realtime-audio-v1` | WebSocket proxy via Outbound API GW |
| `cpt-cyberfabric-llm-gateway-fr-usage-tracking-v1` | Usage Tracker module integration |
| `cpt-cyberfabric-llm-gateway-fr-provider-fallback-v1` | Fallback chain from request config |
| `cpt-cyberfabric-llm-gateway-fr-timeout-v1` | TTFT + total timeout tracking |
| `cpt-cyberfabric-llm-gateway-fr-pre-call-interceptor-v1` | Hook Plugin pre_call invocation |
| `cpt-cyberfabric-llm-gateway-fr-post-response-interceptor-v1` | Hook Plugin post_response invocation |
| `cpt-cyberfabric-llm-gateway-fr-budget-enforcement-v1` | Usage Tracker check_budget / report_usage |
| `cpt-cyberfabric-llm-gateway-fr-rate-limiting-v1` | Distributed rate limiter |
| `cpt-cyberfabric-llm-gateway-fr-batch-processing-v1` | Provider batch API abstraction |
| `cpt-cyberfabric-llm-gateway-fr-audit-events-v1` | Audit Module event emission |

#### Non-functional requirements

| Cypilot ID | Solution short description |
|--------|----------------------------|
| `cpt-cyberfabric-llm-gateway-nfr-scalability-v1` | Stateless design, distributed cache for async jobs |

### 1.3 Architecture Layers

| Layer | Responsibility | Technology |
|-------|---------------|------------|
| API | Request/response handling, validation | REST/OpenAPI |
| Application | Request orchestration, provider routing | Core services |
| Adapters | Provider-specific translation | Provider adapters |
| Infrastructure | External calls, caching | Outbound API GW, distributed cache |

## 2. Principles & Constraints

### 2.1: Design Principles

#### Stateless

**ID**: `cpt-cyberfabric-llm-gateway-principle-stateless`

<!-- cpt-id-content -->
**ADRs**: `cpt-cyberfabric-llm-gateway-adr-stateless`

Gateway does not store conversation history. Consumer provides full context with each request. Exception: temporary async job state.
<!-- cpt-id-content -->

#### Pass-through

**ID**: `cpt-cyberfabric-llm-gateway-principle-pass-through`

<!-- cpt-id-content -->
**ADRs**: `cpt-cyberfabric-llm-gateway-adr-pass-through`

Gateway normalizes but does not interpret content. Tool execution and response parsing are consumer responsibility.
<!-- cpt-id-content -->

### 2.2: Constraints

#### Provider Rate Limits

**ID**: `cpt-cyberfabric-llm-gateway-constraint-provider-rate-limits`

<!-- cpt-id-content -->
Gateway is subject to provider TPM/RPM quotas. Cannot exceed limits imposed by external providers.
<!-- cpt-id-content -->

#### Provider Context Windows

**ID**: `cpt-cyberfabric-llm-gateway-constraint-provider-context-windows`

<!-- cpt-id-content -->
Request size limited by provider context window. Gateway cannot send requests exceeding provider limits.
<!-- cpt-id-content -->

#### Outbound API Gateway Dependency

**ID**: `cpt-cyberfabric-llm-gateway-constraint-outbound-dependency`

<!-- cpt-id-content -->
All external API calls must route through Outbound API Gateway. Direct provider calls are not permitted.
<!-- cpt-id-content -->

#### No Credential Storage

**ID**: `cpt-cyberfabric-llm-gateway-constraint-no-credentials`

<!-- cpt-id-content -->
Gateway does not store provider credentials. Credential injection handled by Outbound API Gateway.
<!-- cpt-id-content -->

#### Content Logging Restrictions

**ID**: `cpt-cyberfabric-llm-gateway-constraint-content-logging`

<!-- cpt-id-content -->
Full request/response content is not logged due to PII concerns. Only metadata (tokens, latency, model, tenant) is logged.
<!-- cpt-id-content -->

## 3. Technical Architecture

### 3.1: Domain Model

**Technology**: GTS (JSON Schema)

**Location**: [`llm_gateway-sdk/schemas/`](../llm_gateway-sdk/schemas/)

**Core Entities**:

*Request/Response (`core/`):*
- [Request](../llm_gateway-sdk/schemas/core/request.v1.schema.json) - Chat completion request (model, messages, tools, stream, async, response_schema, fallback)
- [Response](../llm_gateway-sdk/schemas/core/response.v1.schema.json) - Chat completion response (content, tool_calls, usage, fallback_used, model_used)
- [EmbeddingRequest](../llm_gateway-sdk/schemas/core/embedding_request.v1.schema.json) - Embedding request (model, input, dimensions, encoding_format)
- [EmbeddingResponse](../llm_gateway-sdk/schemas/core/embedding_response.v1.schema.json) - Embedding response (model, data[], usage)
- [StreamChunk](../llm_gateway-sdk/schemas/core/stream_chunk.v1.schema.json) - Streaming response chunk (id, model, delta, finish_reason, usage)
- [Message](../llm_gateway-sdk/schemas/core/message.v1.schema.json) - Conversation element (role, content)
- [Usage](../llm_gateway-sdk/schemas/core/usage.v1.schema.json) - Token counts and cost estimate (input_tokens, output_tokens, cost_estimate)
- [FallbackConfig](../llm_gateway-sdk/schemas/core/fallback_config.v1.schema.json) - Fallback configuration (models[], strategy)
- [Role](../llm_gateway-sdk/schemas/core/role.v1.schema.json) - Enum: system, user, assistant, tool
- [FallbackStrategy](../llm_gateway-sdk/schemas/core/fallback_strategy.v1.schema.json) - Enum: sequential, parallel

*Content (`content/`):*
- [ContentPart](../llm_gateway-sdk/schemas/content/content_part.v1.schema.json) - Abstract base for content types
- [TextContent](../llm_gateway-sdk/schemas/content/text_content.v1.schema.json) - Plain text (text)
- [ImageContent](../llm_gateway-sdk/schemas/content/image_content.v1.schema.json) - Image URL from FileStorage (url)
- [AudioContent](../llm_gateway-sdk/schemas/content/audio_content.v1.schema.json) - Audio URL from FileStorage (url)
- [VideoContent](../llm_gateway-sdk/schemas/content/video_content.v1.schema.json) - Video URL from FileStorage (url)
- [DocumentContent](../llm_gateway-sdk/schemas/content/document_content.v1.schema.json) - Document URL from FileStorage (url)
- [ToolCallContent](../llm_gateway-sdk/schemas/content/tool_call_content.v1.schema.json) - Tool call wrapper (tool_call)
- [ToolResultContent](../llm_gateway-sdk/schemas/content/tool_result_content.v1.schema.json) - Tool result wrapper (tool_result)
- [ToolCall](../llm_gateway-sdk/schemas/content/tool_call.v1.schema.json) - Provider's tool invocation (id, name, arguments)
- [ToolResult](../llm_gateway-sdk/schemas/content/tool_result.v1.schema.json) - Consumer's tool result (tool_call_id, content)

*Tools (`tools/`):*
- [Tool](../llm_gateway-sdk/schemas/tools/tool.v1.schema.json) - Abstract base for tool definitions
- [ToolReference](../llm_gateway-sdk/schemas/tools/tool_reference.v1.schema.json) - Reference to Type Registry (schema_id)
- [ToolInlineGTS](../llm_gateway-sdk/schemas/tools/tool_inline_gts.v1.schema.json) - Inline GTS schema (schema)
- [ToolUnified](../llm_gateway-sdk/schemas/tools/tool_unified.v1.schema.json) - Unified format (name, description, parameters)
- [Schema](../llm_gateway-sdk/schemas/tools/schema.v1.schema.json) - JSON Schema wrapper (json_schema)

*Async (`async/`):*
- [Job](../llm_gateway-sdk/schemas/async/job.v1.schema.json) - Async job (id, status, request, result, error, created_at, expires_at)
- [Batch](../llm_gateway-sdk/schemas/async/batch.v1.schema.json) - Batch request (id, status, requests[], created_at)
- [BatchRequest](../llm_gateway-sdk/schemas/async/batch_request.v1.schema.json) - Individual batch item (custom_id, request, result, error)
- [Error](../llm_gateway-sdk/schemas/async/error.v1.schema.json) - Error info (code, message)
- [JobStatus](../llm_gateway-sdk/schemas/async/job_status.v1.schema.json) - Enum: pending, running, completed, failed, cancelled
- [BatchStatus](../llm_gateway-sdk/schemas/async/batch_status.v1.schema.json) - Enum: pending, in_progress, completed, failed, cancelled

**Relationships**:
- Request → Message: contains 1..*
- Request → Tool: contains 0..*
- Request → Schema: optional response_schema
- Request → FallbackConfig: optional
- EmbeddingRequest → Usage: returns
- EmbeddingResponse → Usage: contains
- Message → Role: has
- Message → ContentPart: contains 1..*
- Response → ContentPart: contains 0..*
- Response → ToolCall: contains 0..*
- Response → Usage: contains 1
- ContentPart ← TextContent, ImageContent, AudioContent, VideoContent, DocumentContent, ToolCallContent, ToolResultContent
- ToolCallContent → ToolCall: wraps
- ToolResultContent → ToolResult: wraps
- Tool ← ToolReference, ToolInlineGTS, ToolUnified
- ToolInlineGTS → Schema: contains
- ToolUnified → Schema: parameters
- Job → JobStatus: has
- Job → Request: references
- Job → Response: optional result
- Job → Error: optional
- Batch → BatchStatus: has
- Batch → BatchRequest: contains 1..*
- BatchRequest → Request: references
- BatchRequest → Response: optional result
- BatchRequest → Error: optional

### 3.2: Component Model

```mermaid
graph TB
    Consumer --> API[API Layer]
    API --> App[Application Layer]
    App --> HP[Hook Plugin]
    App --> UT[Usage Tracker]
    App --> AM[Audit Module]
    App --> Adapters[Provider Adapters]
    Adapters --> OB[Outbound API GW]
    OB --> Provider
    App --> MR[Model Registry]
    App --> TR[Type Registry]
    App --> FS[FileStorage]
```

**Components**:
 - [ ] `p1` - **ID**: `cpt-cyberfabric-llm-gateway-component-api-layer`
   - API Layer
   - Request/response handling, validation, routing
 - [ ] `p1` - **ID**: `cpt-cyberfabric-llm-gateway-component-application-layer`
   - Application Layer
   - Request orchestration, provider selection, response normalization
 - [ ] `p1` - **ID**: `cpt-cyberfabric-llm-gateway-component-provider-adapters`
   - Provider Adapters
   - Provider-specific request/response translation
 - [ ] `p1` - **ID**: `cpt-cyberfabric-llm-gateway-component-hook-plugin`
   - Hook Plugin
   - Pre-call and post-response interception (moderation, PII, transformation)
 - [ ] `p1` - **ID**: `cpt-cyberfabric-llm-gateway-component-usage-tracker`
   - Usage Tracker
   - Budget checks and usage reporting
 - [ ] `p1` - **ID**: `cpt-cyberfabric-llm-gateway-component-audit-module`
   - Audit Module
   - Compliance event logging

**Interactions**:
- Consumer → API Layer: Normalized requests
- Application Layer → Model Registry: Model resolution and availability check
- Application Layer → Type Registry: Tool schema resolution
- Application Layer → FileStorage: Media fetch/store
- Provider Adapters → Outbound API GW: Provider API calls

**Dependencies**:

| Dependency | Role |
|------------|------|
| Model Registry | Model catalog, availability checks |
| Outbound API Gateway | External API calls to providers |
| FileStorage | Media storage and retrieval |
| Type Registry | Tool schema resolution |

### 3.3: API Contracts

**Technology**: REST/OpenAPI

**Location**: <!-- TBD -->

**Endpoints Overview**:
- `POST /chat/completions` - Chat completion (sync/streaming)
- `POST /embeddings` - Embeddings generation
- `POST /jobs` - Create async job
- `GET /jobs/{id}` - Get job status/result
- `DELETE /jobs/{id}` - Cancel job
- `POST /batches` - Create batch
- `GET /batches/{id}` - Get batch status/results
- `WS /realtime` - Realtime audio session

**Error Codes**:

| Error | Description |
|-------|-------------|
| `model_not_found` | Model not in catalog |
| `model_not_approved` | Model not approved for tenant |
| `validation_error` | Invalid request format |
| `capability_not_supported` | Model lacks required capability |
| `budget_exceeded` | Tenant budget exhausted |
| `rate_limited` | Rate limit exceeded |
| `request_blocked` | Blocked by pre-call hook |
| `response_blocked` | Blocked by post-response hook |
| `provider_error` | Provider returned error |
| `provider_timeout` | Provider request timed out |
| `job_not_found` | Job ID does not exist |
| `job_expired` | Job result TTL exceeded |

**Streaming Contract**:

Streaming responses use Server-Sent Events (SSE) format. Each event contains a [StreamChunk](../llm_gateway-sdk/schemas/core/stream_chunk.v1.schema.json).

Format:
```
data: {"id":"chunk-1","model":"gpt-4","delta":{"role":"assistant"}}\n\n
data: {"id":"chunk-2","model":"gpt-4","delta":{"content":"Hello"}}\n\n
data: {"id":"chunk-3","model":"gpt-4","delta":{"content":" world"}}\n\n
data: {"id":"chunk-4","model":"gpt-4","delta":{},"finish_reason":"stop","usage":{"input_tokens":10,"output_tokens":5}}\n\n
data: [DONE]\n\n
```

Chunk structure:
- `id` — Chunk identifier
- `model` — Model used for generation
- `delta.role` — Role (first chunk only)
- `delta.content` — Content fragment
- `delta.tool_calls` — Tool call fragments (index, id, function.name, function.arguments)
- `finish_reason` — Completion reason (final chunk): stop, length, tool_calls, content_filter
- `usage` — Token counts (final chunk only)

The stream terminates with `data: [DONE]` event.

### 3.4: Interactions & Sequences

> **Note**: In the sequence diagrams below, "LLM Gateway" (GW) represents the full gateway stack including Provider Adapters. In practice, the Application Layer delegates to provider-specific adapters, which then call Outbound API Gateway. This is simplified for diagram readability. See Component Model (section 3.2) for the detailed layer structure.

#### Provider Resolution

- [ ] `p1` - **ID**: `cpt-cyberfabric-llm-gateway-seq-provider-resolution-v1`

This sequence is used by all request flows to resolve the target provider. Other diagrams show "Resolve provider" as a simplified step — this is the detailed flow.

```mermaid
sequenceDiagram
    participant GW as LLM Gateway
    participant MR as Model Registry

    GW->>MR: get_tenant_model(ctx, model_id)
    MR->>MR: Check model exists
    MR->>MR: Check tenant approval
    MR->>MR: Get provider health status
    MR-->>GW: ModelInfo + provider + health

    alt Fallback configured
        GW->>GW: Rank providers by health
        GW->>GW: Select best available
    else Single provider
        GW->>GW: Use primary provider
    end
```

**Resolution outcomes**:
- `model_not_found` — model not in catalog
- `model_not_approved` — model not approved for tenant
- `model_deprecated` — model sunset by provider
- Success — returns provider endpoint + health metrics

#### Chat Completion

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-chat-completion-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(model, messages)
    GW->>GW: Resolve provider
    GW->>OB: Provider API call
    OB->>P: Request
    P-->>OB: Response
    OB-->>GW: Response
    GW-->>C: Normalized response + usage
```

#### Streaming Chat Completion

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-streaming-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(stream=true)
    GW->>OB: Streaming request
    OB->>P: Request
    loop Content chunks
        P-->>OB: chunk
        OB-->>GW: chunk
        GW-->>C: normalized chunk
    end
    GW-->>C: usage + done
```

#### Embeddings Generation

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-embeddings-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: embed(model, input[])
    GW->>OB: Embeddings request
    OB->>P: Request
    P-->>OB: vectors[]
    OB-->>GW: vectors[]
    GW-->>C: vectors[] + usage
```

#### Vision (Image Analysis)

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-vision-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant FS as FileStorage
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(messages with image URLs)
    GW->>FS: Fetch images
    FS-->>GW: image content
    GW->>OB: Request with images
    OB->>P: Request
    P-->>OB: Analysis
    OB-->>GW: Analysis
    GW-->>C: Response + usage
```

#### Image Generation

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-image-generation-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider
    participant FS as FileStorage

    C->>GW: chat_completion(image gen model, prompt)
    GW->>OB: Generation request
    OB->>P: Request
    P-->>OB: Generated image
    OB-->>GW: Generated image
    GW->>FS: Store image
    FS-->>GW: URL
    GW-->>C: Response with URL + usage
```

#### Speech-to-Text

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-speech-to-text-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant FS as FileStorage
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(messages with audio URL)
    GW->>FS: Fetch audio
    FS-->>GW: audio content
    GW->>OB: Transcription request
    OB->>P: Request
    P-->>OB: Transcription
    OB-->>GW: Transcription
    GW-->>C: Text response + usage
```

#### Text-to-Speech

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-text-to-speech-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider
    participant FS as FileStorage

    C->>GW: chat_completion(TTS model, text)
    GW->>OB: Synthesis request
    OB->>P: Request
    P-->>OB: Audio
    OB-->>GW: Audio
    GW->>FS: Store audio
    FS-->>GW: URL
    GW-->>C: Response with URL + usage
```

#### Video Understanding

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-video-understanding-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant FS as FileStorage
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(messages with video URL)
    GW->>FS: Fetch video
    FS-->>GW: video content
    GW->>OB: Analysis request
    OB->>P: Request
    P-->>OB: Analysis
    OB-->>GW: Analysis
    GW-->>C: Response + usage
```

#### Video Generation

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-video-generation-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider
    participant FS as FileStorage

    C->>GW: chat_completion(video gen model, prompt)
    GW->>OB: Generation request
    OB->>P: Request
    P-->>OB: Generated video
    OB-->>GW: Generated video
    GW->>FS: Store video
    FS-->>GW: URL
    GW-->>C: Response with URL + usage
```

#### Tool/Function Calling

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-tool-calling-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant TR as Type Registry
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(messages, tools)
    GW->>TR: Resolve schema references
    TR-->>GW: schemas
    GW->>OB: Request with tools
    OB->>P: Request
    P-->>OB: tool_calls[]
    OB-->>GW: tool_calls[]
    GW-->>C: Response with tool_calls[]
    Note over C: Consumer executes tools
    C->>GW: chat_completion(messages + tool_results)
    GW->>OB: Request
    OB->>P: Request
    P-->>OB: Final response
    OB-->>GW: Final response
    GW-->>C: Response
```

#### Structured Output

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-structured-output-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(messages, response_schema)
    GW->>OB: Request with schema
    OB->>P: Request
    P-->>OB: JSON response
    OB-->>GW: JSON response
    GW->>GW: Validate against schema
    GW-->>C: Validated response + usage
```

#### Document Understanding

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-document-understanding-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant FS as FileStorage
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(messages with document URL)
    GW->>FS: Fetch document
    FS-->>GW: document content
    GW->>OB: Analysis request
    OB->>P: Request
    P-->>OB: Analysis
    OB-->>GW: Analysis
    GW-->>C: Response + usage
```

#### Async Jobs

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-async-jobs-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(async=true)
    GW->>OB: Start request
    OB->>P: Request
    P-->>OB: job started
    OB-->>GW: job started
    GW-->>C: job_id

    C->>GW: get_job(job_id)
    GW->>OB: Check status
    OB->>P: Poll
    P-->>OB: result
    OB-->>GW: result
    GW-->>C: result
```

#### Realtime Audio

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-realtime-audio-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: WebSocket connect
    GW->>OB: WebSocket connect
    OB->>P: WebSocket connect

    loop Session
        C->>GW: audio chunk
        GW->>OB: audio chunk
        OB->>P: audio chunk
        P-->>OB: audio/text
        OB-->>GW: audio/text
        GW-->>C: audio/text
    end

    C->>GW: close
    GW-->>C: usage summary
```

#### Provider Fallback

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-provider-fallback-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P1 as Primary Provider
    participant P2 as Fallback Provider

    C->>GW: chat_completion(model)
    GW->>OB: Request
    OB->>P1: Request
    P1-->>OB: failure
    OB-->>GW: failure
    GW->>GW: Select fallback
    GW->>OB: Request
    OB->>P2: Request
    P2-->>OB: Response
    OB-->>GW: Response
    GW-->>C: Response (fallback indicated)
```

#### Timeout Enforcement

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-timeout-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(stream=true)
    GW->>GW: Start TTFT timer
    GW->>OB: Streaming request
    OB->>P: Request

    alt TTFT timeout exceeded
        GW-->>C: provider_timeout (TTFT)
    else First token received
        P-->>OB: first chunk
        OB-->>GW: first chunk
        GW->>GW: Start total timeout timer
        GW-->>C: first chunk
        loop Content chunks
            P-->>OB: chunk
            OB-->>GW: chunk
            GW-->>C: chunk
        end
        alt Total timeout exceeded
            GW-->>C: provider_timeout (total)
        else Completed
            GW-->>C: usage + done
        end
    end
```

#### Pre-Call Interceptor

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-pre-call-interceptor-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant HP as Hook Plugin
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(...)
    GW->>HP: pre_call(request)
    alt Blocked
        HP-->>GW: blocked
        GW-->>C: request_blocked
    else Allowed/Modified
        HP-->>GW: proceed
        GW->>OB: Request
        OB->>P: Request
    end
```

#### Post-Response Interceptor

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-post-response-interceptor-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider
    participant HP as Hook Plugin
    participant C as Consumer

    GW->>OB: Request
    OB->>P: Request
    P-->>OB: Response
    OB-->>GW: Response
    GW->>HP: post_response(response)
    alt Blocked
        HP-->>GW: blocked
        GW-->>C: response_blocked
    else Allowed/Modified
        HP-->>GW: proceed
        GW-->>C: Response
    end
```

#### Rate Limiting

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-rate-limiting-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(...)
    GW->>GW: Check rate limits
    alt Limit exceeded
        GW-->>C: rate_limited
    else Within limits
        GW->>OB: Request
        OB->>P: Request
        P-->>OB: Response
        OB-->>GW: Response
        GW-->>C: Response
    end
```

#### Budget Enforcement

**Use cases**: `cpt-cyberfabric-llm-gateway-fr-budget-enforcement-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`, `cpt-cyberfabric-llm-gateway-actor-usage-tracker`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant UT as Usage Tracker
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: chat_completion(...)
    GW->>UT: check_budget(tenant, model)
    alt Budget exceeded
        UT-->>GW: budget_exceeded
        GW-->>C: budget_exceeded
    else Budget available
        UT-->>GW: ok (remaining quota)
        GW->>OB: Request
        OB->>P: Request
        P-->>OB: Response
        OB-->>GW: Response
        GW->>UT: report_usage(tenant, model, tokens)
        UT-->>GW: ok
        GW-->>C: Response
    end
```

**Budget enforcement flow**:
1. **Pre-request check**: Gateway calls `check_budget()` before processing
2. **Reject if exceeded**: Returns `budget_exceeded` error immediately
3. **Process request**: If budget available, proceed with provider call
4. **Report usage**: After response, report actual token usage to Usage Tracker
5. **Usage Tracker**: Maintains running totals per tenant/model, enforces configured limits

#### Batch Processing

**Use cases**: `cpt-cyberfabric-llm-gateway-usecase-batch-processing-v1`
**Actors**: `cpt-cyberfabric-llm-gateway-actor-consumer`

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant OB as Outbound API Gateway
    participant P as Provider

    C->>GW: create_batch(requests[])
    GW->>OB: Submit batch
    OB->>P: Provider batch API
    P-->>OB: batch_id
    OB-->>GW: batch_id
    GW-->>C: batch_id

    C->>GW: get_batch(batch_id)
    GW->>OB: Check status
    OB->>P: Poll batch
    P-->>OB: status + results[]
    OB-->>GW: status + results[]
    GW-->>C: status + results[]
```

**Tenant isolation**: Gateway is stateless, but batch metadata is stored in distributed cache:
- On `create_batch`: Gateway stores `{batch_id → tenant_id, provider_batch_id, created_at}` in cache
- On `get_batch`: Gateway retrieves tenant_id from cache, validates caller has access
- Cache TTL matches batch expiration policy

This is the same pattern used for async jobs (see ADR-0001).

### 3.5: Database schemas & tables

<!-- Not applicable - Gateway is stateless except for temporary async job state -->

### 3.6: Topology (optional)

<!-- To be defined during implementation -->

### 3.7: Tech stack (optional)

<!-- To be defined during implementation -->

## 4. Additional Context

<!-- To be added as needed -->

## 5. Traceability
 
**ADRs**: `cpt-cyberfabric-llm-gateway-adr-file-storage`, `cpt-cyberfabric-llm-gateway-adr-circuit-breaking`
