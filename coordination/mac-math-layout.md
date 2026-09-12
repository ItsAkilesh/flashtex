# mac-math-layout handoff — FT-020 rev 2

- Agent / task / branch: `mac-math-layout` (Claude Code subagent, parent
  `mac-claude-a`, machine `mac-m1max-a`) / FT-020 rev 2 "operators, accents,
  extensible delimiters, nested styles, font-derived math constants with
  explicit glyph/rule geometry" / `agent/mac-math-layout/math-boxes`
- State: ready for integration (rev 1 and rev 2 acknowledged and implemented).
- Owned paths: `crates/math-layout/**`, `coordination/mac-math-layout.md`,
  `coordination/agents/mac-math-layout.json`.
- Main integrated through: `0a3d8e2` merged into the branch (`84af7cd`); no
  file under `crates/math-layout` exists on main; no conflicts.
- Ready behavior: `crates/math-layout` (`flashtex-math-layout`, edition 2024,
  zero dependencies). `layout(list, style, metrics) -> MathBox` and
  `positioned_runs(box, origin) -> {glyphs, rules}` implement TeXbook Appendix G
  rules 5/6, 9/10, 11 (with `\sqrt[n]` per LaTeX `\r@@t`), 12 (incl. scripted
  single-character bases and `\widehat`/`\widetilde` chains), 13/13a (symbol and
  text operators, limits per style), 15, 17/18, 19, 20, plus tex.web §713
  extensible delimiter/radical stacks from TFM recipes as explicit `Glyph`
  boxes, `\overline`/`\underline` as explicit rules, and `Nucleus::Styled`
  overrides. `CmMathMetrics` embeds cmr/cmmi/cmsy 10/7/5 + cmex10 TFM fixwords
  with TeX's scaling; `MathParams::from_opentype` maps OpenType MATH constants;
  `TimesApproxMetrics` is the no-font fallback.
- Incomplete behavior: no `\mathchoice` (explicit `Styled` overrides instead),
  matrices/arrays, horizontal extensibles; OpenType `MathVariants` not consumed
  (font-engine does not parse it — request recorded in the README); no
  inter-character kerns/ligatures between Ord characters; no `MathFontMetrics`
  implementation on the FT-018 `Face` yet (crate unmerged).
- Interface changes and required consumer actions: none to existing crates.
  Within this crate (unreleased): `Nucleus::Radical` is now a struct variant
  with `degree`; `MathFontMetrics` gained defaulted methods
  `delimiter_extensible`, `radical_extensible`, `text_glyph`. Compiler
  `math.rs` untouched; no runtime-v1 item kinds proposed.
- Validation: `cargo test` — 34 passed (7 unit, 27 golden); `cargo clippy
  --all-targets` clean; `cargo fmt --check` clean. Golden numbers
  cross-checked against pdfTeX 1.40.29 `\showbox` dumps (oracle, scratch dir).
  `crates/math-layout/docs/comparison.md`: seven display formulas (stage 1 A–C,
  stage 2 D–G: extensible brace stack, `\sqrt[3]{\frac{a}{b}}`,
  `\lim_{x\to 0}\frac{\sin x}{x}`, extensible radical over the brace stack)
  against pdfTeX's PDF content stream with a PDFKit (macOS 26.3.1) cross-check:
  every glyph origin and rule within 0.003 bp; extensible pieces match
  piece-for-piece (glyph ids, origins, repeater count). Acceptance (2) is
  enforced by `rules_are_rule_primitives_never_box_drawing_glyphs` over all
  fixtures × 4 styles × 2 providers.
- Needs from others: FT-018 owner — expose OpenType `MathVariants` (vertical
  assemblies + variant lists) and implement `MathFontMetrics` on `Face` via
  `MathParams::from_opentype`; Commander — how rules travel in the runtime
  contract (this crate emits explicit `PositionedRule`s).
- Next action: final report to parent. On request: `MathFontMetrics` for the
  FT-018 `Face`, connector overlap for OpenType assemblies, Ord–Ord kerning.
- Peer revisions reviewed and adaptations:
  - `origin/agent/claude/compiler-foundation` `math.rs` (read at `6b13034`):
    fixed-ratio scripts and U+2500 bars. Adaptation: TeX parameters and explicit
    rules; README integration note. No compiler files edited.
  - `origin/main` through `0a3d8e2` (merged): `font-resources`,
    `rendering-core`, `rendering-v2-proposal.md`, dispatcher changes; nothing
    under `crates/math-layout`. Adaptation: README cites the v2 proposal's
    coordinate and TeX-pt/PDF-pt conventions.
  - `origin/agent/mac-font-engine/tex-fonts` `d0c64cf` (FT-018): `FontId`
    content hash, `GlyphId`, OpenType `MATH` `MathConstants`/italics/top-accent;
    `MathVariants` explicitly not parsed. Adaptation: `MathParams::from_opentype`
    plus a TFM-only extensible path behind trait hooks; MathVariants request in
    README.
  - `origin/agent/mac-validation/native-verification`
    `tools/native-validation/oracle_extract.swift`: model for
    `tools/oracle_glyphs.swift`.
- Resource state: Claude Max 20x plan on `mac-m1max-a` (allocation
  `claude-mac20x-math-layout-stage2` per the rev 2 assignment); quota/usage
  unknown from this session.
- Updated: 2026-09-12T07:20Z
