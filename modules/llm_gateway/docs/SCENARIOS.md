# LLM Gateway Scenarios

## Notation

- `[ ]` — not implemented
- `[x]` — implemented

---

## Content Format

Message content is an array of typed blocks:

```plaintext
content: [
  { type: "text", text: "..." },
  { type: "image", url: "..." },
  { type: "audio", url: "..." },
  { type: "video", url: "..." },
  { type: "document", url: "..." },
  { type: "tool_call", id: "...", schema_id: "...", arguments: {...} },
  { type: "tool_result", tool_call_id: "...", result: {...} }
]
```

| Type | Fields | Direction |
|------|--------|-----------|
| text | text | input/output |
| image | url | input/output |
| audio | url | input/output |
| video | url | input/output |
| document | url | input/output |
| tool_call | id, schema_id, arguments | output |
| tool_result | tool_call_id, result | input |

**Media input**:
- FileStorage URL — Gateway fetches content before sending to provider
- Data URL (`data:<mime>;base64,...`) — Gateway passes directly to provider

**Media output**: Gateway stores via FileStorage, returns URL.

**Tools**: Consumer defines tools via GTS Schema ID. Gateway resolves schema before sending to provider.

---

## Response Format

Gateway fully normalizes `chat_completion` responses to a unified format. Other endpoints (`embed`, `list_models`, `get_job`) have endpoint-specific response structures.

```plaintext
response: {
  content: [...],           // normalized content blocks
  usage: {                  // normalized usage
    prompt_tokens: 100,
    completion_tokens: 50,
    total_tokens: 150
  },
  metadata: {               // normalized metadata
    model: "gpt-4",
    provider: "openai",
    latency_ms: 234,
    request_id: "req_abc123"
  },
  extensions: {             // provider-specific data (optional)
    "openai": { ... },
    "anthropic": { ... }
  }
}
```

### Normalization Guarantees

| Field | Guarantee |
|-------|-----------|
| `content` | Always present, unified block format |
| `usage` | Always present, consistent field names |
| `metadata.model` | Always present, model ID used |
| `metadata.provider` | Always present, identifies source |
| `metadata.latency_ms` | Always present, request duration |
| `metadata.request_id` | Always present, for tracing |
| `metadata.fallback_used` | Optional, true if fallback was triggered |
| `metadata.original_model` | Optional, requested model before fallback |
| `extensions` | Optional, provider-keyed |

### Extensions

Provider-specific data that doesn't fit the normalized schema goes into `extensions`. Consumers should not depend on extensions for core functionality.

**Response extensions** (examples):
```plaintext
extensions: {
  "anthropic": {
    stop_reason: "end_turn",
    cache_creation_input_tokens: 1000,
    cache_read_input_tokens: 500
  },
  "openai": {
    system_fingerprint: "fp_abc123",
    logprobs: [...]
  }
}
```

**Request extensions** (provider hints):
```plaintext
provider_hints: {
  "anthropic": {
    cache_control: { type: "ephemeral" }
  },
  "openai": {
    logprobs: true,
    top_logprobs: 5
  }
}
```

**Principles**:
- Extensions are optional — consumers can ignore them
- Extensions are type-safe — keyed by provider name
- Extensions don't affect Gateway logic — pass-through only
- Model Discovery (S1.12) advertises available extensions per model

---

## Streaming Format

Gateway normalizes streaming events to a unified SSE format.

**Event types**:

| Event | Description |
|-------|-------------|
| `delta` | Content chunk (text, tool_call) |
| `usage` | Final usage metrics (last event before done) |
| `done` | Stream completion |
| `error` | Error occurred |

**Delta event**:
```plaintext
event: delta
data: {
  type: "text_delta" | "tool_call_delta",
  index: 0,           // content block index
  text?: "...",       // for text_delta
  tool_call?: {...}   // for tool_call_delta
}
```

**Usage event** (always emitted before done):
```plaintext
event: usage
data: {
  prompt_tokens: 100,
  completion_tokens: 50,
  total_tokens: 150
}
```

**Done event**:
```plaintext
event: done
data: {
  finish_reason: "stop" | "tool_calls" | "length" | "content_filter",
  metadata: { model: "gpt-4", provider: "openai", latency_ms: 234, request_id: "..." },
  extensions: { ... }
}
```

**Normalization**:
- All providers mapped to same event types
- `tool_calls` always streamed after text (consistent ordering)
- `usage` always in final chunk (not deltas)

