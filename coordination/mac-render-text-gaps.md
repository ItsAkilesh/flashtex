# mac-render-text-gaps — handoff

Lane: Claude Code subagent of mac-claude-a (mac-m1max-a). Owns `crates/render-pipeline`
typesetting/fonts/text (never `vendor/`, never crates/font-engine, paragraph-layout,
math-layout, compiler; `protocol.rs`/`json.rs`/`incremental.rs`/`display.rs` left to
mac-display-delta-impl per the coordinator). Branch `agent/mac-render-pipeline/text-gaps`
from `origin/agent/mac-render-pipeline/unified` 9aaec57a. Bounded ~75–90 min.

## Task (Commander replenishment, issue #2 comment 5646989044; owner packet 5647057936 =
`crates/rendering-core/docs/handoffs/hw1-text-producer/owner-acceptance.md` on main)
1. **Current:** resolve the `\text` spacing / brace-encoding / ligature gaps documented by the
   renderer at main f261b36c (`…/hw1-text-producer/actual-candidate/`). **Done** (033cd5cb).
2. Follow-up 1: `vendor/paragraph-layout/src/linebreak.rs:988` panic guard. **Done** (this commit).
3. Follow-up 2: `\hfill` in section titles (GH38). Compiler token not landed → documented below.

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
- Renderer evidence (main f261b36c): six probes `comment`, `escaped`, `five`, `ligature`,
  `scripts`, `space`; README documents three gaps: `a b` advances by rm-lmr12 slot 32
  (285213/2^20 em) instead of `\fontdimen2`; `\{x\}` placed with OT1 slots 123/125; `ffi`
  emitted as gids 55,55,66. The evidence binary is a MODIFIED producer: adapter.patch
  (incremental.rs/typeset.rs `Nucleus::Text` arms) + the isolated compiler candidate
  (f464c6d6 + 61bd63f7, `crates/preview-controller/docs/handoffs/hw1-text-candidate/candidate.patch`
  sha256 af8fd219…, `hw1-text-comment-fix/comment-fix.patch` 51dbbd0d…) applied to
  vendor/compiler. Neither is on main: `origin/main` `crates/compiler/src/math.rs` has no
  `Nucleus::Text`, and `vendor/compiler` (PIN 745f327) has none either.
- Trailing-`\\` panic: no test; reproduced (below).

## 1. `\text` — what was done (033cd5cb)
- `crates/render-pipeline/src/mathtext.rs` (new): a `\text` argument is an `\hbox` in the
  **text** face (T1 `ec-lmr<size>`, the face paragraphs use) at the math style's size
  (12/8/6 pt), shaped by the existing `Shaper` (TFM ligature/kern program, cmap gids); interword
  glue `\fontdimen2` (+`\fontdimen7` at space factor ≥ 2000) at natural width with
  `adapter::space_factor`. Enters math-layout as an Ord via `Nucleus::Text(handle)` +
  `text_glyph` placeholder of the run's exact width/height/depth; the shaped hbox is substituted
  into the `MathBox` tree after layout (same metrics ⇒ identical Appendix G spacing/scripts).
- `src/typeset.rs`: `convert_math_with(list, sink)` (feature-gated `N::Text` arm), `MathRec`
  carries `text_runs`, `MathRec::otf_glyph/run_glyph` map run glyphs to the text face's own gids,
  `math_items` splits the run at a `\text` space (no space glyph; words are separate items like
  paragraphs) and keeps a ligature cluster's full text (`ffi`). `adapter::space_factor` and
  `typeset::design_size` are `pub(crate)`.
- `Cargo.toml` feature `compiler-text-nucleus` (off by default): the crate builds against the
  current vendor pin; the arm compiles only against a compiler with `Nucleus::Text`.
- Tests `tests/math_text.rs` (6): ligature gid = cmap U+FB03 and width = T1 slot 0x1E; space =
  `\fontdimen2` (3.9166 pt at 12 pt, = 342239/2^20 em, ≠ slot 32); sentence-end factor 3000 adds
  `\fontdimen7`; braces = T1 slots 123/125 with cmap ids (≠ OT1 513365/2^20 em); labels
  `(a)…and` keep gids and Ord/Bin spacing, scripts attach; comment-joined argument identical.
  `src/mathtext.rs` unit tests (2): handle range, sink order.
