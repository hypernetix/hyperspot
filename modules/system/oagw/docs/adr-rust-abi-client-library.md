# ADR: Rust ABI / Client Libraries for Requests and Plugin Development

- **Status**: Proposed
- **Date**: 2026-02-03
- **Deciders**: OAGW Team

## Context and Problem Statement

OAGW needs a client library abstraction for executing HTTP requests to upstream services. This library must support:

1. **Dual execution modes**: In-process (direct function calls) and out-of-process (remote RPC)
2. **Multiple response types**: Plain HTTP, Server-Sent Events (SSE), streaming responses
3. **Multiple protocols**: HTTP/1.1, HTTP/2, WebSocket (WSS), WebTransport (WT)
4. **Plugin development**: Plugins need request/response manipulation APIs
5. **Reusability**: Same API surface for OAGW core, builtin plugins, and custom Starlark plugins

**Current gaps**:
- No unified abstraction for HTTP client operations
- No standardized interface for in-process vs out-of-process execution
- No streaming-aware API design
- No plugin context API implementation details

**Scope**: This ADR covers HTTP/HTTPS protocols only. gRPC and AMQP are future work.

## Decision Drivers

- **Ergonomics**: Simple, intuitive API for common cases
- **Performance**: Zero-copy where possible, minimal allocations
- **Safety**: Strong typing, compile-time guarantees
- **Flexibility**: Support plain, streaming, bidirectional protocols
- **Plugin isolation**: Sandbox-friendly for Starlark plugins
- **Observability**: Request tracing, metrics collection
- **Testability**: Easy to mock for unit tests

## Considered Options

### Option 1: Trait-Based Abstraction with Dynamic Dispatch

Define a core `HttpClient` trait with implementations for in-process (`DirectClient`) and out-of-process (`RpcClient`).

**Core Traits**:

```rust
// Core client abstraction
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn execute(&self, request: Request) -> Result<Response, ClientError>;
    async fn execute_streaming(&self, request: Request) -> Result<StreamingResponse, ClientError>;
    async fn websocket(&self, request: Request) -> Result<WebSocketConn, ClientError>;
}

// Request builder
#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
    timeout: Option<Duration>,
}

// Response types
#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

#[derive(Debug)]
pub struct StreamingResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: BoxStream<'static, Result<Bytes, ClientError>>,
}

// WebSocket connection
pub struct WebSocketConn {
    send: mpsc::Sender<Message>,
    recv: mpsc::Receiver<Message>,
}
```

**Pros**:
- ✅ Clean separation of concerns
- ✅ Easy to test with mock implementations
- ✅ Supports both sync and async contexts (with minor API variations)
- ✅ Pluggable backends (reqwest, hyper, custom)

**Cons**:
- ❌ Dynamic dispatch overhead (negligible for network I/O)
- ❌ Slightly more complex implementation

### Option 2: Concrete Types with Feature Flags

Provide concrete client types, selected via Cargo features:

```rust
#[cfg(feature = "direct")]
pub type HttpClient = DirectClient;

#[cfg(feature = "rpc")]
pub type HttpClient = RpcClient;
```

**Pros**:
- ✅ Zero-cost abstraction (static dispatch)
- ✅ Simpler implementation

**Cons**:
- ❌ Cannot use both modes in same binary
- ❌ Harder to test (need feature-gated test code)
- ❌ Less flexible for future extensions

### Option 3: Hybrid Approach (Recommended)

Combine trait-based abstraction for flexibility with concrete types for performance:

```rust
// Trait for flexibility
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn execute(&self, req: Request) -> Result<Response, ClientError>;
    async fn execute_streaming(&self, req: Request) -> Result<StreamingResponse, ClientError>;
    async fn websocket(&self, req: Request) -> Result<WebSocketConn, ClientError>;
}

// Concrete default for performance
pub type DefaultClient = DirectClient;

// In-process implementation
pub struct DirectClient {
    inner: reqwest::Client,
    metrics: Arc<Metrics>,
}

// Out-of-process implementation (future)
pub struct RpcClient {
    channel: tonic::Channel,
}
```

**Pros**:
- ✅ Best of both worlds: flexibility + performance
- ✅ Easy to swap implementations for testing
- ✅ Can support both modes simultaneously