---

## Dependencies

| Dependency | Role |
|------------|------|
| FileStorage | Fetch input media by URL, store generated content |
| Credential Resolver | API keys for providers |
| Type Registry | Read GTS schemas by ID (tool definitions) |
| Usage Tracker | Token/cost reporting |
| Hook System | `llm.pre_call`, `llm.post_response` interceptors |
| Audit | Request/response logging |

---

## Design Principles

**Stateless**: Gateway does not store conversation history. Consumer provides full message history with each request. Conversation state is managed by a separate component (Chat Engine). Exception: async job state (ID mappings and cached results for sync providers) is stored temporarily until retrieval/expiry.

**Pass-through**: Gateway normalizes requests/responses but does not interpret content. Tool execution, response parsing — consumer responsibility.

---

## Errors

All errors follow RFC 9457 Problem Details format with GTS error codes.

**Code pattern**: `gts.hx.core.errors.err.v1~hx.llm_gateway.<error>.v1`

| Code | Status | Description |
|------|--------|-------------|
| `model_not_found` | 404 | Requested model not available |
| `job_not_found` | 404 | Async job ID not found |
| `job_expired` | 410 | Async job expired (TTL) |
| `validation_error` | 422 | Invalid request parameters |
| `capability_not_supported` | 422 | Model doesn't support requested capability |
| `schema_validation_failed` | 422 | Response doesn't match requested JSON schema |
| `budget_exceeded` | 429 | Tenant budget exhausted |
| `rate_limited` | 429 | Rate limit exceeded (tenant or user level) |
| `request_blocked` | 403 | Blocked by pre-call interceptor |
| `response_blocked` | 403 | Blocked by post-response interceptor |
| `provider_error` | 502 | Provider returned error |
| `provider_timeout` | 504 | Provider request timed out |

**Response example**:
```json
{
  "type": "https://errors.hyperspot.io/gts.hx.core.errors.err.v1~hx.llm_gateway.rate_limited.v1",
  "title": "Rate Limited",
  "status": 429,
  "code": "gts.hx.core.errors.err.v1~hx.llm_gateway.rate_limited.v1",
  "detail": "User rate limit exceeded",
  "instance": "/api/llm-gateway/v1/chat/completions",
  "trace_id": "abc123"
}
```

---

## P1 — Core

### [ ] S1.1 Chat Completion

Consumer sends chat completion request. Gateway resolves provider based on model, executes request, returns response with usage metrics.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant CR as Credential Resolver
    participant P as Provider

    C->>GW: chat_completion(model, messages, params)
    GW->>GW: Resolve provider for model
    GW->>CR: Get API credentials
    CR-->>GW: Credentials
    GW->>P: Provider API call
    P-->>GW: Response + usage
    GW-->>C: Response + usage
```

---

### [ ] S1.2 Streaming Chat Completion

Same as S1.1, but response is streamed via SSE. Gateway normalizes provider events to unified format (see Streaming Format section).

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider

    C->>GW: chat_completion(stream=true)
    GW->>P: Provider streaming call
    loop Content chunks
        P-->>GW: provider delta
        GW-->>C: SSE: delta (normalized)
    end
    P-->>GW: completion
    GW-->>C: SSE: usage
    GW-->>C: SSE: done
```

**Edge cases**:
- Client disconnect → close provider connection, no persistence (consumer responsibility)
- Provider timeout → emit error event, close stream

---

### [ ] S1.3 Embeddings Generation

Consumer sends text(s), Gateway returns vector embeddings. Supports single text or batch input.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider

    C->>GW: embed(model, input[])
    GW->>P: Provider embeddings API
    P-->>GW: vectors[] + usage
    GW-->>C: vectors[] + usage
```

---

### [ ] S1.4 Vision (Image Analysis)

Consumer uploads images to FileStorage, then sends message with image URLs. Gateway routes to vision-capable model.

**Request example**:
```plaintext
messages: [{
  role: "user",
  content: [
    { type: "text", text: "What's in these images?" },
    { type: "image", url: "https://storage.example.com/img1.jpg" },
    { type: "image", url: "https://storage.example.com/img2.jpg" }
  ]
}]
```

```mermaid
sequenceDiagram
    participant C as Consumer
    participant FS as FileStorage
    participant GW as LLM Gateway
    participant P as Provider

    C->>FS: Upload images
    FS-->>C: URLs
    C->>GW: chat_completion(model, messages with URLs)
    GW->>GW: Validate model supports vision
    GW->>FS: Fetch images by URLs
    FS-->>GW: image content[]
    GW->>P: Request with image content
    P-->>GW: Text response + usage
    GW-->>C: Response + usage
