````markdown
# Distributed Tracing Setup

This guide walks you through setting up **OpenTelemetry distributed tracing** with **Jaeger** or **Uptrace** for the
hyperspot framework for testing purposes.

## Overview

The hyperspot framework includes first-class support for distributed tracing with:

- **Automatic trace context extraction** from incoming HTTP requests (W3C Trace Context)
- **Automatic trace context injection** for outgoing HTTP requests
- **Centralized configuration** via YAML
- **TracedClient** for instrumented HTTP calls
- **Integration with existing logging**

## Quick Start with Jaeger

### 1. Start Jaeger (Local Development)

```bash
# Start Jaeger All-in-One with OTLP support
docker run -d --name jaeger \
  -p 16686:16686 \    # UI: http://localhost:16686
  -p 4317:4317 \      # OTLP gRPC
  -p 4318:4318 \      # OTLP HTTP
  -e COLLECTOR_OTLP_ENABLED=true \
  jaegertracing/all-in-one:latest
````

### 2. Configure Tracing

Create a configuration file (e.g., `config/with-tracing.yaml`):

```yaml
server:
  home_dir: "~/.hyperspot"
  host: "127.0.0.1"
  port: 8087

# Enable OpenTelemetry tracing
tracing:
  enabled: true
  service_name: "hyperspot-api"

  exporter:
    kind: "otlp_grpc"
    endpoint: "http://127.0.0.1:4317"
    timeout_ms: 5000

  sampler:
    strategy: "parentbased_ratio"
    ratio: 0.1  # Sample 10% of traces

  propagation:
    w3c_trace_context: true

  resource:
    service.version: "1.0.0"
    deployment.environment: "dev"

logging:
  default:
    console_level: "info"
    file: "logs/hyperspot.log"
```

### 3. Run the Server

```bash
cargo run --bin hyperspot-server -- --config config/with-tracing.yaml
```

### 4. View Traces

Open [http://localhost:16686](http://localhost:16686) and search for service `hyperspot-api`.

---

## Quick Start with Uptrace

[Uptrace](https://uptrace.dev) is a modern tracing UI that works with OpenTelemetry and ClickHouse/Postgres.

### 1. Start Uptrace (Docker Compose)

```yaml
services:
  uptrace:
    image: uptrace/uptrace:2.0.1
    ports:
      - "14318:80"     # Web UI: http://localhost:14318
      - "14317:4317"   # OTLP gRPC
      - "14319:4318"   # OTLP HTTP
    volumes:
      - ./uptrace.yml:/etc/uptrace/config.yml
    depends_on:
      - clickhouse
      - postgres
      - redis

  clickhouse:
    image: clickhouse/clickhouse-server:25.8
    ports: [ "9000:9000" ]

  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: uptrace
      POSTGRES_USER: uptrace
      POSTGRES_PASSWORD: uptrace

  redis:
    image: redis:8.2
```

### 2. Configure Tracing with Uptrace DSN

```yaml
tracing:
  enabled: true
  service_name: "hyperspot-api"

  exporter:
    kind: "otlp_grpc"
    endpoint: "http://127.0.0.1:14317"
    timeout_ms: 5000
    headers:
      uptrace-dsn: "http://project1_secret@localhost:14318?grpc=14317"

  sampler:
    strategy: "always_on"

  resource:
    service.version: "1.3.7"
    deployment.environment: "dev"
    service.namespace: "hyperspot"
```

### 3. Run the Server

```bash
cargo run --bin hyperspot-server -- --config config/with-tracing.yaml
```

### 4. View Traces

Open [http://localhost:14318](http://localhost:14318) and search for service `hyperspot-api`.

---

## Configuration Reference

### Basic Configuration

```yaml
tracing:
  enabled: true                    # Enable/disable tracing
  service_name: "my-service"       # Service name in traces
```

### Exporter Configuration

#### OTLP gRPC (Default)

```yaml
tracing:
  exporter:
    kind: "otlp_grpc"
    endpoint: "http://127.0.0.1:4317"
    timeout_ms: 5000
    headers: # Optional auth headers
      authorization: "Bearer token"
```

#### OTLP HTTP

```yaml
tracing:
  exporter:
    kind: "otlp_http"
    endpoint: "http://127.0.0.1:4318/v1/traces"
    timeout_ms: 5000
```

---

(rest of your original doc unchanged below)

```

### Sampling Strategies

#### Always Sample
```yaml
tracing:
  sampler:
    strategy: "always_on"
```

#### Never Sample

```yaml
tracing:
  sampler:
    strategy: "always_off"
```

#### Ratio-Based Sampling

```yaml
tracing:
  sampler:
    strategy: "parentbased_ratio"
    ratio: 0.1  # Sample 10% of traces
```