- Evidence `crates/render-pipeline/docs/evidence/hw1-text/` (README, before/after per-case
  display.json/stdout/stderr/summary.json, replay.py, adapter.patch, stage-sha256.txt): the six
  probes replayed on scratch builds (candidate compiler applied; same nine resource hashes as
  evidence.json). **Before** (base + adapter.patch): comment/escaped/five/ligature/space
  byte-identical to the evidence; scripts differs only because host MacTeX supplies
  lmroman8-regular.otf (Linux stage lacked it) — `and` run identical. **After**: comment, five,
  scripts unchanged; space `b` at 144879963 (was 144198207: `b−a` = TFM `a` + `\fontdimen2`);
  escaped `x` at 140919024 (was 140788438: T1 brace width = its OTF advance); ligature one glyph
  gid 123 (cmap U+FB03), cluster text `ffi` (was 55,55,66). No diagnostics added/removed.
- Gates: `cargo test --release` 53 passed (incl. `tests/incremental.rs` byte-identical gate,
  118 s under load 33) after the `\text` commit; oracle table (text/06/07) **not re-measured**
  (MacTeX oracle run not affordable under load 20–60 in the bound); by construction unchanged:
  the change touches only `\text` runs (which the pinned compiler cannot produce) and a
  `gid == 0 && ch == ' '` branch of `math_items` that only run glyphs hit. golden_v1 /
  v2_and_math / latex_structure regressions pass.

