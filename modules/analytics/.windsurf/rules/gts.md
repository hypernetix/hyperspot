---
trigger: model_decision
description: When working with data types, schemas, or GTS definitions
---
# GTS Type System

When working with data types, schemas, or type definitions:

## Always Reference GTS Spec
- **GTS Specification**: https://github.com/GlobalTypeSystem/gts-spec
- Read the spec before creating any type definitions
- Follow GTS syntax and conventions exactly

## Key GTS Concepts
- All data schemas use GTS format
- Plugin datasources provide GTS schemas
- Widget data follows GTS structure
- Type safety enforced via GTS validation

## Before Creating Types
1. Read relevant sections from GTS spec
2. Use GTS syntax (not Rust, TypeScript, etc.)
3. Validate against GTS schema rules
4. Document GTS version used