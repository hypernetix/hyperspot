# Outbound API Gateway — PRD

Centralized gateway for external API calls with credentials, reliability, and observability.

## Scenarios

### S1 HTTP Request

```mermaid
sequenceDiagram
    participant C as Consumer
    participant OB as Outbound API Gateway
    participant API as External API

    C->>OB: request(target, method, path, body)
    OB->>OB: Resolve credentials
    OB->>API: HTTP request
    API-->>OB: Response
    OB-->>C: Response
```

### S2 Streaming (SSE)

```mermaid
sequenceDiagram
    participant C as Consumer
    participant OB as Outbound API Gateway
    participant API as External API

    C->>OB: stream_request(target, ...)
    OB->>API: HTTP request
    loop SSE events
        API-->>OB: event
        OB-->>C: event
    end
```

### S3 WebSocket

```mermaid
sequenceDiagram
    participant C as Consumer
    participant OB as Outbound API Gateway
    participant API as External API

    C->>OB: websocket_connect(target, ...)
    OB->>API: WebSocket upgrade
    loop Bidirectional
        C->>OB: message
        OB->>API: message
        API-->>OB: message
        OB-->>C: message
    end
```

## Features

| Feature | Description |
|---------|-------------|
| **Credentials** | Bearer, API key, Basic, OAuth2 |
| **Retry** | Exponential backoff for 429, 5xx |
| **Circuit Breaker** | Fail fast when target is down |
| **Rate Limiting** | Per-target outbound limits |
| **Timeouts** | Connect, read, total |

## Dependencies

| Module | Role |
|--------|------|
| Credential Resolver | API keys, tokens |

## Errors

| Error | HTTP | Description |
|-------|------|-------------|
| `target_not_found` | 404 | Unknown target |
| `connection_timeout` | 504 | Connection timeout |
| `circuit_open` | 503 | Circuit breaker open |
| `rate_limited` | 429 | Outbound rate limit |
