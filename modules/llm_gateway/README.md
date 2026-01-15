# LLM Gateway

Unified interface for LLM inference and embedding generation across providers.

## Capabilities

| Done | Priority | Capability |
|------|----------|------------|
| [ ] | p1 | Completion/chat requests routed to configured provider |
| [ ] | p1 | Embeddings generation for downstream RAG/search |
| [ ] | p2 | Provider routing, fallbacks, retries, and timeouts |
| [ ] | p2 | Request/response interceptors for policy, redaction, safety |
| [ ] | p2 | Per-tenant budgets |
| [ ] | p3 | Cost/latency-aware routing and batching |
| [ ] | p3 | Usage tracking |
| [ ] | p4 | Audit integration |

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

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) — Gateway + Plugin design, component interactions
- [API.md](docs/API.md) — SDK traits, request/response models, errors
- [PROVIDERS.md](docs/PROVIDERS.md) — Provider abstraction, capability matrix
- [SCENARIOS.md](docs/SCENARIOS.md) — Usage scenarios with sequence diagrams
- [CONFIGURATION.md](docs/CONFIGURATION.md) — Gateway and plugin configuration

## Dependencies

| Module | Role |
|--------|------|
| Chat Engine | Consumer — response generation |
| RAG | Consumer — embeddings for semantic search |
| Hook System | Integration — `llm.pre_call`, `llm.post_response` |
| Usage Tracker | Integration — token consumption reporting |
| Credential Resolver | Integration — API key management |
| Audit | Integration — request/response logging |
