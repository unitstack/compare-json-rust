# compare-json

Rust port of [`compare-json`](https://github.com/unitstack/compare-json) — **structured JSON comparison**: find what changed between two JSON values, with control over how keys, values, and arrays are matched.

> Online playground: **[comparejson.com](https://comparejson.com)**

## Why

Most JSON diff tools either output unstructured text or hide the parts that matter (where a key was added, whether a type changed, whether two arrays differ in order or in content). `compare-json` returns a structured list of differences — every entry carries a path, the side it belongs to (`base` / `contrast` / `both`), and the kind of change (`added`, `deleted`, `typeChanged`, `valueChanged`) — so you can render, filter, or program against it.

## Features

- **Deep comparison** of objects, arrays, and primitives.
- **Three array comparison strategies**: `byIndex` (default), `lcs` (minimal diff via Longest Common Subsequence), and `unordered` (multiset match).
- **Case-insensitive** key and/or value matching.
- **Numeric-string equality** — optionally treat `"1"` and `1` as equal.
- **Path tracking** with both segment-array and dot-notation forms.
- **CLI** with table or JSON output, reading from inline strings or files.
- Full **serde** integration — differences serialize to the same JSON shape as the other ports.

## Installation

```bash
cargo install compare-json
```

Or as a library:

```toml
[dependencies]
compare-json = "0.1"
```

## Quick Start

### Library

```rust
use compare_json::{compare_json, ArrayCompareMethod, CompareOptions};
use serde_json::json;

let base = json!({"name": "Alice", "age": 30});
let contrast = json!({"name": "Bob", "age": "30", "email": "bob@test.com"});

let options = CompareOptions::default(); // byIndex arrays, all flags off
let diffs = compare_json(&base, &contrast, &options);
for d in &diffs {
    println!("{} {} {}", d.path_string, d.path_belongs_to, d.diff_type);
}
// name   both     valueChanged
// age    both     typeChanged
// email  contrast added

// With options
let options = CompareOptions {
    array_compare_method: Some(ArrayCompareMethod::Lcs), // or ByIndex, Unordered
    key_case_insensitive: true,
    value_case_insensitive: false,
    numeric_string_equals_number: true,
};
```

### CLI

```bash
compare-json base.json contrast.json
```

```
┌──────────────┬──────────────┐
│ Key          │ Change Type  │
├──────────────┼──────────────┤
│ (Base) a     │ valueChanged │
│ (Base) b     │ deleted      │
│ (Contrast) c │ added        │
└──────────────┴──────────────┘
```

```bash
# Array strategies and case-insensitive matching
compare-json base.json contrast.json -a lcs -k -v

# Machine-readable JSON output
compare-json base.json contrast.json --json-export

# Write the report to a file
compare-json base.json contrast.json -o diff.txt
```

## Options

| Flag | Description |
|------|-------------|
| `-a, --array-compare-method <method>` | Array comparison strategy: `byIndex` (default), `lcs`, `unordered` |
| `-k, --key-case-insensitive` | Compare object keys case-insensitively |
| `-v, --value-case-insensitive` | Compare string values case-insensitively |
| `--numeric-string-equals-number` | Treat numeric strings as equal to numbers |
| `-j, --json-export` | Output the differences as JSON |
| `-o, --output <file>` | Write output to a file instead of stdout |

## Differences from the TypeScript reference

This port intentionally uses native Rust/serde_json semantics instead of replicating
JavaScript quirks. See [COMPATIBILITY.md](./COMPATIBILITY.md) for the full policy. In short:

- Numbers are compared by numeric value (`1` equals `1.0`); integer-vs-integer pairs are
  compared exactly (i64/u64, no 2^53 precision loss), anything involving a float is
  compared as f64 with strict equality (no epsilon approximation).
- Numeric strings are parsed with `str::parse::<f64>` (no JS `Number()` quirks like
  `"" → 0`, whitespace trimming, or hex).
- Object keys are compared in lexicographic order (serde_json's default `BTreeMap`), so
  difference entries may be ordered differently than the TS output; content is the same.
- Key existence checks look at own keys only (the TS reference's `in`-operator
  prototype-chain behavior is a bug and is not reproduced).
- serde_json defaults apply: parser recursion limit of 128 levels and out-of-range number
  literals (e.g. `1e999`) are parse errors.

## Development

```bash
# build library and CLI
cargo build

# run unit tests and CLI end-to-end tests
cargo test
```

The repo layout:

```
src/
├── compare.rs   # core diff engine (+ unit tests)
├── types.rs     # CompareOptions / Difference / enums
├── utils.rs     # type + path helpers
├── lib.rs       # public API re-exports
└── bin/cli.rs   # CLI entry point
tests/
└── cli_e2e.rs   # CLI end-to-end tests (builds the real binary)
```

## License

MIT
