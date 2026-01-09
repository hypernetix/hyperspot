# Settings Module E2E Tests

This directory contains end-to-end tests for the settings module REST API.

## Test Files

- **test_settings_get.py** - Tests for GET /settings/v1/settings endpoint
  - Returns defaults when settings don't exist (lazy creation)
  - Idempotency
  - Authentication handling

- **test_settings_update.py** - Tests for POST /settings/v1/settings endpoint (full update)
  - Creates settings on first call (upsert)
  - Replaces existing settings
  - Validation (max length)
  - Error handling (missing fields)

- **test_settings_patch.py** - Tests for PATCH /settings/v1/settings endpoint (partial update)
  - Updates only provided fields
  - Creates settings if not exist (upsert)
  - Sequential partial updates
  - Empty patch handling

- **test_settings_integration.py** - Integration tests covering full workflows
  - Complete lifecycle: GET -> POST -> PATCH -> GET
  - Idempotency across operations
  - Consistency between POST and PATCH

## Running Tests

### Local mode (with server running on localhost:8087)

Run all settings tests:
```bash
E2E_BASE_URL=http://localhost:8087 \
  testing/e2e/.venv/bin/python -m pytest testing/e2e/modules/settings -vv
```

Run specific test file:
```bash
E2E_BASE_URL=http://localhost:8087 \
  testing/e2e/.venv/bin/python -m pytest testing/e2e/modules/settings/test_settings_get.py -vv
```

Run specific test:
```bash
E2E_BASE_URL=http://localhost:8087 \
  testing/e2e/.venv/bin/python -m pytest testing/e2e/modules/settings/test_settings_patch.py::test_patch_settings_theme_only -vv
```

### With authentication

```bash
E2E_BASE_URL=http://localhost:8087 \
E2E_AUTH_TOKEN=your_token_here \
  testing/e2e/.venv/bin/python -m pytest testing/e2e/modules/settings -vv
```

## Test Coverage

The E2E tests cover:

- ✅ GET endpoint - lazy creation, idempotency
- ✅ POST endpoint - full update, upsert behavior, validation
- ✅ PATCH endpoint - partial update, field independence
- ✅ Error handling - validation failures, missing fields
- ✅ Integration - complete workflows, consistency
- ✅ Authentication - proper handling of auth requirements

Total: 27 test cases
