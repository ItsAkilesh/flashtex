# rendering-v2 → exact export: measured gap against the pdflatex-lmodern reference

Agent `mac-pdf` (parent `mac-claude-a`, `mac-m1max-a`), branch
`agent/mac-pdf/v2-adapter`. Written 2026-09-12 from an actual run; every
number below came out of the commands in "Reproduction". This is the first
end-to-end path that feeds the exact route from FlashTeX's own producer, and
the deliverable is the **measured gap per category**, not parity: the
upstream layout is not pdfTeX's and nothing here claims it is.

## Pipeline under test

```
fixture body  ──►  flashtex-render --v2 --secnumdepth 0  ──►  display_list envelope (rendering-v2, bp_2pow20 ticks)
              ──►  flashtex-pdf-exact from-v2 LIST.json --out OUT.pdf
                     glyph_run → GlyphRun/PlacedGlyph by original GID, every glyph at its exact tick origin
                     fonts by content hash → lmroman*.otf / latinmodern-math.otf → GID-preserving CID-keyed CFF subset
                     rule → Op::rule; page size from ticks
              ──►  flashtex-pdf-exact classify REF.pdf OUT.pdf  +  CoreGraphics 144 dpi raster comparison
```

| item | value |
|---|---|
| producer | `flashtex-render` built from `origin/agent/mac-render-pipeline/unified` at `ba5611f` (`git archive` into a scratch directory, `cargo build --release --bin flashtex-render`, as `tools/typing-bench/run.sh` does), invoked with `--secnumdepth 0 --v2`, one request per fixture, body from `\begin{document}` only (the harness strips the preamble) |
| adapter | `flashtex-pdf-exact from-v2` from this branch (`crates/pdf/src/v2.rs`), default font search (MacTeX 2026 Latin Modern directories) |
| reference | MacTeX 2026 pdflatex (pdfTeX 1.40.29) with the harness `lm` preamble (`\documentclass[12pt]{article}`, T1 fontenc, `lmodern`, `geometry` margin 1in, `\parindent` 0, `secnumdepth` 0, `\pagestyle{empty}`); the same references as `exact-export-classification.md` |
| comparison | `flashtex-pdf-exact classify` (categories as in `exact-export-classification.md`; fonts unmatched by resource name are paired by base font name so program/metadata gaps are measured) and CoreGraphics `CGContextDrawPDFPage` at 144 dpi. `% px differ` counts any RGB difference (anti-aliasing included); `ref-only`/`ours-only` count pixels that carry ink on one side only, i.e. differences a reader can see |

## Result, per fixture

