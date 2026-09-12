# Proposal: consuming `flashtex-vector-graphics` from rendering-v2

Status: **proposal, documentation only.** Nothing here is implemented in any
consumer, and nothing changes runtime-v1. Rendering-v2
(`protocol/rendering-v2.schema.json` on
`origin/agent/commander-render-schema/rendering-v2-schema`, reviewed at
`41cacfc`) is under Commander review; the feature-gating decisions below are
the Commander's to make. This document exists so the ABI conversation can
start from concrete field mappings rather than from the crate's Rust types.

## 1. What v2 already has that this crate can feed

The v2 draft defines page items `glyph_run` and `rule`, integer coordinates in
`bp_2pow20` ticks (big points × 2^20), RGBA unpremultiplied sRGB `paint`, and
per-item provenance as either `sources` (1..128 ranges) or a
`synthetic_reason` string.

`item::Rule` maps onto the v2 `rule` with no schema change:

| crate (`Rule`, f64 pt, top-left) | v2 `rule` (integer ticks) | note |
| --- | --- | --- |
| `rect.x` | `x` | `round_ties_even(x · 2^20)` |
| `rect.y` | `top` | same rounding; v2 already uses a top-left origin, so no flip |
| `rect.width`, `rect.height` | `width`, `height` | `positive_tick` (≥ 1): a rule that rounds to 0 ticks in either axis is **dropped with a diagnostic**, never widened silently |
| `paint.color`, `paint.alpha` | `paint {r,g,b,a}` | `Color::to_rgb()` + alpha. Gray and RGB are exact; CMYK goes through the crate's naive conversion and must be labelled lossy in the diagnostic if used |
| `source: Some(range)` | `sources: [range]` | identical `{path, start_byte, end_byte}` byte semantics; the v2 validator additionally checks the range lies inside the snapshot document and on UTF-8 boundaries |
| `source: None` | `synthetic_reason` | producer supplies the reason (e.g. `"fraction bar"`); the crate carries no reason string today, so a producer-side map from `ItemId` to reason is needed, or `Rule` gains an optional `synthetic_reason` field (open question 5) |

The v2 validator computes `page.height - top - height` with checked
arithmetic, so the producer must clamp rules to the page before rounding, or
accept `invalid_display_list` rejections for off-page geometry.

Rounding rule proposed for **all** pt → tick conversion: multiply by 2^20 in
f64, then round half to even, then check the result fits in the schema's
`±(2^53 − 1)` bound. The conversion is exact for any value with ≤ 20
fractional binary digits, which covers every TeX scaled-point quantity
(2^16 sp per pt) with room to spare.

## 2. Proposed feature-gated v2 extensions (for Commander decision)

The remaining primitives have no v2 counterpart. They are proposed as
**opt-in features** using v2's existing `render_capabilities` /
`render_format_selected` negotiation, so a consumer that does not advertise a
feature never receives the item and the producer must fall back (§4). Nothing
is proposed for runtime-v1.

| proposed feature | item `kind` | fields (all coordinates in ticks) |
| --- | --- | --- |
| `path_fill` | `path_fill` | `path`, `fill_rule` (`nonzero`/`evenodd`), `paint`, provenance |
| `path_stroke` | `path_stroke` | `path`, `stroke {width, cap, join, miter_limit, dash}`, `paint`, provenance |
| `image` | `image` | `image_id`, `x`, `top`, `width`, `height`, `transform`, `alpha`, provenance; plus a top-level `images` resource table like v2's `fonts` (`image_id`, `sha256`, `byte_length`, `format` ∈ {`png`, `jpeg`}, `pixel_width`, `pixel_height`) |

`path` is the crate's JSON path array (`["m",x,y]`, `["l",x,y]`,
`["q",cx,cy,x,y]`, `["c",…]`, `["z"]`) with tick integers instead of f64.
`transform` is `[a,b,c,d,e,f]` where `a..d` are plain numbers (dimensionless)
and `e,f` are ticks. `stroke.width` and dash lengths are ticks.

