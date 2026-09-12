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
consumer and dependency sources must match acceptance baselinec158f4e; changes
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
