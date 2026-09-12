# Compiler compatibility and recovery corpus

Owner: FT-011 / commander-corpus. Created September 12, 2026. Contract:
[Runtime v1](../../../docs/contracts/runtime-v1.md).

These 14 original small projects define requirements for the original Rust
compiler. **No case currently claims verified compiler support.** Validation here
checks fixture and manifest integrity, not rendering or full LaTeX compatibility.
The complete product requirement remains larger than this finite corpus.

Run from repository root:

```sh
python3 tests/tex-corpus/validate.py
python3 tests/tex-corpus/validate.py --emit-request included-file > /tmp/flashtex-include-request.jsonl
```

The second command emits exactly one runtime-v1 JSON line, with all project source
files included. It can be sent to a compatible compiler process when available;
it does not invoke any installed TeX engine. No network, package installation,
external assets, or paid inference is needed. Python 3.9+ is sufficient.

## Manifest and expected behavior

`manifest.json` defines each case's entry file, complete source inventory, features,
compatibility profile, and expectations. Every expectation cites a source witness:
project-relative file, zero-based UTF-8 start byte, exclusive end byte, and the
exact original source text. These ranges are fixture evidence, not assertions that
an entire expanded macro must map one-to-one to its invocation. Literal navigation
cases require original file/range identity. Expansion provenance beyond that is an
open contract design decision; do not fabricate exact per-glyph provenance.

| Cases | Requirement |
|---|---|
| plain-paragraphs | Text order and paragraph boundary |
| macro-arguments, macro-scope | Argument substitution and local binding restoration |
| unicode-literals, literal-source-map | Unicode preservation and byte-accurate navigation |
| math-inline-display | Inline scripts and a real two-dimensional fraction |
| included-file, include-scope | Included source identity and caller execution scope |
| comments-escapes | TeX comment line removal and escaped literal characters |
| unknown-command, unclosed-group, extra-closing-group, missing-include | Explicit diagnostic plus usable provisional output |
| tikz-required | Required package/diagram support; unsupported must remain explicit |

Profiles separate basic LaTeX, Unicode-capable LaTeX, and TikZ package behavior.
Unicode cases require an encoding/font profile with the necessary glyph coverage;
this corpus supplies no font and does not assert identical output across engines.
TikZ is intentionally an outstanding compatibility case, not a request to hide a
reference compiler behind the product or substitute a static image.

`visible_text` specifies semantic text order; output may split literal runs into
items and normalize ordinary source whitespace. `math_layout` and `diagram_layout`
need layout/visual assertions, not string comparison. `source_navigation` checks
clickable source identity against the exact input revision. `diagnostic` plus
`recovered_text` specifies FlashTeX recovery behavior, not a promise that all
reference TeX engines recover identically. Diagnostic wording is not prescribed;
location, severity, recovery explanation, and remaining content matter.

## Evidence required before calling a feature supported

A future compiler runner must record the compiler commit/build command, input
case/revision, selected compatibility profile, result status, diagnostics, and the
actual output artifact or display list. It should validate every returned source
range against that revision and retain failed cases. Record observed support in a
separate result artifact, with pass/fail/unsupported/unverified per expectation;
do not edit the fixture to match a compiler defect or mark unsupported as passing.
For unsupported required constructs, an explicit diagnostic passes only the
honesty/recovery gate, not the feature's rendering gate.

For performance and incrementality, use paired edits and compare incremental
output with a clean compile of the identical edited sources; this static corpus
alone provides neither a latency measurement nor proof of cache correctness.
Full project completion additionally requires native builds, PDF/export checks,
real capture conversion, package compatibility, and end-to-end validation.

## Execute the compiler and retain evidence

```sh
python3 tests/tex-corpus/run.py \
  --compiler-sha FULL_40_CHARACTER_BUILD_REVISION \
  --build-command 'cargo build --offline --manifest-path crates/compiler/Cargo.toml' \
  --output /tmp/flashtex-corpus-results.json \
  -- /absolute/path/to/flashtex-compiler
python3 -m unittest discover -s tests/tex-corpus -v
```

The runner starts one bounded subprocess per case, sends one request, closes its
stdin, and requires one correlated response before the default 10-second timeout.
Executable arguments are passed directly without a shell. Supply a trusted local
compiler executable. Requests, raw responses/stderr, exact source SHA supplied by
the operator, manifest hash, checks, and elapsed times are retained. The runner
records the build command; it does not verify that an arbitrary binary came from
that SHA. Time includes process startup and is not the live-preview benchmark.