```

**Multiple images**: supported in single message

---

### [ ] S1.5 Image Generation

Consumer sends text prompt to image generation model. Gateway generates image, stores via FileStorage, returns URL.

**Request example**:
```plaintext
model: "dall-e-3"
messages: [{
  role: "user",
  content: [
    { type: "text", text: "A sunset over mountains, oil painting style" }
  ]
}]
params: { size: "1024x1024", quality: "hd" }
```

**Response example**:
```plaintext
content: [
  { type: "image", url: "https://storage.example.com/generated/abc123.png" }
]
usage: { ... }
```

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider
    participant FS as FileStorage

    C->>GW: chat_completion(model, messages, params)
    GW->>GW: Validate model supports image generation
    GW->>P: Generation request
    P-->>GW: Image (base64 or provider URL)
    GW->>FS: Store image
    FS-->>GW: stored_url
    GW-->>C: Response with image URL + usage
```

**Parameters**: size, quality, style, count (via request params)

---

### [ ] S1.6 Speech-to-Text (Transcription)

Consumer uploads audio to FileStorage, then sends message with audio URL. Gateway returns transcribed text.

**Request example**:
```plaintext
messages: [{
  role: "user",
  content: [
    { type: "text", text: "Transcribe this audio" },
    { type: "audio", url: "https://storage.example.com/recording.mp3" }
  ]
}]
```

```mermaid
sequenceDiagram
    participant C as Consumer
    participant FS as FileStorage
    participant GW as LLM Gateway
    participant P as Provider

    C->>FS: Upload audio
    FS-->>C: URL
    C->>GW: chat_completion(model, messages with URL)
    GW->>GW: Validate model supports speech-to-text
    GW->>FS: Fetch audio by URL
    FS-->>GW: audio content
    GW->>P: Audio content
    P-->>GW: Transcription + usage
    GW-->>C: Text response + usage
```

**Parameters**: language hint, timestamps, word-level timing (via request params)

---

### [ ] S1.7 Text-to-Speech (Synthesis)

Consumer sends text to TTS model. Gateway synthesizes audio, stores via FileStorage, returns URL.

**Request example**:
```plaintext
model: "tts-1-hd"
messages: [{
  role: "user",
  content: [
    { type: "text", text: "Hello, welcome to our platform." }
  ]
}]
params: { voice: "alloy", speed: 1.0 }
```

**Response example**:
```plaintext
content: [
  { type: "audio", url: "https://storage.example.com/generated/xyz789.mp3" }
]
usage: { ... }
```

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider
    participant FS as FileStorage

    C->>GW: chat_completion(model, messages, params)
    GW->>GW: Validate model supports text-to-speech
    GW->>P: Text payload
    P-->>GW: Audio (base64 or provider URL)
    GW->>FS: Store audio
    FS-->>GW: stored_url
    GW-->>C: Response with audio URL + usage
```

**Parameters**: voice, speed, pitch (via request params)

**Streaming**: for real-time synthesis, stream audio chunks directly (no storage)

---

### [ ] S1.8 Video Understanding

Consumer uploads video to FileStorage, then sends message with video URL. Gateway returns analysis.

**Request example**:
```plaintext
messages: [{
  role: "user",
  content: [
    { type: "text", text: "Describe what happens in this video" },
    { type: "video", url: "https://storage.example.com/clip.mp4" }
  ]
}]
```

```mermaid
sequenceDiagram
    participant C as Consumer
    participant FS as FileStorage
    participant GW as LLM Gateway
    participant P as Provider

    C->>FS: Upload video
    FS-->>C: URL
    C->>GW: chat_completion(model, messages with URL)
    GW->>GW: Validate model supports video
    GW->>FS: Fetch video by URL
    FS-->>GW: video content
    GW->>P: Request with video content
    P-->>GW: Analysis + usage
    GW-->>C: Text response + usage
