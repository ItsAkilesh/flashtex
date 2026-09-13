# Proposal: `display-list-v2-links` — links, destinations and outline for `display-list-v2`

Status: **PROPOSAL ONLY** (agent `kabir-claude`, hyperref coverage). Additive
and opt-in. It changes no frozen contract: `protocol/rendering-v2.schema.json`,
`docs/contracts/runtime-v1*.md` and the `display_list` line of a request that
does not ask for this capability stay byte-for-byte as they are. Needs a
consumer co-sign (Mac painter + PDF export, `mac-claude-a`) before the Mac side
relies on it. Nothing in `apps/` changes with this document.

Producer inputs already exist on the compiler side (`crates/compiler`
`Parsed::hyperref`, PR "compiler: hyperref options, autoref/nameref,
link/destination/bookmark records"); the PDF writer side exists in
`crates/pdf` (`navigation::Navigation`, `exact::render_exact_with`). This
document is the bridge the render pipeline would emit between them.

## 1. Summary

1. New layout capability `display-list-v2-links`, accepted only together
   with `display-list-v2`. Unknown to old producers (never echoed), never
   sent by old consumers.
2. When accepted and the document loads hyperref, the `display_list` line
   gains one top-level `navigation` object (§3). When not accepted, the line
   is unchanged; layout is identical either way (links never move text).
3. `required_features` is **not** extended: a consumer that ignores
   `navigation` still paints the page correctly. Link text colour under
   `colorlinks` is ordinary glyph-run paint and is present with or without
   the capability.
4. The runtime-v1 `compile_result` has no link item and is unchanged.

## 2. Request

```json
{"type": "compile", "payload": {"layout_capabilities": ["display-list-v2", "display-list-v2-links"], "...": "..."}}
```

## 3. `navigation` object

All coordinates use the envelope's `coordinate_unit` (`bp_2pow20` ticks,
y down from the page's top-left), like every other item.

```json
"navigation": {
  "links": [
    {
      "page": 1,
      "rect": [x0, y0, x1, y1],
      "class": "link",
      "border": ["0", "0", "1"],
      "color": ["1", "0", "0"],
      "target": {"destination": "section.1"},
      "source": {"document": "main.tex", "start": 120, "end": 139}
    },
    {
      "page": 1,
      "rect": [x0, y0, x1, y1],
      "class": "url",
      "border": ["0", "0", "1"],
      "color": ["0", "1", "1"],
      "target": {"uri": "https://example.com"},
      "source": {"document": "main.tex", "start": 150, "end": 175}
    }
  ],
  "destinations": {
    "section.1": {"page": 1, "x": 0, "y": 0, "view": "xyz"}
  },
  "outline": [
    {"title": "Introduction", "level": 1, "destination": "section.1"}
  ],
  "outline_open": false,
  "page_mode": "use_outlines",
  "open_action": "fit_first_page",
  "info": {"title": "", "author": "", "subject": "", "keywords": "", "creator": "LaTeX with hyperref"}
}
```

- `links[]`: one entry **per line piece**. A link that breaks across lines
  (pdfTeX splits `\url` and long `\href` text this way) is several entries
  with the same `target` and `source`, in reading order. `class` is
  hyperref's colour class (`link`, `url`, `cite`, `file`); `border`/`color`
  are the verbatim decimals hyperref writes (`\@pdfborder`,
  `\@linkbordercolor`, ...; `0 0 0` under `colorlinks`/`hidelinks`, `/C` is
  still present). `target` is exactly one of `destination` (a key of
  `destinations`; producers must not emit dangling names — an undefined
  `\ref` has no link, as in pdfTeX) or `uri`. `source` is the compiler's link
  span (exact UTF-8 byte range), for editor hit-testing and staleness checks.
- `destinations`: every named destination the document defines
  (`section.1`, `subsection.2.1`, `section*.3`, `equation.4`, `figure.1`,
  `table.1`, `Item.2`, `theorem.1`, `cite.key`, `name.level` from
  `\pdfbookmark`, `\hypertarget` names). `Doc-Start` and `page.<n>` are
  implied for every hyperref document and may be omitted.
- `outline[]`: document order; `level` is hyperref's level (section 1,
  subsection 2, ..., `\pdfbookmark[0]` 0), already level-checked. The tree is
  derived by the consumer: an entry's parent is the nearest earlier entry with
  a smaller level.
- `page_mode`: `use_outlines` when hyperref's `bookmarks` option is on (the
  default), else `use_none`. `open_action`: `fit_first_page` (hyperref's
  `pdfstartview=Fit`).

## 4. Geometry the producer must match (pdfTeX)

Measured against pdflatex in `crates/compiler/tests/hyperref_oracle/expected`
(32 cases: rectangles, destinations and word boxes are recorded there):

- **Link rectangle** (pdfTeX `\pdfstartlink` with hyperref, `\pdflinkmargin`
  = 1pt, hpdftex.def 320): horizontally from the first to the last glyph edge
  of the piece, vertically the height and depth of the enclosing line box,
  all four sides grown by 1pt. The pipeline must derive height/depth from the
  line it set, not from glyph ink.
- **Destination** (`/XYZ left top`): hyperref raises anchors
  (`\Hy@raisedlink`, `\HyperRaiseLinkDefault`); the exact raise per construct
  (heading, equation, caption, item) is to be pinned from the recorded
  `destinations` before this proposal leaves draft. Until then consumers must
  treat `y` as approximate.
- Acceptance for the producer: link rectangles and destination coordinates
  within 0.5 bp of the oracle on the 32 cases, with the same number of line
  pieces per link.

## 5. Consumers

- **PDF export (`crates/pdf`, exact route)**: map `navigation` 1:1 onto
  `navigation::Navigation` (`page` 1-based to index, ticks to decimals with
  the existing y flip) and call `exact::render_exact_with`. Implemented and
  oracle-tested on the writer side; `from_v2` does not read `navigation` yet.
- **Mac preview**: may hit-test `links[].rect` for click-through (internal
  targets scroll to `destinations[..]`, URIs open only after the app's own
  scheme policy); may show `outline` as a sidebar. Painting borders in the
  preview is optional (PDF viewers draw `/Border`, the page content does not).

## 6. Open questions for co-sign

1. Should `source` carry the compile revision explicitly (as the image
   proposal's items do not), or is the envelope's revision binding enough?
2. URI scheme policy: the writer escapes and bounds URIs but does not filter
   schemes (pdfTeX does not); the Mac preview should apply its own allowlist
   before opening one.