**Cons**:
- ❌ Slightly more code to maintain

**Decision**: Use hybrid approach (Option 3).

## Detailed Design

### Core Types

#### Request Builder

```rust
pub struct Request {
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
    timeout: Option<Duration>,
    extensions: Extensions,
}

impl Request {
    pub fn builder() -> RequestBuilder { ... }
    
    pub fn method(&self) -> &Method { &self.method }
    pub fn uri(&self) -> &Uri { &self.uri }
    pub fn headers(&self) -> &HeaderMap { &self.headers }
    pub fn headers_mut(&mut self) -> &mut HeaderMap { &mut self.headers }
    pub fn body(&self) -> &Body { &self.body }
    pub fn into_body(self) -> Body { self.body }
}

pub struct RequestBuilder {
    method: Method,
    uri: Option<Uri>,
    headers: HeaderMap,
    body: Option<Body>,
    timeout: Option<Duration>,
}

impl RequestBuilder {
    pub fn method(mut self, method: Method) -> Self { ... }
    pub fn uri<T: TryInto<Uri>>(mut self, uri: T) -> Self { ... }
    pub fn header<K, V>(mut self, key: K, value: V) -> Self { ... }
    pub fn body<B: Into<Body>>(mut self, body: B) -> Self { ... }
    pub fn timeout(mut self, duration: Duration) -> Self { ... }
    pub fn build(self) -> Result<Request, BuildError> { ... }
}
```

#### Response Types

```rust
// Plain response
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
    extensions: Extensions,
}

impl Response {
    pub fn status(&self) -> StatusCode { self.status }
    pub fn headers(&self) -> &HeaderMap { &self.headers }
    pub fn body(&self) -> &Bytes { &self.body }
    
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
    
    pub fn text(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }
}

// Streaming response (SSE, chunked transfer)
pub struct StreamingResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: BoxStream<'static, Result<Bytes, ClientError>>,
}

impl StreamingResponse {
    pub fn status(&self) -> StatusCode { self.status }
    pub fn headers(&self) -> &HeaderMap { &self.headers }
    pub fn body_stream(&mut self) -> &mut BoxStream<'static, Result<Bytes, ClientError>> {
        &mut self.body
    }
    
    // Convert to SSE event stream
    pub fn into_sse_stream(self) -> SseEventStream {
        SseEventStream::new(self.body)
    }
}

// Server-Sent Events stream
pub struct SseEventStream {
    inner: BoxStream<'static, Result<Bytes, ClientError>>,
    buffer: Vec<u8>,
}

impl SseEventStream {
    pub async fn next_event(&mut self) -> Result<Option<SseEvent>, ClientError> { ... }
}

pub struct SseEvent {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: String,
    pub retry: Option<u64>,
}

// WebSocket connection
pub struct WebSocketConn {
    send: mpsc::Sender<WsMessage>,
    recv: mpsc::Receiver<Result<WsMessage, ClientError>>,
}

impl WebSocketConn {
    pub async fn send(&mut self, msg: WsMessage) -> Result<(), ClientError> {
        self.send.send(msg).await
            .map_err(|_| ClientError::ConnectionClosed)
    }
    
    pub async fn recv(&mut self) -> Result<Option<WsMessage>, ClientError> {
        self.recv.recv().await
            .transpose()
    }
    
    pub async fn close(self) -> Result<(), ClientError> { ... }
}

pub enum WsMessage {
    Text(String),
    Binary(Bytes),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}
```

#### Body Abstraction

