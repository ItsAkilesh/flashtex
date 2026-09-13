# flashtex-paragraph-layout

Original Rust implementation of TeX-style paragraph line breaking, horizontal
and vertical metrics, and deterministic page breaking, behind an integration
API that consumes font metrics from the font engine and produces positioned
glyph runs for the preview and PDF back ends. Edition 2024, no external crates,
no TeX engine involved.

```
cargo test            # 72 tests: unit, golden (hand-derived numbers), pdflatex oracles (incl. 30-paragraph hyphenation corpus)
```

## API tour

```rust
use flashtex_paragraph_layout::*;
use flashtex_paragraph_layout::core14::Core14Times;

// 1. Items: boxes, glue, penalties, kerns.
let hyph = LiangHyphenator::english();       // pdflatex's default patterns; ExplicitDiscretionary / NoHyphenation also shipped
let mut b = ParagraphBuilder::new(&hyph);
b.text(&Core14Times::ROMAN, 12.0, "A naïve reader at the café ", 0)?;
b.text(&Core14Times::BOLD, 12.0, "expects", 28)?;
let items: Vec<Item> = b.finish(Glue::fil());   // strips trailing glue, adds \parfillskip + forced break

// 2. Lines. `layout_paragraph` validates dimensions/item count itself (NaN,
// overflow, oversized input) and returns a typed error for bad input, same
// as `try_layout_paragraph` below.
let params = LineBreakParams::article_12pt_letter_1in();   // 469.755pt measure, TeX defaults
let lines: Lines = layout_paragraph(&items, &params)?;
for line in &lines.lines {
    for run in &line.runs {
        // run.x, run.baseline_y (paragraph frame), run.font (FontId), run.size,
        // run.glyphs[i].{gid, x_offset, advance, cluster}, run.source, run.is_hyphen
    }
}
// lines.breaks[i].{item, ratio, badness, fitness, demerits, hyphenated}
// lines.stats.{pass, total_demerits, overfull, underfull, hyphenated_lines}

// 3. Pages.
let blocks = vec![ParagraphBlock::body(lines)];
let pages: Pages = layout_pages(&blocks, &PageParams::article_12pt_letter_1in_tex_pt());
// pages.pages[p].runs: PositionedRun in absolute page coordinates
// pages.overflow: every line that did not fit (never dropped)
```

`Item` is the only input the breaker reads. Callers with shaped output from the
font engine build boxes directly with `GlyphRun::from_shaped(font_id, size,
units_per_em, ascender, descender, &[ShapedGlyph], source_range)` and add
`Item::Glue`/`Item::Penalty`/`Item::Kern` themselves; `ParagraphBuilder` is the
convenience path for text (it implements TeX `\spacefactor` spacing, cross-run
kerns, ligatures and discretionaries through a `Hyphenator`).

### Font metrics input

`FontMetricsSource` (in `src/metrics.rs`) is the contract: `font_id()` (opaque
32-byte content-addressed `FontId`), `units_per_em`, `advance(ch)`, `kern(l, r)`,
`glyph_id(ch)`, `ascender`, `descender`, `line_gap`, `space` and the TeX
`\fontdimen`-style `space_stretch` / `space_shrink` / `extra_space`, plus an
optional `ligature(l, r)`. All in font units; the builder scales to points.

