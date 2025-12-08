CI := 1

OPENAPI_URL ?= http://127.0.0.1:8087/openapi.json
OPENAPI_OUT ?= docs/api/api.json

# -------- Utility macros --------

define ensure_tool
	@command -v $(1) >/dev/null || (echo "Installing $(1)..." && cargo install $(1))
endef

# -------- Defaults --------

# Show the help message with list of commands (default target)
help:
	@python3 scripts/make_help.py Makefile

# -------- Code formatting --------

.PHONY: fmt

# Check code formatting
fmt:
	cargo fmt --all -- --check

# -------- Code safety checks --------
#
# Tool Comparison - What Each Tool Checks:
# +-------------+----------------------------------------------------------------------+
# | Tool        | Checks Performed                                                     |
# +-------------+----------------------------------------------------------------------+
# | clippy      | - Idiomatic Rust patterns (e.g., use of .iter() vs into_iter())      |
# |             | - Common mistakes (e.g., unnecessary clones, redundant closures)     |
# |             | - Performance issues (e.g., inefficient string operations)           |
# |             | - Style violations (e.g., naming conventions, formatting)            |
# |             | - Suspicious constructs (e.g., comparison to NaN, unused results)    |
# |             | - Complexity warnings (e.g., too many arguments, cognitive load)     |
# +-------------+----------------------------------------------------------------------+
# | kani        | - Memory safety proofs (buffer overflows, null pointer dereferences) |
# |             | - Arithmetic overflow/underflow in all possible execution paths      |
# |             | - Assertion violations (panics, unwrap failures)                     |
# |             | - Undefined behavior detection                                       |
# |             | - Concurrency issues (data races, deadlocks) with #[kani::proof]     |
# |             | - Custom invariants and postconditions verification                  |
# +-------------+----------------------------------------------------------------------+
# | geiger      | - Unsafe blocks in your code and dependencies                        |
# |             | - FFI (Foreign Function Interface) calls                             |
# |             | - Raw pointer dereferences                                           |
# |             | - Mutable static variables access                                    |
# |             | - Inline assembly usage                                              |
# |             | - Dependency tree visualization of unsafe code usage                 |
# +-------------+----------------------------------------------------------------------+
# | lint        | - Compiler warnings treated as errors (unused variables, imports)    |
# |             | - Dead code detection                                                |
# |             | - Type inference failures                                            |
# |             | - Deprecated API usage                                               |
# |             | - Missing documentation warnings                                     |
# |             | - Ensures clean compilation across all targets and features          |
# +-------------+----------------------------------------------------------------------+

.PHONY: clippy kani geiger safety lint

# Run clippy linter
clippy:
	$(call ensure_tool,cargo-clippy)
	cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::perf

## The Kani Rust Verifier for checking safety of the code
kani:
	@command -v kani >/dev/null || \
		(echo "Installing Kani verifier..." && \
		 cargo install --locked kani-verifier)
	cargo kani --workspace --all-features

## Run Geiger scanner for unsafe code in dependencies
geiger:
	$(call ensure_tool,cargo-geiger)
	cd apps/hyperspot-server && cargo geiger --all-features

## Check there are no compile time warnings
lint:
	RUSTFLAGS="-D warnings" cargo check --workspace --all-targets --all-features

# Run all code safety checks
safety: clippy kani lint # geiger
	@echo "OK. Rust Safety Pipeline complete"

# -------- Code security checks --------

.PHONY: deny security

## Check licenses and dependencies
deny:
	$(call ensure_tool,cargo-deny)
	@command -v cargo-deny >/dev/null || (echo "Installing cargo-deny..." && cargo install cargo-deny)
	cargo deny check

# Run all security checks
security: deny
	@echo "OK. Rust Security Pipeline complete"

# -------- API and docs --------

.PHONY: openapi

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

# -------- Development and auto fix --------

.PHONY: dev dev-fmt dev-clippy dev-test

## Run tests in development mode
dev-test:
	cargo test --workspace

## Auto-fix code formatting
dev-fmt:
	cargo fmt --all

## Auto-fix clippy warnings
dev-clippy:
	cargo clippy --workspace --all-targets --fix --allow-dirty

# Auto-fix formatting and clippy warnings
dev: dev-fmt dev-clippy dev-test

# -------- Tests --------

.PHONY: test test-sqlite test-pg test-mysql test-db test-users-info-pg

# Run all tests
test:
	cargo test --workspace

## Run SQLite integration tests
test-sqlite:
	cargo test -p modkit-db --features "sqlite,integration" -- --nocapture

## Run PostgreSQL integration tests
test-pg:
	cargo test -p modkit-db --features "pg,integration" -- --nocapture

## Run MySQL integration tests
test-mysql:
	cargo test -p modkit-db --features "mysql,integration" -- --nocapture

# Run all database integration tests
test-db: test-sqlite test-pg test-mysql

## Run users_info module integration tests
test-users-info-pg:
	cargo test -p users_info --features "integration" -- --nocapture

# -------- E2E tests --------

.PHONY: e2e e2e-local e2e-docker

# Run E2E tests in Docker (default)
e2e: e2e-docker

# Run E2E tests locally
e2e-local:
	python3 scripts/ci.py e2e

## Run E2E tests in Docker environment
e2e-docker:
	python3 scripts/ci.py e2e --docker

# -------- Code coverage --------

.PHONY: coverage coverage-unit coverage-e2e-local check-prereq-e2e-local

# Generate code coverage report (unit + e2e-local tests)
coverage:
	@command -v cargo-llvm-cov >/dev/null || (echo "Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov)
	python3 scripts/coverage.py combined

# Generate code coverage report (unit tests only)
coverage-unit:
	@command -v cargo-llvm-cov >/dev/null || (echo "Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov)
	python3 scripts/coverage.py unit

## Ensure needed packages and programs installed for local e2e testing
check-prereq-e2e-local:
	python scripts/check_local_env.py --mode e2e-local

# Generate code coverage report (e2e-local tests only)
coverage-e2e-local: check-prereq-e2e-local
	@command -v cargo-llvm-cov >/dev/null || (echo "Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov)
	python3 scripts/coverage.py e2e-local

# -------- Main targets --------

.PHONY: all check ci build quickstart example

# Start server with quickstart config
quickstart:
	mkdir -p data
	cargo run --bin hyperspot-server -- --config config/quickstart.yaml run

## Run server with example module
example:
	cargo run --bin hyperspot-server --features users-info-example -- --config config/quickstart.yaml run

# Run all quality checks
check: fmt clippy test security

# Run CI pipeline
ci: check

# Make a release build using stable toolchain
build:
	cargo +stable build --release

# Run all necessary quality checks and tests and then build the release binary
all: check test test-sqlite build
	@echo "consider to run 'make test-db' and 'make e2e-local' as well"