#### Simple Ratio (No Parent Context)

```yaml
tracing:
  sampler:
    strategy: "ratio"
    ratio: 0.05  # Sample 5% of traces
```

### Resource Attributes

Add metadata to all spans:

```yaml
tracing:
  resource:
    service.version: "1.2.3"
    deployment.environment: "production"
    service.namespace: "hyperspot"
    k8s.cluster.name: "prod-cluster"
    k8s.namespace.name: "hyperspot-ns"
```

### HTTP Options

```yaml
tracing:
  http:
    inject_request_id_header: "x-request-id"
    record_headers:
      - "user-agent"
      - "x-forwarded-for"
      - "authorization"  # Be careful with sensitive headers
```

## Using TracedClient

### In Your Module

```rust
use modkit::TracedClient;

#[async_trait]
impl MyModule {
    async fn call_external_api(&self) -> Result<String> {
        let client = TracedClient::default();

        // Trace context is automatically injected
        let response = client
            .get("https://api.example.com/data")
            .await?;

        let data = response.text().await?;
        Ok(data)
    }
}
```

### Converting Existing reqwest::Client

```rust
use modkit::TracedClient;

let reqwest_client = reqwest::Client::new();
let traced_client = TracedClient::from(reqwest_client);

// Or using Into trait
let traced_client: TracedClient = reqwest_client.into();
```

### Advanced Usage

```rust
use modkit::TracedClient;

let client = TracedClient::default ();

// Build custom requests
let request = client.inner()
.post("https://api.example.com/upload")
.json( & my_data)
.build() ?;

// Execute with tracing
let response = client.execute(request).await?;
```

## Manual Span Creation

Create custom spans for business logic:

```rust
use tracing::{info_span, Instrument};

async fn process_user_data(user_id: u64) -> Result<()> {
    // Create a span for this operation
    let span = info_span!("process_user", user.id = user_id);

    async {
        // Your business logic here
        info!("Processing user {}", user_id);

        // Child operations will be traced automatically
        let client = TracedClient::default();
        let user_data = client.get(&format!("https://api.example.com/users/{}", user_id)).await?;

        Ok(())
    }.instrument(span).await
}
```

## Production Deployment

### Docker Compose with Jaeger

```yaml
  services:
    jaeger:
      image: ${REGISTRY:-}jaegertracing/jaeger:${JAEGER_VERSION:-latest}
      ports:
        - "16686:16686"
        - "4317:4317"
        - "4318:4318"
      environment:
        - LOG_LEVEL=debug
        - COLLECTOR_OTLP_ENABLED=true
      networks:
        - jaeger-example

  networks:
    jaeger-example:
```

### Environment Variable Overrides

You can override any config via environment variables:

```bash
# Enable tracing
export APP__TRACING__ENABLED=true
export APP__TRACING__SERVICE_NAME=hyperspot-prod

# Configure exporter
export APP__TRACING__EXPORTER__KIND=otlp_grpc
export APP__TRACING__EXPORTER__ENDPOINT=http://jaeger:4317

# Configure sampling
export APP__TRACING__SAMPLER__STRATEGY=parentbased_ratio
export APP__TRACING__SAMPLER__RATIO=0.01  # 1% sampling in prod
```

## Troubleshooting

### No Traces Appearing

1. **Check Jaeger is running**: Visit http://localhost:16686
2. **Verify endpoint**: Ensure `exporter.endpoint` matches Jaeger's OTLP port
3. **Check sampling**: Set `sampler.strategy: "always_on"` for testing
4. **View logs**: Look for "OpenTelemetry tracing initialized" message

### Performance Impact

1. **Use sampling in production**: Set appropriate `ratio` (0.01 = 1%)
2. **Monitor resource usage**: Tracing adds some CPU/memory overhead
3. **Batch export**: The framework uses batched export by default

### Trace Context Not Propagating

1. **Check headers**: Ensure upstream sends `traceparent` header
2. **Verify propagation**: Set `propagation.w3c_trace_context: true`
3. **Use TracedClient**: Ensure outgoing calls use `TracedClient`

## Observability Best Practices

### Structured Attributes

Use consistent attribute names:

```rust
tracing::info_span!(
    "user_operation",
    user.id = user_id,
    user.email = %user_email,
    operation.type = "create",
    operation.result = "success"
)
```

### Error Handling

Mark spans with errors:

```rust
let span = tracing::info_span!("risky_operation");
let _guard = span.enter();

match risky_operation().await {
Ok(result) => {
span.record("operation.result", "success");
Ok(result)
}
Err(e) => {
span.record("error", true);
span.record("error.message", % e);
span.record("operation.result", "error");
Err(e)
}
}
```
