# mac-paragraph-layout — FT-019 rev 1

Agent / task / branch: `mac-paragraph-layout` (Claude Code subagent, parent
`mac-claude-a`, mac-m1max-a) / FT-019 rev 1 paragraph line breaking + metrics
crate / `agent/mac-paragraph-layout/linebreak`

State: in progress (checkpoint 1 pushed: skeleton, metrics trait + Core-14
Times adapter, first-fit and Knuth–Plass total-fit breakers, vertical layout and
page breaking, oracle comparison test)

Owned paths: `crates/paragraph-layout/**`, `coordination/mac-paragraph-layout.md`,
`coordination/agents/mac-paragraph-layout.json`

Main integrated through: `9da7e48` (branch base; no compiler files touched)

Ready behavior:
- `flashtex-paragraph-layout` (edition 2024, zero external crates):
  `FontMetricsSource` trait (advance/kern/ligature/glyph id/ascender/descender/
  space glue), `FontId` opaque content-addressed identity; `core14::Core14Times`
  adapter (Roman/Bold/Italic widths from de1020c, 272 ASCII kern pairs per face
  from the Adobe AFMs, fi/fl ligatures, psnfss `ptm*8t` space glue).
- `ParagraphBuilder`: text -> boxes/glue/penalties/kerns with TeX `\spacefactor`
  spacing, cross-run kerns, `\-` discretionaries via `Hyphenator`
  (`NoHyphenation`, `ExplicitDiscretionary`).
- `layout_paragraph`: Knuth–Plass total-fit (TeX passes, badness, fitness,
  demerits, tie rule, artificial demerits => overfull reporting) and first-fit;
  justified and `\raggedright`; positioned runs with original glyph ids and
  source clusters.
- `layout_pages`: topskip, interline glue, parskip, heading skips,
  keep-with-next, baseline grid, widow/orphan (2 lines), overflow reports.
- Oracle evidence: `tests/oracle_wrap_sample.rs` reproduces pdflatex variant C
  for wrap-sample: 102/102 line starts, mean|dx| 0.001 bp, max 0.002 bp,
  mean|dy| 0.016 bp (max 0.132 bp = bold glyph-box descent difference).

Incomplete behavior:
- Golden tests for justified stretch/shrink, hyphenation with `\-`, kerning
  affecting a break, baseline grid, widows/orphans, page overflow, determinism
  (next checkpoint); README and `docs/comparison.md` (next checkpoint).
- Not modelled: `\hbox` nesting, floats, footnotes, math (FT-020), automatic
  pattern hyphenation, ff/ffi/ffl composites, `\addvspace` skip merging,
  `\looseness`, per-character heights/depths (font ascender/descender used).

Interface changes and required consumer actions: none on main. New crate only;
FT-018 should implement `FontMetricsSource` (see `crates/paragraph-layout/src/metrics.rs`).

Validation: `cargo test` in `crates/paragraph-layout` — 9 tests pass
(7 unit + 2 oracle) at this checkpoint.

Needs from others: FT-018 branch `agent/mac-font-engine/tex-fonts` not yet
pushed at 2026-09-12T05:57Z; adapter unblocks this task.

Next action: golden tests + README + comparison doc; second push.

Peer revisions reviewed and adaptations:
- origin/main `9da7e48`: compiler `layout.rs` is still the greedy placeholder
  (612x792 bp page, 6pt paragraph gap, 1.2 line spacing); nothing to adapt, the
  crate is independent. de1020c `metrics.rs` widths copied with provenance.
- issue #10 oracle findings (no kerning -> early wraps; +14.75 pt after headings;
  6 pt paragraph gap): kerning, `\section` skips (18.9/12.42 pt) and parskip 0
  are modelled and verified against the oracle word boxes.

Resource: Claude Max 20x plan on mac-m1max-a (allocation
`claude-mac20x-paragraph`, shared account quota; remaining quota unknown to
this worker).

Updated: 2026-09-12T05:57Z
