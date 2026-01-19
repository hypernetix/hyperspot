# LLM Gateway

Unified interface for LLM inference across providers. Stateless, pass-through design.

## Capabilities

### P1 — Core

| Done | Capability |
|------|------------|
| [ ] | Chat completion (sync and streaming) |
| [ ] | Embeddings generation |
| [ ] | Vision (image analysis) |
| [ ] | Image generation |
| [ ] | Speech-to-text (transcription) |
| [ ] | Text-to-speech (synthesis) |
| [ ] | Video understanding |
| [ ] | Video generation |
| [ ] | Document understanding |
| [ ] | Tool/function calling |
| [ ] | Structured output (JSON mode) |
| [ ] | Model discovery |
| [ ] | Async jobs (long-running operations) |
| [ ] | Realtime audio (WebSocket) |
| [ ] | Usage tracking |

### P2 — Reliability & Governance

| Done | Capability |
|------|------------|
| [ ] | Provider fallback |
| [ ] | Retry with backoff |
| [ ] | Timeout enforcement |
| [ ] | Pre-call interceptor |
| [ ] | Post-response interceptor |
| [ ] | Per-tenant budget enforcement |
| [ ] | Request cancellation |
| [ ] | Rate limiting (tenant/user) |

### P3 — Optimization

| Done | Capability |
|------|------------|
| [ ] | Cost-aware routing |
| [ ] | Embeddings batching |

### P4 — Enterprise

| Done | Capability |
|------|------------|
| [ ] | Audit events |

## Module Structure

```plaintext
modules/llm_gateway/
├── llm_gateway-sdk/         # Public API traits, models, errors
├── llm_gateway-gw/          # Gateway implementation
└── plugins/
    ├── providers/
    │   ├── openai_plugin/       # OpenAI-compatible providers
    │   ├── anthropic_plugin/    # Claude API
    │   └── ollama_plugin/       # Local models via Ollama
    ├── hooks/
    │   ├── noop_hook_plugin/    # Default no-op (passthrough)
    │   └── ...                  # Custom hook plugins
    ├── usage/
    │   ├── noop_usage_plugin/   # Default no-op
    │   └── ...                  # Custom usage tracking
    └── audit/
        ├── noop_audit_plugin/   # Default no-op
        └── ...                  # Custom audit logging
```

## Documentation

- [PRD.md](docs/PRD.md) — Product requirements, scenarios with sequence diagrams
- ARCHITECTURE.md — Gateway + Plugin design, component interactions `TODO`
- API.md — SDK traits, request/response models, errors `TODO`
- PROVIDERS.md — Provider abstraction, capability matrix `TODO`
- CONFIGURATION.md — Gateway and plugin configuration `TODO`

## Dependencies

| Module | Role |
|--------|------|
| Model Registry | Model catalog, availability checks |
| Outbound API Gateway | External API calls to providers |
| FileStorage | Fetch input media, store generated content |
| Type Registry | Read GTS schemas by ID (tool definitions) |

## Consumers

| Module | Usage |
|--------|-------|
| Chat Engine | Response generation |
| RAG | Embeddings for semantic search |
