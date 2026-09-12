# flashtex-fontmetrics

Advance widths and vertical metrics for the seven PDF base-14 text faces that
macOS ships metric-compatible fonts for: Times-Roman, Times-Bold, Times-Italic,
Times-BoldItalic, Helvetica, Helvetica-Bold, Courier. Zero dependencies; the
tables are committed Rust data, so the crate builds on any platform.

This is an independent proposal for FT-005 (the compiler's placeholder glyph
widths). It does not modify `crates/compiler` or `crates/pdf`; the compiler
owner decides whether to adopt it.

```sh
cd crates/fontmetrics
cargo test
cargo run --quiet --example measure -- Times-Roman 12 "Hello"   # 26.664000 0
```

## Purpose

Three components place the same text and must agree on how wide it is:

- `crates/compiler` decides where each word goes (`x_pt`, `baseline_y_pt`),
  today using a placeholder of `0.5 × font_size` per glyph and `0.28 × font_size`
  per space (`crates/compiler/src/layout.rs`).
- The Mac preview draws the compiler's items with the system Times at those
  positions. Real Times is narrower than 0.5 em on average for lowercase but
  wider for `m`, `w`, capitals, so words overlap or gap unevenly.
- `crates/pdf` writes the same items with the base-14 `Times-Roman` and
  `WinAnsiEncoding`, where every viewer substitutes a Times whose advances are
  the standard Adobe integers.

If the compiler measures words with the advances in this crate, all three see
the same widths: the preview draws Apple's metric-compatible Times, the PDF
viewer draws its metric-compatible Times, and both match the table to within
half a unit per glyph.

## API

```rust
use flashtex_fontmetrics::{advance, measure, space_pt, width_pt, Face, VerticalMetrics};

advance(Face::TimesRoman, 'H');                 // Some(722), in 1/1000 em
width_pt(Face::TimesRoman, "Hello", 12.0);      // 26.664
space_pt(Face::TimesRoman, 12.0);               // 3.0
let m = measure(Face::TimesRoman, "aλb", 10.0); // m.width_pt, m.unknown_chars == 1
let v = VerticalMetrics::at(Face::TimesRoman, 12.0); // ascender 9.0, descender -3.0, cap_height, x_height
Face::TimesRoman.postscript_name();             // "Times-Roman", for /BaseFont
Face::TimesRoman.metrics();                     // &'static FaceMetrics: raw table + ascender/descender/cap/x/space/notdef
```

Characters outside the table are charged the face's `.notdef` width, defined
here as the width of `?` because that is the character `crates/pdf` writes for
them; `measure` reports how many there were so a caller can warn.
`descender` is negative, as in a PDF `FontDescriptor /Descent`.

## How the tables were generated

`tools/dump-metrics.swift` creates each face with `CTFontCreateWithName` at
1000 pt, so one CoreText advance unit equals one 1/1000-em glyph-space unit.
For every scalar in the WinAnsi repertoire it looks up the glyph with
`CTFontGetGlyphsForCharacters` and reads the horizontal advance with
`CTFontGetAdvancesForGlyphs`, one glyph at a time (no kerning, no ligatures,
no shaping), rounds to the nearest integer, and emits `src/tables.rs`:

```sh
swift crates/fontmetrics/tools/dump-metrics.swift > crates/fontmetrics/src/tables.rs
```

Vertical metrics come from `CTFontGetAscent`, `CTFontGetDescent` (negated),
`CTFontGetCapHeight`, and `CTFontGetXHeight` at the same size.

Repertoire: U+0020–U+007E, U+00A0–U+00FF, and the 27 Windows-1252 0x80–0x9F
specials (`€ ‚ ƒ „ … † ‡ ˆ ‰ Š ‹ Œ Ž ‘ ’ “ ” • – — ˜ ™ š › œ ž Ÿ`): 218 code
points per face, all present in every face. U+00A0 measures as the space
advance and U+00AD as the hyphen advance, matching how PDF `WinAnsiEncoding`
maps those bytes.

## Provenance

- Generated on macOS 26.3.1 (build 25D2128), Apple Swift 6.2.4, CoreText.
- Fonts are the macOS system fonts, resolved by CoreText to these PostScript
  names and files (2048 units/em TrueType):

  | Face | PostScript name | File |
  |---|---|---|
  | Times-Roman | `Times-Roman` | `/System/Library/Fonts/Times.ttc` |
  | Times-Bold | `Times-Bold` | `/System/Library/Fonts/Times.ttc` |
  | Times-Italic | `Times-Italic` | `/System/Library/Fonts/Times.ttc` |
  | Times-BoldItalic | `Times-BoldItalic` | `/System/Library/Fonts/Times.ttc` |
  | Helvetica | `Helvetica` | `/System/Library/Fonts/Helvetica.ttc` |
  | Helvetica-Bold | `Helvetica-Bold` | `/System/Library/Fonts/Helvetica.ttc` |
  | Courier | `Courier` | `/System/Library/Fonts/Courier.ttc` |

