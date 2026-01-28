# Copyright 2026 HyperSpot Contributors
# SPDX-License-Identifier: Apache-2.0

FROM gcr.io/oss-fuzz-base/base-builder-rust

# Install system dependencies required for building
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libprotobuf-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install latest nightly toolchain (project requires Rust 1.92+)
# rust-src is required for sanitizer builds
RUN rustup toolchain install nightly --force && \
    rustup default nightly && \
    rustup component add rust-src --toolchain nightly

# Copy source code
COPY . $SRC/hyperspot
WORKDIR $SRC/hyperspot

# Copy build script
COPY .clusterfuzzlite/build.sh $SRC/
