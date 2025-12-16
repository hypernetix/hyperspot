# Event Broker Module

## Overview

The Event Broker is a high-performance event streaming module for HyperSpot. It provides a REST API for producing and
consuming events with support for batch operations, cursor-based pagination, and long-polling. Events are typed using
the Global Type System (GTS) notation and stored with monotonically increasing sequences for reliable ordering and
replay.

The module acts as a lightweight event log, enabling decoupled communication between microservices and real-time data
streaming use cases.

## Architecture

The Event Broker follows HyperSpot's modular architecture with clean layer separation:

```
event_broker/
├── src/
│   ├── contract/          # Public API (transport-agnostic)
│   │   ├── client.rs      # Native Rust client trait (EventBrokerApi)
│   │   ├── model.rs       # Core models: Event, Topic, EventType
│   │   └── error.rs       # Contract errors: EventBrokerError
│   ├── domain/            # Business logic
│   │   ├── service.rs     # Event service (produce, consume, storage)
│   │   ├── repository.rs  # Repository traits (event persistence)
│   │   └── events.rs      # Domain events
│   ├── api/               # REST adapters
│   │   └── rest/
│   │       ├── dto.rs     # REST DTOs (with serde)
│   │       ├── handlers.rs # HTTP handlers
│   │       ├── routes.rs  # Route registration
│   │       ├── mapper.rs  # DTO ↔ Model conversions
│   │       └── error.rs   # Problem Details mapping
│   ├── infra/             # Infrastructure
│   │   └── storage/       # Event storage implementation
│   │       ├── repository.rs
│   │       └── entity.rs
│   ├── gateways/          # Client implementations
│   │   └── local.rs       # Local client for ClientHub
│   ├── module.rs          # Module registration
│   └── config.rs          # Typed configuration
└── README.md
```

### Key Design Principles

