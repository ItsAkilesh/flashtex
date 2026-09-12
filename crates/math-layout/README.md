# flashtex-math-layout

Original Rust implementation of TeX's math typesetting rules (TeXbook
Appendix G, `tex.web` §§720–767) that turns a math list into explicit boxes
carrying glyph identity and rule geometry. No TeX engine runs at runtime and
no external crates are used (edition 2024).

Owner: FT-020 (`mac-math-layout`), revision 2. Status: model, spacing,
fractions, scripts, radicals (with degree), fixed and extensible delimiters
and radicals, operators (symbol and text, limits per style), accents
(including over scripted bases, `\widehat`/`\widetilde`), `\overline`/
`\underline`, explicit style overrides, oracle comparisons. See
"Unsupported" for the honest scope.

## API

```rust
use flashtex_math_layout::{Atom, MathList, Style, CmMathMetrics, layout, positioned_runs};

// \frac{a}{b} + x^2
let list = MathList::new(vec![
    Atom::frac(MathList::symbols("a"), MathList::symbols("b")),
    Atom::symbol('+'),
    Atom::symbol('x').with_sup(MathList::symbols("2")),
]);
let metrics = CmMathMetrics::latex_10pt();
let root = layout(&list, Style::TEXT, &metrics);          // MathBox tree
let runs = positioned_runs(&root, (72.0, 700.0));         // glyphs + rules in pt
```

- `MathList` / `Atom` (`mathlist.rs`): atoms with the eight TeX classes
  (`Ord Op Bin Rel Open Close Punct Inner`), a nucleus (`Symbol`, `List`,
  `Fraction`, `Radical { radicand, degree }`, `Accent`, `Delimited`, `Text`
  for `\lim`/`\sin`-style upright operators, `Overline`, `Underline`,
  `Styled { style, body }` for `{\displaystyle …}` overrides, `Empty`),
  optional sub/superscript lists and a `Limits` mode
  (`DisplayLimits`/`Limits`/`NoLimits`). `Atom::symbol(ch)` classifies
  Unicode symbols after plain.tex's `\mathcode`s; explicit constructors
  (`Atom::bin`, `Atom::rel`, …) override.
- `Style` (`style.rs`): D, T, S, SS plus the cramped flag, with the
  TeXbook transitions `sup()`, `sub()`, `num()`, `denom()`, `cramped()`.
- `spacing::between(left, right, style)`: the 8×8 TeXbook table in thin/
  medium/thick mu (3/4/5 mu), suppressing the parenthesised entries in
  script styles. `mu = quad/18` of the current style's family-2 font, with
  TeX's integer truncation (`MathParams::mu`).
- `layout(list, style, metrics) -> MathBox` and
  `layout_with_report(...) -> Layout { root, limitations }`.
