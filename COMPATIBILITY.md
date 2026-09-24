# Cross-Language Compatibility

`@compare-json/core` (TypeScript) is the reference implementation. Ports exist in
[Go](https://github.com/unitstack/compare-json-go),
[Python](https://github.com/unitstack/compare-json-python), and
[Rust](https://github.com/unitstack/compare-json-rust).

**Policy:** each port uses its own language's native semantics and idioms. Ports do NOT
replicate JavaScript engine quirks. The main comparison functionality — the set of
differences, their `pathSegments` / `pathString` / `pathBelongsTo` / `diffType` — must be
correct and deterministic; cosmetic or platform-level differences are acceptable.

## Guaranteed to match across all implementations

- Difference entry JSON shape: `pathSegments` (string[]), `pathString` (dot notation with
  `[N]` array indices), `pathBelongsTo` (`base` | `contrast` | `both`),
  `diffType` (`added` | `deleted` | `typeChanged` | `valueChanged`).
- Options: `arrayCompareMethod` (`byIndex` default / `lcs` / `unordered`),
  `keyCaseInsensitive`, `valueCaseInsensitive`, `numericStringEqualsNumber`.
- Object recursion: added/deleted detection, case-insensitive key pairing, output paths
  always use the base-side key.
- Array strategies: `byIndex` positional recursion; `lcs` minimal diff via longest common
  subsequence (same tie-breaking); `unordered` multiset matching.
- `typeChanged` boundaries: number vs string, null vs object, array vs object, etc.
- `valueCaseInsensitive` applies to strings only.
- `pathString` rules: first segment as-is; later segments matching `[<digits>]` are
  appended directly, everything else is joined with `.`.

## Intentional differences (do not "fix" the ports to match these)

| Behavior | TypeScript (reference) | Go | Python | Rust |
|---|---|---|---|---|
| Numeric-string parsing (`numericStringEqualsNumber`) | JS `Number(s)`: trims whitespace, `"" → 0`, hex `0x10 → 16`, `Infinity` | `strconv.ParseFloat` | `float()` | `str::parse::<f64>` |
| Key iteration order | `Object.keys`: integer-like keys numerically first, then insertion order | lexicographic (sorted) | insertion order | lexicographic (BTreeMap) |
| Integers > 2^53 | precision loss (`JSON.parse`) | precision loss (float64) | exact (unbounded int) | exact (i64/u64) |
| Key existence check | `in` operator — **known bug**: hits the prototype chain (`"toString" in {}` is true, so an `added` key named `toString` is silently swallowed) | own keys only (correct) | own keys only (correct) | own keys only (correct) |
| Non-standard JSON (`NaN`, `Infinity` literals) | parse error | parse error | accepted by `json.loads` | parse error |
| Number equality | `===` (so `1` == `1.0`) | float64 `==` | `==` (`1` == `1.0`) | numeric comparison (`1` == `1.0`); integers compared exactly, floats as f64 |
| Parser recursion limit | none practical | none practical | ~1000 (`RecursionError`) | 128 (serde_json default) |
| Table column width | UTF-16 code units | bytes | code points | bytes |

Notes:

- **Key order** affects only the ordering of difference entries in the output, not their
  content. In case-insensitive mode with multiple case-variant keys in one object (e.g.
  `{"name": ..., "Name": ...}`), pairing order follows each language's key order, so the
  reported diffs may differ from the TS result. This is a pathological edge case.
- The TS `in`-operator prototype-chain behavior is a bug in the reference implementation
  (`object.ts`: uses `in` instead of `Object.hasOwn`). Ports intentionally do not
  reproduce it; the reference may be fixed in a future release.
- Rust previously used an `EPSILON` approximate comparison for numeric strings; this was
  removed — all implementations now use strict numeric equality after parsing.
