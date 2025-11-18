CI := 1

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

# Development commands
dev-fmt:
	cargo fmt --all

dev-clippy:
	cargo clippy --workspace --all-targets --fix --allow-dirty

dev-test:
	cargo test --workspace

example:
	cargo run --bin hyperspot-server --features users-info-example -- --config config/quickstart.yaml

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