- **Topic**: A topic is a named logical event stream. Topics are identified
  by [GTS topic identifiers](https://github.com/globalTypeSystem/gts-spec/). All sequencing, offsets, and ordering
  semantics are scoped to a single topic; no ordering guarantees exist across topics.
- **Sequential ordering**: Events are ordered within each topic by a monotonically increasing sequence number. Sequence
  numbers are assigned per topic and are unique within that topic (not global), providing deterministic ordering and
  replay inside the topic.
- **Immutable log**: Each topic is an append-only immutable log. Once written, events are never modified or reordered;
  retention may remove older segments without affecting ordering. All immutability and ordering guarantees apply per
  topic.
- **Typed events (GTS + JSON Schema validation)**: Events are identified by GTS type identifiers and validated against a
  JSON Schema associated with the event type. See: https://github.com/globalTypeSystem/gts-spec/
- **Offset-based fetching**: Consumers fetch by sequence offset within a topic (offset = last seen sequence). Subsequent
  reads return events with sequence greater than the offset, enabling efficient forward-only traversal and replay.

### GTS Types

#### Topic

**Base Type**: `gts.hx.core.events.topic.v1~`
Examples:

- `gts.hx.core.events.topic.v1~vendor.users.v1`
- `gts.hx.core.events.topic.v1~vendor.orders.v1`

**Schema**

```json
{
  "$id": "https://example.com/person.schema.json",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "gts.hx.core.events.topic.v1~",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "gts-identifier"
    },
    "description": {
      "type": "string",
      "maxLength": 2048
    },
    "retention": {
      "type": "string",
      "format": "iso8601duration"
    },
    "idempotentRetention": {
      "type": "string",
      "format": "iso8601duration"
    }
  },
  "required": [
    "id",
    "streaming"
  ]
}
```

**Topic Instance Example**:


- **Event Type**: `gts.hx.core.events.event_type.v1~`
    - Examples:
        - `gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1`
        - `gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_updated.v1`

- **Event Instance**: `gts.hx.core.events.event.v1~`
    - Examples:
        - `gts.hx.core.events.event.v1~vendor.users.user_registered.v1`
        - `gts.hx.core.events.event.v1~vendor.orders.order_placed.v1`

## Features

### Event Production

- **Single event publishing**: `POST /v1/events` with idempotency support
- **Batch publishing**: `POST /v1/events:batch` (up to 100 events) atomic writes, per-event error reporting
- **Automatic sequence assignment**: Server-assigned monotonic sequence numbers
- **Idempotency**: Duplicate sequence detection and conflict resolution

### Event Consumption

- **Cursor-based listing**: `GET /v1/events` with filtering and pagination
- **Long-polling**: `GET /v1/events:poll` - wait up to 30s for new events
- **Sequence-based replay**: Read from any historical sequence position
- **Type filtering**: Filter by single or multiple event types

### Data Management

- **Sequential ordering**: Guaranteed ordering within the log
- **Efficient storage**: Optimized for high-throughput writes and range reads
- **Type registry via ABI**: Topics and event types registered through local Rust client (not REST)

## Data Models

### Topic

**Fields:**

- `id` (string, GTS Identifier) - Unique Global Topic Identifier
- `description` (string, optional) - Human-readable description
- `streaming` (object|bool) - Streaming configuration (e.g. retention policy)
- `createdAt` (string, ISO 8601) - Timestamp of topic creation

**Example:**

```json
{
  "id": "gts.hx.core.events.topic.v1~",
  "description": "User-related events",
  "streaming": {
    "type": "file",
    "location": "file://var/lib/hyperspot/events",
    "retention": "PT30D"
  },
  "idempotentRetention": "PT24H",
  "createdAt": "2025-12-03T12:00:00.000Z"
}
```

### Event Type

**Fields:**

- `id` (string, GTS Identifier) - Unique Global Event Type Identifier
- `description` (string) - Human-readable description
- `allowedSubjecTypes` (array of strings, GTS Identifiers) - Allowed subject types
- `subjectRefTemplate` (string) - Subject reference template // TODO: do we still need this?
- `dataSchema` (object) - JSON Schema defining event payload structure
- `createdAt` (string, ISO 8601) - Timestamp of event type creation

**Example:**

```json
{
  "id": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
  "description": "Event emitted when a new tenant is created",
  "allowedSubjectTypes": [
    "gts.vendor.tenants.tenant.v1~"
  ],
  "subjectRefTemplate": "/tenants/{tenantId}",
  "dataSchema": {
    "type": "object",
    "properties": {
      "subject": {
        "type": "string",
        "format": "uuid"
      },
      "data": {
        "type": "object",
        "properties": {
          "userId": {
            "type": "string"
          },
          "email": {
            "type": "string",
            "format": "email"
          }
        },
        "required": [
          "userId",
          "email"
        ]
      }
    },
    "required": [
      "subject",
      "data"
    ]
  },
  "createdAt": "2025-12-03T12:00:00.000Z"
}
```

### Event

**Fields:**

- `id` (string, UUID) - Unique Event Identifier
- `type` (string, GTS Identifier) - Event type identifier
- `occurredAt` (string, ISO 8601) - Timestamp when the event occurred
- `createdAt` (string, ISO 8601) - Timestamp when the event was created in the broker
- `sequence` (integer) - Monotonically increasing sequence number
- `source` (string) - Origin of the event
- `subject` (string) - Subject reference (id)
- `subjectType` (string, GTS Identifier) - Subject type identifier
- `tenant` (string, UUID) - Tenant identifier
- `traceParent` (string, optional) - W3C Trace Context parent
- `data` (object) - Event payload (validated against event type schema)

**Example:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "type": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
  "occurredAt": "2025-12-03T12:01:59.829Z",
  "createdAt": "2025-12-03T12:01:59.829Z",
  "sequence": 12345,
  "source": "example-service",
  "subject": "1053828c-57b1-4c2b-97dc-ccff6225651e",
  "subjectType": "gts.vendor.tenants.tenant.v1",
  "tenant": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "traceParent": "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
  "data": {
    "userId": "user-12345",
    "email": "user@gmail.com"
  }
}
```

### Consumer

**Fields:**

- `id` (string, UUID) - Unique Consumer Identifier, generated on creation
- `consumerGroup` (string) - Unique consumer group identifier that multiple instances can share
- `topic` (string, GTS Identifier) - Topic identifier
- `types` (array of strings, GTS Identifiers) - Filtered event types, wildcards allowed
- `subjectTypes` (array of strings, GTS Identifiers) - Filtered subject types, wildcards allowed
- `CEL` (string, optional) - CEL expression for advanced filtering
- `sessionTimeout` (string, ISO 8601 Duration) - Consumer session timeout, e.g. "PT30S"
- `createdAt` (string, ISO 8601) - Timestamp of consumer creation
- `lastSeenAt` (string, ISO 8601) - Timestamp of last poll activity
- `expiresAt` (string, ISO 8601) - Timestamp when the consumer expires

## API Endpoints

### Sequence Resolution

#### GET /v1/topics

List available topics.

**Query Parameters:**

- `topic` - Optional. Filter by topic GTS Identifier, wildcards allowed
- `limit` - Optional. Records per page (default: 100, max: 100)

**Request:**

```bash
GET /v1/topics?topic=gts.hx.core.events.topic.v1~vendor.users.v1
```

**Response:** `200 OK`

```json
{
  "topics": [
    {
      "id": "gts.hx.core.events.topic.v1~vendor.users.v1",
      "description": "User-related events",
      "streaming": {
        "type": "file",
        "location": "file://var/lib/hyperspot/events",
        "retention": "PT30D"
      },
      "createdAt": "2025-12-03T12:00:00.000Z"
    }
  ]
}
```

#### GET /v1/topics/segments

Get topic segments for partitioned topics. The exact segment structure depends on the storage backend.

**Query Parameters:**

- `topic` - Required. Topic GTS Identifier
- `$orderby` - Optional. Sort order (default: rangeStart asc)
- `limit` - Optional. Records per page (default: 100, max: 100)

**Request:**

```bash
GET /v1/topics/segments?topic=gts.hx.core.events.topic.v1~vendor.users.v1
```

**Response:** `200 OK`

```json
{
  "topic": "gts.hx.core.events.topic.v1~vendor.users.v1",
  "start": 123456,
  "startTime": "2025-12-03T00:00:00.000Z",
  "end": 234567,
  "endTime": "2025-12-05T00:00:00.000Z",
  "segments": [
    {
      "location": "file://var/lib/hyperspot/events/vendor.users.v1/segment-00001",
      "startSequence": 0,
      "startTime": "2025-12-03T00:00:00.000Z",
      "endSequence": 100000,
      "endTime": "2025-12-04T00:00:00.000Z",
      "amount": 100000,
      "createdAt": "2025-12-03T00:00:00.000Z"
    },
    {
      "location": "file://var/lib/hyperspot/events/vendor.users.v1/segment-00002",
      "startSequence": 100001,
      "startTime": "2025-12-04T00:00:00.000Z",
      "endSequence": 200000,
      "endTime": "2025-12-05T00:00:00.000Z",
      "amount": 100000,
      "createdAt": "2025-12-04T00:00:00.000Z"
    }
  ],
  "page_info": {
    "next_cursor": "<opaque>",
    "prev_cursor": "<opaque>",
    "limit": 20
  }
}
```

### Produce Events

#### POST /v1/events

Publish a single event to the broker.

**Request:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "type": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
  "occurredAt": "2025-12-03T12:01:59.829Z",
  "source": "example-service",
  "subject": "1053828c-57b1-4c2b-97dc-ccff6225651e",
  "subjectType": "gts.vendor.tenants.tenant.v1",
  "tenant": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "traceParent": "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
  "data": {
    "userId": "user-12345",
    "email": "user@example.com"
  }
}
```

