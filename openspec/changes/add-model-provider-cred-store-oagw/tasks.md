## 1. Specification (this change)
- [ ] 1.1 Define module responsibilities and boundaries for `model_provider`, `cred_store`, and `oagw`.
- [ ] 1.2 Define complete domain models (GTS schemas/instances + anonymous DB entities) for all three modules.
- [ ] 1.3 Define gateway (Rust native + REST) contracts for all three modules.
- [ ] 1.4 Define gateway↔plugin contracts and plugin selection rules.
- [ ] 1.5 Define module startup procedure for GTS schema + instance registration (and dependency ordering).
- [ ] 1.6 Provide OpenAPI 3.1 specifications for the three REST APIs (paths + components).
- [ ] 1.7 Run `openspec validate add-model-provider-cred-store-oagw --strict` and fix all issues.

## 2. Follow-up implementation (out of scope for this proposal)
- [ ] 2.1 Create module crates following `guidelines/NEW_MODULE.md` (SDK + gateway + plugins).
- [ ] 2.2 Implement DB migrations and repositories using `modkit-db` Secure ORM.
- [ ] 2.3 Implement REST handlers + OpenAPI wiring with `OperationBuilder`.
- [ ] 2.4 Implement plugin discovery and selection using `types_registry` + `ClientHub` scoped clients.
- [ ] 2.5 Add end-to-end tests and example plugins (OpenAI, Vault, HTTP/SSE).
