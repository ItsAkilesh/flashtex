# flashtex-render-pipeline

Original Rust rendering pipeline for FlashTeX and the `flashtex-render` worker
binary. It turns the compiler's parse tree into positioned pages through the
sibling crates (font-engine shaping, paragraph-layout Knuth–Plass line
breaking, math-layout Appendix G boxes, document-style geometry) plus this
crate's own TeX page builder, and emits the rendering-v2 display list with a
runtime-v1 `compile_result` fallback. No TeX engine runs anywhere in the
product path; pdflatex is used only as a test oracle (see
`docs/oracle-evidence.md`).

```sh
cd crates/render-pipeline
cargo build --release
cargo test --release            # 29 tests; the Latin Modern ones skip (loudly) without the fonts
FLASHTEX_COMPILER=$PWD/target/release/flashtex-render   # drop-in worker for the Mac app
```

## Pipeline

```
compiler::parser::parse_project      exact byte spans per document (Span.document)
  -> adapter.rs                      styles/gaps/ligatures/accents re-derived from source bytes,
                                     heading numbers, \ref/\pageref, \newpage, secnumdepth
  -> shape.rs (tfm.rs / font-engine) TFM widths, kerns, ligatures, heights (ec-lm*.tfm, 2^20
                                     units per em) with glyph ids from the OTF cmap; font-engine
                                     GSUB/GPOS for text outside T1; clusters keep source bytes
  -> typeset.rs (paragraph-layout)   total-fit Knuth–Plass, TeX space factor (§1034), \parindent,
                                     \quad, \/ italic correction, glue sized by the font at the space
  -> mathtex.rs / math-layout        TeX's math metrics (lmmi/lmsy/lmex = CM TFMs, rm-lmr*.tfm),
                                     Appendix G, explicit rules; Latin Modern Math glyphs painted
                                     (mathfont.rs: OpenType MATH fallback when no TFMs are installed)
  -> pagebuild.rs                    TeX §980–1028 page builder (penalty costs, \topskip, \maxdepth,
                                     per-block \baselineskip, \nointerlineskip, \raggedbottom)
  -> display.rs                      display list v2 (ticks, original GIDs, clusters, carets, rules)
  -> v1.rs                           runtime-v1 fallback with negotiated layout capabilities
```

Distinct identifier types (`ids.rs`): `EncodingCode` (a T1/OT1 slot),
`char`, and `GlyphId` are separate types; TFM codes are never cast to glyph
ids. Every glyph in the display list carries the font's original glyph id.

## Fonts

The default face is Latin Modern (`lmroman<size>-{regular,bold,italic}.otf`
by optical size following `t1lmr.fd`, `latinmodern-math.otf` for math).
Layout uses the TeX font metrics pdfLaTeX uses (`ec-lm*.tfm` for text,
`rm-lmr*.tfm` for the math roman family, the CM-identical `lmmi`/`lmsy`/
`lmex` values embedded in math-layout), found next to the OTFs
(`fonts/opentype/...` → `fonts/tfm/...`) or in `FLASHTEX_TFM_DIRS`; the
OTFs supply the outlines and glyph ids. Without the TFMs the OpenType
metrics are used and a `math_metrics_opentype` / `tfm_missing` diagnostic
says so (`tfm_missing` is a warning per face).
Times is used only when the document selects it (`\usepackage{times}`) and is
metric-only (`core14-afm`, no bytes). Fonts are read at run time from, in
order: `FLASHTEX_FONT_DIRS` (colon separated), `--font-dir`, `FLASHTEX_LM_DIR`,
a `Fonts` directory next to the executable or in the app bundle's
`Resources`, then MacTeX/BasicTeX 2025/2026 and Debian TeX Live paths
(`fonts::DEFAULT_FONT_DIRS`). A missing Latin Modern face is an **error
diagnostic** with Times metrics substituted, never a silent fallback; a
missing math face reports `math_font_unavailable` and typesets no math.

## `flashtex-render` (runtime-v1 worker)

