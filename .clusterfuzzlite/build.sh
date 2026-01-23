#!/bin/bash -eu
# Copyright 2026 HyperSpot Contributors
# SPDX-License-Identifier: Apache-2.0

# Update Rust toolchain to latest nightly (project requires Rust 1.92+)
# This runs in ClusterFuzzLite context where RUSTUP_HOME=/rust/rustup
rustup toolchain install nightly --force
rustup default nightly
rustup component add rust-src --toolchain nightly

cd $SRC/hyperspot

# Build all fuzz targets with optimization
cargo fuzz build -O

# Copy all fuzz target binaries to $OUT
FUZZ_TARGET_OUTPUT_DIR=fuzz/target/x86_64-unknown-linux-gnu/release
for f in fuzz/fuzz_targets/*.rs; do
    FUZZ_TARGET_NAME=$(basename ${f%.*})
    if [ -f "$FUZZ_TARGET_OUTPUT_DIR/$FUZZ_TARGET_NAME" ]; then
        cp "$FUZZ_TARGET_OUTPUT_DIR/$FUZZ_TARGET_NAME" "$OUT/"
    fi
done
