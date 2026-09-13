# flashtex-unicode-tex

XeLaTeX/LuaLaTeX compatibility layer for FlashTeX (task KC-104, owner
`kabir-claude`, mac-m5pro-kabir). Pure Rust, edition 2024, no external crates;
depends only on `crates/font-engine` (font program parsing, `default-features = false`).
Nothing outside `crates/unicode-tex` is modified; adoption is proposed in
[`CONTRACT.md`](CONTRACT.md).

```sh
cd crates/unicode-tex
cargo test --release                              # 25 unit tests + oracle comparison
cargo test --release --test oracle -- --nocapture # per-line PASS/FAIL table
KC104_DEBUG=arial_sans cargo test --release --test oracle -- --nocapture  # word-level dump
```

## Modules

| module | what it does |
| --- | --- |
| `detect` | Lexical engine detection: magic comments, fontspec/unicode-math/polyglossia, XeTeX-only (`xeCJK`, `mathspec`, `\XeTeX…`) and LuaTeX-only (`luacode`, `luatexja`, …) packages, `iftex` conditionals/requirements, pdfTeX-only packages. Chooses `CompileMode::{Legacy8Bit, UnicodeFonts{engine}}`, answers `\ifxetex`/`\ifluatex`/`\iftutex`, and emits located diagnostics for Lua code (`\directlua`, `\luaexec`, `\latelua`, `luacode` environments; downgraded to warnings inside engine conditionals). |
| `fontspec` | `\setmainfont/\setsansfont/\setmonofont/\newfontfamily/\newfontface/\fontspec/\defaultfontfeatures/\addfontfeatures/\setmathfont` with options before and after the name; `Path, Extension, UprightFont/BoldFont/ItalicFont/BoldItalicFont/SlantedFont/SmallCapsFont, *Features, Scale (factor, MatchLowercase, MatchUppercase), Ligatures, Numbers, Letters, Kerning, Fractions, VerticalPosition, Contextuals, Style, StylisticSet, CharacterVariant, RawFeature, Color, Opacity, LetterSpace, WordSpace, PunctuationSpace, HyphenChar, Renderer, Script, Language, Mapping`. Unknown options are kept, never dropped. `fontspec.cfg` defaults (`Ligatures=TeX` for rm/sf; `WordSpace={1,0,0}`, `PunctuationSpace=WordSpace` for tt), face requests with `*` expansion, NFSS substitution chain, `FeaturePlan`. |
| `locate` | `FontLocator` trait (for font-resources to implement) and `DirectoryLocator` (explicit directories, reads only table directory + `name` + `OS/2` by seek; TTC faces; family/full/PostScript matching; bold/italic by weight/width/slope). TeX Live name→file table, Latin Modern optical-size selection of `TU/lmr`. |
| `texlig` | `Ligatures=TeX`: XeTeX `tex-text.map` (character level) and luaotfload `tlig` (glyph must exist). |
| `otl` | Raw GSUB/GPOS: script/language/feature selection (`latn`→`DFLT`→`dflt`), GSUB 1/4/7, GPOS 1/2/9, lookup flags via GDEF; unimplemented lookup types are reported per feature. |
| `shaper` | Word shaping per engine: TeX ligatures → cmap (+ canonical composition under HarfBuzz via font-engine) → GSUB → GPOS → mark advances zeroed → legacy `kern` table only under HarfBuzz when GPOS has no `kern`. `pair_adjustment` for GPOS pairs with the space glyph (LuaTeX node mode). |
| `glue` | Interword glue per engine + WordSpace/PunctuationSpace/LetterSpace; LaTeX space factor. |
| `measure` | Word positions inside one line at natural glue. |
| `unimath` | Unicode math alphanumerics (with Letterlike holes, Greek block order, digits), `math-style` ISO/TeX/french/upright/literal, `bold-style`, `sans-style`, `nabla`/`partial`, `mathrm/mathit/mathbf/mathsf/mathtt = text|sym`, all `\sym…`/`\math…` alphabet commands, options from `\usepackage[..]{unicode-math}`, `\setmathfont[..]`, `\unimathsetup`. |
| `mathfont` | MATH constants (parsed by font-engine) in pt and the LuaTeX `\Umath…` parameter each initialises. |

## Engine facts established here (TeX Live 2026, all measured)