Reads `compile` envelopes on stdin (JSON Lines), answers one `compile_result`
per line. Unknown `protocol_version`/`type`, oversized lines, invalid UTF-8
and unsafe paths get the compiler's error envelopes. Options: `--v2 out.json`
(display list of the last request), `--pdf out.pdf` (through the pdf
sibling's negotiated route), `--font-dir DIR`, `--class-options OPTS`
(body-only input; default `12pt` like the compiler's implicit preamble),
`--secnumdepth N` (default 2; the visual-oracle preamble is 0), `--timing`.

### Layout-capability negotiation (`docs/contracts/runtime-v1-layout-capabilities.md`)

`payload.layout_capabilities` is validated (≤16 unique non-empty strings of
≤64 bytes, else the request fails, never the worker); the accepted subset
(`rules-v1`, `font-hints-v1`, request order, never unrequested) is echoed in
the reply, also on failures. `rules-v1` turns every v2 rule (fraction bars,
radical rules) into `{"kind":"rule",x_pt,y_pt,width_pt,
height_pt,source}` with top-left semantics; without it the legacy U+2500
approximation is emitted (run of box-drawing characters at the size whose
0.0857 em equals the rule height). `font-hints-v1` adds
`font:{family,weight,style}` (`Latin Modern Roman`/`Latin Modern Math`/
`Times`) to text items. `display-list-v2` (mac-preview-v2's proposal,
`docs/contracts/runtime-v1-display-list-v2.md` on its branch, ACKed here)
makes the worker write the rendering-v2 `display_list` envelope as one
sibling line right after the `compile_result` (same `id`, project,
revision, document digests) for `ok`/`recovered` results; a line over the
16 MiB reply limit declines the capability for that request with a
`display-list-v2 declined:` warning. The v1 payload is derived from the
immutable v2 display list per request, so the same source with the same
accepted set is byte-identical whether or not a previous request warmed the
worker. Replies larger than 16 MiB (the Mac reader's line limit) fail the
request explicitly rather than being cut off (`FLASHTEX_MAX_REPLY_BYTES`
lowers the limit for tests).

## Runtime-v1 items

One text item per glyph run (a styled word segment) with the exact UTF-8
source range including the document path (multi-file `\input` projects keep
each item's own file); one item per glyph for math, since each has its own
position. Coordinates are PDF points, y downward, rounded to 1/1000 pt.

## What is implemented, honestly

Paragraphs (justified, `\parindent`, `\\`, `~`, TeX ligatures `--`/`---`/
quotes, `\'e`-style accents composed to precomposed characters), `\textbf`,
`\emph`, `\textit`, `\section`/`\subsection` with LaTeX numbering
(`secnumdepth`), `\label`/`\ref`/`\pageref` (bounded 3-pass convergence),
inline and display math (`$`, `\[`, `$$`, `equation` with `(n)` flush right,
`\frac`, `\sqrt`, scripts, operators with display limits), `\newpage`/
`\clearpage`/`\pagebreak`, page breaking with TeX's cost model, US Letter
`article` at 10/11/12pt with `geometry` margins. Diagnostics carry source
ranges and codes (`font_unavailable`, `missing_glyph`, `overfull_hbox`,
`overfull_vbox`, `unsupported_script`, `math_limitation`, `labels_unstable`).

Not implemented (reported, not approximated silently): hyphenation, lists
(`\item` markers are set as plain paragraphs, no hanging indent), figures
(`\includegraphics` is dropped by the compiler; captions are plain
paragraphs), `\left`/`\right`, Greek letters and most control-word math
symbols (the compiler's math parser rejects them), tables, footnotes,
two-column, page numbers/headers (`\pagestyle{empty}` behaviour only),
non-Latin scripts (`unsupported_script`), RTL.

## Sibling pins and requested API changes

The crate builds against vendored copies of the sibling crates (see
`vendor/VENDORING.md`; each directory has a `PIN` file). Requested changes,
also listed in `docs/proposals/rendering-abi.md`:

- **compiler**: expose style scopes (`\textbf`/`\emph`), interword gaps,
  class options, `\parindent`, `secnumdepth` and page-break commands in the
  parse tree instead of leaving them to source re-scanning; number only
  `equation` displays (today every closed `\[`/`$$` increments the counter);
  keep `\newpage` as a block boundary instead of an unsupported-command error.
- **rendering-core / schema**: accept `format: "opentype-cff"` (face 0) as a
  font profile — Latin Modern ships as OpenType CFF; every other rule of the
  schema validates on all 18 fixtures (see `docs/oracle-evidence.md`).
- **font-resources**: `inspect_opentype_cff` (OTTO wrapper over the existing
  `cff` module) so rendering-core's validator/outlines can consume Latin
  Modern; this crate's `cff.rs` (Type 2 bounds) then goes away.
- **paragraph-layout**: penalty-based page builder (this crate's
  `pagebuild.rs` would move there), per-block `\baselineskip`.
- **pdf**: a glyph-run entry point (font id + original GIDs + tick
  positions) so `--pdf` stops re-encoding text by character.
- **font-engine**: none new; the preview JSON export overlaps with `--v2`.

## Latency

Warm worker, Apple M1 Max, release build, per request including JSON in and
out (see `docs/oracle-evidence.md` for the per-fixture table): 0.2 ms
(one line) to 9 ms (three dense pages, 1800 items). The first request adds
5–8 ms of font loading. This is the compile side of the Commander's
typing-to-visible gate (< 200 ms including layout); the Mac paint side is
measured separately by the Mac shell.