**Response:** `201 Created`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "type": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
  "occurredAt": "2025-12-03T12:01:59.829Z",
  "createdAt": "2025-12-03T12:01:59.829Z",
  "sequence": 12345,
  "source": "example-service",
  "subject": "1053828c-57b1-4c2b-97dc-ccff6225651e",
  "subjectType": "gts.vendor.tenants.tenant.v1",
  "tenant": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "traceParent": "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
  "data": {
    "userId": "user-12345",
    "email": "user@example.com"
  }
}
```

**Headers:**

- `Location: /v1/events`
- `Producer-Id: <opaque id>` - (optional) Unique identifier for the event producer instance

**Errors:**

- `400 Bad Request` - Invalid GTS type format or invalid schema
- `422 Unprocessable Entity` - Validation failed (data doesn't match event type schema)

---

#### POST /v1/events:batch

Publish multiple events in a single batch request. Push per topic is atomic.

**Request:**

```json
{
  "events": [
    {
      "id": "829501ea-99dd-484f-a227-3e958747b8ea",
      "type": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
      "occurredAt": "2025-12-03T12:01:59.829Z",
      "source": "tenant-service",
      "subject": "1053828c-57b1-4c2b-97dc-ccff6225651e",
      "subjectType": "gts.vendor.tenants.tenant.v1",
      "tenant": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "data": {
        "userId": "user-12345",
        "email": "user@example.com"
      }
    },
    {
      "id": "78d4c214-f688-43d7-9b1c-164ea146cda0",
      "type": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_updated.v1",
      "occurredAt": "2025-12-03T12:02:00.000Z",
      "source": "tenant-service",
      "subject": "1053828c-57b1-4c2b-97dc-ccff6225651e",
      "subjectType": "gts.vendor.tenants.tenant.v1",
      "tenant": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "data": {
        "userId": "user-12345",
        "email": "newemail@example.com"
      }
    }
  ]
}
```

**Response:** `207 Multi-Status`
Multiple status possible when pushed events to a different topics. Per topic write is atomic.

```json
{
  "data": {
    "results": [
      {
        "index": 0,
        "status": 201,
        "event": {
          "id": "550e8400-e29b-41d4-a716-446655440000",
          "type": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
          "occurredAt": "2025-12-03T12:01:59.829Z",
          "createdAt": "2025-12-03T12:01:59.829Z",
          "sequence": 12345,
          "source": "example-service",
          "subject": "1053828c-57b1-4c2b-97dc-ccff6225651e",
          "subjectType": "gts.vendor.tenants.tenant.v1",
          "tenant": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
          "data": {
            "userId": "user-12345",
            "email": "user@example.com"
          }
        }
      },
      {
        "index": 1,
        "status": 422,
        "error": {
          "type": "https://errors.hyperspot.com/EVENT_BROKER_VALIDATION_ERROR",
          "title": "Validation failed",
          "status": 422,
          "detail": "Event data does not match schema for type gts.mycompany.app.events.user_updated.v1~"
        }
      }
    ]
  },
  "meta": {
    "total": 2,
    "succeeded": 1,
    "failed": 1
  }
}
```

**Limits:**

- Maximum 100 records per batch
- Total payload size: 1MB

---

### Consume Events

The Event Broker supports **3 consumption patterns** for different use cases:

#### **Pattern 1: Direct Event Listing**

Simple pagination with filters via query parameters - for one-time queries or historical data access.
The exact set of filters depends on topic streaming configuration, therefore might vary between topics.

**Example:**

```bash
GET /v1/events?topic=gts.hx.core.events.topic.v1~&offset=10000&limit=50
```

#### **Pattern 2: Long-Poll Single Consumer**

One consumer instance polling for new events with inline filters via query parameters.

**Example:**

```bash
GET /v1/events:poll?topic=gts.hx.core.events.topic.v1~&offset=12345&timeout=30&limit=25
```

#### **Pattern 3: Long-Poll Multiple Consumer Instances**

Multiple instances, each with its own consumer created via `POST /v1/consumers` for various complex filters.
Each instance polls using its own `consumer_id`. The consumer resource holds unique consumer group id,
distribution strategy, filter criteria and session timeout.

**Example:**

```bash
# Step 1: Create consumer with complex filters
# Consumer Instance 1
POST /v1/consumers
# Returns: {"data": {"id": "daca7bfd-89e6-4944-949c-327dd3fef133", ...}}

