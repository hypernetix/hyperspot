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
| [ ] | Usage tracking |

### P4 — Enterprise

| Done | Capability |
|------|------------|
| [ ] | Audit events |

## Module Structure

```
modules/llm_gateway/
├── llm_gateway-sdk/         # Public API traits, models, errors
├── llm_gateway-gw/          # Gateway implementation
└── plugins/
    ├── openai_plugin/       # OpenAI-compatible providers
    ├── anthropic_plugin/    # Claude API
    └── ollama_plugin/       # Local models via Ollama
```

## Documentation

- [SCENARIOS.md](docs/SCENARIOS.md) — Usage scenarios with sequence diagrams
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) — Gateway + Plugin design, component interactions
- [API.md](docs/API.md) — SDK traits, request/response models, errors
- [PROVIDERS.md](docs/PROVIDERS.md) — Provider abstraction, capability matrix
- [CONFIGURATION.md](docs/CONFIGURATION.md) — Gateway and plugin configuration

## Dependencies

| Module | Role |
|--------|------|
| FileStorage | Fetch input media, store generated content |
| Credential Resolver | API key management |
| Type Registry | Read GTS schemas by ID (tool definitions) |
| Usage Tracker | Token/cost reporting |
| Hook System | `llm.pre_call`, `llm.post_response` interceptors |
| Audit | Request/response logging |

## Consumers

| Module | Usage |
|--------|-------|
| Chat Engine | Response generation |
| RAG | Embeddings for semantic search |
