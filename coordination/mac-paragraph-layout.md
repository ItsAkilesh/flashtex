# mac-paragraph-layout — FT-019 rev 1

Agent / task / branch: `mac-paragraph-layout` (Claude Code subagent, parent
`mac-claude-a`, mac-m1max-a) / FT-019 rev 1 paragraph line breaking + metrics
crate / `agent/mac-paragraph-layout/linebreak`

State: ready for integration (checkpoint 2: golden tests, README,
`docs/comparison.md`, `GlyphRun::from_shaped` for FT-018 output). Follow-ups
available on request: pattern hyphenator, `\parshape`/hanging indent,
per-character heights.

Owned paths: `crates/paragraph-layout/**`, `coordination/mac-paragraph-layout.md`,
`coordination/agents/mac-paragraph-layout.json`

Main integrated through: `9da7e48` (branch base). Reviewed origin/main
`ae3f4de` (new crates rendering-core, font-resources, edit-ledger,
project-index, conversion-jobs, document-runtime; compiler now carries
de1020c's `metrics.rs` and a wider `layout.rs` that is still greedy with
`LINE_SPACING 1.2` / `PARAGRAPH_GAP_PT 6`, so the integration note in the
README still applies; no overlap with `crates/paragraph-layout`): branch
applies cleanly, no merge needed. No compiler files touched.

Ready behavior:
- `flashtex-paragraph-layout` (edition 2024, zero external crates):
  `FontMetricsSource` trait (advance/kern/ligature/glyph id/ascender/descender/
  space glue), `FontId` opaque 32-byte content-addressed identity (+`to_hex`);
  `core14::Core14Times` adapter (Roman/Bold/Italic widths from de1020c, 272
  ASCII kern pairs per face from the Adobe AFMs, fi/fl ligatures, psnfss
  `ptm*8t` space glue 250/150/60/extra 60) with provenance.
- `ParagraphBuilder`: text -> boxes/glue/penalties/kerns with TeX `\spacefactor`
  spacing, cross-run kerns, `\-` discretionaries via `Hyphenator`
  (`NoHyphenation`, `ExplicitDiscretionary`); `GlyphRun::from_shaped` consumes
  already-shaped glyphs (original gid, advance units, cluster range).
- `layout_paragraph`: Knuth–Plass total-fit (TeX passes, integer badness,
  fitness classes, demerits, `<=` tie rule, artificial demerits => overfull
  reported never dropped) and first-fit; justified and `\raggedright`;
  positioned runs keep font id, glyph ids, source clusters.
- `layout_pages`: topskip, TeX interline glue across blocks, parskip, heading
  skips, keep-with-next, baseline grid, 2-line widow/orphan control,
  deterministic page breaking with overflow reports.
- Tests (24): 7 unit; 15 golden in `tests/golden.rs` with hand-derived numbers
  (justified stretch/shrink ratios, badness, demerits and x positions; ragged
  spaces; first-fit vs total-fit divergence incl. the TeX tie rule; `\-`
  hyphen run with marker cluster; overfull reporting; kerning-decided break
  with Times A/V; explicit kern discard; orphan, widow, keep-with-next, long
  paragraph, page overflow; baseline grid; determinism); 2 oracle in
  `tests/oracle_wrap_sample.rs`.
- Oracle evidence (`docs/comparison.md`): pdflatex 1.40.29 variant C for
  wrap-sample reproduced — 102/102 line starts, mean|dx| 0.001 bp, max
  0.002 bp, mean|dy| 0.016 bp (max 0.132 bp = PDFKit bold glyph-box descent).
- README: API, TeX/LaTeX defaults, coordinate frames and pt/bp conversion,
  not-modelled list, integration proposal for the compiler lead.

Incomplete behavior / not modelled: `\hbox` nesting, `\parshape`, floats,
footnotes, math (FT-020), automatic pattern hyphenation, ff/ffi/ffl composites,
`\addvspace` merging, `\looseness`, `\flushbottom`, per-character
heights/depths (font ascender/descender used).

Interface changes and required consumer actions: none on main. New crate only.
FT-018 should implement `FontMetricsSource` for `font_engine::Face` or feed
`Shaped` clusters to `GlyphRun::from_shaped` (mapping in README); the
compiler's AST adapter is described in README "Proposed integration".

Validation: `cargo test` in `crates/paragraph-layout` — 24 passed;
`cargo clippy --all-targets` 0 warnings; `cargo fmt` applied.

Needs from others: none blocking. Commander review/integration.

Next action: respond to review; follow-ups on request.

Peer revisions reviewed and adaptations:
- origin/main `9da7e48` -> `ae3f4de`: compiler `layout.rs` now uses real
  Times widths but remains greedy with 1.2 line spacing and 6 pt gaps (my
  crate is the proposed replacement, per README); new rendering-core `font_adapter.rs` uses string font ids and
  hex sha256 -> added `FontId::to_hex()`. de1020c `metrics.rs` widths copied
  with provenance.
- issue #10 oracle findings (no kerning -> early wraps; +14.75 pt after
  headings; 6 pt paragraph gap; 14.45 vs 14.4 pitch): kerning, `\section`
  skips (18.9/12.42 pt), parskip 0 and the TeX-pt/bp unit difference are
  modelled and verified against the oracle word boxes.
- origin/agent/mac-font-engine/tex-fonts `6a20750` (FT-018; merged main
  36e501e, `lib.rs`/`shape.rs` API unchanged since `bd60519`): read `lib.rs`
  (`Face` trait, `FontId.content_sha256: [u8; 32]`) and `shape.rs`
  (`Shaped`/`Cluster`/`Glyph` with `source_range`): our `FontId` is the same 32
  bytes; added `ShapedGlyph` + `GlyphRun::from_shaped`; adapter documented, not
  stacked on the unmerged branch.

Resource: Claude Max 20x plan on mac-m1max-a (allocation
`claude-mac20x-paragraph`, shared account quota; remaining quota unknown to
this worker).

Updated: 2026-09-12T06:12Z