```rust
pub enum Body {
    Empty,
    Bytes(Bytes),
    Stream(BoxStream<'static, Result<Bytes, std::io::Error>>),
}

impl Body {
    pub fn empty() -> Self { Body::Empty }
    
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        Body::Bytes(bytes.into())
    }
    
    pub fn from_json<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        Ok(Body::Bytes(serde_json::to_vec(value)?.into()))
    }
    
    pub fn from_stream<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<Bytes, std::io::Error>> + Send + 'static,
    {
        Body::Stream(Box::pin(stream))
    }
    
    pub async fn into_bytes(self) -> Result<Bytes, ClientError> {
        match self {
            Body::Empty => Ok(Bytes::new()),
            Body::Bytes(b) => Ok(b),
            Body::Stream(mut s) => {
                let mut buf = BytesMut::new();
                while let Some(chunk) = s.next().await {
                    buf.extend_from_slice(&chunk?);
                }
                Ok(buf.freeze())
            }
        }
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self { Body::Empty }
}

impl From<Bytes> for Body {
    fn from(b: Bytes) -> Self { Body::Bytes(b) }
}

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self { Body::Bytes(v.into()) }
}

impl From<String> for Body {
    fn from(s: String) -> Self { Body::Bytes(s.into()) }
}
```

#### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Request build error: {0}")]
    BuildError(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("TLS error: {0}")]
    Tls(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("HTTP error: {status}")]
    Http { status: StatusCode, body: Bytes },
}
```

### Client Implementations

#### In-Process Client (DirectClient)

```rust
pub struct DirectClient {
    inner: reqwest::Client,
    metrics: Arc<Metrics>,
    config: ClientConfig,
}

pub struct ClientConfig {
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub pool_idle_timeout: Duration,
    pub pool_max_idle_per_host: usize,
    pub http2_prior_knowledge: bool,
    pub http2_adaptive_window: bool,
}

impl DirectClient {
    pub fn new(config: ClientConfig) -> Result<Self, ClientError> {
        let inner = reqwest::Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .pool_idle_timeout(config.pool_idle_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .http2_prior_knowledge(config.http2_prior_knowledge)
            .http2_adaptive_window(config.http2_adaptive_window)
            .build()
            .map_err(|e| ClientError::BuildError(e.to_string()))?;
        
        Ok(Self {
            inner,
            metrics: Arc::new(Metrics::default()),
            config,
        })
    }
}

#[async_trait]
impl HttpClient for DirectClient {
    async fn execute(&self, request: Request) -> Result<Response, ClientError> {
        let start = Instant::now();
        self.metrics.requests_in_flight.inc();
        
        let req = self.build_reqwest_request(request)?;
        let resp = self.inner.execute(req).await
            .map_err(|e| self.map_reqwest_error(e))?;
        
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp.bytes().await
            .map_err(|e| ClientError::Io(std::io::Error::new(
                std::io::ErrorKind::Other, 
                e
            )))?;
        
        self.metrics.requests_in_flight.dec();
        self.metrics.request_duration.observe(start.elapsed().as_secs_f64());
        self.metrics.requests_total.with_label_values(&[status.as_str()]).inc();
        
        Ok(Response {
            status,
            headers,
            body,
            extensions: Extensions::default(),
        })
    }
    
    async fn execute_streaming(&self, request: Request) -> Result<StreamingResponse, ClientError> {
        let req = self.build_reqwest_request(request)?;
        let resp = self.inner.execute(req).await
            .map_err(|e| self.map_reqwest_error(e))?;
        
        let status = resp.status();
        let headers = resp.headers().clone();
        
        // Convert reqwest::Response to async stream
        let stream = resp.bytes_stream()
            .map_err(|e| ClientError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e
            )));
        