**Groups are deliberately not proposed for transport.** The recommendation is
that producers ship the crate's *device list* (`DisplayList::flatten()`),
not the authored tree: transforms are applied, group opacity is folded into
`paint.a`, and clips are attached per item as a `clips` array of
`{kind:"rect",…}` / `{kind:"path",…}` in page space. Consumers then never
implement a transform or clip stack, and the v2 validator can keep checking
plain rectangles. If a consumer wants clip-free items it can advertise no
`clip` feature and the producer must rasterise-free fall back (§4). This
mirrors how the crate's `json::write_device_list` is already shaped.

Hit-testing stays consumer-side and needs no protocol support: the device
list carries every leaf's `source`, and `DeviceList::hit`/`source_at` are the
reference semantics (topmost first; strokes hit within `width/2 + tolerance`;
clips honoured). A Swift consumer would reimplement these on the JSON, so the
crate's tests double as the conformance description.

## 3. `crates/pdf` integration (after ABI agreement)

`pdf::content_stream(&DisplayList) -> PdfFragment` is already shaped for the
writer on `origin/agent/mac-pdf/pdf-output`:

1. Append `fragment.content` to the page's content stream after the text
   blocks, wrapped in `q … Q` (the fragment does not undo top-level colour
   and line-width changes).
2. For each `ExtGState { name, alpha }` add `/name << /Type /ExtGState /ca a
   /CA a >>` to `/Resources /ExtGState` (`ExtGState::dictionary()` builds the
   value).
3. For each `ImageResource { name, content_hash }` create one image XObject
   from the bytes the hash identifies and register it under `/name` in
   `/Resources /XObject`. Decoding is the writer's concern; the crate never
   sees pixels.
4. No change to number formatting: `pdf::num` matches the writer's `num()`
   (three decimals, trimmed, `-0` → `0`), and a top-level rule is emitted as
   `x (H - y - h) w h re f`, byte-identical to the writer's fraction bars.

The fragment keeps strokes in local space via `cm`, so line widths and dashes
are exact under any affine transform; only the device list (used for
preview/hit-testing) approximates anisotropic stroke scaling.

## 4. Fallback when a feature is not selected

Producers must not silently drop geometry. Proposed order of degradation:

- `path_fill`/`path_stroke` not selected: emit a `warning` diagnostic per
  dropped item with its `sources`, and, where the path is an axis-aligned
  rectangle (`Path::rect` under an axis-aligned transform), emit a `rule`
  instead.
- `image` not selected: emit a `rule` placeholder of the placed box with a
  `synthetic_reason` of `"image placeholder"` and a `warning` diagnostic.
- `clips` present but consumer cannot clip: producer pre-intersects
  rectangular clips into rules (`Rect::intersect`) and drops items whose
  clip has no rectangular intersection, with a diagnostic; path clips are
  reported as unsupported.

## 5. Open questions for the ABI

1. Tick rounding of Bézier control points changes curve shape by < 2^-20 bp;
   acceptable, but should the schema state it.
2. Should `stroke.width` be allowed to be 0 (PDF's thinnest line) or must it
   be `positive_tick`.
3. Image resources: declare by `sha256` like fonts (recommended, matches
   `Image::content_hash = "sha256:<hex>"`), and whether pixel data travels
   inline base64 (as captures do) or by path.
4. Clip representation: per-item `clips` array (recommended) versus a
   `clip_id` table.
5. `synthetic_reason` for source-less rules: add an optional field to
   `item::Rule` (crate change, trivial) or keep it producer-side.
6. Colour: v2 is RGBA-only. Either the schema gains `gray`/`cmyk` paints or
   producers accept the crate's documented naive CMYK → RGB conversion and
   say so in a diagnostic.

None of these block the crate; each is a one-line mapping change once decided.
