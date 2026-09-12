# Temporary Opus font-bundling acceptance

Codex continuation of the preserved mac-fonts-bundle lane, authorized for
2026-09-12 18:00:50–18:20:50 UTC. Baseline mac-shell `7a2f63f4`;
implementation checkpoint `56f9da95`. No compiler implementation or main writes.

* Official local CTAN archive: 19 supplementary faces independently verified
  byte-for-byte; archive SHA/size/member paths in `archive-verification.json`.
* Packaging self-test: 32 passed, zero failures; verifies missing/drifted/stray
  faces and existing metric/resource staging. Full preflight verifies 51 entries.
* Native BundledFacesTests with optical producer `f762f82a`: five passed,
  zero skips/failures, including actual CoreText resolution and missing-Roman8
  negative control with host TeX reads denied.
* Signed app-only export: seven fixtures passed under `9aaec57a`, seven under
  optical `f762f82a`, with host TeX and source font-directory reads denied.
  Reports preserve all compiler errors/recoveries: this is a font packaging gate,
  not full compiler compatibility or TeX reference pixel parity.
* Optical profile emits Roman8 and Roman6. Exact export of the original valid
  list refuses a copy missing Roman8, exit 1 naming its face/hash/byte length.
* Existing TFM acceptance with explicit bundled environment: seven passed,
  zero failures. Requiring implicit discovery fails one control with older
  `f762f82a`; use explicit FLASHTEX_TFM_DIRS as the native app does.

The pinned `9aaec57a` binary does not exercise Roman8/Roman6 in these cases.
The original test's assumption that deleting Roman8 must emit `font_unavailable`
was incorrect: optical producer `f762f82a` emits an `lmr8:` math-resource warning.
The corrected test requires actual optical faces and classifies this fallback;
other math-family profile warnings remain compiler limitations.

`base-app-exports/report.json` and `optical-app-exports/report.json` identify
actual signed binary hashes, declared source revisions, requests, diagnostics,
exports and correlations. Declared revision provenance comes from preserved
builds; these test binaries are not product adoption. Raw display/producer
captures are losslessly gzip-compressed (`gzip -dc file.gz`); report argv and
hashes describe original decompressed bytes and historical scratch paths.
Export PDFs are retained. Contact sheet was inspected: pages render legibly,
with unsupported macro recoveries visibly retained. It is not a reference oracle.

Reproduce from the repository root after making an app containing both helpers:

```sh
python3 apps/mac/scripts/faces-acceptance.py --app /absolute/FlashTeX.app \
  --evidence /fresh/evidence/directory --require-optical
FLASHTEX_RENDER=/absolute/optical/flashtex-render \
  swift test --package-path apps/mac --filter BundledFacesTests
bash apps/mac/scripts/texmf-acceptance.sh --app /absolute/FlashTeX.app \
  --evidence /fresh/metrics/directory
```

The harness bounds each producer/export subprocess to 45 seconds, preserves
failures, and requires matching request/project/revision on compile and display
frames. The missing-face export control requires the error to name Roman8.
The six existing real-world fixtures plus one nested-math fixture are this gate's
scope; broad unsupported compiler fixtures and upstream panics are not claimed.

The full Swift regression run with real helper environment stalled in
`DisplayListDeltaTests.testAcknowledgingAnOlderSnapshotYieldsFull`.
It was stopped after roughly four minutes by SIGTERM to the exact owned xctest
PID; SwiftPM exited 1 and released its lock. It is incomplete, not a passing
suite. Log/status are retained. Final focused optical tests passed five of five
again after the stop. Parent ShellModel/test infrastructure remains out of scope.

Final affected-consumer regression at published code `22bddb4c`: 18 tests,
zero failures/skips (BundledFacesTests, PreviewFontsTests,
FontHintResolutionTests, RuleGeometryTests, LayoutCapabilityPDFExportTests).
Recovery issue: https://github.com/flash-tex/flashtex/issues/50.
Draft stacked review: https://github.com/flash-tex/flashtex/pull/49.

The broad probe inherited FLASHTEX_COMPILER pointing to the parent's ordinary
v1 compiler; only FLASHTEX_RENDER was overridden. The delta test expected a
second display-list response and its blocking availableData read prevented the
nominal120s deadline from firing. This is a configuration mismatch exposing a
parent test timeout defect; it is not evidence of a font regression. The sample
and final flushed log identify DisplayListDeltaTests, correcting the provisional
DisplayCandidate location inferred from buffered output.