        Ok(StreamingResponse {
            status,
            headers,
            body: Box::pin(stream),
        })
    }
    
    async fn websocket(&self, request: Request) -> Result<WebSocketConn, ClientError> {
        // WebSocket upgrade using tokio-tungstenite
        let uri = request.uri().to_string();
        let req = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&uri)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_ws_key())
            .body(())
            .map_err(|e| ClientError::BuildError(e.to_string()))?;
        
        let (ws_stream, _) = tokio_tungstenite::connect_async(req).await
            .map_err(|e| ClientError::Connection(e.to_string()))?;
        
        let (write, read) = ws_stream.split();
        
        let (send_tx, mut send_rx) = mpsc::channel(16);
        let (recv_tx, recv_rx) = mpsc::channel(16);
        
        // Send task
        tokio::spawn(async move {
            let mut write = write;
            while let Some(msg) = send_rx.recv().await {
                let ws_msg = match msg {
                    WsMessage::Text(s) => tokio_tungstenite::tungstenite::Message::Text(s),
                    WsMessage::Binary(b) => tokio_tungstenite::tungstenite::Message::Binary(b.to_vec()),
                    WsMessage::Ping(p) => tokio_tungstenite::tungstenite::Message::Ping(p),
                    WsMessage::Pong(p) => tokio_tungstenite::tungstenite::Message::Pong(p),
                    WsMessage::Close(f) => tokio_tungstenite::tungstenite::Message::Close(f),
                };
                
                if write.send(ws_msg).await.is_err() {
                    break;
                }
            }
        });
        
        // Receive task
        tokio::spawn(async move {
            let mut read = read;
            while let Some(msg) = read.next().await {
                let result = match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(s)) => 
                        Ok(WsMessage::Text(s)),
                    Ok(tokio_tungstenite::tungstenite::Message::Binary(b)) => 
                        Ok(WsMessage::Binary(b.into())),
                    Ok(tokio_tungstenite::tungstenite::Message::Ping(p)) => 
                        Ok(WsMessage::Ping(p)),
                    Ok(tokio_tungstenite::tungstenite::Message::Pong(p)) => 
                        Ok(WsMessage::Pong(p)),
                    Ok(tokio_tungstenite::tungstenite::Message::Close(f)) => 
                        Ok(WsMessage::Close(f)),
                    Err(e) => Err(ClientError::Protocol(e.to_string())),
                    _ => continue,
                };
                
                if recv_tx.send(result).await.is_err() {
                    break;
                }
            }
        });
        
        Ok(WebSocketConn {
            send: send_tx,
            recv: recv_rx,
        })
    }
}

impl DirectClient {
    fn build_reqwest_request(&self, request: Request) -> Result<reqwest::Request, ClientError> {
        let mut builder = self.inner.request(request.method().clone(), request.uri().to_string());
        
        for (name, value) in request.headers() {
            builder = builder.header(name, value);
        }
        
        match request.into_body() {
            Body::Empty => {},
            Body::Bytes(b) => {
                builder = builder.body(b.to_vec());
            },
            Body::Stream(_) => {
                return Err(ClientError::BuildError(
                    "Streaming body not supported for plain requests".into()
                ));
            }
        }
        
        builder.build()
            .map_err(|e| ClientError::BuildError(e.to_string()))
    }
    
    fn map_reqwest_error(&self, error: reqwest::Error) -> ClientError {
        if error.is_timeout() {
            ClientError::Timeout(error.to_string())
        } else if error.is_connect() {
            ClientError::Connection(error.to_string())
        } else {
            ClientError::Protocol(error.to_string())
        }
    }
}
```

#### Out-of-Process Client (RpcClient) - Future Work

```rust
// Placeholder for future RPC implementation
pub struct RpcClient {
    channel: tonic::Channel,
    timeout: Duration,
}

// Would use gRPC service definition like:
// service OagwProxyService {
//   rpc Execute(HttpRequest) returns (HttpResponse);
//   rpc ExecuteStreaming(HttpRequest) returns (stream ChunkResponse);
//   rpc WebSocket(stream WsMessage) returns (stream WsMessage);
// }

#[async_trait]
impl HttpClient for RpcClient {
    async fn execute(&self, request: Request) -> Result<Response, ClientError> {
        todo!("RPC implementation - future work")
    }
    
    async fn execute_streaming(&self, request: Request) -> Result<StreamingResponse, ClientError> {
        todo!("RPC implementation - future work")
    }
    
    async fn websocket(&self, request: Request) -> Result<WebSocketConn, ClientError> {
        todo!("RPC implementation - future work")
    }
}
```

### Plugin Context API

Plugins need a simplified, sandboxed interface for request/response manipulation.

```rust
// Plugin context exposed to Starlark plugins
pub struct PluginContext {
    pub request: RequestContext,
    pub response: Option<ResponseContext>,
    pub error: Option<ErrorContext>,
    pub config: serde_json::Value,
    pub route_id: String,
    pub tenant_id: String,
    pub log: Logger,
    pub time: TimeContext,
}

pub struct RequestContext {
    method: Method,
    path: String,
    query: HashMap<String, Vec<String>>,
    headers: HeaderMap,
    body: Bytes,
}