`core14::Core14Times` (Roman/Bold/Italic) is the shipped adapter: Adobe Core 14
AFM widths (copied from de1020c's `crates/compiler/src/metrics.rs`), the AFMs'
272 ASCII kern pairs per face, `fi`/`fl` ligatures, and the psnfss `ptm*8t`
space glue (250 / 150 / 60 / extra 60 units), with provenance in the file
header.

**FT-018 (`crates/font-engine`, branch `agent/mac-font-engine/tex-fonts`)**
maps directly: its `FontId.content_sha256: [u8; 32]` is our `FontId(..)`;
`Face::advance(gid)`, `Face::kerning(l, r)`, `Face::glyph_id(ch)`,
`Face::vertical_metrics()` implement the trait one-to-one; and its
`Shaped.clusters[*].glyphs[*]` (original gid, advance in font units,
`source_range`) feed `GlyphRun::from_shaped` so nothing is re-measured. The
adapter is a dozen lines in whichever crate depends on both; it is not written
here to avoid stacking on an unmerged branch.

### Output contract

* `PositionedRun.font` and every `gid` and `cluster` (source byte range) are the
  input's values, untouched. Ligatures keep a multi-byte cluster; a hyphen from a
  `\-` discretionary carries the marker's byte range (empty for automatic points).
* Glyph origin = `(run.x + glyph.x_offset, run.baseline_y)`. `advance` is the pen
  movement to the next glyph (kern included).
* No new runtime-v1 item kinds are proposed. For rendering-v2's glyph runs
  (absolute origins, ticks of 1/2^20 bp), convert `x`/`baseline_y` as described
  below and multiply by 2^20.

## Coordinates and units

The crate is unit-agnostic: all inputs (sizes, widths, skips) and outputs share
one linear unit. Two frames:

| Frame | Origin | x | y |
|---|---|---|---|
| `Lines` (paragraph) | left edge of the measure, top of the first line | rightward, before `left_skip` | `baseline_y` downward; first baseline = first line's height |
| `Pages` | page top-left | `margin_left` + paragraph x | absolute baseline from the page top |

PDF: `x_pdf = x`, `y_pdf = page_height − baseline_y` (same as rendering-v2).

TeX points vs PDF points: LaTeX sizes (12pt, 14.5pt, 1in = 72.27pt) are TeX
points; PDF and the current compiler's `*_pt` fields are big points
(1/72 in). Run the layout in TeX points with `PageParams::article_12pt_letter_1in_tex_pt()`
and convert every output with `tex_pt_to_bp` (× 72/72.27), or run directly in
bp by pre-scaling the font size (12 pt → 11.955 bp) and every parameter. The
oracle comparison (`docs/comparison.md`) uses the former.

## Parameters and TeX / LaTeX article defaults

`LineBreakParams` (`article_12pt_letter_1in()` sets these):

| Field | TeX name | Default | Notes |
|---|---|---|---|
| `line_width` | `\hsize` | 469.75499 pt | 6.5 in; 12pt article with `geometry margin=1in` |
| `mode` | — | `Justified` | `RaggedRight` = LaTeX `\raggedright` (`\rightskip 0pt plus 1fil`, interword shrink kept) |
| `algorithm` | — | `TotalFit` | `FirstFit` greedy for comparison |
| `pretolerance` / `tolerance` | same | 100 / 200 | pass 1 ignores automatic hyphens; `pretolerance < 0` skips pass 1 |
| `emergency_stretch` | same | 0 | > 0 enables a third pass |
| `line_penalty` | `\linepenalty` | 10 | |
| `adj_demerits` | `\adjdemerits` | 10000 | fitness classes differing by > 1 |
| `double_hyphen_demerits` / `final_hyphen_demerits` | same | 10000 / 5000 | |
| `parindent` | `\parindent` | 0 pt | article: 15 pt (10pt), 17.62 pt (12pt); FlashTeX/oracle use 0 |
| `left_skip` / `right_skip` | same | 0 pt | |
| `baselineskip` | `\baselineskip` | 14.5 pt | 10pt article 12 pt; 11pt 13.6 pt; 12pt 14.5 pt |
| `lineskip` / `lineskiplimit` | same | 1 pt / 0 pt | |

`ParagraphBuilder`: `hyphen_penalty` / `ex_hyphen_penalty` 50 (`\hyphenpenalty`,
`\exhyphenpenalty`), `french_spacing` false (`\nonfrenchspacing`).

`PageParams` (`article_12pt_letter_1in_tex_pt()`):

| Field | TeX name | Default |
|---|---|---|
| page 614.295 × 794.97 pt, margins 72.27 pt | Letter, `margin=1in` | |
| `topskip` | `\topskip` | 12 pt (10pt article: 10 pt) |
| `max_depth` | `\maxdepth` | 6 pt (`.5\topskip`) |
| `parskip` | `\parskip` | 0 pt plus 1 pt (inserted before every paragraph; discarded at a page top) |
| `baselineskip` / `lineskip` / `lineskiplimit` | | 14.5 / 1 / 0 pt |
| `baseline_grid` | — | `None`; `Some(pitch)` snaps every baseline to `topskip + k·pitch` |
| `club_lines` / `widow_lines` | `\clubpenalty` / `\widowpenalty` style | 2 / 2 lines kept together |

`ParagraphBlock::section_heading_12pt` encodes LaTeX `\section` in a 12pt
article: before 3.5 ex (18.9 pt), after 2.3 ex (12.42 pt), keep-with-next. Use
`\Large` = 17.28 pt bold for the heading text itself.

## Hyphenation (FT-064)

* `LiangHyphenator` (`src/liang.rs`): Liang pattern matching over the TeX
  pattern files vendored verbatim in `patterns/` (licences in their headers,
  summarised in the module docs). `LiangHyphenator::english()` loads Knuth's
  `hyphen.tex`, which is what TeX Live's `language.dat` loads for pdflatex's
  default `english`; `en_us_max()` loads hyph-utf8 `hyph-en-us.tex`
  (`usenglishmax`). `add_exceptions("ta-ble as-so-ciate")` is `\hyphenation`;
  `left_min`/`right_min` are `\lefthyphenmin`/`\righthyphenmin` (2/3) and apply
  to exceptions too. Words over 63 letters, and chunks containing an explicit
  hyphen or `\-`, get no automatic points (TeX §894–903).
* `ParagraphBuilder::text` only hyphenates a word that directly follows glue
  (never a paragraph's first word, as in TeX); an explicit `-` is followed by an
  empty discretionary with `ex_hyphen_penalty`; `\-` and automatic points use
  `hyphen_penalty` and add a hyphen glyph when the line breaks there.
* `ParagraphBuilder::discretionary(font, size, pre, post, nobreak, source)` is
  `\discretionary{pre}{post}{nobreak}`: `Penalty::post_break` starts the next
  line and the `Penalty::replace_count` no-break items are dropped at a break.
* The breaker ignores automatic points in the `\pretolerance` pass, charges
  `\doublehyphendemerits`/`\finalhyphendemerits`, and counts pre-/post-break
  widths in the lines they end/start.
* Oracle: `tests/hyphenation_oracle.rs` checks that 30 paragraphs (91 lines, 20
  hyphenated) break exactly where pdflatex breaks them (`tests/oracle/`,
  regenerate with `python3 tests/oracle/gen_hyphenation_oracle.py`; TeX is
  never used outside that generator).

## Algorithms

* **Total-fit**: Knuth–Plass with TeX's specifics — legal breaks (glue after a
  box, penalty < 10000, kern before glue), discardables dropped after a break,
  integer badness `round(100·r³)` capped at 10000, fitness classes from badness
  (very loose > 99, loose > 12, decent, tight > 12 when shrinking), demerits
  `(linepenalty + badness)² ± penalty²` + hyphen/adjacent charges, the final
  break treated as hyphenated for `\finalhyphendemerits`, best node per fitness
  class with the `<=` tie rule, node creation for classes within
  `adj_demerits` of the minimum, deactivation of overfull predecessors, and
  artificial demerits on the final pass so an unbreakable overflow is set as
  an overfull line and reported instead of failing.
* **First-fit**: natural width ≤ measure, break at the last legal point; forced
  breaks honoured; overflows reported.
* **Vertical**: TeX interline glue (`baselineskip − prev_depth − height`, or
  `lineskip` below `lineskiplimit`) across paragraphs and skips, glue discarded
  at page tops, `topskip` for the first line, raggedbottom pages, greedy page
  filling with the line-count constraints above, keep-with-next for headings.

Everything is deterministic: no hashing, randomness, or time; equal inputs give
`==` outputs (`tests/golden.rs::layout_is_deterministic`).

## Not modelled (honest limits)

* `\hbox`/`\vbox` nesting, `\parshape`/`\hangindent`, `\looseness`, floats,
  footnotes, marginpars, math (FT-020), `\flushbottom`, `\addvspace` merging,
  vertical glue stretch/shrink, per-character heights/depths (the font
  ascender/descender is used), infinite-order *shrink*.
* Hyphenation of words set in several fonts, non-ASCII `\lccode`s beyond
  Unicode `is_alphabetic`, `\uchyph=0`, and TeX's ligature/kern
  reconstitution at automatic hyphen points (we kern across the point; no kern
  is applied across an explicit `\-`).
* Ligature/kern interaction across a discretionary (TeX reconstitutes; we kern
  around the break point as a separate `Item::Kern`).
* Right-to-left or vertical scripts; combining marks (a mark is a glyph with
  its own advance from the metrics source).

## Proposed integration (for the compiler lead)

The AST adapter is the compiler's; the suggested shape:

1. For each paragraph, walk inlines and call `ParagraphBuilder::text(font, size,
   text, byte_offset)` per styled span (`\textbf` → Bold face, `\emph` → Italic),
   `line_break()` for `\\`, and `finish(Glue::fil())`. Headings become their own
   paragraph with `\Large` bold and `ParagraphBlock::section_heading_12pt`.
2. Call `layout_paragraph` with `LineBreakParams::article_12pt_letter_1in()`
   (or the class/geometry-derived values), then `layout_pages`.
3. Emit runtime-v1 text items **per positioned run** (one `TextItem` per
   `PositionedRun` with `x_pt`/`baseline_y_pt` converted to bp, `span` from
   `run.source`), which is a strict refinement of today's one-item-per-word
   output and needs no new item kinds; rendering-v2 glyph runs map field-for-field.
4. Replace the compiler's `LINE_SPACING 1.2` / `PARAGRAPH_GAP_PT 6` / greedy
   placeholder with the parameters above; issue #10's three vertical findings
   (heading +14.75 pt, 6 pt paragraph gap, 14.45 vs 14.4 pitch) are all
   resolved by that switch, as `docs/comparison.md` shows.
5. When FT-018 lands, implement `FontMetricsSource` for `font_engine::Face` (or
   feed `Shaped` clusters to `GlyphRun::from_shaped`) and delete nothing here:
   the Core 14 adapter remains the fallback for the standard PDF fonts.
