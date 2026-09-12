# Experimental Rust rendering core

Original Rust types, bounded JSON parsing and semantic validation for the
experimental `protocol/rendering-v2.schema.json` proposal. This crate does not
change the compiler, native preview, PDF writer or runtime-v1 wire behavior.

```sh
cargo test --manifest-path crates/rendering-core/Cargo.toml
cargo clippy --manifest-path crates/rendering-core/Cargo.toml -- -D warnings
```

`parse(&[u8])` enforces a 32 MiB message limit, exact typed fields, protocol version
and known message/item types. Call `Envelope::validate(Some(&offer))` afterwards:
parsing alone does not negotiate capabilities or validate cross-record references.
Offers and explicit rejections can be validated with no preceding offer. Selection
IDs must match the offer; display IDs identify their own compiler request.

`DisplayList::validate` checks font manifests, original GID bounds, contiguous pages,
finite RGBA, exact fixed-point coordinates and checked geometry sums, explicit rule
geometry, logical UTF-8 clusters, caret/hit regions and source provenance. Logical
cluster text partitions the run and is extracted once per cluster. Multiple glyphs
may refer to one cluster; consumers must not repeat its text once per glyph.

`DisplayList::validate_resources` additionally takes exact `SourceSnapshot` maps,
font byte maps and a `FontValidator` adapter. It verifies SHA256/length before calling
the separately owned font loader, compares units/GID count, and checks source UTF-8
boundaries against the exact document revision. The hook intentionally avoids a
second competing font parser. `font_adapter::StaticTrueTypeLoader` now calls the original `font-resources`
crate's `inspect_static_truetype` API. `validate_with_collection` additionally
requires exact descriptors from a loader-owned immutable, license-bound collection.

All resource evidence retains `paintable: false`: source/font identity checks are
not proof that a consumer safely paints those outlines, that shaping is correct,
or that native/PDF output matches. Production activation requires compiler/PDF/Mac
agreement on the proposal and their acceptance gates.

Coordinates use `Tick(i64)`, bounded to JSON's exact ±(2^53−1) integer range, with
1,048,576 ticks per PDF point. `Tick::from_tex_sp` converts TeX scaled points once
using exact integer arithmetic and round-to-nearest/ties-to-even; TeX and PDF point
sizes are deliberately different. JSON floating-point geometry is rejected.

Tests use declared synthetic glyph IDs and a deliberately fake font-validation
callback to exercise mapping and resource substitution; these are not real fonts
or visual correctness evidence. Fixtures retain the experimental schema provenance
from commit `41cacfc`; no image, network provider, or external TeX engine is run.

`hit_test::PageIndex` builds a per-page spatial index from declared hit rectangles
and rules. Queries require the matching project/revision, use exact half-open
fixed-point rectangles clipped to page bounds, and resolve overlaps in paint order.
Caret selection uses only supplied caret positions; absent carets return a whole
logical cluster. TeX source ranges and synthetic provenance remain unchanged. Shared
cluster metadata avoids copying caret/source arrays for every rectangle.

`cache::DisplayCache` accepts an authoritative `RenderIdentity` before work starts.
The identity includes project revision, source digests/revisions, font manifests
and a compiler/configuration digest. A `RenderTicket` binds that expectation to a
cache generation. Source/font/config changes invalidate prior tickets immediately;
late frames cannot replace a newer expectation. Installation verifies exact source
and font bytes, keeps immutable font bytes alongside the complete display list,
and rejects conflicting geometry for one immutable identity. It never combines
pages from different revisions. Identical completed installs reuse one `Arc`.

Callers must obtain the current frame through its ticket before painting or querying
its index. Previously borrowed Arcs remain readable but are no longer current after
invalidation. Cache accounting limits retained serialized payload/font bytes;
allocator/index overhead and Arcs retained externally are additional memory.


Real resource integration probe (no rendering or glyph-shaping claim):

```sh
cargo run --manifest-path crates/rendering-core/Cargo.toml --example resource_probe -- /path/to/font.ttf
```

