# Raw helper binder attribution — FT023 revision13

`tools/profile_helper_binding.py` instruments an isolated archive of published
3040a0ce; it never edits production helper/rendering sources. Its three inputs are
the unchanged actual f5524794 raw-helper captures already pinned by the strict
acceptance tests. Run it with `--report PATH` using the existing offline Rust tools.

Two release-mode passes use three warmups and 20 samples per input. The timing pass
disables allocation counter updates. The separate allocation pass records calls
and cumulative requested capacity, including realloc requests; these numbers are
neither retained memory nor peak RSS. The global allocator still has a flag check
in timing mode. Phase timestamps and profiling scaffolding also add overhead, so
these observations are attribution, not calibrated production/native latency.

The temporary markers separate whole-event Value parse/drop, typed wrapper decode,
policy/source snapshot construction, existing pipeline pairing, immutable-resource
binding and typed-wrapper/bound-result drops. Pairing and resource binding include
their existing strict parsers and validators; no equivalent operation was removed
for the measurement. Profiling output and other scope cleanup are outside phase
times, so phase sums must not be presented as complete end-to-end latency.

Source hashes, exact base, archive hash, tool versions, per-sample measurements and
phase summaries are retained in the report. Missing or drifted instrumentation
anchors fail the command instead of measuring a different function. No fallback
parser, native switch, new corpus or relaxed duplicate/finite/depth rule is added.

Observed host after the run: `11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz`, `Linux-6.19.10-300.fc44.x86_64-x86_64-with-glibc2.43`.
Commander and both active product peers released benchmark windows; this is still
a single-process instrumented observation, not CPU-frequency-controlled calibration.

| Capture bytes | Preflight median ms | Preflight allocation calls | Requested bytes | Typed wrapper calls |
| --- | --- | --- | --- | --- |
| 364573 | 8.04 | 31787 | 3690626 | 10 |
| 739757 | 19.01 | 64487 | 7534526 | 10 |
| 1150821 | 14.64 | 100500 | 11849804 | 10 |

The preflight is the largest individual timed phase. Pairing and resource binding
combined are larger: for the largest capture they measure 9.87 ms and 9.84 ms, versus
14.64 ms preflight. Median timings are not monotonic across capture sizes (the
middle input measures 19.01 ms preflight), so do not extrapolate throughput or claim
an end-to-end speedup. Repeated parser work also remains a separate target.

The allocation evidence supports testing a serde visitor that validates the same
finite/depth syntax without constructing Value. Such a candidate must retain typed
original-byte duplicate checks, unknown-extension validation, current source and
immutable resource authority. No replacement has been implemented in this checkpoint.
Measured report: `tools/evidence/raw-binding-attribution.json` relative to crate root.

## Equivalent visitor candidate

The following implementation narrowly reuses the unchanged private runtime
`Syntax` serde visitor from 7817e4e8 in rendering-owned
`src/helper_candidate/syntax.rs`. It replaces only the initial whole-event Value
preflight. Typed original-byte wrapper/payload decoding and all subsequent source,
resource, duplicate and export-currentness checks remain intact. Equivalence tests
compare the old Value gate on finite/nonfinite/extreme numbers, unknown nested
fields, duplicate keys, invalid Unicode/UTF-8 and the recursion boundary. Existing
three actual raw captures continue to export the same pinned PDFs.

The profiler now accepts an explicit `--base` commit and labels Value versus
Syntax preflight separately. Its default still reproduces the original3040a0ce
baseline. A candidate performance claim requires a separate completed report;
unit tests alone do not prove allocation or latency improvement.

## Measured candidate result

Candidate 02da0568 completes the same 138 instrumented bindings. All measured
Syntax preflights request zero allocations/capacity, removing 31,787 / 64,487 /
100,500 calls and 3,690,626 / 7,534,526 / 11,849,804 requested bytes from the three
captures. Every other phase's allocation-call/capacity median is exactly unchanged.
These are cumulative requests, not retained-memory or RSS measurements. Escaped
strings can require serde scratch storage; zero allocation is scoped to the three
captured inputs, not arbitrary JSON.

Observed candidate preflight medians are 0.92 / 2.01 / 3.54 ms. Old and new timing
passes were separate instrumented processes; other phase times also vary. Do not
present their difference as a calibrated whole-binder or native speedup. The
allocation reduction is the robust measured result. See
`tools/evidence/syntax-binding-attribution.json` and
`tools/evidence/syntax-allocation-comparison.json` relative to crate root.

The production source/dependency guard was deliberately rebased to tested 02da0568
and all five established producer/reference fixtures replayed. Original PDF
hashes, raster hashes and extraction outcomes exactly match the b797b21a baseline.
Reference pixel differences remain 0 / 602 / 1244 / 979 / 337; display-math reference
ToUnicode remains an explicit oracle limitation. The expected exit 3 therefore
records existing reference gaps, not a new output regression. Evidence:
`tools/evidence/syntax-five-fixtures.json` relative to crate root.