GET /v1/events:poll?consumer_id=daca7bfd-89e6-4944-949c-327dd3fef133&offset=100

# Consumer Instance 2
POST /v1/consumers
# Returns: {"data": {"id": "cdb75c04-19e7-42ad-9999-d23f9d7c3392", ...}}

GET /v1/events:poll?consumer_id=cdb75c04-19e7-42ad-9999-d23f9d7c3392&offset=100
```

---

#### POST /v1/consumers

Create a consumer with complex filters for long-polling. This endpoint is optional and can be used to manage consumer
state for multiple instances.
The consumer is a temporary resource that holds filter criteria for a `sessionTimeout` period.

**Request:**

```json
{
  "consumerGroup": "tenant-service-instances",
  "topic": "gts.hx.core.events.topic.v1~",
  "types": [
    "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
    "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_updated.v1"
  ],
  "subjectTypes": [
    "gts.vendor.tenants.tenant.v1~"
  ],
  "CEL": "has(data) && data.userId == 'user-12345'",
  "sessionTimeout": "PT30S"
}
```

**Response:** `201 Created`

```json
{
  "data": {
    "id": "cdb75c04-19e7-42ad-9999-d23f9d7c3392",
    "consumerGroup": "tenant-service-instances",
    "topic": "gts.hx.core.events.topic.v1~",
    "types": [
      "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
      "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_updated.v1"
    ],
    "subjectTypes": [
      "gts.vendor.tenants.tenant.v1~"
    ],
    "CEL": "has(data) && data.userId == 'user-12345'",
    "createdAt": "2025-12-03T12:05:00.000Z",
    "expiresAt": "2025-12-03T12:05:30.000Z"
  }
}
```

#### GET /v1/events

List events with cursor-based pagination.

**Query Parameters:**

- `offset` - Required. Last seen sequence number (returns events with sequence > offset)
- `topic` - Optional. Filter by topic (GTS Identifier). Required when `consumer_id` is not provided
- `consumer_id` - Optional. Reference to pre-created consumer (from `POST /v1/consumers`). When provided, `topic` is
  ignored
- `limit` - Optional. Records per page (default: 100, max: 100)

**Note:** Either `topic` or `consumer_id` must be provided.

**Example:**

```bash
GET /v1/events?topic=gts.hx.core.events.topic.v1~&offset=10000&limit=50
```

**Response:** `200 OK`

```json
{
  "items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "type": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
      "occurredAt": "2025-12-03T12:01:59.829Z",
      "createdAt": "2025-12-03T12:01:59.829Z",
      "sequence": 12345,
      "source": "example-service",
      "subject": "1053828c-57b1-4c2b-97dc-ccff6225651e",
      "subjectType": "gts.vendor.tenants.tenant.v1",
      "data": {
        "userId": "user-12345",
        "email": "user@example.com"
      }
    }
  ]
}
```

---

#### GET /v1/events:poll

Long-poll for new events (consumer pattern).

**Query Parameters:**

- `offset` - Required. Last seen sequence number (returns events with sequence > offset)
- `topic` - Optional. Filter by topic (GTS Identifier). Required when `consumer_id` is not provided
- `consumer_id` - Optional. Reference to pre-created consumer (from `POST /v1/consumers`). When provided, `topic` is
  ignored
- `limit` - Optional. Records per page (default: 100, max: 100)
- `timeout` - Optional. Wait timeout in seconds (default: 30, max: 30)

**Note:** Either `topic` or `consumer_id` must be provided.

**Semantics:**

- `offset=100` means "I have seen all events up to and including sequence 100"
- Returns events where `sequence > 100` (starting from 101)

**Behavior:**

1. If events with `sequence > offset` available → return 200 with records immediately
2. If no new events → wait up to `timeout` seconds
3. If timeout expires → return 200 with empty array
4. Client should reconnect with the last seen sequence as offset

**Example:**

```bash
GET /v1/events:poll?topic=gts.hx.core.events.topic.v1~&offset=12345&timeout=30&limit=25
```

**Response:** `200 OK`

```json
{
  "items": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "type": "gts.hyperspot.core.events.type.v1~vendor.tenants.tenant_created.v1",
      "occurredAt": "2025-12-03T12:02:35.829Z",
      "createdAt": "2025-12-03T12:02:35.829Z",
      "sequence": 12346,
      "source": "order-service",
      "subject": "order-789",
      "subjectType": "gts.mycompany.app.entities.order.v1~",
      "data": {
        "orderId": "order-789",
        "total": 99.99
      }
    }
  ]
}
```

**Errors:**

- `400 Bad Request` - Missing or invalid offset parameter
- `408 Request Timeout` - Timeout exceeded (should not happen with 30s max)

---

## Configuration

Configuration is managed through the HyperSpot configuration file (`config/quickstart.yaml`):

```yaml
modules:
  event_broker:
    # Storage backend
    storage:
      type: "memory"  # Options: memory, database
      retention_days: 30  # How long to keep events

    # Performance tuning
    batch:
      max_size: 100  # Max events per batch request
      max_payload_bytes: 1048576  # 1MB max payload

    # Long-polling configuration
    polling:
      default_timeout_secs: 30
      max_timeout_secs: 30
      poll_interval_ms: 100  # Internal check interval

    # Consumer settings
    consumer:
      default_limit: 25
      max_limit: 200
