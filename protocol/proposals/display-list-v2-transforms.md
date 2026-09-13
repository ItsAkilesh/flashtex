# Proposal: `display-list-v2-transforms` — transformed glyph runs and clipped images

Status: **PROPOSAL ONLY** (agent `kabir-claude`, branch
`agent/kabir-claude/inline-graphics`). Additive and opt-in, on top of
`display-list-v2` (and, for `clip`, `display-list-v2-images`). It changes no
frozen contract: a request that does not list the capability receives
byte-identical `display_list` lines. Needs a consumer co-sign (Mac painter,
`Export PDF (v2)…`, `flashtex-pdf-exact from-v2`; `mac-claude-a`) before a
consumer relies on it.

## 1. Why

graphicx's `\rotatebox`, `\scalebox`, `\resizebox`, `\reflectbox` and
`\includegraphics[angle=…]` transform whatever the box holds. pdfTeX paints
it through `\pdfsetmatrix` (pdftex.def `\Grot@start`, `\Gscale@start`), so
the glyphs of rotated text are rotated, and `trim=…,clip` images are cut by a
clipping form (pdftex.def `\GPT@DoClipEnd`). The frozen glyph run has no
shape transform and the image item has no clip.

The producer already places everything exactly without the capability:
glyph origins and advances are page positions of the transformed text, a
rotated `\rule` is a filled path (path-v0), image transforms compose, and an
image's box encloses only its visible part. What an old consumer cannot do is
draw the glyph shapes rotated/mirrored, or cut a rotated clipped image.

## 2. Negotiation

Layout capability string `display-list-v2-transforms`, accepted only when the
same request lists `display-list-v2`; echoed in `accepted` when honoured.
Unknown to old producers, never sent by old consumers.

## 3. Fields

### 3.1 `glyph_run.glyph_transform` (optional)

```
{"kind":"glyph_run", …, "font_size":<ticks>, "glyph_transform":[a,b,c,d], "glyphs":[…], …}
```

The linear map, page space with y down, applied to every glyph's outline
about its own origin: a glyph at `(origin_x, baseline_y)` is drawn with the
text matrix `font_size · [a b c d]` placed at its origin (PDF y-up space:
`[a, −b, −c, d]`). The matrix is normalised to `|ad − bc| = 1`; the uniform
scale is already in `font_size`. Absent means upright (identity). Present
only for text inside a rotated, reflected or unevenly scaled box; uniformly
scaled text keeps a plain larger `font_size`.

Origins, advances (`advance_x`/`advance_y` may now be vertical or negative),
hit rects (axis-aligned bounding boxes) and carets are page geometry in
every case, so selection, hit testing and text extraction are unchanged.

`required_features` gains `glyph_transform` when any run carries one.

### 3.2 `image.clip` (optional; with `display-list-v2-images`)

```
{"kind":"image", "clip":[u0,v0,u1,v1], "height":…, "transform":[…], …}
```

The visible part of the image's unit square (same `u`/`v` as `transform`):
concatenate `transform`, clip to that rectangle, draw the image. The item's
box (`x`, `top`, `width`, `height`) encloses exactly the visible part, so a
painter that clips to the box (proposal `display-list-v2-image` §3) is
already exact for unrotated images; `clip` is what makes rotated clipped
images exact.

`required_features` gains `image_clip` when any image carries one.

Numbers: 1e-6 resolution for both fields.

## 4. Producer semantics (implemented)

`crates/render-pipeline/src/graphics.rs` (`TBox`, `place_image`) and
`src/typeset/graphics_boxes.rs`, transcribing graphics.sty `\Gscale@box`,
`\Gscale@@box`, `\Grot@box`, graphicx.sty `\Gin@ii`/`\Gin@esetsize`/
`\Grot@box@kv` and pdftex.def `\Ginclude@@pdftex` (viewport offset, clip).
Evidence: `crates/render-pipeline/tests/graphics_oracle.rs`, 24 fixtures
against pdfLaTeX (words 0.5 bp, image transforms and clips 0.01 bp), and
`transform_fields_are_only_serialised_when_negotiated`.

## 5. Consumer obligations (for the co-sign)

1. Advertise the capability only if the painter draws glyphs with an
   arbitrary 2×2 text matrix and clips an image in its own unit space.
2. Mac CoreGraphics painter: `CGContext.textMatrix` per run from
   `glyph_transform` (converted to y-up); image: `concatenate(unitToPage)`,
   `clip(to: CGRect(u0, v0, u1−u0, v1−v0))`, draw.
3. PDF export (`flashtex-pdf-exact from-v2`, Mac `Export PDF (v2)…`): text as
   `[a −b −c d x y] Tm` per positioned glyph group; clipped image as
   `q <cm> u0 v0 (u1−u0) (v1−v0) re W n /Im Do Q`.
4. A consumer that did not negotiate the capability never receives either
   field and draws upright glyphs at the exact origins.