| fixture | from-v2 | content | fonts | page geometry | CoreGraphics 144 dpi vs pdflatex-lmodern | pipeline diagnostics |
|---|---|---|---|---|---|---|
| 01-plain-paragraph | 1 page(s), 13 glyph run(s), 55 glyph(s) (0 continued by hmtx advance), 0 rule(s), 29547 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 1/1; p1 0.01% px differ; ink 4355 vs 4355, ref-only 0, ours-only 0 | none |
| 02-wrapping-paragraph | 1 page(s), 210 glyph run(s), 896 glyph(s) (0 continued by hmtx advance), 0 rule(s), 96447 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 1/1; p1 0.16% px differ; ink 72584 vs 72578, ref-only 74, ours-only 68 | none |
| 03-section-heading | 1 page(s), 15 glyph run(s), 79 glyph(s) (0 continued by hmtx advance), 0 rule(s), 54935 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 1/1; p1 0.02% px differ; ink 9373 vs 9372, ref-only 12, ours-only 11 | none |
| 04-bold-emph | 1 page(s), 12 glyph run(s), 54 glyph(s) (3 continued by hmtx advance), 0 rule(s), 104089 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 1/1; p1 0.01% px differ; ink 4863 vs 4863, ref-only 0, ours-only 0 | none |
| 05-unicode | 1 page(s), 11 glyph run(s), 46 glyph(s) (0 continued by hmtx advance), 0 rule(s), 28931 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 1/1; p1 0.01% px differ; ink 3904 vs 3904, ref-only 0, ours-only 0 | none |
| 06-math-inline | 1 page(s), 13 glyph run(s), 45 glyph(s) (0 continued by hmtx advance), 2 rule(s), 140263 bytes | ContentOperators | FontMetadata; FontProgram; FontResources | same | pages 1/1; p1 0.09% px differ; ink 3432 vs 3440, ref-only 162, ours-only 170 | none |
| 07-math-display | 1 page(s), 10 glyph run(s), 47 glyph(s) (1 continued by hmtx advance), 1 rule(s), 139697 bytes | ContentOperators | FontMetadata; FontProgram; FontResources | same | pages 1/1; p1 0.21% px differ; ink 3889 vs 3971, ref-only 1396, ours-only 1478 | none |
| 08-two-page | 3 page(s), 1800 glyph run(s), 7680 glyph(s) (0 continued by hmtx advance), 0 rule(s), 628112 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 3/3; p1 0.56% px differ; ink 254503 vs 254497, ref-only 240, ours-only 234; p2 0.55% px differ; ink 254661 vs 254649, ref-only 232, ours-only 220; p3 0.24% px differ; ink 112626 vs 112614, ref-only 108, ours-only 96 | none |
| 09-mixed-document | 1 page(s), 50 glyph run(s), 206 glyph(s) (2 continued by hmtx advance), 3 rule(s), 203293 bytes | ContentOperators | FontMetadata; FontProgram; FontResources | same | pages 1/1; p1 0.70% px differ; ink 17757 vs 17803, ref-only 3289, ours-only 3335 | none |
| 10-unicode-paragraph | 1 page(s), 81 glyph run(s), 425 glyph(s) (0 continued by hmtx advance), 0 rule(s), 62869 bytes | ContentOperators | FontMetadata; FontProgram; FontResources | same | pages 1/1; p1 0.16% px differ; ink 34516 vs 34285, ref-only 822, ours-only 591 | missing_glyph |
| 11-nested-lists | 1 page(s), 29 glyph run(s), 147 glyph(s) (0 continued by hmtx advance), 0 rule(s), 36762 bytes | ContentOperators | FontMetadata; FontProgram; FontResources | same | pages 1/1; p1 1.01% px differ; ink 11160 vs 11146, ref-only 9457, ours-only 9443 | none |
| 12-justified-paragraphs | 1 page(s), 360 glyph run(s), 1536 glyph(s) (0 continued by hmtx advance), 0 rule(s), 146226 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 1/1; p1 0.28% px differ; ink 124408 vs 124396, ref-only 148, ours-only 136 | none |
| 13-math-display-rich | 1 page(s), 17 glyph run(s), 72 glyph(s) (2 continued by hmtx advance), 3 rule(s), 145014 bytes | ContentOperators | FontMetadata; FontProgram; FontResources | same | pages 1/1; p1 0.44% px differ; ink 5244 vs 6172, ref-only 3388, ours-only 4316 | compiler, compiler |
| 14-math-inline-dense | 1 page(s), 69 glyph run(s), 147 glyph(s) (1 continued by hmtx advance), 4 rule(s), 155485 bytes | ContentOperators | FontMetadata; FontProgram; FontResources | same | pages 1/1; p1 0.81% px differ; ink 10622 vs 10971, ref-only 5829, ours-only 6178 | compiler, overfull_hbox |
| 15-three-page-sections | 3 page(s), 1806 glyph run(s), 7695 glyph(s) (0 continued by hmtx advance), 0 rule(s), 652110 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 3/3; p1 0.46% px differ; ink 208434 vs 208422, ref-only 240, ours-only 228; p2 0.46% px differ; ink 208508 vs 208496, ref-only 240, ours-only 228; p3 0.46% px differ; ink 208516 vs 208504, ref-only 240, ours-only 228 | compiler, compiler |
| 16-heading-page-break | 2 page(s), 962 glyph run(s), 4109 glyph(s) (0 continued by hmtx advance), 0 rule(s), 371875 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 2/2; p1 0.56% px differ; ink 254503 vs 254497, ref-only 240, ours-only 234; p2 0.17% px differ; ink 80091 vs 80080, ref-only 73, ours-only 62 | none |
| 17-apostrophes | 1 page(s), 25 glyph run(s), 132 glyph(s) (0 continued by hmtx advance), 0 rule(s), 36418 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 1/1; p1 0.01% px differ; ink 9482 vs 9482, ref-only 0, ours-only 0 | none |
| 18-ligatures | 1 page(s), 36 glyph run(s), 170 glyph(s) (0 continued by hmtx advance), 0 rule(s), 39690 bytes | ContentOperators | FontMetadata; FontProgram | same | pages 1/1; p1 0.04% px differ; ink 14798 vs 14796, ref-only 34, ours-only 32 | none |

