# flashtex-pdf

Original Rust PDF writer for FlashTeX (task FT-009). Takes runtime-v1
`compile_result` pages (`docs/contracts/runtime-v1.md`) and writes a PDF 1.4
document by hand: catalog, page tree, page objects with `MediaBox`, content
streams, cross-reference table, trailer. No TeX engine is involved anywhere,
and the crate has zero external dependencies (the JSON reader is in-tree, like
`crates/compiler`), so it builds offline and deterministically.

```sh
cd crates/pdf
cargo test
cargo run --bin flashtex-pdf -- --out out.pdf < ../../protocol/fixtures/compile-result.json
cargo run --bin flashtex-pdf -- result.json --out out.pdf --verify
# Opt-in: embed a subset of a Unicode TrueType font for characters outside
# WinAnsi and Symbol (see "Font embedding" below before redistributing output).
cargo run --bin flashtex-pdf -- result.json --out out.pdf --embed-font /path/to/font.ttf
FLASHTEX_UNICODE_FONT=/path/to/font.ttf cargo run --bin flashtex-pdf -- result.json --out out.pdf
open out.pdf
```

## What is implemented

- Library: `flashtex_pdf::render_pdf(&CompileResult) -> Result<PdfOutput, PdfError>`
  and `render_envelope(&str)` for a raw runtime-v1 envelope. `PdfOutput` carries
  `bytes` and `warnings`; a non-empty warning list means the document was
  approximated somewhere and the caller should surface it, not hide it.
- CLI `flashtex-pdf [INPUT.json] --out OUTPUT.pdf [--verify]`. Reads the envelope
  from a file or stdin, prints warnings to stderr, exits 1 for an unsupported
  `protocol_version`/`type` or unrenderable pages, 2 for usage errors.
  `--verify` re-reads the produced bytes and checks the xref table before writing.
- One page object per runtime-v1 page with `MediaBox [0 0 width_pt height_pt]`.
  Any page size is accepted; nothing assumes US Letter.
- Every `kind: text` item becomes one `BT x y Td … ET` block with
  `y = height_pt - baseline_y_pt`, i.e. the runtime-v1 top-left origin converted
  to PDF's bottom-left origin. Coordinates are written to a thousandth of a point.
  Inside the block the item is split into font runs, each `/Fn size Tf (bytes) Tj`:
  `/F1` Times-Roman (WinAnsiEncoding) for everything WinAnsi covers, `/F2` the
  base-14 `Symbol` font (its built-in encoding) for Greek letters and the
  mathematical operators listed in `src/encoding.rs`. Because the runs share one
  text object, the viewer advances between them with the real base-14 widths;
  this crate ships no width tables.
- **Fraction rules.** The FT-002 compiler has no rule primitive in runtime-v1, so
  it emits a fraction bar as a text item consisting only of U+2500 (`─`) repeated
  N times, each assumed 0.5 em wide, with the item's baseline at the bar's bottom
  edge and thickness 0.06 em of the parent size (the item itself is set at 0.7 of
  the parent). Such items are rendered as filled rectangles (`x y w h re f`) of
  width `N × 0.5 × font_size_pt` and thickness `0.06/0.7 × font_size_pt`, never
  as glyphs and with no warning. Convention: *compiler emits fraction rules as
  U+2500 runs; rendered as rules.* A real rule item type in runtime-v1 would
  replace this and is Commander's call.
