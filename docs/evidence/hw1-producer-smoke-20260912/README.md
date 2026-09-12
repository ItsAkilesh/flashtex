# Original and edited HW1 producer smoke baseline

Codex reused the existing real-world corpus runner read-only, with its scratch
work redirected outside the owner's tree and every subprocess timeout reduced
to 20 seconds. No TeX, PDF export or rasterization was invoked. Each of six
original and six accepted edited HW1 sources was sent to both existing producers.

**24/24 jobs exited zero, returned one correctly correlated runtime-v1
compile_result and did not time out. All 24 results were recovered.** The compiler
reported 346 diagnostics across twelve jobs; the bundled render helper reported
80. Counts across different producers are not a support score. Measured worker
wall time totaled 0.062 seconds for compiler and 0.235 seconds for render; these
small-fixture runs do not establish interactive latency.

Both artifact hashes stayed unchanged, and **both build-source revisions are
unknown**:

- compiler: `1d615ef71e59435a45846aaf5b763d4e17589e6ce5ee1b55d4a47859c1c98238`
- bundled renderer: `7a7c63e427541c2fed9eb23ac376afcd3c15ec108a8a7568abdf4614a4a2abd1`

The renderer's actual diagnostic output names flattened arrays/cases, dropped
math glue, approximate math outline profiles, unsupported paragraph registers
and font sizes, quote/center fallback and dropped hrule. These are findings about
this artifact. Several corresponding diagnostic strings are absent from the
current root render-pipeline sources; do not infer current-main gaps from this
older/unverified bundle. A fresh renderer with independently verified build
provenance must be tested before release acceptance.

`report.json`/`report.md` retain the existing harness's per-construct diagnostic
inventory and source spans. `input-provenance.json` pins all original/edited
sources and reference PDFs plus runner and artifact hashes; `hashes-after.json`
verifies they were unchanged. `reply-validation.json` independently checks
single-reply count, IDs, protocol/version/type/revision and worker exit state.
`raw-producer-traces.zip` preserves source copies, raw replies, render display
lists, reports and reconstructed requests. The requests were reconstructed from
the unchanged inputs with the runner's exact deterministic recipe after execution;
they were not a separate stdin recording. Accepted reference PDFs are addressed
by hash in the repository and are omitted from this trace archive.

Reproduction uses `tools/real-world-corpus/run.py --fixtures SCRATCH_FIXTURES
--no-oracle --no-pixels --compiler ABS_COMPILER --render ABS_RENDER --pdf-exact ''
--pdf-v1 '' --out SCRATCH_REPORT`, with twelve source/reference directories and
the explicit artifact provenance above. The existing runner's HERE scratch path
and run() timeout were overridden only in memory; its owned files were not edited.

This is transport/recovery diagnostic evidence, not rendered correctness, TeX
support, PDF parity, current-main acceptance or a released application claim.
No main/control writes, owner files, paid provider/Claude calls, children,
purchases or resource allocations were changed.