All 18 fixtures went through `from-v2` without error (31 font subsets, all
`CffSubset`, none embedded whole), every output passed the xref self-check,
page count and MediaBox match the reference on every page, and PDFKit
extracts the same text as from the reference (checked on 01 and 18; the fi/ffi
ligatures come back as letters through the cluster ToUnicode).

## Reading the categories

**ContentOperators on every fixture** is expected and is two things at once:

1. *Representation.* pdfTeX writes one `TJ` array per line with kerning
   integers and rounds every operand (`11.9552 Tf`, `72 708.045 Td`); the
   exact route writes the pipeline's authoritative absolute origin for every
   glyph (`1 0 0 1 72 708.0448322296142578125 Tm`, `/F1 11.9551677703857421875
   Tf`, 5 operators vs 149 on 01). Both are exact for their producer; the
   decimals are the pipeline's ticks written verbatim, which is what the
   route promises. No glyph joined a string on real output (the pipeline's
   `advance_x` is TFM-based and never equals `hmtx × size` exactly), so the
   streams are larger than pdfTeX's; that is a size cost, not a fidelity one.
2. *Layout.* Where the pipeline lays out differently from LaTeX, the
   operator streams differ in substance; the raster column says where.

**FontProgram/FontMetadata on every fixture**: pdflatex embeds Latin Modern
as Type 1 (`lmr12` subset, `FontFile` Length1/2/3, `/Differences`
encoding); the exact route embeds the OpenType CFF face as a CID-keyed subset
(`FontFile3 /CIDFontType0C`, `Identity-H`, CID = original GID). Same design,
different program format and dictionaries. Program bytes therefore never
match, by construction.

**FontResources only on the math fixtures (06, 07, 09, 13, 14) and on 10/11**:
pdflatex sets math in Computer Modern Type 1 (`CMMI`, `CMSY`, `CMR`) while the
pipeline uses `latinmodern-math.otf`, genuinely different font sets. On 10 and
11 the reference carries a *second* font dictionary for the same
`LMRoman12-Regular` face (pdfTeX uses one dictionary per 8-bit encoding, and
codes 128–255 such as the list bullet on 11 and the Latin-Extended letters on
10 live in another one), whereas the exact route has exactly one resource per
face; the classifier reports the unpaired second dictionary.

**Raster, the honest measure of the layout gap** (`ref-only` / `ours-only` ink
pixels at 144 dpi, page 1 unless noted):

| fixtures | ink-only pixels | reading |
|---|---|---|
| 01, 04, 05, 17 | 0 / 0 | binary-ink identical; the `% px differ` is anti-aliasing from sub-thousandth-point position/size differences |
| 02, 03, 08 (3 pp), 12, 15 (3 pp), 16 (2 pp), 18 | 11–240 per page out of 9k–255k ink pixels (≤ 0.1%) | stray edge pixels, no visible layout difference; consistent with the pipeline's own word-box evidence (dx ≤ 0.09 pt) |
| 06 | 162 / 170 | inline math: Latin Modern Math glyphs vs Computer Modern glyphs, same positions to the pixel elsewhere |
| 07, 09, 13, 14 | 1.4k–6.2k | display math: different math fonts plus pipeline layout differences (13: `\left`/`\right` unsupported, 14: `\nu` unsupported and an overfull line, both reported by the pipeline) |
| 10 | 822 / 591 | one `missing_glyph` (U+01C5) reported by the pipeline; text otherwise aligned |
| 11 | 9457 / 9443, best vertical registration 8 px | lists are not implemented upstream (`\item` set as plain paragraphs), so the whole body shifts |

