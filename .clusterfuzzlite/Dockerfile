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

# Update Rust toolchain to nightly (project requires Rust 1.92+)
RUN rustup default nightly && rustup update

# Copy source code
COPY . $SRC/hyperspot
WORKDIR $SRC/hyperspot

# Copy build script
COPY .clusterfuzzlite/build.sh $SRC/