```

**Considerations**:
- Provider-specific duration/size limits

---

### [ ] S1.9 Video Generation

Consumer sends text prompt to video generation model. Gateway generates video, stores via FileStorage, returns URL.

**Request example**:
```plaintext
model: "sora-1"
messages: [{
  role: "user",
  content: [
    { type: "text", text: "A cat playing piano in a jazz club" }
  ]
}]
params: { duration: 10, resolution: "1080p" }
```

**Response example**:
```plaintext
content: [
  { type: "video", url: "https://storage.example.com/generated/vid456.mp4" }
]
usage: { ... }
```

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider
    participant FS as FileStorage

    C->>GW: chat_completion(model, messages, params)
    GW->>GW: Validate model supports video generation
    GW->>P: Generation request
    P-->>GW: Video (URL or chunks)
    GW->>FS: Store video
    FS-->>GW: stored_url
    GW-->>C: Response with video URL + usage
```

**Parameters**: duration, resolution, aspect_ratio (via request params)

**Async mode**: Video generation is often long-running. Use `async=true` to get job_id immediately and poll for result (see S1.14 Async Jobs).

---

### [ ] S1.10 Tool/Function Calling

Consumer sends request with tools defined by GTS Schema ID. Gateway resolves schemas, forwards to provider. Model may return tool calls. Consumer executes tools, sends results back.

**Tool Schema ID**: `gts.hx.core.faas.func.v1~<vendor>.<app>.<namespace>.<func_name>.v1`

**Request example**:
```plaintext
tools: [
  { schema_id: "gts.hx.core.faas.func.v1~acme.crm.contacts.search.v1" },
  { schema_id: "gts.hx.core.faas.func.v1~acme.crm.orders.create.v1" }
]
```

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant TR as Type Registry
    participant P as Provider

    C->>GW: chat_completion(model, messages, tools[schema_ids])
    GW->>TR: Get schemas by IDs
    TR-->>GW: GTS schemas
    GW->>GW: Convert to provider format
    GW->>P: Request with provider-specific tools
    P-->>GW: tool_calls[]
    GW-->>C: Response with tool_calls[] (schema_id preserved)
    Note over C: Consumer executes tools
    C->>GW: chat_completion(messages + tool_results)
    GW->>P: Request with tool results
    P-->>GW: Final response
    GW-->>C: Response
```

**Gateway role**:
- Reads schemas from Type Registry
- Converts to provider-specific format (OpenAI functions, Anthropic tools, etc.)
- Does not execute tools — consumer handles execution

---

### [ ] S1.11 Structured Output

Consumer requests JSON output with schema validation. Gateway passes schema to provider, validates response.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider

    C->>GW: chat_completion(model, messages, response_format)
    GW->>P: Request with JSON schema
    P-->>GW: JSON response
    GW->>GW: Validate against schema
    GW-->>C: Validated JSON + usage
```

**response_format**: `{ type: "json_schema", schema: {...} }`

**Validation failure**: Return `schema_validation_failed` with raw response in Problem extensions.

---

### [ ] S1.12 Model Discovery

Consumer queries available models and their capabilities.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway

    C->>GW: list_models(filter?)
    GW->>GW: Aggregate from configured providers
    GW-->>C: models[]
```

**Model info**:
- id, name, provider
- capabilities: chat, vision, audio, video, embeddings, tools, json_mode
- context_window, max_output_tokens
- pricing (per 1M tokens)
- supported_extensions: provider-specific features available for this model

**Example**:
```plaintext
{
  id: "claude-3-5-sonnet",
  name: "Claude 3.5 Sonnet",
  provider: "anthropic",
  capabilities: ["chat", "vision", "tools", "json_mode"],
  context_window: 200000,
  max_output_tokens: 8192,
  pricing: { prompt: 3.0, completion: 15.0 },
  supported_extensions: {
    "anthropic": ["cache_control", "extended_thinking"]
  }
}
```

---

### [ ] S1.13 Document Understanding

Consumer uploads document to FileStorage, sends URL. Gateway fetches document, passes to capable model.

**Supported formats**: Provider-dependent (typically PDF, images)

```mermaid
sequenceDiagram
    participant C as Consumer
    participant FS as FileStorage
    participant GW as LLM Gateway
    participant P as Provider

    C->>FS: Upload document
    FS-->>C: URL
    C->>GW: chat_completion(model, messages with document URL)
    GW->>GW: Validate model supports documents
    GW->>FS: Fetch document
    FS-->>GW: document content
    GW->>P: Request with document content
    P-->>GW: Analysis + usage
    GW-->>C: Response + usage
