# Six clean edited HW1 TeX oracles

Codex generated each HW1 probe's recorded exact edit from clean auxiliary state
using its declared three-pass MacTeX workflow. **Six one-page edited PDFs, no
warnings, all six final pages inspected individually at recorded 96 DPI RGB
Ghostscript settings.** Original sources and reference PDFs remain unchanged.

The accepted edited source, PDF, TeX logs and full engine/dependency/source/PDF
hashes are in `tests/extended-tex-corpus/edited-references/CASE_ID/`.
`edited-reference-index.json` pins the manifest and edit definitions, both
original and edited sources, the exact edit, clean auxiliary state, engine and
PDF hashes. The full development suite passes **15 tests**, including exact
edited-source/PDF/provenance integrity and rejection of unknown/negative profiles
before creating output or invoking TeX.

```sh
python3 tools/extended-tex-corpus/edited_reference.py \
  --only hw1-blackboard-macros --only hw1-array-cases \
  --only hw1-delimiters-kerns --only hw1-title-size-scope \
  --only hw1-paragraph-registers --only hw1-heading-hfill \
  --output /private/tmp/flashtex-hw1-edited-oracles-20260912 --render
```

Each edit produces an actual raster change relative to its original MacTeX
oracle. `oracle-differences.json` records exact changed-pixel counts, bounding
boxes, full-page dimensions, original/edited PDF and raw RGB raster hashes, and
the pinned Ghostscript version/executable and arguments. Both PDFs were rendered
at 96 DPI with TextAlphaBits=4 and GraphicsAlphaBits=4. Comparison uses full raw
RGB pixels, with no translation, cropping, rescaling or tolerance. This merely
checks that the fixture edit affects its oracle; it is **not** FlashTeX parity.

Inspected effects: the blackboard macro changes dependent uses to Q; array column
alignment changes the first array's cells; the selected math-space row widens;
the smaller scoped title changes its glyphs and centered width while explicit
row spacing preserves the following content; paragraph glue propagates while the
local register override remains; fixed
heading space moves both points labels while other fill/quote content stays.
The hfill probe's unchanged explanatory prose deliberately still describes the
original right-edge check; the edited heading macro now uses fixed quad space.

Next acceptance gate: use these exact edited bytes and PDFs for complete original
compiler + renderer comparisons. The companion incremental runner already passed
84 raw warm/clean consistency comparisons against a pinned local compiler whose
build-source revision is unknown. Neither consistency nor TeX oracle generation
establishes compiler support, rendered parity, device acceptance or current-main
correctness. Warm book auxiliary-file workflows remain a separate requirement.
No paid calls, children, owner worktree edits or main/control writes were made.
