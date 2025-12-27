# Fuzz Testing for epub-builder

This directory contains fuzz tests to find XML parsing errors in generated EPUB files.

## Prerequisites

Install cargo-fuzz (requires nightly Rust):

```bash
cargo install cargo-fuzz
```

## Running Fuzz Tests

### Metadata Fuzzer
Tests all metadata fields (title, author, description, subjects, series, custom meta, etc.):

```bash
cargo +nightly fuzz run metadata_fuzz
```

### TOC Fuzzer
Tests table of contents entries with various titles and nested structures:

```bash
cargo +nightly fuzz run toc_fuzz
```

## Options

Run for a specific duration:
```bash
cargo +nightly fuzz run metadata_fuzz -- -max_total_time=3600  # 1 hour
```

Run with more parallel jobs:
```bash
cargo +nightly fuzz run metadata_fuzz -- -jobs=4 -workers=4
```

Use a specific seed:
```bash
cargo +nightly fuzz run metadata_fuzz -- -seed=12345
```

## Reproducing Crashes

If the fuzzer finds a crash, it will save the input to `fuzz/artifacts/metadata_fuzz/`. To reproduce:

```bash
cargo +nightly fuzz run metadata_fuzz fuzz/artifacts/metadata_fuzz/crash-<hash>
```

## Known Vulnerability Areas

Based on code analysis, these fields are high priority for fuzzing:

1. **Custom OPF metadata** (`add_metadata_opf`) - Uses `encode_html()` which doesn't escape quotes, but values go into XML attributes
2. **Author name_last_first** - Recently fixed (commit af25561)
3. **Series name** - Goes into both element content and attributes
4. **Accessibility summary** - Custom string in metadata

## What the Fuzzers Check

1. Generate EPUB with random metadata values
2. Extract the `content.opf`, `toc.ncx`, and `nav.xhtml` from the generated zip
3. Parse each XML file with quick-xml
4. Panic if any XML parsing errors are found
