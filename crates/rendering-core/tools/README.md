# Pinned original/reference replay

From the repository root, run one command with the existing pinned toolchain:

```
PATH="$HOME/.cargo/bin:$PATH" python3 crates/rendering-core/tools/replay_original.py --metrics-root /path/to/official-lm-root --report /tmp/flashtex-reference-report.json
```

The root must contain the four unmodified official LM2.004 files under
`fonts/tfm/public/lm/` and their license at
`doc/fonts/lm/GUST-FONT-LICENSE.TXT`. Exact SHA256 values are embedded in the
runner. It snapshots these assets, the request, LM12 font/license and actual
pdfTeX reference after verification; no fonts are downloaded or installed.

The runner extracts untouched published producer65dbe7d, builds it offline with
two jobs, requires status `ok`, zero diagnostics and exactly one page, then calls
the existing immutable registry/CFF searchable export example. The current
consumer and dependency sources must match acceptance baselinecdd5e979; changes
require a deliberate reviewed rebaseline. Original producer output is never
rewritten. Source-preamble transformation remains recorded and explicit.

It calls the existing PDF owner comparison API through `pdf_compare`, Poppler26.01
`pdftotext`/`pdftoppm` and Pillow12.1 only for decoded RGB comparison. Cargo/rustc
versions are pinned as well. The reference PDF is the committed established
pdfTeX artifact; it is not generated from, or re-emitted by, FlashTeX. The report
retains raw/operator comparisons separately from extraction and144DPI RGB
comparison. Different PDF bytes/operators do not masquerade as raster mismatch.

Exit statuses:

- `0 accepted`: this one fixture has zero diagnostics and exact text/RGB equality.
- `2 refused`: assets or producer diagnostics invalidate the acceptance setup.
- `3 mismatch`: measured page geometry, extracted text or pixels differ.
- `4 unknown`: tools/version/source baseline differ, timeout, or unsupported/truncated comparison.
- `1 error`: an execution or report-processing failure prevented measurement.

The JSON report is written on every handled outcome. Subprocesses have bounded
timeouts. Builds use disposable directories under rendering-core/target to avoid
exhausting shared /tmp, and temporary snapshots are cleaned on completion. Cargo
registry dependencies must already exist; no install, login, purchase or network
fallback is attempted. This does not imply all-document/all-resolution parity.

Observed checks: complete runner accepted the rooted-assets fixture; an absent
asset root returned2/refused; an intentionally fake version-only pdftotext
executable returned4/unknown before compilation. No rasterizer was replaced in
an acceptance run. Reports are retained under tools/evidence with the runner's
publication checkpoint.

## Three established fixtures (v2 runner)

Add `--fixture plain`, `--fixture inline-math`, `--fixture display-math`, `--fixture wrapping`, or
`--fixture all`. Default remains plain. `replay-fixtures.json` pins each actual
source, request, reference PDF/engine metadata, font and license; its own digest
is pinned in the runner. Metrics remain separately pinned. Consumer baseline is
nowcdd5e979 and includes the multi-font exporter plus the reference CMap regression.
Source/dependency drift returns unknown before builds. Rebaseline is an explicit
review decision, never automatic repair. One archived producer build serves all
selected fixtures; each is measured even when another differs.

The v2 report nests each result under `fixtures`, including asset hashes, exact
PDF comparison, `raster.equal`, `extracted_text.equal`, and a separate
`text_comparison_validity`. `limited_linear_text` is deliberately narrower than
an exhaustive Unicode/structure/accessibility guarantee. Display math is an
`oracle_limitation`: its pinned reference has no ToUnicode entry for shown
summation code88. This fact is verified by the pinned existing-reader Rust test;
the Python runner does not introduce another CMap parser. It never rewrites ∑ to
X. Even zero raster differences cannot yield acceptance with this incomplete
text oracle. A real raster difference still reports mismatch independently.

Aggregate severity is execution error, unknown, refused, mismatch, accepted;
all per-fixture results remain available. The report records executable hashes
for invoked tools, version strings, producer binary/archive and runner/manifest
hashes. Rustup-managed cargo/rustc paths may be shims; those hashes identify the
invoked executable, while the version strings identify the selected toolchain.
No claim of complete hermetic build provenance follows.

Validation: `python3 -m unittest discover -s crates/rendering-core/tools -p
 test_replay_original.py` checks six decision/setup gates using explicitly
synthetic inputs. The real `--fixture all` report separately reproduces0/602/1244
pixel differences and true/true/false text equality. `next-gap.json` selects only
existing02-wrapping-paragraph from the pinned Mac closure matrix for the next
investigation; its different rasterizer evidence is not compared numerically as
though it were this Linux run.

The added `wrapping` selection replays existing02 with the same resources. Its
actual report has13 matching line memberships/210 words, equal text, and979
differing pixels. `all` now includes this fourth established fixture; historical
three-fixture reports retain their original scope and runner hashes.

The `ligatures` selection adds existing18 with original GID/source-interval and
reference ToUnicode checks. Its measured337-pixel mismatch remains separate from
matching three-line word membership and extracted text. `all` now covers the
five explicitly selected fixtures, not the complete corpus. The aggregate in
completed-fixtures.json retains the actual report hashes for each checkpoint;
it does not pretend these were one simultaneous benchmark run.

Explicit source rebaseline cdd5e979 follows the additive helper_candidate adapter
and164 passing Rust tests. It changes no producer/immutable export implementation;
a fresh five-fixture run verifies exact prior PDFs and0/602/1244/979/337 raster
counts. Historical report hashes are preserved; no unknown drift is auto-approved.