Results distinguish `pass`, `fail`, `unsupported`, and `unverified`. A failed
machine check takes precedence; explicit unsupported diagnostics prevent a case
pass; a remaining manual semantic/layout gate prevents a case pass. Machine checks
cover correlation, source-range integrity, required text order, literal source
anchors and selected recovery diagnostics. A `pass` is limited to those stated
checks, not native click behavior, complete layout fidelity, or general feature
support. Display-list text joining assumes v1 word/run reading order; disagreements
need inspection of retained items before attributing a defect to the engine.

[First compiler evidence](evidence/compiler-9f1033b.json) uses an offline build of
reviewed compiler commit `9f1033ba75176236bc5fc66b5e64b8ebbfc15a37`, extracted into
a temporary directory without editing the compiler owner's checkout. This is a
failure/unsupported baseline, not a compatibility certificate. The original
extra-closing-group witness was corrected to point to the stray body brace rather
than the earlier closing brace in `\\documentclass{article}`.

## Persistent-process versus clean-build equivalence

`edit-scenarios.json` defines five paired-edit sequences: a Unicode byte shift,
macro definition edit, included-file edit, malformed input repaired, and valid
input made malformed then repaired. Each sequence starts one persistent compiler,
sends a cold request, an unchanged-source request at a new revision, and each edit
at a newer revision. Every step is independently compiled in a fresh process.

```sh
python3 tests/tex-corpus/incremental.py \
  --compiler-sha FULL_40_CHARACTER_BUILD_REVISION \
  --build-command 'cargo build --offline --manifest-path crates/compiler/Cargo.toml' \
  --output /tmp/flashtex-incremental-results.json \
  -- /absolute/path/to/flashtex-compiler
```

Comparison checks status, complete pages/display items, diagnostics, and actual
PDF bytes when a path is supplied. PDF path names are not compared; a missing
promised artifact fails. Each response must match request ID, project and revision.
A stale or mismatched response is retained and rejected, never adopted as output
for the latest revision. Fake-process tests exercise stale responses and divergent
persistent state; this sequential harness does **not** establish that the native
app drops asynchronously delivered stale previews.

[Recorded equivalence evidence](evidence/incremental-9f1033b.json) at compiler
`9f1033ba75176236bc5fc66b5e64b8ebbfc15a37` passes all five sequences (16 comparisons).
That compiler explicitly performs full recompilation and has no cache reuse.
Identically wrong or unsupported results can be equivalent: the separate semantic
corpus still records five failed and nine unsupported cases. This check prevents
future caching from changing output; it does not prove incrementality exists.

Raw requests, responses, diagnostics, stderr and individual timings are retained.
Cold and clean timings include process creation; warm/edit timings measure the
JSONLines request/response on an already running process. They are single local
samples from tiny fixtures, not end-to-end editor latency, representative-document
benchmarks, or a sub-200ms product guarantee. Semantic source witnesses from the
baseline corpus are deliberately not reused after edits move those ranges; only
source integrity against the actual edited revision is used here.

## Deterministic non-regression gate

`compare.py` compares a candidate semantic corpus result against the explicit
`compiler-9f1033b.json` baseline. Supply the exact expected source revision and
build command for both artifacts; candidate evidence must be no more than one
hour old by default (`--max-candidate-age-seconds` explicitly adjusts that gate).
The historical baseline is exempt from the age limit. A source revision is an
operator-attested build identity, not cryptographic proof of executable origin.

```sh
python3 tests/tex-corpus/compare.py \
  --baseline-build-command 'EXACT BUILD COMMAND RECORDED IN BASELINE' \
  --candidate /tmp/flashtex-corpus-results.json \
  --candidate-sha FULL_40_CHARACTER_BUILD_REVISION \
  --candidate-build-command 'EXACT BUILD COMMAND USED FOR CANDIDATE' \
  --output /tmp/flashtex-corpus-comparison.json
```

The gate rejects wrong SHA/build provenance, old candidate timestamps, changed
manifest hashes, missing/duplicate cases, requests that differ from current source
fixtures, and checks/status summaries inconsistent with replaying the retained raw
compiler response. It never executes the recorded build command. If fixtures or
checker semantics change, regenerate comparable evidence deliberately; do not edit
old results until a regression disappears.