- `MathBox` (`boxes.rs`): `kind` ∈ `Glyph { font_id, gid, ch, size }`,
  `Rule`, `HBox(children)`, `VBox(children)`, `Kern`, `Glue { mu }`, with
  `width`/`height`/`depth` in pt. Children carry explicit `(dx, dy)`
  offsets from the parent's reference point (left end of the baseline),
  `dy` positive downwards (TeX's `shift_amount` sign).
- `positioned_runs(&box, origin_top_left) -> PositionedRuns { glyphs, rules }`:
  `PositionedGlyph { font_id, gid, ch, x, baseline_y, size }` and
  `PositionedRule { x, y, w, h }` in pt with a y-down axis; `origin` is the
  box's top-left corner, so the baseline is at `origin.1 + root.height`.
  For a PDF page (y up): `y_pdf = page_height - y`; place a glyph's text
  matrix at `(x, y_pdf(baseline_y))` and fill a rule as the rectangle
  `(x, y_pdf(y + h), w, h)` — the same convention as
  `docs/contracts/rendering-v2-proposal.md`. Units are the metrics
  provider's points: TeX points (1/72.27 in) for `CmMathMetrics`, so a
  consumer emitting PDF points multiplies by 7200/7227 once, as that
  proposal specifies; `TimesApproxMetrics` is already in PDF points.

## Metrics providers

`MathFontMetrics` (`metrics.rs`) is what the engine needs from a font set:

| method | purpose |
| --- | --- |
| `params(size)` | `MathParams`: x_height, quad, num1–3, denom1–2, sup1–3, sub1–2, sup_drop, sub_drop, delim1–2, axis_height (family-2 σ parameters), default_rule_thickness, big_op_spacing1–5 (family-3 ξ parameters), script_space, null_delimiter_space, delimiter_factor, delimiter_shortfall |
| `glyph(ch, size)` | the `Glyph` (font id, gid, width/height/depth, italic correction, skew) for a symbol |
| `large_operator(ch, size)` | display-size variant of a big operator (Rule 13) |
| `delimiter_sizes(ch, size)` / `radical_sizes(size)` | size lists, smallest first (Rule 19 / 11) |
| `accent_sizes(ch, size)` | accent variants, narrowest first (Rule 12) |
| `delimiter_extensible(ch, size)` / `radical_extensible(size)` | `Extensible { top, mid, bot, rep }` recipe used when every size is too small (default `None`) |
| `text_glyph(ch, size)` | upright operator text (`\operator@font`), default = `glyph` |
| `font_name(id)` | human-readable identity for reports and renderers |

`FontId` is opaque and provider-defined; once the FT-018 font engine
(`crates/font-engine` on `agent/mac-font-engine/tex-fonts`, which parses the
OpenType `MATH` table) is the
provider, it maps to a content-addressed font handle and `gid` to the font's
own glyph id; `MathParams::from_opentype` maps its `MathConstants` to the
TeX parameters with the LuaTeX correspondence. Until then two adapters ship:

- **`CmMathMetrics`** (`cm.rs`, `cm_tfm.rs`) — Computer Modern. The σ
  parameters are `\fontdimen` 5–22 of `cmsy10/7/5.tfm`, the ξ parameters
  `\fontdimen` 8–13 of `cmex10.tfm`, extracted by `tools/gen_cm_tfm.py`
  from the TFM files of the `cm` package (TeX Live 2026) and embedded as raw
  fixwords. `tfm::scale` reproduces TeX's fixword scaling (`tex.web`
  §571–572), so dimensions agree with pdfTeX's `\showbox` output to the
  scaled point (e.g. the rule thickness is 0.39998pt, not 0.4). Fixed
  registers use plain.tex/LaTeX values: `\scriptspace=0.5pt`,
  `\nulldelimiterspace=1.2pt` (78643sp), `\delimiterfactor=901`,
  `\delimitershortfall=5pt`. Family 3 stays at 10pt in every style
  (`ExtensionSizing::Fixed`), matching plain.tex (`\scriptfont3=\tenex`)
  and LaTeX (`omxcmex.fd`: `<->sfixed*cmex10`). Unicode → family/code
  follows plain.tex's `\mathcode`/`\mathchardef`/`\delcode`. Glyph ids are
  the TFM character codes (identical to the Type 1 font encoding slots).
  `gid` and `font_name` therefore identify the exact glyph to draw.
- **`TimesApproxMetrics`** (`times.rs`) — a Times-based fallback so nothing
  blocks on fonts: Adobe core-14 Times-Roman/Times-Italic/Symbol advance
  widths for the covered characters, category-estimated heights/depths,
  and the Computer Modern σ/ξ ratios (Times has no math parameters). No size
  chains: the engine uses the base glyph and reports `DelimiterTooSmall` /
  `RadicalTooSmall`. Glyph ids are Unicode scalars.

## What the engine implements (Appendix G)

- Rules 5/6: Bin→Ord conversion at list edges and next to Op/Rel/Open/Punct.
- Rules 9/10: `\overline` = overbar(x, 3θ, θ) on the cramped body;
  `\underline` = body, kern 3θ, rule θ, extra θ below.
- Rule 11: radicals — `ψ = θ + θ/4` (text) or `θ + x_height/4` (display),
  sign chosen from the size list, excess depth split around the radicand,
  overbar = kern θ' + rule θ' + kern ψ, θ' = height of the sign glyph.
  `\sqrt[n]` follows LaTeX's `\r@@t`: `\mkern5mu`, the degree in
  scriptscript style raised by 0.6(h − d), `\mkern-10mu`.
- Rule 12: accents — accent widened along its chain while ≤ base width,
  centred with the base glyph's skew kern, lowered by min(h, x_height). For
  an accented single character with scripts, the scripts move under the
  accent and the box grows (tex.web §742).
- Rule 13/13a: operators — display-size variant, axis centring, italic
  correction handling, limits stacked with ξ9–ξ13 gaps and ±δ/2 shifts.
- Rule 15: fractions — num1/denom1 (display) or num2/denom2 (text), rule
  centred on the axis, clearance 3θ (display) or θ, `\atop` when θ = 0,
  null delimiter space on both sides.
- Rule 17/18: scripts — sup1/sup2/sup3 by style, sub1/sub2, sup_drop/
  sub_drop for box nuclei, 4θ separation, x_height*4/5 lift, `\scriptspace`,
  δ offset of the superscript.
- Rule 19: `\left … \right` — δ = max(h − a, d + a), size =
  max(2δ·factor, 2δ − shortfall), centred on the axis.
- `var_delimiter` (tex.web §713): when no size fits, the extensible recipe
  is stacked — bot, n×rep, mid, n×rep, top — as explicit `Glyph` boxes in a
  `VBox`; the same path builds tall radical signs.
- Rule 20: inter-atom glue from the spacing table.