```

---

### [ ] S1.14 Async Jobs

Consumer explicitly requests async mode via `async=true`. Gateway abstracts provider behavior — consumer always gets consistent response type.

**Behavior matrix**:

| Request | Provider | Gateway action |
|---------|----------|----------------|
| `async=false` (default) | sync | Return result immediately |
| `async=false` (default) | async | Poll internally, return result when done |
| `async=true` | sync | Simulate job (execute, store result, return job_id) |
| `async=true` | async | Return job_id, consumer polls |

**Request example**:
```plaintext
chat_completion(model, messages, params, async=true)
```

**Async response** (always when `async=true`):
```plaintext
{ job_id: "job_abc123", status: "pending" }
```

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider
    participant FS as FileStorage

    C->>GW: chat_completion(..., async=true)
    GW->>P: Start generation
    alt Provider is async
        P-->>GW: provider_job_id
        GW->>GW: Store mapping (gw_job_id → provider_job_id)
    else Provider is sync
        P-->>GW: Result
        GW->>GW: Store result with gw_job_id
    end
    GW-->>C: { job_id: gw_job_id, status: "pending" }

    C->>GW: get_job(gw_job_id)
    alt Provider is async
        GW->>P: Check status
        P-->>GW: status / result
    else Provider is sync (result cached)
        GW->>GW: Retrieve stored result
    end
    GW-->>C: { status: "completed", content: [...] }
```

**Job states**: pending → processing → completed | failed

**Storage**: Gateway stores job mapping (gw_job_id → provider_job_id or cached result).

**Cleanup**: Jobs expire after configurable TTL.

**Use cases**:
- Video generation (long-running)
- Large batch operations
- Fire-and-forget with later retrieval

---

### [ ] S1.15 Realtime Audio

Bidirectional audio streaming via WebSocket for voice conversations.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider

    C->>GW: WebSocket connect
    GW->>P: WebSocket connect

    loop Session
        C->>GW: audio chunk
        GW->>P: audio chunk
        P-->>GW: audio chunk / text
        GW-->>C: audio chunk / text
    end

    C->>GW: close
    GW->>P: close
    GW-->>C: usage summary
```

**WebSocket events** (different from SSE streaming):

| Event | Direction | Description |
|-------|-----------|-------------|
| `audio_delta` | bidirectional | Audio chunk |
| `text_delta` | server→client | Transcribed/generated text |
| `function_call` | server→client | Tool invocation request |
| `session_end` | server→client | Session completed with usage |

**Use case**: Voice assistants, real-time translation

---

### [ ] S1.16 Usage Tracking

Gateway reports usage metrics to Usage Tracker after each request.

**Reported**: tokens (prompt, completion), cost estimate, latency, provider, model

**Attribution**: tenant_id, user_id, conversation_id, model_id

---

## P2 — Reliability & Governance

### [ ] S2.1 Provider Fallback

When primary provider fails, Gateway automatically switches to next provider in fallback chain.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P1 as Primary Provider
    participant P2 as Fallback Provider

    C->>GW: chat_completion(model)
    GW->>P1: Request
    P1-->>GW: 5xx / timeout
    GW->>GW: Select fallback (capability match)
    GW->>P2: Request
    P2-->>GW: Response
    GW-->>C: Response (metadata.fallback_used: true)
```

**Response metadata** (when fallback triggered):
- `metadata.fallback_used: true`
- `metadata.original_model: "gpt-4o"` (requested model)
- `metadata.model: "claude-3-5-sonnet"` (actual model used)
- `metadata.provider: "anthropic"` (actual provider used)

**Trigger conditions**:
- HTTP 5xx
- Connection timeout
- Rate limit (429) with exhausted retry budget

---

### [ ] S2.2 Retry with Backoff

Transient failures trigger automatic retry with exponential backoff before fallback.

```mermaid
sequenceDiagram
    participant GW as LLM Gateway
    participant P as Provider

    GW->>P: Request
    P-->>GW: 503
    GW->>GW: Wait (backoff)
    GW->>P: Retry #1
    P-->>GW: 503
    GW->>GW: Wait (backoff * multiplier)
    GW->>P: Retry #2
    P-->>GW: 200 OK
```

**Retryable**: 429, 5xx
**Non-retryable**: 4xx (except 429)

---

### [ ] S2.3 Timeout Enforcement

