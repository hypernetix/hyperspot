CI := 1

OPENAPI_URL ?= http://127.0.0.1:8087/openapi.json
OPENAPI_OUT ?= docs/api/api.json

# E2E Docker args
E2E_ARGS ?= --features users-info-example

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
# | dylint      | - Project-specific architectural conventions (custom lints)          |
# |             | - DTO declaration and placement (only in api/rest folders)           |
# |             | - DTO isolation (no references from domain/contract layers)          |
# |             | - API endpoint versioning requirements (e.g., /users/v1/users)       |
# |             | - Contract layer purity (no serde, HTTP types, or ToSchema)          |
# |             | - Layer separation and dependency rules enforcement                  |
# |             | - Use 'make dylint-list' to see all available custom lints           |
# +-------------+----------------------------------------------------------------------+

.PHONY: clippy kani geiger safety lint dylint dylint-list dylint-test gts-docs gts-docs-vendor gts-docs-release gts-docs-vendor-release gts-docs-test

# Run clippy linter (excludes gts-rust submodule which has its own lint settings)
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

## Validate GTS identifiers in .md and .json files (DE0903)
# Uses gts-docs-validator from apps/gts-docs-validator
# Vendor enforcement is available via the gts-docs-vendor target (--vendor x)
gts-docs:
	cargo run -p gts-docs-validator -- \
		--exclude "target/*" \
		--exclude "docs/api/*" \
		docs modules libs examples

## Validate GTS docs with vendor check (ensures all IDs use vendor "x")
gts-docs-vendor:
	cargo run -p gts-docs-validator -- \
		--vendor x \
		--exclude "target/*" \
		--exclude "docs/api/*" \
		docs modules libs examples

## Validate GTS identifiers (release build)
gts-docs-release:
	cargo run --release -p gts-docs-validator -- \
		--exclude "target/*" \
		--exclude "docs/api/*" \
		docs modules libs examples

## Validate GTS docs with vendor check (release build)
gts-docs-vendor-release:
	cargo run --release -p gts-docs-validator -- \
		--vendor x \
		--exclude "target/*" \
		--exclude "docs/api/*" \
		docs modules libs examples

## Run tests for GTS documentation validator
gts-docs-test:
	cargo test -p gts-docs-validator

## List all custom project compliance lints (see dylint_lints/README.md)
dylint-list:
	@cd dylint_lints && \
	DYLINT_LIBS=$$(find target/release -maxdepth 1 \( -name "libde*@*.so" -o -name "libde*@*.dylib" -o -name "de*@*.dll" \) -type f | sort -u); \
	if [ -z "$$DYLINT_LIBS" ]; then \
		echo "ERROR: No dylint libraries found. Run 'make dylint' first to build them."; \
		exit 1; \
	fi; \
	for lib in $$DYLINT_LIBS; do \
		echo "=== $$lib ==="; \
		cargo dylint list --lib-path "$$lib"; \
	done

## Test dylint lints on UI test cases (compile and verify violations)
dylint-test:
	@cd dylint_lints && cargo test

# Run project compliance dylint lints on the workspace (see `make dylint-list`)
dylint:
	@command -v cargo-dylint >/dev/null || (echo "Installing cargo-dylint..." && cargo install cargo-dylint)
	@command -v dylint-link >/dev/null || (echo "Installing dylint-link..." && cargo install dylint-link)
	@cd dylint_lints && cargo build --release
	@TOOLCHAIN=$$(rustc --version --verbose | grep 'host:' | cut -d' ' -f2); \
	RUSTUP_TOOLCHAIN=$$(cat dylint_lints/rust-toolchain.toml 2>/dev/null | grep 'channel' | cut -d'"' -f2 || echo "nightly"); \
	cd dylint_lints/target/release && \
	for lib in $$(ls libde*.dylib libde*.so de*.dll 2>/dev/null | grep -v '@'); do \
		case "$$lib" in \
			*.dylib) EXT=".dylib" ;; \
			*.so) EXT=".so" ;; \
			*.dll) EXT=".dll" ;; \
		esac; \
		BASE=$${lib%$$EXT}; \
		TARGET="$$BASE@$$RUSTUP_TOOLCHAIN-$$TOOLCHAIN$$EXT"; \
		cp -f "$$lib" "$$TARGET" 2>/dev/null || true; \
	done; \
	cd ../../.. && \
	DYLINT_LIBS=$$(find dylint_lints/target/release -maxdepth 1 \( -name "libde*@*.so" -o -name "libde*@*.dylib" -o -name "de*@*.dll" \) -type f | sort -u); \
	if [ -z "$$DYLINT_LIBS" ]; then \
		echo "ERROR: No dylint libraries found after build."; \
		exit 1; \
	fi; \
	LIB_ARGS=""; \
	for lib in $$DYLINT_LIBS; do \
		LIB_ARGS="$$LIB_ARGS --lib-path $$lib"; \
	done; \
	cargo +$$RUSTUP_TOOLCHAIN dylint $$LIB_ARGS --workspace