- These are Apple's metric-compatible equivalents of the base-14 fonts, **not
  Adobe's AFM files**. Because the source fonts are 2048/em, the raw advances
  are fractions such as 722.168 (= 1479/2048 × 1000); rounding recovers the
  Adobe integers for the Latin repertoire (Times-Roman `H` 722, `e` 444, `l`
  278, `o` 500, `—` 1000, `Œ` 889). Two observed departures from the Adobe
  AFMs, kept as measured because they are what the Mac preview draws:
  Times-Roman `€` is 744 here (Adobe: 500), and Apple's Courier gives `°` 400,
  `±` 549, `µ` 576, `÷` 549 instead of 600. Every ASCII printable in Courier
  is 600.

## Validation

`cargo test`: 9 passed, 0 failed. `cargo clippy --all-targets -- -D warnings`
clean. `cargo build --release` succeeds. Tests cover: "Hello" in Times-Roman at
12 pt is 26.664 pt (CoreText measured 26.660; assertion is 26.66 ± 0.5 and
exact 26.664); accented Latin-1 letters share the advance of their base letter
in Times-Roman (`é` = `e` = 444, checked for the a/e/i/o/u/n/c families in
both cases); every ASCII printable in Courier is 600; every table is sorted,
duplicate-free, non-zero, and covers the full 218-point repertoire; unknown
characters are charged `.notdef` and counted; space and vertical metrics scale.

`tools/verify-against-coretext.swift` re-measures strings with CoreText at
12 pt and compares to the Rust width obtained through `examples/measure.rs`:

```sh
swift crates/fontmetrics/tools/verify-against-coretext.swift
```

Result on the generating machine (15 cases: a 60-glyph pangram and a 40-glyph
accented/Windows-1252 sentence in each of the seven faces, plus "Hello"):
0 failures against the rounding bound of 0.5 units per glyph; max |delta|
0.070 pt (Courier, 60 glyphs: Apple's Courier advance is 600.098 units, so the
integer table drifts 0.0012 pt per glyph at 12 pt); "Hello" delta +0.0038 pt;
6 of 15 cases within a fixed 0.01 pt, the rest within 0.032 pt except Courier.
A fixed 0.01 pt bound therefore holds for words, not for whole lines; the
per-glyph bound is the honest guarantee. `--tolerance PT` sets a fixed bound.

Note for anyone extending the verifier: Foundation's `Process` rewrites
arguments to decomposed (NFD) Unicode on macOS, which turns `é` into `e` plus
U+0301 and shows up as spurious unknown characters. The script therefore feeds
the text on stdin (`measure ... -`).

## Limitations

- Per-glyph advances only: no kerning pairs, no ligatures (`fi`, `fl`), no
  shaping. TeX's Computer Modern metrics also differ; this is a base-14 model.
- WinAnsi repertoire only. Greek, mathematical symbols, CJK, emoji, and
  combining marks are unknown and charged the width of `?`.
- No font embedding and no glyph outlines; this crate is metrics only.
- Measured from Apple's fonts on one macOS release. Regenerate and diff
  `src/tables.rs` if a macOS update changes the system fonts.
- Vertical metrics are the font's hhea/OS2-derived values as CoreText reports
  them, not TeX's `\baselineskip`.

## Proposal for the compiler owner (FT-005)

1. Add `flashtex-fontmetrics = { path = "../fontmetrics" }` to
   `crates/compiler/Cargo.toml` (still no external dependencies).
2. In `src/layout.rs`, replace `glyph_width(text, size)` (0.5 × size per char)
   with `flashtex_fontmetrics::width_pt(Face::TimesRoman, text, size)`.
3. Replace `SPACE_RATIO * size` with
   `flashtex_fontmetrics::space_pt(Face::TimesRoman, size)` (0.25 em, not 0.28).
4. Keep one item per word; only the widths change, so source spans and
   click-to-source are unaffected.
5. Optionally use `measure` and surface `unknown_chars` as a warning diagnostic,
   mirroring the `?` substitution `crates/pdf` already reports.
6. Line breaks will move; update the layout tests' expected `x_pt` values from
   the new widths rather than from the placeholder.
7. `\textbf`/`\textit` can later select `TimesBold`/`TimesItalic`; the tables
   are already present.
8. `crates/pdf` then places words consistently with no change on its side: it
   already writes `Times-Roman` with `WinAnsiEncoding`, whose viewer-side
   advances are the same integers these tables round to.
9. The Mac preview drawing system Times at the compiler's positions stops
   overlapping, since those positions were computed from the same font.
10. Use `VerticalMetrics::at(face, size)` for ascender/descender when the
    compiler starts computing line height from the font instead of a constant.