Output reports newly passing, regressed, still passing/failing/unsupported/unverified,
and changed-but-incomplete cases in deterministic case order. Unsupported versus
unverified is not treated as an ordered correctness score. Any previously passing
check that stops passing blocks the gate, even inside a case that still fails or
has otherwise improved. A new passing case cannot offset another regression.
Exit 0 means no detected regression, 1 means regression, and 2 means evidence was
rejected. Nine unsupported and five failing cases can therefore pass a comparison
against themselves; **this does not mean the compiler or project is complete**.

[Baseline self-check](evidence/baseline-self-check.json) demonstrates that distinction
and includes hashes of both input artifacts. It is a validator check using the
same compiler evidence on both sides, not progress by a newer compiler revision.

## PDF artifact structure and placement evidence

`pdf_check.py` consumes verified semantic-run evidence, invokes a supplied trusted
PDF writer on each retained compile-result, and keeps one PDF per case plus a
linked `results.json`. It records compiler/writer revisions and build commands,
the compiler-evidence hash, per-PDF byte hashes, renderer stderr/warnings, original
request identity, and compiler semantic case status. Existing PDF artifacts are
not overwritten. Run with exact provenance values:

```sh
python3 tests/tex-corpus/pdf_check.py \
  --results /tmp/flashtex-corpus-results.json \
  --compiler-sha FULL_COMPILER_SHA \
  --compiler-build-command 'EXACT COMPILER BUILD COMMAND' \
  --renderer /absolute/path/to/flashtex-pdf \
  --renderer-sha FULL_PDF_WRITER_SHA \
  --renderer-build-command 'EXACT PDF BUILD COMMAND' \
  --output-dir /tmp/flashtex-pdf-evidence-new
```

The independent checker targets the current uncompressed text-only writer profile:
PDF 1.4 header, terminal `startxref`/EOF, every classic xref object offset, trailer
root/size, flat page tree/count, page MediaBoxes, stream lengths, text-run counts,
and each font size/baseline position against the display list (0.0011pt tolerance
for the writer's decimal rounding). Baseline anchors must lie within page bounds.
Unsupported stream/operator profiles are explicit, not accepted as validated PDF.
This is not a general PDF parser or conformance certification.

[Actual PDF evidence](evidence/pdf-a0855dd/results.json) uses renderer
`a0855dd46ecebd70f2e02ea1a56a9f48499b1034`, built offline from an isolated archive of
`origin/agent/mac-pdf/pdf-output`. All 14 artifacts pass these structural/placement
checks. The underlying compiler still has five failing and nine unsupported cases.
WinAnsi substitutions remain visible in renderer warnings, and Unicode/font/math
correctness remains unverified. Glyph ink bounds and advances need real font
metrics; checking the baseline anchor does not prove the entire word fits inside
the page. Native rendering, visual comparison, and the app's export workflow remain
separate acceptance gates. PDFs here are test evidence, not user-ready documents.

## PDF text consistency and explicit substitution

```sh
python3 tests/tex-corpus/pdf_consistency.py \
  --pdf-evidence tests/tex-corpus/evidence/pdf-a0855dd/results.json \
  --compiler-evidence tests/tex-corpus/evidence/compiler-9f1033b.json \
  --output /tmp/flashtex-pdf-consistency.json
```

This additional checker decodes each escaped PDF literal string and compares its
bytes with the runtime text encoded in the current writer's WinAnsi profile. Any
unrepresentable codepoint must become `?` and have a matching warning naming the
page, item and lost codepoints. A warning for another item cannot excuse loss.
Dropped/duplicated items, changed text bytes, shifted placement and out-of-page
baseline anchors fail. Page/item/source identity stays attached to every result.
Escaping, multiline placement, distinct pages, Unicode loss, controls, duplicate
items and corruption are tested with deterministic generated artifacts.

Input compiler/PDF evidence hashes, original request identity and compiler semantic
status must match. Existing PDF hashes are checked before reading content. The
checker preserves failing/unsupported compiler labels even when PDF serialization
is consistent. A consistency pass with `text_preserved: false` explicitly records
loss; it does not satisfy faithful Unicode rendering or a visual acceptance gate.

[Actual text consistency evidence](evidence/pdf-consistency-a0855dd.json) rechecks
all 14 artifacts from exact writer `a0855dd46ecebd70f2e02ea1a56a9f48499b1034`:
14 serialization-consistency passes, **two cases with warned substitutions**.
Compiler semantic status remains five failing and nine unsupported cases. No
native visual/font/math correctness is established by these byte-level checks.