### Cross-owner requests (no forks made)
1. **compiler** (owner: compiler lead / Astra): adopt `Nucleus::Text` — the isolated candidate
   `candidate.patch` (af8fd219…) + `comment-fix.patch` (51dbbd0d…) on base 1c02a2bc — on main;
   then this lane re-pins `vendor/compiler` and removes the `compiler-text-nucleus` gate. The
   pipeline consumes `Nucleus::Text(String)` with normalized single spaces and literal `{`/`}`
   exactly as the candidate emits them; unsupported nested commands keep the candidate's
   diagnostics (their literal `\name` text is shaped as plain characters — T1 has `\`).
2. **incremental.rs** (mac-display-delta-impl lane / whoever re-pins): the two arms the
   renderer's adapter.patch adds are required with the repin (hash tag 3 + content; shift leaf):
   ```
   @@ hash_math @@
   +            Nucleus::Text(text) => {
   +                3u8.hash(h);
   +                text.hash(h);
   +            }
   @@ shift_math @@
   -            Nucleus::Symbol(_) => {}
   +            Nucleus::Symbol(_) | Nucleus::Text(_) => {}
   ```
   (`docs/evidence/hw1-text/adapter.patch`, incremental.rs half; verified in the scratch build.)
3. **math-layout** (owner mac-math-layout): `Nucleus::HBox(MathBox)` — a pre-typeset box as a
   nucleus (TeX §1076: `\hbox` in math is an Ord). Replaces `mathtext.rs`'s placeholder +
   `substitute` indirection; no behaviour change expected.

## 2. Trailing `\\` panic — guard (this commit)
Reproduction (worker, base 9aaec57a and 033cd5cb): body `Alpha beta\\` + blank line (also
`Alpha beta\\ ` + blank, `Alpha beta\\` + newline before `\end{document}`, `Alpha\\\\`):
`thread 'main' panicked at vendor/paragraph-layout/src/linebreak.rs:988:21`, worker exit 101 —
one keystroke kills the worker. `\\[2pt]` and a lone `\\` paragraph do not panic.
Guard: `Context::refuse_trailing_break` in `src/typeset.rs` (paragraph_block): if the
horizontal list ends `… penalty(≤ −10000) | penalty 10000, \parfillskip, penalty −10000`, the
paragraph is refused with error diagnostic `paragraph_final_linebreak` pointing at the last
word/formula before the `\\` (the adapter's `LineBreak` item carries no span; adding one
touches `incremental.rs`, left to its lane), and nothing is typeset for it. Worker now answers
`recovered`/`failed` with the diagnostic (repro script: scratch `repro/repro.py`).
Tests `tests/paragraph_guard.rs` (2): five variants → typed error, refused paragraph absent,
following paragraph typeset; a `\\` inside a paragraph still typesets.
Alternative the owner may prefer: drop the final break and set the paragraph (TeX sets an
empty last line + "Underfull \hbox"); refusal was implemented as instructed.

### Report for the paragraph-layout owner (mac-paragraph-layout, pin 70209e2)
`layout_paragraph(&items, &params)` panics at `src/linebreak.rs:988` (`&items[start..brk]`,
start > brk) when the list is
`[Box(w), Glue(fil), Penalty(−10000), Penalty(10000), Glue(fil), Penalty(−10000)]`
— a forced break immediately followed by the paragraph-end sequence (TeX §1096 after `\\`):
TeX produces an empty final line (Underfull \hbox badness 10000); the breaker should yield a
line with no boxes rather than index with `start` past the second forced break. Expected: two
lines, the second empty (width 0, `\parfillskip` only). Pipeline call site:
`crates/render-pipeline/src/typeset.rs::paragraph_block` (guarded meanwhile).

## 3. `\hfill` in section titles (GH38) — compiler token not landed
`origin/main` compiler (f1f1f21e) has no `\hfill` token (grep `hfill` in crates/compiler/src →
none; the candidate's `unsupported_heading_commands_remain_explicit` test still expects the
"not supported in this inline argument" diagnostic at the `\problem` invocation). What the
pipeline will do once the compiler exposes an `Inline::HFill { span }` (→ adapter `Item::HFill`):
in `Context::hlist`, push `pl::Item::Glue(pl::Glue::fil())` (TeX `\hfill` = `\hskip 0pt plus
1fill`; paragraph-layout has first-order fil only — `fil` is the correct strength when no other
fil glue is on the line, which is the section-title case; two `\hfill`s share the line equally
either way) with no break allowed at it in a heading (headings are one `\raggedright`-free
line here; a `penalty INF` before the glue), so `\subsection*{Problem 1 \hfill [10 points]}`
sets the points flush right of `\textwidth`; the run before it keeps its space factor. Until
then the compiler's `\hfill … not supported in this inline argument` diagnostic stands and the
title is set without the fill (no silent approximation). No code was written for this.

## 4. GH43 — run index / handle bounds (fixed on this branch)
Slot addressing (`font_id = RUN_FONT_BASE + slot`, `gid` within a 65536-entry chunk, runs occupy
consecutive slots from `TextRun::first_slot`; `glyph_at`/`owns`/`run_of`) replaces the u16 cast;
handles span U+F0000..=U+10FFFF (`MAX_TEXT_ATOMS` 131072) and `TextSink::atom` refuses beyond
without state change (`math_text_overflow` error). Tests: `mathtext::tests::slot_addressing_
does_not_alias_across_chunks`, `handles_round_trip_and_stay_private_use`, `sink_gives_ordinary_
atoms_in_order_and_refuses_beyond_the_handle_space`, `tests/math_text.rs::a_run_beyond_one_chunk_
addresses_every_entry_without_aliasing`. Text-enabled route check `docs/evidence/hw1-text/issue43.
{py,json}` (oversized runs, 131073 atoms → explicit reply-limit failure, then a valid request in
the same session). Six probes byte-identical to `after/`. `cargo test --release`: 57 passed.

## Checkpoint
- Branch `agent/mac-render-pipeline/text-gaps`; commits 82fcaf71 (audit), 033cd5cb (`\text`),
  this commit (guard + handoff). Base 9aaec57a; consumed main d5440b0 (through base); read
  main f261b36c evidence and a5e7fa7c/f1f1f21e owner packet.
- Dirty files: none after this commit. Scratch (not committed): probe builds and repro under
  the session scratchpad.
- Next commands for a successor: `cd crates/render-pipeline && cargo test --release`; on
  compiler adoption: re-pin `vendor/compiler`, apply the incremental.rs arms, build with the
  feature, re-run `docs/evidence/hw1-text/replay.py after` against the stage, then remove the
  feature gate; add a `\text` fixture to the oracle harness.
- Resource state: shared 20x Max quota on mac-m1max-a; not observable from a subagent. Load
  during the lane 8–60 (other lanes building); no MacTeX oracle run.
- Parent-retained Mac files: none touched, no diffs requested.