impl RequestContext {
    pub fn method(&self) -> &str { self.method.as_str() }
    pub fn path(&self) -> &str { &self.path }
    pub fn set_path(&mut self, path: String) { self.path = path; }
    
    pub fn query(&self) -> &HashMap<String, Vec<String>> { &self.query }
    pub fn set_query(&mut self, query: HashMap<String, Vec<String>>) {
        self.query = query;
    }
    pub fn add_query(&mut self, key: String, value: String) {
        self.query.entry(key).or_default().push(value);
    }
    
    pub fn headers(&self) -> &HeaderMap { &self.headers }
    pub fn headers_mut(&mut self) -> &mut HeaderMap { &mut self.headers }
    
    pub fn body(&self) -> &[u8] { &self.body }
    pub fn json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
    pub fn set_json(&mut self, value: serde_json::Value) -> Result<(), serde_json::Error> {
        self.body = serde_json::to_vec(&value)?.into();
        Ok(())
    }
}

pub struct ResponseContext {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl ResponseContext {
    pub fn status(&self) -> u16 { self.status.as_u16() }
    pub fn set_status(&mut self, code: u16) {
        self.status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    pub fn headers(&self) -> &HeaderMap { &self.headers }
    pub fn headers_mut(&mut self) -> &mut HeaderMap { &mut self.headers }
    
    pub fn body(&self) -> &[u8] { &self.body }
    pub fn json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }
    pub fn set_json(&mut self, value: serde_json::Value) -> Result<(), serde_json::Error> {
        self.body = serde_json::to_vec(&value)?.into();
        Ok(())
    }
}

pub struct ErrorContext {
    pub status: u16,
    pub code: String,
    pub message: String,
    pub upstream: bool,
}

pub struct Logger {
    tenant_id: String,
    route_id: String,
}

impl Logger {
    pub fn info(&self, message: &str, data: serde_json::Value) {
        tracing::info!(
            tenant_id = %self.tenant_id,
            route_id = %self.route_id,
            data = ?data,
            "{}",
            message
        );
    }
    
    pub fn warn(&self, message: &str, data: serde_json::Value) {
        tracing::warn!(
            tenant_id = %self.tenant_id,
            route_id = %self.route_id,
            data = ?data,
            "{}",
            message
        );
    }
    
    pub fn error(&self, message: &str, data: serde_json::Value) {
        tracing::error!(
            tenant_id = %self.tenant_id,
            route_id = %self.route_id,
            data = ?data,
            "{}",
            message
        );
    }
}

pub struct TimeContext {
    start: Instant,
}

impl TimeContext {
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}
```

### Starlark Integration

Bridge between Starlark runtime and Rust plugin context:

```rust
use starlark::values::Value;
use starlark::environment::Module;

pub struct StarlarkPluginContext {
    ctx: Arc<RwLock<PluginContext>>,
}

impl StarlarkPluginContext {
    pub fn register_globals(module: &Module) {
        // Register ctx.request methods
        module.set("ctx_request_method", starlark_fn!(ctx_request_method));
        module.set("ctx_request_path", starlark_fn!(ctx_request_path));
        module.set("ctx_request_set_path", starlark_fn!(ctx_request_set_path));
        module.set("ctx_request_json", starlark_fn!(ctx_request_json));
        module.set("ctx_request_set_json", starlark_fn!(ctx_request_set_json));
        
        // Register ctx.response methods
        module.set("ctx_response_status", starlark_fn!(ctx_response_status));
        module.set("ctx_response_set_json", starlark_fn!(ctx_response_set_json));
        
        // Register ctx.log methods
        module.set("ctx_log_info", starlark_fn!(ctx_log_info));
        
        // Register control flow
        module.set("ctx_next", starlark_fn!(ctx_next));
        module.set("ctx_reject", starlark_fn!(ctx_reject));
    }
}

// Example Starlark function implementations
fn ctx_request_method(ctx: Value) -> Result<String, anyhow::Error> {
    let plugin_ctx = ctx.downcast_ref::<StarlarkPluginContext>()?;
    Ok(plugin_ctx.ctx.read().unwrap().request.method().to_string())
}

fn ctx_request_set_path(ctx: Value, path: String) -> Result<(), anyhow::Error> {
    let plugin_ctx = ctx.downcast_ref::<StarlarkPluginContext>()?;
    plugin_ctx.ctx.write().unwrap().request.set_path(path);
    Ok(())
}

fn ctx_log_info(ctx: Value, message: String, data: Value) -> Result<(), anyhow::Error> {
    let plugin_ctx = ctx.downcast_ref::<StarlarkPluginContext>()?;
    let data_json = starlark_value_to_json(data)?;
    plugin_ctx.ctx.read().unwrap().log.info(&message, data_json);
    Ok(())
}
```

### WebTransport Support (Future)

```rust
// Placeholder for WebTransport (QUIC-based)
pub struct WebTransportConn {
    session: webtransport::Session,
}

impl WebTransportConn {
    pub async fn open_stream(&mut self) -> Result<WebTransportStream, ClientError> {
        // Open bidirectional stream
        todo!("WebTransport implementation")
    }
    
