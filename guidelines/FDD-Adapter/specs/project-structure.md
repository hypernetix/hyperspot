# Project Structure Specification

**Architecture Root**: `architecture/`
**Source Root**: `modules/`, `libs/`, `apps/`

---

## Architecture Files

```
architecture/
├── BUSINESS.md              # Business context (FDD Layer 2)
├── DESIGN.md                # Overall architecture design (FDD Layer 3)
├── ADR.md                   # Architecture Decision Records
└── features/
    ├── FEATURES.md          # Features manifest (FDD Layer 4)
    └── feature-{slug}/
        ├── DESIGN.md        # Feature design (FDD Layer 5)
        └── CHANGES.md       # Implementation plan (FDD Layer 6)
```

---

## Source Code Structure

```
modules/                     # Application modules
├── {module}/
│   ├── src/
│   │   ├── domain/          # Domain models (GTS types)
│   │   ├── api/
│   │   │   ├── rest/        # REST API (DTOs with serde)
│   │   │   └── grpc/        # gRPC API
│   │   ├── core/            # Business logic
│   │   └── infra/           # Infrastructure
│   ├── tests/               # Integration tests
│   └── Cargo.toml

libs/                        # Shared libraries
├── modkit/                  # Core module system
├── modkit-auth/             # Authentication
├── modkit-db/               # Database utilities
├── modkit-odata/            # OData parsing
└── modkit-security/         # Security context

apps/                        # Applications
└── hyperspot-server/        # Main server application

proto/                       # gRPC protocol definitions
docs/                        # Documentation & API specs
testing/                     # E2E tests
```

---

## Guidelines Structure

```
guidelines/
├── FDD/                     # FDD core methodology
│   ├── workflows/           # Workflow definitions
│   └── requirements/        # Requirements specs
├── FDD-Adapter/             # Project adapter
│   ├── AGENTS.md            # Adapter navigation
│   └── specs/               # Detailed specifications
└── GTS/                     # GTS specification
```
