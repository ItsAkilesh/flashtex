# Adoption contract: `flashtex-unicode-tex`

Proposal only. No file outside `crates/unicode-tex` is changed by KC-104; each
step below is the owning crate's decision. All APIs named here exist in this
crate and are exercised by `cargo test` (unit tests + `tests/oracle.rs`).

## 1. compiler (`crates/compiler`)

**When:** at document load, before choosing fonts.

```rust
let d = flashtex_unicode_tex::detect(&source);
match d.mode {
    CompileMode::Legacy8Bit => { /* current pdfLaTeX path, unchanged */ }
    CompileMode::UnicodeFonts { engine } => { /* Unicode-font mode below */ }
}
for diag in &d.diagnostics { /* map to compiler diagnostics: code, line, byte_offset, severity */ }
```

Unicode-font mode means:

1. UTF-8 input characters go to glyphs directly; `inputenc` is ignored (both
   engines do); no LICR conversion; TeX input ligatures only where
   `FeaturePlan::tex_ligatures` is true (fontspec gives it to `\rmfamily`/
   `\sffamily`, not `\ttfamily`/`\newfontfamily`/`\fontspec`).
2. Engine conditionals answer `d.conditional_value("\\ifxetex")` etc.
3. Font commands: `fontspec::parse_font_commands(&preamble)`; per NFSS family
   and shape, `fontspec::face_request(name, options, slot)` →
   `FontLocator::locate` → `font_engine::load_from_path_index`. When no
   `\setmainfont` is given, fontspec's default is
   `locate::latin_modern_default_file(size, bold, italic)` with `Ligatures=TeX`.
4. `FeaturePlan::resolve(role, &defaultfontfeatures, options, slot)` is stored
   with the font instance; `Scale` multiplies the size (`Scale::factor` with the
   current and new x-heights for `MatchLowercase`).
5. Space factor: `glue::space_factor(word, frenchspacing, sf)`; glue:
   `glue::interword_glue(face, size, profile, plan)` and `InterwordGlue::natural(sf)`.
6. `unimath::MathOptions::from_source(&preamble)` for math letters (§3).
7. `\directlua`/`luacode`: do not execute; keep the diagnostic.

Compiler-owned open points: macro expansion that produces font commands
(detection is lexical); `\addfontfeatures` scoping; polyglossia language → `Language=`.

## 2. render-pipeline (`crates/render-pipeline/src/shape.rs`)

`Shaper` currently routes TFM faces through the TFM and everything else through
`font_engine::shape`. Proposal: a third route for faces loaded in Unicode-font
mode:

```rust
let fi = measure::FontInstance { face: &tt_face, size_pt, plan, profile: EngineProfile::new(engine) };
let s = fi.shape(word);                 // ShapedText { glyphs: [ShapedGlyph { gid, source, advance }], units_per_em, missing, notes }
let width = s.width_pt(size_pt, fi.letter_space_pt());
```

* `ShapedGlyph.gid` is an original font-engine `GlyphId`; `source` is the byte
  range in the word (ligatures merge ranges) — maps to `SCluster` 1:1 by source.
* Advances already include GPOS/legacy kerning per engine; mark glyphs have 0.
* Interword: `fi.glue().natural(sf)`; for LuaLaTeX (`Renderer::LuaNode`) add
  `fi.letter_space_pt()` and `shaper::pair_adjustment(face, last, space, ..)` +
  `pair_adjustment(face, space, first, ..)` (exactly what `measure::layout_line` does).
* Missing characters are dropped (both engines log "Missing character").
* Cache key: `(FontId, FeaturePlan, EngineProfile)`; results are size-independent
  except `letter_space_pt`.

## 3. math-layout (`crates/math-layout`)

math-layout already has `MathFontMetrics`, and font-engine's
`adapters::math::OpenTypeMathFace` fills it from the MATH table. This crate adds
what unicode-math decides *before* layout:

* character selection: `unimath::map_in(command, ch, &opts)` /
  `map_expression` return the Unicode math alphanumeric to look up in the math
  font (`\mathbb{R}`→ℝ, `h`→ℎ, `\symbf{\alpha}`→𝜶 under TeX bold-style);
  `AlphabetCommand::TextFont{..}` means "use the text font" (default for
  `\mathrm/\mathit/\mathbf/\mathsf/\mathtt`).
* `mathfont::MathFontParams::umath_pt(size)` gives the 20 LuaTeX `\Umath`
  parameters verified against LuaLaTeX (names in `UMATH_PARAMETERS`) and
  `script_sizes(size)`; math-layout's parameter struct can be filled from these
  for Unicode-math documents.

Not provided (math-layout's scope): MathVariants/stretchy construction, italic
correction and kerning in scripts, `\Umath` spacing tables.

## 4. font-engine (`crates/font-engine`)

No change required. Suggested upstreaming, if the owner wants it: move
`otl::Layout::{select, apply_gsub, apply_gpos}` (script/language/feature
selection, GSUB 1/4, GPOS 1/2) into font-engine's `ShapeOptions` as
`script`, `language`, `features: BTreeSet<Tag>`, `legacy_kern: bool`, after
which `shaper.rs` becomes a thin policy layer. `locate::parse_name_table`
(ids 2/4/16/17) could likewise replace font-engine's id-1/6-only name parsing.

## 5. font-resources (`crates/font-resources`)

Implement `locate::FontLocator` over a pinned manifest (hash-checked files,
license provenance) so Unicode-font documents compile reproducibly;
`DirectoryLocator` remains for development and the oracle tests.

## Verification an adopter can reuse

`tools/gen_fixtures.py` + `tools/run_oracle.py` regenerate the oracle for new
fixtures; `tests/oracle.rs` shows the full chain from preamble text to word
positions and asserts the current floors (228/230 lines within 0.5pt,
429/430 fontdimens, 272/272 math alphabet lines, 160/160 `\Umath`).