```

## OpenAPI Documentation

Interactive API documentation is available at:

- **Swagger UI**: `http://localhost:8087/docs`
- **OpenAPI JSON**: `http://localhost:8087/v1/openapi.json`

## Error Handling

All errors follow RFC 9457 Problem Details format:

```json
{
  "type": "https://errors.hyperspot.com/EVENT_BROKER_INVALID_OFFSET",
  "title": "Invalid offset",
  "status": 400,
  "detail": "Offset must be a positive integer",
  "instance": "/v1/events:poll",
  "traceId": "01J9X..."
}
```

### Error Codes

| Code                            | Status | Description                            |
|---------------------------------|--------|----------------------------------------|
| `EVENT_BROKER_INVALID_TYPE`     | 400    | Invalid GTS type format                |
| `EVENT_BROKER_INVALID_OFFSET`   | 400    | Invalid offset value (query parameter) |
| `EVENT_BROKER_INVALID_SEQUENCE` | 400    | Invalid sequence value (event field)   |
| `EVENT_BROKER_SEQUENCE_EXISTS`  | 409    | Sequence already exists                |
| `EVENT_BROKER_NOT_FOUND`        | 404    | Event not found                        |
| `EVENT_BROKER_VALIDATION_ERROR` | 422    | Request validation failed              |
| `EVENT_BROKER_BATCH_TOO_LARGE`  | 400    | Batch exceeds size limit               |