Verified locally with installed LiberationSans-Regular.ttf, SHA256
`76d04c18ea243f426b7de1f3ad208e927008f961dc5945e5aad352d0dfde8ee8`,
2048 units per em and 2620 glyphs. The probe uses a synthetic positioned-text
fixture and keeps `paintable: false`; the font itself is not copied into this crate.
Font license/embedding permission remains explicit loader metadata and is never
inferred from successful parsing or converted from unknown to allowed.

`transform::ViewportTransform` maps canonical ticks to exact rational viewport
coordinates with positive zoom, translation and optional y-axis reversal. It clips
source rectangles before transformation and preserves half-open boundary inclusion
even when the y axis flips. `PageIndex::hit_test_exact` retains fractional query
positions for caret choice; integer floor is used only for exact membership tests
against integer rectangle edges. No pixel rounding changes caret selection.
Rational denominators are bounded to one million; checked distance arithmetic
returns an explicit error if an extreme query exceeds its i128 budget.

`wire::parse_validated` and `wire::serialize_validated` form the opt-in consumer
boundary. They validate capabilities and semantic references, return typed errors
for unsupported versions/messages/primitives, and never emit a partial oversized
frame. The serializer defaults to the caller's requested cap, bounded above by
32 MiB. Resource verification remains separate from wire validity.

```sh
cargo run --manifest-path crates/rendering-core/Cargo.toml --example validate_display -- \
  crates/rendering-core/tests/fixtures/synthetic-display-list.json \
  crates/rendering-core/tests/fixtures/capabilities.json
```

The offline harness validates, serializes, reparses and verifies a stable canonical
roundtrip. It reports `paintable: false`; a synthetic glyph fixture is not a native
rendering or font-shaping test.

`outlines::PreparedOutlines` validates exact display/font descriptors once, then
resolves original glyph IDs through the immutable font loader. It preserves font
SHA, component instances, logical cluster/source ranges and explicit synthetic
provenance. Loader-supplied quadratic paths and implied points are placed with exact
checked rational size/baseline arithmetic; font coordinates point upward and page
coordinates downward. `hinting_applied` remains false. Unsupported loader cases
return errors, never silently empty paths or replacement fonts.

```sh
cargo run --manifest-path crates/rendering-core/Cargo.toml --example outline_probe -- \
  /path/to/font.ttf /path/to/LICENSE.txt A
```

Local LiberationSans `A` probe: original GID 36, 17 points, two contours and
17 placed path commands; exact font hash recorded above. Supplied installed license
SHA256 was `93fed46019c38bbe566b479d22148e2e8a1e85ada614accb0211c37b2c61c19b`.
No glyph shaping, hint execution, raster painting or reference-TeX parity is claimed.

`glyph_cache::GlyphPathCache` retains immutable expanded quadratic paths by font
SHA256, face, original GID and `UnhintedExactComponentsV1` policy. Font aliases share
geometry only when the actual bytes match. LRU limits bound entry count and retained
path/component payload estimates; map/allocator overhead and caller-held Arcs are
additional memory. Unsupported/font-error results and oversized-path budget failures
remain explicit cached outcomes, preventing repeated expansion attempts.
`PreparedOutlines::glyph_cached` applies each glyph's exact size/origin after lookup.

The developer outline probe now reports expansion-cache measurements separately.
One local debug-build LiberationSans `A` run measured one cold expansion at 59,456ns
and 1,000 cache lookups totaling 901,476ns (one expansion, 1,000 hits). These are
single-run font-cache observations, not native paint or edit-to-preview latency.

`batch::PreparedBatchSource` verifies immutable font descriptors and source snapshots
once, then creates atomic consumer-neutral page batches. Batches preserve glyph/rule
paint order, sRGB paint, exact quadratic commands and source/synthetic provenance.
The canonical rectangular clip is intersected with the page; `None` in the returned
`visible_clip` means empty. Consumers must apply that clip when painting curves.
Glyph ink is never culled using source hit boxes. Operation/path limits fail the
whole batch explicitly; immutable cached expansions remain reusable on retry.
Collection validation uses loader-owned verified bytes without copying or reparsing
fonts. This adapter does not activate runtime-v2 or establish native paint parity.