For comparison, the same fixture 01 through the runtime-v1 route
(`flashtex-compiler` → `flashtex-pdf --embed-font auto`) differs from the
reference in 6,257 pixels (`exact-export-classification.md`); through
`flashtex-render --v2` → `from-v2` it differs in 0 ink pixels and 0.01% of
anti-aliased pixels. The pipeline's own `docs/oracle-evidence.md` measured
13.5% differing pixels on the dense pages through the v1 export route and
attributed it to ink weight; through the exact route the dense pages show
240 ink-only pixels out of 254,503 (0.09%) with ink counts within 6 pixels of
the reference, so that attribution no longer holds for this route.

## What this does and does not establish

- Established: a real FlashTeX producer can drive the exact route end to end
  with original glyph ids, exact tick positions, embedded GID-preserving
  subsets, typed rules and searchable text, and the output is deterministic
  (test) and structurally valid (self-check on all 18).
- Established: for text-only fixtures the remaining visible gap to pdflatex
  is at the anti-aliasing level; the remaining measurable gaps are the
  pipeline's documented limitations (lists, `\left`/`\right`, some symbols,
  hyphenation) and its choice of math font.
- Not established: parity of operator streams or font programs with pdfTeX
  (different representation and program format by design), or anything
  outside these 18 fixtures, this font, this page size and CoreGraphics.

## Needs from other owners (not edited here)

- **render-pipeline**: the envelope's `fonts[].sha256` is font-engine's
  `content_sha256` = SHA-256(bytes ‖ face_index as big-endian u32), not
  SHA-256(bytes) as `docs/contracts/rendering-v2-proposal.md` describes and
  `crates/rendering-core`'s validator checks (`digest(bytes) == font.sha256`,
  "font digest mismatch"). `from-v2` accepts both and reports which matched;
  emitting SHA-256(bytes) would make the envelope validate as is.
- **render-pipeline**: `core14-afm` (Times, metric-only) carries no program;
  the exact route refuses runs in it. Either ship a redistributable Times
  program as a real resource or keep Times on the v1 route only.
- **rendering-core / schema**: `format: "opentype-cff"` (the pipeline's
  documented deviation from the schema's `static-truetype` const); `from-v2`
  accepts it and verifies the file really has CFF outlines.
- **rendering-core**: cluster ActualText is reduced to per-glyph ToUnicode
  here; a marked-content `/ActualText` contract would need `BDC`/`EMC` in the
  bounded operator set (not added; no conflicts were observed on the corpus).
- **this crate (mac-pdf)**: `latinmodern-math.otf` subsets are ~110 KB for
  7–39 glyphs because the bounded CFF subsetter keeps the local/global
  subroutine INDEXes whole (733,736-byte source); pruning unused subroutines
  is the size follow-up. Streams carry one `Tm` per glyph; a `TJ` form with
  exact adjustments would need the pipeline to emit hmtx-consistent advances.

The rendering-core outline route (`crates/rendering-core/src/pdf_export.rs`,
paths only, no embedded fonts or searchable text) and this glyph-run route
are complementary: the outline route proves operator/geometry exactness for
paths; this route carries fonts and text.

## Reproduction

```sh
# producer (scratch archive, never in the product path of this crate)
git archive origin/agent/mac-render-pipeline/unified crates/render-pipeline | tar -x -C /tmp/rp
cargo build --release --manifest-path /tmp/rp/crates/render-pipeline/Cargo.toml --bin flashtex-render
printf '%s\n' '{"protocol_version":1,"id":"x","type":"compile","payload":{"project_id":"p","revision":1,"entry_path":"main.tex","documents":[{"path":"main.tex","text":"\\begin{document}Hello world.\\end{document}\n"}]}}' \
  | /tmp/rp/crates/render-pipeline/target/release/flashtex-render --secnumdepth 0 --v2 list.json > /dev/null
# adapter + comparison
cd crates/pdf && cargo build --release
target/release/flashtex-pdf-exact from-v2 list.json --out ours.pdf
target/release/flashtex-pdf-exact classify reference.pdf ours.pdf
```

`cargo test` (78 tests: 38 unit, 12 `tests/exact.rs`, 3 `tests/v2.rs`, 25
`tests/render.rs`) and `cargo clippy --all-targets` are clean at the commit
that adds this file. `tests/v2.rs` carries the unmodified `flashtex-render`
envelope for fixture 01 (`tests/fixtures/v2-plain-paragraph.json`) and
resolves Latin Modern by content hash from the installed TeX Live tree.