## Validation

`cargo test` (34 tests): unit tests for styles, spacing, parameters, the
OpenType MATH mapping, and `tests/golden.rs` — nested scripts, stacked
fractions in text and display, radical overbar geometry and sign selection,
`\sqrt[3]`, `\left(\frac{a}{b}\right)` sizing, the extensible brace and
radical stacks piece by piece, `\sum` with limits (display) and scripts
(text), `\lim`/`\sin`, `\int` italic offset, `\hat{x}`, `\hat{x}^2`,
`\widehat{xyz}`, `\overline`/`\underline`, style overrides, spacing
classes, Bin→Ord, determinism, origin translation, the Times fallback with
reported shortfalls, and an all-fixtures check that every bar is a `Rule`
primitive and no U+2500 box-drawing glyph appears (CM and Times, 4 styles). Every expected number is asserted at pdfTeX's `\showbox`
precision and its derivation is written next to it.

`cargo run --example dump -- [--display] [index]` prints `\showbox`-style
trees and flattened runs for the fixtures in `src/fixtures.rs`.

`docs/comparison.md` compares seven expressions against pdfTeX (the oracle,
never in the product path), including the extensible brace and radical
stacks: every glyph origin and rule matches within 0.003 bp, the reference's
own output rounding. `tools/oracle_compare.py` and
`tools/oracle_glyphs.swift` regenerate it; `examples/emit_runs.rs` feeds it.

## Unsupported / limitations

- No `\mathchoice` (use `Nucleus::Styled` for explicit overrides), no
  matrices/arrays/`\overset`, no stretchy accents beyond the `cmex`
  `\widehat`/`\widetilde` chains, no horizontal extensible constructs.
- Extensible delimiters and radicals come from TFM recipes only. The
  OpenType `MathVariants` table (vertical glyph construction/assembly for
  Latin Modern Math) is **not** consumed: `crates/font-engine`
  (`agent/mac-font-engine/tex-fonts`) parses `MathConstants`, italics
  correction and top-accent attachment but states `MathVariants` is not
  parsed. Request to the FT-018 owner: expose `MathVariants` vertical
  assemblies (parts with start/end connector lengths, advance, extender
  flag) and glyph variant lists; this crate's `delimiter_extensible` /
  `radical_extensible` / `delimiter_sizes` are the hooks, and connector
  overlap (absent in TFM) would be added to `stack_extensible` then.
  Providers without recipes (Times) still fall back to the largest glyph and
  report `Limitation::DelimiterTooSmall` / `RadicalTooSmall`.
- No inter-character kerning or ligatures between adjacent Ord characters
  (TeX's `math_text_char` rule); italic corrections are applied.
- Accents on a scripted *list* base (`\hat{xy}^2`) accent the base and
  place the scripts after it; TeX's script-swap applies only to a single
  character, which is implemented.
- Glue has no stretch/shrink; line breaking is out of scope.
- `CmMathMetrics::scaled(base)` scales the 10/7/5pt design proportionally;
  real LaTeX picks `cmr8`/`cmr6` at 11–12pt and keeps `cmex10` at 10pt.
- The Times adapter's heights/depths are estimates; only its advance widths
  are transcribed metrics.

## Integration note (compiler lead)

The compiler's `math.rs` currently emits fraction bars as runs of U+2500 and
scales scripts by fixed ratios (0.7 / 0.5) with hand-picked shifts. This
crate replaces that arithmetic with TeX's, and its output carries what the
preview and PDF writer need without re-deriving anything:

1. Convert the compiler's `MathList` into this crate's `MathList` (the
   nucleus variants line up: `Symbol`, `Fraction`, `Radical`; scripts are
   the same optional lists). Class comes from `Atom::symbol`.
2. Call `layout_with_report` with `Style::DISPLAY` for `\[…\]` and
   `Style::TEXT` for `$…$`; surface `limitations` as diagnostics.
3. Flatten with `positioned_runs` at the formula's top-left (runtime-v1
   already uses points with a top-left origin). Each `PositionedGlyph` maps
   onto a runtime-v1 `text` item (`x_pt`, `baseline_y_pt`, `font_size_pt`)
   with the provider's font identity and gid carried by the compiler's
   internal item so preview and PDF draw the same glyph. Rules are
   explicit `PositionedRule { x, y, w, h }`; runtime-v1 has no rule item
   and this crate proposes none (Commander owns v1). Until the contract
   grows one, the compiler can keep its current U+2500 text-run encoding
   for the transport while taking the bar's position, width and thickness
   from the rule geometry, so preview and PDF agree.
4. Choose the provider: `CmMathMetrics` when the renderer draws Computer
   Modern (FT-018), `TimesApproxMetrics` otherwise. Implement
   `MathFontMetrics` on the font engine once it exposes metrics.
