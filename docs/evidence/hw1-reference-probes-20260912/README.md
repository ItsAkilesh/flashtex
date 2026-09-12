# Six isolated HW1 reference probes

Owner: Codex `mac-reference-corpus`, continuing the user's direct reference-test
assignment alongside Opus. Branch `agent/mac-reference-corpus/hw1-probes` stacks
on the existing reference-corpus PR42; no compiler/app implementation changed.

All six positive projects compile with installed MacTeX pdfLaTeX into one-page
PDFs with no warnings. Exact source/engine/dependency/PDF hashes and command
results are in each `tests/extended-tex-corpus/references/hw1-*/reference.json`.
The existing runner uses no shell escape and a 60-second per-process bound.
Every final page was individually inspected using its recorded 96 DPI RGB
Ghostscript rendering. Heading hfill initially lacked a paragraph boundary;
the source was fixed, freshly compiled/rendered, and reinspected before acceptance.

| Probe | Requirement it isolates | Local runtime-v1 diagnostics |
| --- | --- | ---: |
| hw1-blackboard-macros | All 26 blackboard capitals, macros, scripts/styles, setminus | 45 |
| hw1-array-cases | l/c/r columns, vertical rule, explicit row gaps, cases, @{} padding | 30 |
| hw1-delimiters-kerns | Manual sizes, middle/null delimiters, positive/negative math space | 52 |
| hw1-title-size-scope | Large/LARGE, nested size/weight restoration, center/rule/vertical gaps | 14 |
| hw1-paragraph-registers | parindent/parskip and local register restoration | 12 |
| hw1-heading-hfill | Starred heading macro, normal-font points, equal fill, quote width | 20 |

The local original compiler returned `recovered` for all six, 173 diagnostics
total. This baseline is diagnostic evidence only. Its binary hash was verified
unchanged before/after:
`1d615ef71e59435a45846aaf5b763d4e17589e6ce5ee1b55d4a47859c1c98238`.
Its build-source revision was not independently established; do not call this a
current-main or render-pipeline compatibility result. `baseline.json` preserves
diagnostics and binary identity; adjacent request/stdout/stderr files preserve the
exact input/output. A clean process received each emitted runtime-v1 request with
a 20-second timeout. The final corrected hfill source was used in the baseline.

Acceptance: compare every page and its geometry, fonts, glyphs, spacing, text
order and diagnostics. PDF bytes and pinned-raster pixels are separate gates.
Do not translate, crop, rescale, blur or adjust tolerance to make a candidate
match. An explicit limitation remains an unimplemented feature.

Validation: `reference.py --validate` accepts 55 projects; the existing seven
integrity/byte-edit/profile/timeout/request tests pass. All old reference sources
and PDFs remain unchanged. Current total: 55 projects, 48 positive PDFs/57 pages,
seven negative cases, eight exact edit scenarios. The newly generated source
hashes match the accepted PDFs' provenance; no old oracle was regenerated.
