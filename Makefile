CI := 1

OPENAPI_URL ?= http://127.0.0.1:8087/openapi.json
OPENAPI_OUT ?= docs/api/api.json

.PHONY: check fmt clippy test audit deny security ci

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

audit:
	@command -v cargo-audit >/dev/null || (echo "Installing cargo-audit..." && cargo install cargo-audit)
	cargo audit

deny:
	@command -v cargo-deny >/dev/null || (echo "Installing cargo-deny..." && cargo install cargo-deny)
	cargo deny check

security: audit deny

check: fmt clippy test security

ci: check

# Generate OpenAPI spec from running hyperspot-server
openapi:
	@command -v curl >/dev/null || (echo "curl is required to generate OpenAPI spec" && exit 1)
	@echo "Starting hyperspot-server to generate OpenAPI spec..."
	# Запускаем сервер в фоне
	cargo run --bin hyperspot-server --features users-info-example -- --config config/quickstart.yaml &
	@SERVER_PID=$$!; \
	echo "hyperspot-server PID: $$SERVER_PID"; \
	echo "Waiting for $(OPENAPI_URL) to become ready..."; \
	tries=0; \
	until curl -fsS "$(OPENAPI_URL)" >/dev/null 2>&1; do \
		tries=$$((tries+1)); \
		if [ $$tries -gt 30 ]; then \
			echo "ERROR: hyperspot-server did not become ready in time"; \
			kill $$SERVER_PID >/dev/null 2>&1 || true; \
			exit 1; \
		fi; \
		sleep 1; \
	done; \
	echo "Server is ready, fetching OpenAPI spec..."; \
	mkdir -p $$(dirname "$(OPENAPI_OUT)"); \
	curl -fsS "$(OPENAPI_URL)" -o "$(OPENAPI_OUT)"; \
	echo "OpenAPI spec saved to $(OPENAPI_OUT)"; \
	echo "Stopping hyperspot-server..."; \
	kill $$SERVER_PID >/dev/null 2>&1 || true; \
	wait $$SERVER_PID 2>/dev/null || true

# Development commands
dev-fmt:
	cargo fmt --all

dev-clippy:
	cargo clippy --workspace --all-targets --fix --allow-dirty

dev-test:
	cargo test --workspace

example:
	cargo run --bin hyperspot-server --features users-info-example -- --config config/quickstart.yaml
# Quick start helpers
quickstart:
	mkdir -p data
	cargo run --bin hyperspot-server -- --config config/quickstart.yaml run

example:
	cargo run --bin hyperspot-server --features users-info-example -- --config config/quickstart.yaml run

# Integration testing with testcontainers
.PHONY: test-sqlite test-pg test-mysql test-all test-users-info-pg

# modkit-db only
test-sqlite:
	cargo test -p modkit-db --features "sqlite,integration" -- --nocapture

test-pg:
	cargo test -p modkit-db --features "pg,integration" -- --nocapture

test-mysql:
	cargo test -p modkit-db --features "mysql,integration" -- --nocapture

test-all: test-sqlite test-pg test-mysql

# example module (Postgres only)
test-users-info-pg:
	cargo test -p users_info --features "integration" -- --nocapture

# E2E testing
.PHONY: e2e e2e-local e2e-docker

# Run E2E tests against server in Docker  (default mode)
e2e: e2e-docker

# Explicit local mode (same as default e2e)
e2e-local:
	python3 scripts/ci.py e2e

# Run E2E tests in Docker environment
e2e-docker:
	python3 scripts/ci.py e2e --docker
coverage:
	@echo "Code coverage is not implemented yet"
	@exit -1
