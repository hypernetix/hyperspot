# Multi-stage build for hyperspot-server API backend
# Stage 1: Builder
FROM rust:1.89 AS builder

# Install protobuf-compiler for prost-build
RUN apt-get update && \
    apt-get install -y --no-install-recommends protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Copy all workspace members
COPY apps/hyperspot-server ./apps/hyperspot-server
COPY libs ./libs
COPY modules ./modules
COPY examples ./examples
COPY config ./config

# Build the hyperspot-server binary in release mode
# Using --bin to build only the specific binary
RUN cargo build --bin hyperspot-server --features users-info-example

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder /build/target/debug/hyperspot-server /app/hyperspot-server
# Copy config used in CMD
COPY --from=builder /build/config /app/config

# Expose the HTTP port (default 8080 for E2E)
EXPOSE 8087

# Run the binary with minimal config suitable for E2E tests
# Using --mock flag to use in-memory SQLite for any modules that need DB
CMD ["/app/hyperspot-server", "--config", "/app/config/quickstart.yaml"]