Gateway enforces connection, read, and total request timeouts. On timeout → retry → fallback → error.

---

### [ ] S2.4 Pre-Call Interceptor

Before sending request to provider, Gateway invokes registered hooks. Hooks can allow, block, or modify the request.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant HK as Hook Endpoint
    participant P as Provider

    C->>GW: chat_completion(...)
    GW->>HK: pre_call hook
    alt BLOCK
        HK-->>GW: block + reason
        GW-->>C: request_blocked
    else OVERRIDE
        HK-->>GW: modified request
        GW->>P: Modified request
    else ALLOW
        HK-->>GW: allow
        GW->>P: Original request
    end
```

---

### [ ] S2.5 Post-Response Interceptor

After receiving provider response, Gateway invokes hooks. Hooks can allow, block, or modify the response.

```mermaid
sequenceDiagram
    participant GW as LLM Gateway
    participant P as Provider
    participant HK as Hook Endpoint
    participant C as Consumer

    GW->>P: Request
    P-->>GW: Response
    GW->>HK: post_response hook
    alt BLOCK
        HK-->>GW: block + reason
        GW-->>C: response_blocked
    else OVERRIDE
        HK-->>GW: modified response
        GW-->>C: Modified response
    else ALLOW
        HK-->>GW: allow
        GW-->>C: Original response
    end
```

---

### [ ] S2.6 Per-Tenant Budget Enforcement

Gateway checks tenant budget before execution. Rejects if budget exhausted, deducts actual usage after completion.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant UT as Usage Tracker
    participant P as Provider

    C->>GW: chat_completion(...)
    GW->>UT: Check budget (tenant_id, estimated_tokens)
    alt Budget exhausted
        UT-->>GW: DENIED
        GW-->>C: budget_exceeded
    else Budget available
        UT-->>GW: OK
        GW->>P: Request
        P-->>GW: Response + usage
        GW->>UT: Deduct usage (actual_tokens)
        GW-->>C: Response
    end
```

---

### [ ] S2.7 Request Cancellation

Consumer cancels in-progress request. Gateway propagates cancellation to provider, cleans up resources.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider

    C->>GW: chat_completion(request_id)
    GW->>P: Request
    C->>GW: cancel(request_id)
    GW->>P: Cancel request
    P-->>GW: cancelled
    GW-->>C: { status: "cancelled" }
```

**Streaming**: Close SSE connection, emit cancellation event.

**Async jobs**: Mark job as cancelled, stop polling provider.

**Partial results**: Optionally return partial response if available.

---

### [ ] S2.8 Rate Limiting

Gateway enforces rate limits at multiple levels. Rejects requests exceeding any limit.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant GW as LLM Gateway
    participant P as Provider

    C->>GW: chat_completion(...)
    GW->>GW: Check tenant limit
    GW->>GW: Check user limit
    alt Any limit exceeded
        GW-->>C: rate_limited
    else All limits OK
        GW->>P: Request
        P-->>GW: Response
        GW-->>C: Response
    end
```

**Levels** (checked in order):

| Level | Key | Description |
|-------|-----|-------------|
| Tenant | tenant_id | Aggregate limit for all users in tenant |
| User | tenant_id + user_id | Per-user limit within tenant |

**Limit types**:
- Requests per second (RPS)
- Concurrent requests
- Tokens per minute (TPM)

**Problem extensions**: `level` (tenant|user), `retry_after` (seconds)

**Headers**: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`, `Retry-After`

---

## P3 — Optimization

### [ ] S3.1 Cost-Aware Routing

When multiple providers can serve request, Gateway selects based on cost/latency score.

**Factors**:
- Model pricing (per token)
- Current provider latency (from metrics)
- Required capabilities filter

**Selection**: lowest weighted score among capable providers.

---

### [ ] S3.2 Embeddings Batching

Gateway collects embedding requests within time window, batches to single provider call.

**Constraints**:
- No cross-tenant batching
- Partial failure → return error per failed item

---

## P4 — Enterprise

### [ ] S4.1 Audit Events

Gateway emits audit events for compliance tracking.

| Event | Trigger |
|-------|---------|
| llm.request.started | Request initiated |
| llm.request.completed | Request succeeded |
| llm.request.failed | Request failed |
| llm.request.blocked | Blocked by interceptor |
| llm.fallback.triggered | Fallback activated |

**Content policy**: full | redacted | metadata_only (per tenant)
