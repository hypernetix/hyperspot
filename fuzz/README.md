# Fuzzing Infrastructure for HyperSpot

This directory contains fuzzing infrastructure using [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) and [ClusterFuzzLite](https://google.github.io/clusterfuzzlite/).

## Overview

Continuous fuzzing helps discover:
- Memory safety issues (buffer overflows, use-after-free)
- Panics and crashes
- Logic bugs in parsers and validators
- Performance issues (algorithmic complexity attacks)

## Quick Start

### Prerequisites

```bash
# Install cargo-fuzz (requires nightly)
cargo install cargo-fuzz
```

### Running Fuzzing Locally

```bash
# Build all fuzz targets
make fuzz-build

# Run all targets (smoke test - 30s each)
make fuzz

# Run specific target for longer
make fuzz-run FUZZ_TARGET=fuzz_odata_filter FUZZ_SECONDS=300

# List all available targets
make fuzz-list
```

### Using CI Script

```bash
# Build fuzz targets
python scripts/ci.py fuzz-build

# Run smoke tests
python scripts/ci.py fuzz --seconds 60

# Run specific target
python scripts/ci.py fuzz-run fuzz_odata_filter --seconds 300
```

## Fuzz Targets

| Target | Priority | Component | Description |
|--------|----------|-----------|-------------|
| `fuzz_odata_filter` | HIGH | OData parsing | Fuzzes $filter query string parser |
| `fuzz_odata_cursor` | HIGH | Pagination | Fuzzes cursor decoder (base64+JSON) |
| `fuzz_yaml_config` | HIGH | Configuration | Fuzzes YAML config parser |
| `fuzz_html_parser` | MEDIUM | file_parser | Fuzzes HTML document parser |
| `fuzz_pdf_parser` | MEDIUM | file_parser | Fuzzes PDF document parser |
| `fuzz_json_config` | MEDIUM | Configuration | Fuzzes JSON parser |
| `fuzz_odata_orderby` | MEDIUM | OData parsing | Fuzzes $orderby token parser |
| `fuzz_markdown_parser` | LOW | file_parser | Fuzzes Markdown parser |

## Continuous Fuzzing in CI

ClusterFuzzLite runs automatically:
- **On Pull Requests:** 10 minutes per target
- **On main branch:** 1 hour per target
- **Scheduled (nightly):** 1 hour per target

Results are uploaded as artifacts and issues are created automatically.

## Reproducing Crashes

If a crash is found:

```bash
# Crashes are saved in artifacts/
cd fuzz
cargo fuzz run fuzz_odata_filter artifacts/fuzz_odata_filter/crash-*
```

## Corpus Management

```bash
# Minimize corpus (remove redundant inputs)
make fuzz-corpus FUZZ_TARGET=fuzz_odata_filter

# Add seed inputs
echo 'name eq "test"' > corpus/fuzz_odata_filter/my_seed.txt
```

## Best Practices

1. **Don't panic on invalid input** - use `Result` types
2. **Limit resource usage** - add timeouts and size limits
3. **Add seed corpus** - good inputs speed up fuzzing
4. **Run locally before PR** - catch issues early

## Cleaning Up

```bash
# Remove all fuzzing artifacts
make fuzz-clean

# Or using CI script
python scripts/ci.py fuzz-clean
```

## Resources

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [ClusterFuzzLite docs](https://google.github.io/clusterfuzzlite/)
- [libFuzzer options](https://llvm.org/docs/LibFuzzer.html#options)

## Troubleshooting

### cargo-fuzz not found

```bash
cargo install cargo-fuzz
```

### Nightly toolchain issues

```bash
rustup install nightly
# Use +nightly flag with cargo-fuzz commands:
cargo +nightly fuzz run fuzz_odata_filter
```

### Build failures

Make sure all dependencies are available:

```bash
cd fuzz
cargo check --all
```
