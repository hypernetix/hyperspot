# LLM Gateway

Unified interface for LLM inference across providers. Stateless, pass-through design.

## Capabilities

### P1 — Core

- [ ] Chat completion (sync and streaming)
- [ ] Embeddings generation
- [ ] Vision (image analysis)
- [ ] Image generation
- [ ] Speech-to-text (transcription)
- [ ] Text-to-speech (synthesis)
- [ ] Video understanding
- [ ] Video generation
- [ ] Document understanding
- [ ] Tool/function calling
- [ ] Structured output (JSON mode)
- [ ] Async jobs (long-running operations)
- [ ] Realtime audio (WebSocket)
- [ ] Usage tracking

### P2 — Reliability & Governance

- [ ] Provider fallback
- [ ] Timeout enforcement
- [ ] Pre-call interceptor
- [ ] Post-response interceptor
- [ ] Per-tenant budget enforcement
- [ ] Rate limiting (tenant/user)

### P3 — Optimization

- [ ] Batch processing

### P4 — Enterprise

- [ ] Audit events

## Module Structure

```plaintext
modules/llm-gateway/
├── docs/                    # Documentation
│   ├── PRD.md
│   ├── DESIGN.md
│   └── ADR/
├── llm-gateway-sdk/         # Public API traits, models, errors
│   └── schemas/             # GTS domain model schemas
├── llm-gateway-gw/          # Gateway implementation (planned)
└── plugins/                 # (planned)
    ├── providers/
    │   ├── openai_plugin/       # OpenAI-compatible providers
    │   ├── anthropic_plugin/    # Claude API
    │   └── ollama_plugin/       # Local models via Ollama
    ├── hooks/
    │   ├── noop_hook_plugin/    # Default no-op (passthrough)
    │   └── ...                  # Custom hook plugins
    ├── usage/
    │   ├── noop_usage_tracker/  # Default no-op
    │   └── ...                  # Custom usage tracking
    └── audit/
        ├── noop_audit_module/   # Default no-op
        └── ...                  # Custom audit logging
```

## Documentation

- [PRD.md](docs/PRD.md) — Product requirements, use cases, acceptance criteria
- [DESIGN.md](docs/DESIGN.md) — Technical architecture, components, sequence diagrams
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
