CI := 1

OPENAPI_URL ?= http://127.0.0.1:8087/openapi.json
OPENAPI_OUT ?= docs/api/api.json

.PHONY: check fmt clippy test test-sqlite test-pg test-mysql test-all test-users-info-pg audit deny security ci

# Default target - run necessary quality checks and tests and build the release binary
all: check fmt clippy test test-sqlite security build
	@echo "consider to run 'make test-all' and 'make e2e-local' as well"

# Check code formatting
fmt:
	cargo fmt --all -- --check

# Run clippy linter
clippy:
	cargo clippy --workspace --all-targets -- -D warnings

# Run all tests
test:
	cargo test --workspace

# Check for security vulnerabilities
audit:
	@command -v cargo-audit >/dev/null || (echo "Installing cargo-audit..." && cargo install cargo-audit)
	cargo audit

# Check licenses and dependencies
deny:
	@command -v cargo-deny >/dev/null || (echo "Installing cargo-deny..." && cargo install cargo-deny)
	cargo deny check

# Run all security checks
security: audit deny

# Run all quality checks
check: fmt clippy test security

# Run CI pipeline
ci: check

# Make a release build
build:
	cargo build --release

# Show this help message
help:
	@awk '/^# / { desc=substr($$0, 3) } /^[a-zA-Z0-9_-]+:/ && desc { printf "%-20s - %s\n", $$1, desc; desc="" }' Makefile | sort

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

# Auto-fix code formatting
dev-fmt:
	cargo fmt --all

# Auto-fix clippy warnings
dev-clippy:
	cargo clippy --workspace --all-targets --fix --allow-dirty

# Run tests in development mode
dev-test:
	cargo test --workspace

# Start server with quickstart config
quickstart:
	mkdir -p data
	cargo run --bin hyperspot-server -- --config config/quickstart.yaml run

# Run server with example module
example:
	cargo run --bin hyperspot-server --features users-info-example -- --config config/quickstart.yaml run

.PHONY: test-sqlite test-pg test-mysql test-all test-users-info-pg

# Run SQLite integration tests
test-sqlite:
	cargo test -p modkit-db --features "sqlite,integration" -- --nocapture

# Run PostgreSQL integration tests
test-pg:
	cargo test -p modkit-db --features "pg,integration" -- --nocapture

# Run MySQL integration tests
test-mysql:
	cargo test -p modkit-db --features "mysql,integration" -- --nocapture

# Run all database integration tests
test-all: test-sqlite test-pg test-mysql

# Run users_info module integration tests
test-users-info-pg:
	cargo test -p users_info --features "integration" -- --nocapture

.PHONY: e2e e2e-local e2e-docker

# Run E2E tests in Docker (default)
e2e: e2e-docker

# Run E2E tests locally
e2e-local:
	python3 scripts/ci.py e2e

# Run E2E tests in Docker environment
e2e-docker:
	python3 scripts/ci.py e2e --docker

# Generate code coverage report
coverage:
	@echo "Code coverage is not implemented yet"
	@exit -1