- `(`, `)`, and `\` are escaped in literal strings.
- **Font embedding (opt-in).** `RenderOptions { embed_font }`, `--embed-font
  PATH|auto`, or `FLASHTEX_UNICODE_FONT` embed a subset of a Unicode TrueType
  font as `/F3` for characters neither base-14 font has. See below.
- Structural self-check (`flashtex_pdf::verify`) that parses the header, `startxref`,
  every xref entry, and confirms each offset lands on `N 0 obj`, plus a
  content-stream reader that recovers `Tf`/`Td`/`Tj` runs and `re f` rules for tests.

## Font embedding

Off by default. When enabled, every character that WinAnsi and Symbol cannot
show is looked up in the supplied `.ttf`; the glyphs used (plus `.notdef` and
the parts of any composite glyph) are copied into a new, densely renumbered
TrueType program and embedded as `/FontFile2` under a Type0 font with a
CIDFontType2 descendant, `Identity-H` encoding (two bytes per glyph, written as
hex strings), `/CIDToGIDMap /Identity`, a `/W` widths array, and a `ToUnicode`
CMap so text extraction and search return the original characters. The subset
carries `head`, `hhea`, `maxp`, `hmtx`, `loca`, `glyf`, and, if present,
`cvt `, `fpgm`, `prep`; table checksums and `head.checkSumAdjustment` are
computed and `truetype::verify_checksums` reads them back in tests. The parser,
subsetter, and CMap writer are hand-written (`src/truetype.rs`,
`src/embed.rs`); the crate still has no dependencies.

Characters the embedded font also lacks still become `?` and are named in
`warnings` (one font-level warning listing them, plus the per-item warning).
Nothing is ever dropped silently. The CLI prints every warning and a final
`note: N warning(s)` line; exit status stays 0 because the PDF was written.

**Which font.** `--embed-font PATH` uses that file. `--embed-font auto` (or
setting `FLASHTEX_UNICODE_FONT` alone) uses `FLASHTEX_UNICODE_FONT` if set,
otherwise, on macOS only, the first of these that exists:

1. `/System/Library/Fonts/Supplemental/Times New Roman.ttf` — serif, matches
   the base-14 Times visually; covers Latin, Greek, Cyrillic, but **not** CJK,
   `ℝ`, or emoji.
2. `/System/Library/Fonts/Supplemental/Arial Unicode.ttf` — sans, ~50 000
   glyphs including CJK and most symbols; no emoji outlines.

Both are Apple-supplied system fonts. Their licences permit use on the Mac;
**embedding a subset into a PDF you redistribute is a licensing question the
user must answer for themselves.** This crate does not choose a font unless
asked, and it prints which one it embedded. No font is committed to this
repository. Only TrueType outlines (`glyf`) are supported: CFF/OpenType
(`OTTO`) and `.ttc` collections are rejected with a message, not guessed at.

**Limits of the embedded route.** Glyph advances come from the font, so text
spacing inside a run is right, but the *positions* of items still come from
the compiler's own metrics, which do not know about this font. There is no
shaping: combining marks, ligature substitution, and complex scripts are
written glyph-by-glyph from the `cmap`. Emoji fonts with colour tables
(`sbix`, `COLR`) are not supported. The subset is uncompressed (no Flate).

## Limitations, stated plainly

- **Fonts: base-14 `Times-Roman` (WinAnsiEncoding) and `Symbol` only, neither
  embedded.** This is a placeholder until FlashTeX embeds its own fonts. Glyph
  shapes and advance widths come from whatever the viewer substitutes for the
  standard 14.
- **Heading weight is not reproduced.** The compiler sets headings in Times-Bold
  (`layout.rs::font_for_size`, sizes above 12pt) and measures them with bold
  widths, but runtime-v1 carries only `font_size_pt`, no font identity. This
  writer deliberately does **not** infer bold from size (issue #9: that would
  also embolden math scripts and other non-body sizes); every Times run is
  Times-Roman until the contract gains a font/weight field. Proposed to
  Commander as a runtime-v1 addition; until then headings export in regular
  weight at the right size and position, with widths slightly narrower than the
  compiler assumed.
- **Math is rendered via base-14 Symbol without embedding.** Greek (α…ω, Α…Ω,
  ς ϑ ϕ ϖ ϒ) and the operators in `src/encoding.rs::symbol_byte` (√ ∑ ∏ ∫ ∞ ± ×
  ÷ ≤ ≥ ≠ ≈ ≡ ∂ ∇ ∈ ∉ ⊂ ⊃ ⊆ ⊇ ∪ ∩ → ← ↑ ↓ ↔ ⇒ ⇐ ⇔ ∀ ∃ ¬ ∧ ∨ ′ ″ ° · … ∅ ℵ ℜ ℑ
  ℘ ⊗ ⊕ ∠ ∝ ∼ ∗ ∣ ⟨ ⟩ − ≅ ∴ ⊥ ∋ ◊) are written as single Symbol bytes. Symbol's
  glyph design does not match Times, has no bold/italic, and cannot stretch
  delimiters or radicals; the radical sign is a plain glyph with no overbar.
  Codes were transcribed from the Adobe Symbol encoding (PDF 32000-1 Annex D.5)
  and only codes the author is certain of are included; none were omitted
  from the requested list.
- **Unicode:** characters representable in WinAnsi (ASCII, Latin-1 such as
  `é ï ñ ü`, and the Windows-1252 block: `— – … € “ ” ‘ ’ Œ œ Š š Ž ž Ÿ ƒ ‰ • ™`)
  are written as their single WinAnsi byte via Times; characters Symbol covers
  are written via Symbol; with embedding enabled, anything the supplied font
  has is written through the embedded subset. Whatever remains (by default:
  Cyrillic, CJK, blackboard bold, emoji, control characters, combining marks)
  is written as `?` and reported in `warnings` with its code point. It is
  never silently dropped.
- **Text and U+2500 rules only.** Item kinds other than `text` are skipped with
  a warning. No images, general paths, links, or annotations.
- **Metrics come from the compiler, not from here.** The writer places each item
  exactly where `x_pt`/`baseline_y_pt` say. The FT-002 compiler currently
  estimates glyph widths as `0.5 × font_size`, so word gaps in the PDF will look
  uneven against real Times widths until the compiler uses real metrics. That is
  a compiler limitation; this crate does not re-measure or re-flow text.
- No compression, no object streams, no outline/bookmarks, no metadata beyond
  `/Producer` and `/Creator`. Output is plain PDF 1.4 and larger than it needs
  to be for big documents.
- Zero pages is an error (`PdfError::Invalid`), as are non-positive page sizes,
  non-positive font sizes, and non-finite coordinates.

## Export is always white

The writer takes no theme or colour input. The only colour operator emitted is
`0 g` (black text in DeviceGray) and no background rectangle is painted, so the
page renders white in every viewer regardless of what the Mac preview does in
dark mode. `tests/render.rs::export_is_white_and_theme_independent` guards this.

## Verification performed

- `cargo test`: 32 tests (18 unit, 14 integration) covering the fixture's page
  count and MediaBox, a two-page synthetic result with distinct page sizes,
  multiline placement (every `Td` equals `(x_pt, height_pt - baseline_y_pt)`),
  WinAnsi encoding (`é` is byte `0xE9`, `—` is `0x97`), unrepresentable
  characters (`中`, `😀`, `ℝ`) becoming `?` with warnings, delimiter escaping,
  unsupported item kinds, bad envelopes, determinism, the CLI, and the xref
  self-check. On macOS an additional test writes the fixture PDF and asserts
  `/usr/bin/sips -g pixelWidth -g pixelHeight` reports `612` × `792`.
- Issue #9 reproduction: `tests/fixtures/math-compile-result.json` is the exact
  `flashtex-compiler` (de1020c) output for `$\frac{a}{b}+\alpha+\sqrt{x}$`
  (`tests/fixtures/math-compile-request.json`). The test asserts zero warnings,
  one `re f` rule of width 8.4pt and thickness 0.72pt at x=72 with its bottom
  edge on baseline 84.72, `α` emitted via `/F2` as byte `0x61`, `√` via `/F2` as
  `0xD6`, and every other item via `/F1` at the compiler's coordinates. A mixed
  item (`x∈ℝ→∞`) is checked to switch `/F1`/`/F2` inside one text object and
  both fonts are present in every page's `/Resources`.
- Embedding: with a font found by the same discovery the CLI uses (skipped
  with a message otherwise), `ж中ℝ😀` is rendered; the test asserts a Type0 /
  CIDFontType2 / `FontFile2` / `ToUnicode` object chain, that each character
  is either written through `/F3` with a ToUnicode entry mapping its glyph id
  back to the code point or named in a warning, that the embedded program
  re-parses with `verify_checksums` passing and glyph count equal to the
  subset's map (used glyphs + `.notdef` + composite parts), that advances
  survive subsetting, and that `sips` opens the CLI's output. Off by default
  and bad font files are errors, not silent fallbacks.
- Manual: the fixture and a two-page Unicode sample were rendered, opened by
  `sips` (`format: pdf`, `pixelWidth: 612.000`, `pixelHeight: 792.000`), and
  rasterised to PNG; the heading, three baselines, accented characters, escaped
  parentheses, the `?` substitution, a bottom-margin line, and page two all
  appeared where expected on a white page. The issue #9 math PDF rasterised by
  macOS shows `a` over a drawn bar over `b`, then `+α+√x`, with no `?`.
- Embedding, rasterised by macOS: with Arial Unicode, `Latin café — Greek αβγ
  — Cyrillic жизнь — CJK 中文 — ℝ ∫ 😀` shows every script and `ℝ`, with `?`
  only for the emoji; `Ǆǅ Ŵŷ ő` (composite glyphs) render correctly. With
  Times New Roman the same line shows `?` for CJK and `ℝ` exactly as warned.

## Integration with the Mac app

The Rust worker (`crates/compiler`) still reports `pdf_path: null`, as the
contract permits. To attach export:

1. In the worker, after producing a `compile_result`, call
   `flashtex_pdf::render_envelope` (or `render_pdf` on the already-built pages),
   write the bytes to a per-project temporary file, and set `pdf_path` to that
   path. Forward `warnings` as `warning` diagnostics with `source: null` so the
   UI shows them; do not drop them.
2. Alternatively, until the worker is wired, `apps/mac` can shell out to the
   `flashtex-pdf` binary next to `flashtex-compiler`, feeding it the
   `compile_result` line it already holds and reading `--out`.
3. The Mac "Export PDF" action then copies `pdf_path` to the user's chosen
   location. The export never depends on the preview theme.

Neither step changes the runtime-v1 contract; `pdf_path` already exists for
this purpose. Adding a `pdf_warnings` field would be a contract change and
belongs to Commander.