## Performance Characteristics

- **Write throughput**: 10,000+ events/sec (in-memory)
- **Read latency**: <10ms (p99, warm cache)
- **Long-poll overhead**: Minimal (event-driven notification)
- **Storage**: Linear growth with event count (consider retention policies)

# Design Decisions

## Idempotent Event Publishing

Problem: Ensuring that duplicate event submissions do not result in multiple stored events.

Solution: Idempotence key for an event is calculated based on `id`, `topic`, `type`, `tenant`, `subject`,
and `occurredAt` fields in combination with the `Producer-Id` header if provided. Idempotence key TTL is managed by
the topic configuration `idempotentRetention`.

Security Note: The idempotence key does not include the `data` field to avoid performance overhead and potential
leakage of sensitive information. Therefore, two events with identical idempotence keys but different `data` payloads
will be considered duplicates. Clients must ensure uniqueness of the `id` field to avoid unintended deduplication.
Using Producer-Id can help isolate idempotence scope per producer instance.

## Consumption Filters Update

Problem: Allowing smooth updates to consumer filters without losing position in the event stream.

Solution: Consumers are identified by a unique `consumer_id` and can be created with specific filter criteria.
When a consumer's filters need to be updated, a new consumer instance should be created with the desired filters.
The old consumer can be deleted once the new one is active. This approach ensures that each consumer maintains
its own position in the event stream without conflicts.

## Event Ordering and Pagination

The Event Broker uses **offset-based pagination** (not cursor-based) for event consumption.

- **Offset = Sequence Number**: The `offset` parameter represents the last seen event sequence number within the
  specified topic (sequence numbers are scoped per-topic).
- **Deterministic Ordering**: Events are ordered within a topic by monotonically increasing `sequence` field. Sequence
  numbers are unique within the topic (not global), so ordering and replay semantics apply per-topic only.
- **Replay Capability**: Consumers can replay from any historical sequence number
- **Consumer Progress Tracking**: Offset represents explicit consumer position in the event log