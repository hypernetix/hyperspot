# Model Registry — PRD

Model catalog with tenant-level availability and approval workflow.

## Scenarios

### S1 Get Tenant Model

LLM Gateway resolves model availability for current tenant.

```mermaid
sequenceDiagram
    participant LLM as LLM Gateway
    participant MR as Model Registry

    LLM->>MR: get_tenant_model(ctx, model_id)
    MR-->>LLM: model info + provider
```

### S2 List Tenant Models

LLM Gateway queries available models for current tenant.

```mermaid
sequenceDiagram
    participant LLM as LLM Gateway
    participant MR as Model Registry

    LLM->>MR: list_tenant_models(ctx, filter)
    MR-->>LLM: models[]
```

### S3 Model Discovery

Registry polls providers for available models via Outbound API Gateway.

```mermaid
sequenceDiagram
    participant MR as Model Registry
    participant OB as Outbound API Gateway
    participant P as Provider API

    MR->>OB: GET /models
    OB->>P: Request (with credentials)
    P-->>OB: models[]
    OB-->>MR: models[]
    MR->>MR: Upsert + create pending approvals
```

### S4 Model Approval

New models require tenant admin approval before becoming available.

**Statuses**: `pending` → `approved` | `rejected` | `revoked`

**Auto-approval**: configurable per tenant/provider

## Dependencies

| Module | Role |
|--------|------|
| Outbound API Gateway | Provider API calls |
| Tenant Resolver | Tenant context |

## Errors

| Error | HTTP | Description |
|-------|------|-------------|
| `model_not_found` | 404 | Model not in catalog |
| `model_not_approved` | 403 | Model not approved for tenant |
| `model_deprecated` | 410 | Model sunset |
