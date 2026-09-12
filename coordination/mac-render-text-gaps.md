# mac-render-text-gaps — handoff

Lane: Claude Code subagent of mac-claude-a (mac-m1max-a). Owns `crates/render-pipeline`
(never `vendor/`, never crates/font-engine, paragraph-layout, math-layout, compiler).
Branch `agent/mac-render-pipeline/text-gaps` from `origin/agent/mac-render-pipeline/unified`
9aaec57a. Bounded ~75–90 min.

## Task (Commander replenishment, issue #2 comment 5646989044)
1. **Current:** resolve the `\text` spacing / brace-encoding / ligature gaps documented by the
   renderer at main f261b36c (`crates/rendering-core/docs/handoffs/hw1-text-producer/actual-candidate/`).
2. Follow-up 1: `vendor/paragraph-layout/src/linebreak.rs:988` panic guard at the pipeline call site.
3. Follow-up 2: `\hfill` in section titles (GH38) — typesetting side behind the existing diagnostic.

## Coverage audit (mandatory first step)
Grepped `crates/render-pipeline/{src,tests,examples}`, `apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`:

- `\text{...}` in math: **no test anywhere** in render-pipeline (`grep 'text{'` → 0 hits in
  src/tests/examples), none in the Mac test suite, none in the live report or docs/evidence.
- Existing related coverage (not the gap):
  - `crates/render-pipeline/src/tfm.rs::ec_lmr12_widths_ligatures_and_kerns_are_tfms` — T1
    ligature (`office` → ffi slot 0x1E), kern, `\fontdimen2` for *paragraph* text.
  - `crates/render-pipeline/src/shape.rs::latin_modern_shaping_applies_ligatures_and_kerning` —
    OpenType GSUB/GPOS path.
  - `crates/render-pipeline/src/adapter.rs::space_factor_follows_sentence_punctuation` — §1034
    space factor for paragraph glue.
  - `crates/render-pipeline/tests/v2_and_math.rs`, `metrics_provenance.rs` — math roman family
    (`rm-lmr*`) glyph/GID provenance; no `\text`.
- Renderer evidence (main f261b36c, extracted to scratch): six probes `comment`, `escaped`,
  `five`, `ligature`, `scripts`, `space` with request.jsonl + display.json; README documents
  three gaps: `a b` advances by rm-lmr12 slot 32 (285213/2^20 em) instead of the space
  `\fontdimen2`; `\{x\}` placed with OT1 slots 123/125; `ffi` emitted as three glyphs 55,55,66
  (no ligature program run). The evidence binary is a MODIFIED producer: adapter.patch
  (incremental.rs/typeset.rs `Nucleus::Text` arms) + the isolated compiler candidate
  (f464c6d6 + 61bd63f7, `crates/preview-controller/docs/handoffs/hw1-text-candidate/candidate.patch`)
  applied to vendor/compiler. Neither is on main: `origin/main` `crates/compiler/src/math.rs`
  has no `Nucleus::Text`, and `vendor/compiler` (PIN 745f327) has none either.

Conclusion: the gap is entirely uncovered on this branch; the compiler side is a cross-owner
prerequisite (adapter request below), the typesetting side is mine.

## Design decisions
- `\text` content is an `\hbox` in the *text* font (T1 `ec-lmr<size>`, the face the paragraph
  path already uses), not the math roman family (`rm-lmr`, OT1): that is what fixes braces
  (T1 123/125 = braces) and ligatures (T1 lig program `f f i` → 0x1E) with the same
  `Shaper`/TFM code the paragraphs use. Sizes follow the math style (12/8/6 at 12 pt).
- Interword space = `\fontdimen2` (+`\fontdimen7` at space factor ≥ 2000, §1041–1044) at
  natural width (an `\hbox` is never stretched); TeX's space factor from `adapter::space_factor`.
- math-layout has no box nucleus. The pipeline registers each shaped run and lays it out
  through the existing `Nucleus::Text` + `MathFontMetrics::text_glyph` interface with a
  placeholder glyph of the run's exact width/height/depth, then substitutes the shaped hbox
  into the returned `MathBox` tree (same metrics ⇒ identical spacing/script placement).
  API request to math-layout (owner mac-math-layout): `Nucleus::HBox(MathBox)` for pre-typeset
  boxes (TeX §1076 `\hbox` in math → Ord); the placeholder path then goes away.
- No vendor edits. The compiler `Nucleus::Text` arm is feature-gated
  (`compiler-text-nucleus`) so the crate builds against the current pin; the probes are built
  in a scratch tree with the candidate compiler applied for evidence only.

## Checkpoint
- Branch `agent/mac-render-pipeline/text-gaps`, base 9aaec57a; consumed main: d5440b0 (through
  the base), evidence read at f261b36c.
- Dirty files: this handoff, `coordination/agents/mac-render-text-gaps.json`.
- Next commands: implement `crates/render-pipeline/src/mathtext.rs`; `cargo test --release`
  in `crates/render-pipeline`; scratch probe replay.
- Load at start: 8.37 (1-min).