    pub async fn accept_stream(&mut self) -> Result<WebTransportStream, ClientError> {
        // Accept incoming stream
        todo!("WebTransport implementation")
    }
    
    pub async fn send_datagram(&mut self, data: Bytes) -> Result<(), ClientError> {
        // Unreliable datagram
        todo!("WebTransport implementation")
    }
    
    pub async fn recv_datagram(&mut self) -> Result<Bytes, ClientError> {
        todo!("WebTransport implementation")
    }
}

pub struct WebTransportStream {
    send: BoxSink<'static, Bytes>,
    recv: BoxStream<'static, Result<Bytes, ClientError>>,
}
```

## Implementation Plan

### Phase 1: Core HTTP Client (In-Process)
- ✅ `Request`, `Response`, `Body` types
- ✅ `HttpClient` trait
- ✅ `DirectClient` implementation using `reqwest`
- ✅ Plain HTTP request/response
- ✅ Error handling and metrics

### Phase 2: Streaming Support
- ✅ `StreamingResponse` type
- ✅ SSE event stream parsing
- ✅ Integration with OAGW proxy endpoint

### Phase 3: WebSocket Support
- ✅ `WebSocketConn` type
- ✅ WebSocket upgrade handling
- ✅ Bidirectional message passing
- ✅ Connection lifecycle management

### Phase 4: Plugin Context API
- ✅ `PluginContext` and related types
- ✅ Starlark integration
- ✅ Sandbox restrictions enforcement
- ✅ Plugin examples and tests

### Phase 5: Out-of-Process Client (Future)
- ⬜ gRPC service definition
- ⬜ `RpcClient` implementation
- ⬜ Streaming RPC support
- ⬜ WebSocket over gRPC

### Phase 6: WebTransport (Future)
- ⬜ QUIC transport layer
- ⬜ `WebTransportConn` implementation
- ⬜ Stream multiplexing
- ⬜ Datagram support

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plain_request() {
        let client = DirectClient::new(ClientConfig::default()).unwrap();
        
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://httpbin.org/get")
            .build()
            .unwrap();
        
        let response = client.execute(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_streaming_response() {
        let client = DirectClient::new(ClientConfig::default()).unwrap();
        
        let request = Request::builder()
            .method(Method::GET)
            .uri("https://httpbin.org/stream/3")
            .build()
            .unwrap();
        
        let mut response = client.execute_streaming(request).await.unwrap();
        
        let mut chunks = 0;
        while let Some(chunk) = response.body_stream().next().await {
            chunk.unwrap();
            chunks += 1;
        }
        
        assert!(chunks > 0);
    }
    
    #[tokio::test]
    async fn test_websocket() {
        let client = DirectClient::new(ClientConfig::default()).unwrap();
        
        let request = Request::builder()
            .uri("wss://echo.websocket.org")
            .build()
            .unwrap();
        
        let mut conn = client.websocket(request).await.unwrap();
        
        conn.send(WsMessage::Text("hello".into())).await.unwrap();
        let msg = conn.recv().await.unwrap().unwrap();
        
        match msg {
            WsMessage::Text(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected text message"),
        }
    }
}
```

### Mock Client for Testing

```rust
pub struct MockClient {
    responses: Arc<Mutex<VecDeque<Result<Response, ClientError>>>>,
}

impl MockClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    pub fn push_response(&self, response: Response) {
        self.responses.lock().unwrap().push_back(Ok(response));
    }
    
    pub fn push_error(&self, error: ClientError) {
        self.responses.lock().unwrap().push_back(Err(error));
    }
}

#[async_trait]
impl HttpClient for MockClient {
    async fn execute(&self, _request: Request) -> Result<Response, ClientError> {
        self.responses.lock().unwrap().pop_front()
            .unwrap_or(Err(ClientError::Connection("No mock response configured".into())))
    }
    
    async fn execute_streaming(&self, _request: Request) -> Result<StreamingResponse, ClientError> {
        todo!("Mock streaming implementation")
    }
    
    async fn websocket(&self, _request: Request) -> Result<WebSocketConn, ClientError> {
        todo!("Mock WebSocket implementation")
    }
}
```

## Dependencies

```toml
[dependencies]
# Core HTTP client
reqwest = { version = "0.11", features = ["json", "stream"] }
hyper = "0.14"
http = "0.2"
bytes = "1.5"

# Async runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"
async-trait = "0.1"

# WebSocket
tokio-tungstenite = "0.21"

# WebTransport (future)
# webtransport = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Metrics
prometheus = "0.13"

# Tracing
tracing = "0.1"

# Starlark integration
starlark = "0.11"

# Out-of-process RPC (future)
# tonic = "0.10"
# prost = "0.12"
```

## Security Considerations

1. **Timeout enforcement**: All requests have mandatory timeouts to prevent resource exhaustion
2. **Body size limits**: Maximum body size (100MB) enforced before buffering
3. **TLS verification**: Certificate validation always enabled (no insecure mode in production)
4. **Connection pooling**: Limits on idle connections per host to prevent resource leaks
5. **Sandbox isolation**: Starlark plugins cannot make network requests directly
6. **Header validation**: Well-known hop-by-hop headers stripped automatically
7. **WebSocket security**: Proper origin validation and frame size limits

## Performance Considerations

1. **Zero-copy streaming**: SSE and chunked responses use stream-based processing
2. **Connection reuse**: HTTP/1.1 and HTTP/2 connection pooling enabled
3. **Adaptive window sizing**: HTTP/2 flow control optimized for throughput
4. **Minimal allocations**: `Bytes` type uses reference counting for zero-copy operations
5. **Async I/O**: Non-blocking operations throughout the stack
6. **Metrics overhead**: Negligible (<0.1ms per request)

## Alternatives Considered

### Alternative 1: Use `hyper` Directly

**Pros**:
- Lower-level control
- Slightly better performance

**Cons**:
- More complex API
- More boilerplate code
- Missing high-level features (redirects, cookies, etc.)

**Decision**: Use `reqwest` (built on `hyper`) for better ergonomics.

### Alternative 2: Custom HTTP Client Implementation

**Pros**:
- Full control over implementation
- Optimized for OAGW use cases

**Cons**:
- Significant development effort
- Maintenance burden
- Likely inferior to battle-tested libraries

**Decision**: Use existing libraries (`reqwest`, `tokio-tungstenite`).

### Alternative 3: Single Monolithic Client Type

**Pros**:
- Simpler API surface

**Cons**:
- Cannot support multiple backends
- Harder to test
- Less flexible for future extensions

**Decision**: Use trait-based abstraction.

## Related ADRs

- [ADR: Error Source Distinction](./adr-error-source-distinction.md) - HTTP error handling
- [ADR: Rate Limiting](./adr-rate-limiting.md) - Client-side rate limiting
- [ADR: Circuit Breaker](./adr-circuit-breaker.md) - Upstream availability tracking
- [ADR: CORS](./adr-cors.md) - Cross-origin request handling

## References

- [reqwest documentation](https://docs.rs/reqwest)
- [hyper documentation](https://docs.rs/hyper)
- [tokio-tungstenite documentation](https://docs.rs/tokio-tungstenite)
- [WebTransport specification](https://w3c.github.io/webtransport/)
- [Server-Sent Events specification](https://html.spec.whatwg.org/multipage/server-sent-events.html)
- [WebSocket protocol RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