| topic | XeTeX 0.999998 | LuaHBTeX 1.24 + luaotfload |
| --- | --- | --- |
| renderer | HarfBuzz for OpenType; **AAT (CoreText) for fonts with `morx`** (Helvetica Neue, Menlo) | node mode, `script=latn;language=dflt` from fontspec |
| legacy `kern` table | used when GPOS lacks `kern` (TNR `AV` kerned) | ignored (TNR `AV` = A+V) |
| GPOS pairs with space | never (words shaped separately) | applied across glue (Arial `space Y`, `A space`) |
| space / stretch / shrink / extra | U+0020 advance, /2, /3, /3 | same; stretch=shrink=extra=0 when `post.isFixedPitch` |
| fixed-pitch without `\setmonofont` | stretch kept (Menlo `\newfontfamily`: 3.01pt) | zero |
| `LetterSpace=x` | x% of size between glyphs, and added to `\fontdimen2` (stretch/shrink/extra derived from it) | x% between glyphs, and one kern at each interword glue; fontdimens unchanged |
| `WordSpace=a` / `{a,b,c}` | scales space, stretch, shrink; extra untouched | same |
| `Ligatures=TeX` | TECkit map on characters | `tlig` feature (target glyph required); same pairs incl. `"`→”, `<<`, `,,`, `!``, `?`` |
| `Scale=MatchLowercase` | x-height from CoreText for AAT fonts (Helvetica Neue 5.15068pt) | x-height from font (5.17pt) |
| unicode-math `\mathbf{x}` | text bold `x` (default `mathbf=text`) | same |

## Oracle results

51 one-page fixtures (43 text, 8 unicode-math) × {XeLaTeX, LuaLaTeX}, generated
by `tools/gen_fixtures.py`, compiled by `tools/run_oracle.py` (TeX Live 2026 in
`/Library/TeX/texbin`, poppler `pdftotext -bbox-layout` 26.09.0 — installed via
Homebrew for this task). Fonts: Times New Roman (4 styles), Helvetica Neue
(TTC), Menlo (TTC), Georgia, Arial, Courier New, TeX Gyre Termes/Pagella/Heros/
Cursor, Latin Modern Roman OTF (default, 10pt/12pt optical sizes), Latin Modern
Math, STIX Two Math, New Computer Modern Math.

Current (`cargo test --release --test oracle -- --nocapture`):

| check | result |
| --- | --- |
| text lines, every word's left and right edge within 0.5pt | **228/230** (XeLaTeX 113/115, LuaLaTeX 115/115); median line max-error 0.009pt |
| words within 0.5pt | 1673/1678 |
| fontdimens (size, space, stretch, shrink, extra) within 0.02pt | 429/430 |
| unicode-math alphabet lines (exact characters in the PDF) | 272/272 |
| `\Umath` parameters within 0.011pt (8 LuaLaTeX math fixtures, 3 math fonts, × 20 parameters) | 160/160 |

The two failing lines are `helvneue_matchlowercase` under XeLaTeX: XeTeX takes
the x-height of AAT fonts from CoreText (not reproducible from the font file
alone), so the scaled size is 8.33656pt instead of 8.36777pt. Lines pass
despite XeTeX's AAT `morx` ff/ffi ligatures on Helvetica Neue because the
ligature glyphs there are within 0.4pt of their components; `morx` itself is
not implemented (reported as a note).

The test asserts these floors (228 lines, 429 fontdimens, all math) so
regressions fail; it reports and skips instead when fonts are missing.

## Shaping decision

`crates/font-engine` already parses cmap/hmtx/OS/2/post/MATH, composes marks
and applies GSUB `liga` + GPOS `kern` under a fixed script choice. It cannot
select a script/language, enable arbitrary features (`smcp`, `onum`, `dlig`,
`tnum`, …) or run single substitutions, and it always uses the legacy `kern`
table — all of which change XeTeX/LuaTeX output. Rather than edit a crate owned
elsewhere, this crate reads GSUB/GPOS itself (`otl.rs`, ~500 lines) and reuses
font-engine for everything else. rustybuzz (MIT) was not adopted: every
FlashTeX crate is dependency-free (font-engine README), no workspace dependency
policy admits external crates, and HarfBuzz parity would still not model
luaotfload node mode. Not implemented (reported, never silent): contextual/
chaining lookups (GSUB 5/6/8, GPOS 7/8), multiple/alternate substitution,
mark/cursive positioning (advances unaffected), AAT `morx`/`kerx`, Graphite,
complex scripts, vertical text.

## Other limits

* `detect` is lexical; macros that expand to font commands are not seen.
* `DirectoryLocator` is a development locator; reproducible builds should use a
  pinned `FontLocator` (see CONTRACT.md). XeTeX finds TeX Live fonts by file
  name only; luaotfload also by name — `tex_font_file` covers common families.
* `SizeFeatures`, `FontFace`, `NFSSFamily`, `Opacity`, `HyphenChar` are parsed
  and stored but have no effect here.
* Line breaking, justification, math layout and PDF output belong to other crates.