# Run all code safety checks
safety: clippy kani lint dylint # geiger
	@echo "OK. Rust Safety Pipeline complete"

# -------- Code security checks --------

.PHONY: deny security

## Check licenses and dependencies
deny:
	$(call ensure_tool,cargo-deny)
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
	# Run server in background
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
	python3 scripts/ci.py e2e --docker $(E2E_ARGS)

markdown-check:
	broken-md-links docs
	broken-md-links examples
	broken-md-links dylint_lints
	broken-md-links guidelines

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

# -------- Fuzzing --------

.PHONY: fuzz fuzz-build fuzz-list fuzz-run fuzz-clean fuzz-corpus

## Install cargo-fuzz (required for fuzzing)
fuzz-install:
	@command -v cargo-fuzz >/dev/null || \
		(echo "Installing cargo-fuzz..." && cargo install cargo-fuzz)

## Build all fuzz targets
fuzz-build: fuzz-install
	cd fuzz && cargo +nightly fuzz build

## List all available fuzz targets
fuzz-list: fuzz-install
	cd fuzz && cargo +nightly fuzz list

## Run a specific fuzz target (use FUZZ_TARGET=name)
## Example: make fuzz-run FUZZ_TARGET=fuzz_odata_filter FUZZ_SECONDS=60
fuzz-run: fuzz-install
	@if [ -z "$(FUZZ_TARGET)" ]; then \
		echo "ERROR: FUZZ_TARGET is required. Example: make fuzz-run FUZZ_TARGET=fuzz_odata_filter"; \
		exit 1; \
	fi
	cd fuzz && cargo +nightly fuzz run $(FUZZ_TARGET) -- -max_total_time=$(or $(FUZZ_SECONDS),60)

## Run all fuzz targets for a short time (smoke test)
fuzz: fuzz-build
	@echo "Running all fuzz targets for 30 seconds each..."
	@cd fuzz && \
	for target in $$(cargo +nightly fuzz list); do \
		echo "=== Fuzzing $$target ==="; \
		cargo +nightly fuzz run $$target -- -max_total_time=30 || true; \
	done
	@echo "Fuzzing complete. Check fuzz/artifacts/ for crashes."

## Clean fuzzing artifacts and corpus
fuzz-clean:
	rm -rf fuzz/artifacts/
	rm -rf fuzz/corpus/*/
	rm -rf fuzz/target/

## Minimize corpus for a specific target
fuzz-corpus: fuzz-install
	@if [ -z "$(FUZZ_TARGET)" ]; then \
		echo "ERROR: FUZZ_TARGET is required. Example: make fuzz-corpus FUZZ_TARGET=fuzz_odata_filter"; \
		exit 1; \
	fi
	cd fuzz && cargo +nightly fuzz cmin $(FUZZ_TARGET)

# -------- Main targets --------

.PHONY: all check ci build quickstart example

# Start server with quickstart config
quickstart:
	mkdir -p data
	cargo run --bin hyperspot-server -- --config config/quickstart.yaml run

## Run server with example module
example:
	cargo run --bin hyperspot-server --features users-info-example,tenant-resolver-example -- --config config/quickstart.yaml run

oop-example:
	cargo build -p calculator --features oop_module
	cargo run --bin hyperspot-server --features oop-example,users-info-example,tenant-resolver-example -- --config config/quickstart.yaml run

# Run all quality checks
check: fmt clippy security dylint-test dylint gts-docs test

# Run CI pipeline
ci: check

# Make a release build using stable toolchain
build:
	cargo +stable build --release

# Run all necessary quality checks and tests and then build the release binary
all: build check test-sqlite e2e-local fuzz
	@echo "consider to run 'make test-db' as well"
