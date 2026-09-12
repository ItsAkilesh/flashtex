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
- Every `kind: text` item becomes `BT /F1 size Tf x y Td (text) Tj ET` with
  `y = height_pt - baseline_y_pt`, i.e. the runtime-v1 top-left origin converted
  to PDF's bottom-left origin. Coordinates are written to a thousandth of a point.
- `(`, `)`, and `\` are escaped in literal strings.
- Structural self-check (`flashtex_pdf::verify`) that parses the header, `startxref`,
  every xref entry, and confirms each offset lands on `N 0 obj`, plus a
  content-stream reader that recovers `Tf`/`Td`/`Tj` for tests.

## Limitations, stated plainly

- **Font: base-14 `Times-Roman` with `WinAnsiEncoding` only.** This is a
  placeholder until FlashTeX embeds its own fonts. No font program is embedded,
  so glyph shapes and advance widths come from whatever the viewer substitutes.
  There is one font for everything: headings differ only in size; bold, italic,
  and math fonts are not selected.
- **Unicode:** characters representable in WinAnsi (ASCII, Latin-1 such as
  `é ï ñ ü`, and the Windows-1252 block: `— – … € “ ” ‘ ’ Œ œ Š š Ž ž Ÿ ƒ ‰ • ™`)
  are written as their single WinAnsi byte. Anything else (mathematical symbols,
  Greek, CJK, emoji, control characters) is written as `?` and reported in
  `warnings` with its code point. It is never silently dropped.
- **Text only.** Item kinds other than `text` are skipped with a warning. No
  images, rules, paths, links, or annotations.
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

- `cargo test`: 20 tests (11 unit, 9 integration) covering the fixture's page
  count and MediaBox, a two-page synthetic result with distinct page sizes,
  multiline placement (every `Td` equals `(x_pt, height_pt - baseline_y_pt)`),
  WinAnsi encoding (`é` is byte `0xE9`, `—` is `0x97`), unrepresentable
  characters (`∫`, `😀`) becoming `?` with warnings, delimiter escaping,
  unsupported item kinds, bad envelopes, determinism, the CLI, and the xref
  self-check. On macOS an additional test writes the fixture PDF and asserts
  `/usr/bin/sips -g pixelWidth -g pixelHeight` reports `612` × `792`.
- Manual: the fixture and a two-page Unicode sample were rendered, opened by
  `sips` (`format: pdf`, `pixelWidth: 612.000`, `pixelHeight: 792.000`), and
  rasterised to PNG; the heading, three baselines, accented characters, escaped
  parentheses, the `?` substitution, a bottom-margin line, and page two all
  appeared where expected on a white page.

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
