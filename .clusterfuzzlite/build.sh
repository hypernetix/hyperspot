#!/bin/bash -eu
# Copyright 2026 HyperSpot Contributors
# SPDX-License-Identifier: Apache-2.0

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
