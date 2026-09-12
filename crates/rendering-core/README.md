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

`tex_adapter` binds original 8-bit TFM codes through the font loader's explicit
encoding manifest. Physical and virtual runs retain original GIDs, font/TFM hashes,
exact metrics/kerns and original input intervals. VF rules convert their lower-left
reference to the page's top-edge convention. Callers explicitly select
`ExactRationalNoTexRounding`; exact rational values never imply TeX scaled-point
rounding. Unsupported virtual commands, .notdef, absent bindings and arithmetic
budgets fail explicitly.

`EncodedRun::batch` emits existing unhinted exact quadratic draw batches using only
loader-bound immutable fonts and supplied logical UTF-8/source interval mappings.
Source snapshots must match declared revision/digest. Internal batch conversion preserves rational origins/sizes through exact checked
quadratic placement; `ExactRule` retains fractional virtual-rule bounds. No Unicode inference, interval
interpolation, implicit rounding or production runtime activation occurs. Flat/VF
equivalence fixtures are original synthetic data, not a reference-TeX oracle.

`graph_cache::GraphCache` immutably borrows one fully declared resource graph and
caches exact nested packets by resource hash key and encoded character. A cache
cannot be reassigned to another graph: file hashes alone do not identify encoding
declarations or virtual local-font bindings. Cached packets retain the complete
root-to-leaf source chain. LRU entry/payload limits and explicit cached unsupported
resource or oversize outcomes bound retained work; loader limits bound expansion
separately. Payload figures exclude map/allocator overhead and externally held Arcs.
Tests explicitly distinguish identical font/TFM hashes with different encodings.

`place_path_exact` accepts checked rational font sizes and origins, retaining exact
quadratic coordinates with bounded i128/u128 arithmetic. `ExactClip` supports exact
intersection and half-open membership; `batch_with_exact_clip` returns an
`ExactDrawBatch` whose exact clip is authoritative for the consumer. Curves remain
unflattened, and the original integer wire schema is unchanged. Overflow is an
explicit error. Tests cover fractional glyph origins/scales, VF rule bounds, clips,
precision exhaustion and integral-path equivalence.

`tex_adapter::nested_run` consumes cached nested graph packets and verifies every
physical placement against a supplied font/TFM/encoding binding. It produces the
same exact `EncodedRun` API, retaining encoded input intervals and rational nested
scale/offset/rule geometry. `NestedRun::source_chains()` is indexed by the original
run operation index. Prefer `NestedRun::batch`, which attaches the correct chain
to each retained primitive by stable item/glyph identity after culling. Missing or conflicting bindings fail before any partial run is
returned. Synthetic tests compare flat and two-level virtual glyph/rule runs and
verify that differing encoding declarations cannot be substituted at this boundary.

`TracedBatch` keeps source chains attached to primitives, so vector reordering does
not relabel provenance. Primitive identity is scoped by project, revision and page;
`PrimitiveId` retains original item/glyph indices rather than output-vector indices.
The nested run and chain storage are immutable behind accessors. An adversarial
fixture culls an initial rule, keeps a glyph and later rule, reorders the retained
primitives, then applies a fractional clip that removes the later rule. Every
retained chain still points to its original virtual-font command.

`cubic::CffConsumer` is a separate opt-in CFF1 consumer. It verifies an explicitly
supplied raw table digest and encapsulates parsed dictionaries immutably. The
font loader applies exact FontMatrix into font/text space; this adapter then
scales once and flips the baseline into exact page coordinates. Cubic controls
remain cubic. Callers supply `HintPolicy` explicitly; `Unhinted` retains validated
hint metadata while `hinting_applied` remains false. CID/CFF2 and other unsupported
loader operations remain explicit failures. Original GID zero is rejected.

```sh
cargo run --manifest-path crates/rendering-core/Cargo.toml --example cff_probe -- /path/to/raw-table.cff
```

An offline installed STIXTwoText-Regular probe retained 55,177 exact commands across
2,220 non-.notdef glyphs, with zero rejected placements and one skipped .notdef.
CFF table SHA256: `c5d11bab6a95e75a568e1b72fd30fdd5e4c95abe68a72f02c0c4329ee948b532`;
containing installed OTF SHA256: `c4864ca6ec071c2d31d0d8309001faa1ee3517fffb53a31a405a697b71f52ca1`.
The probe used rational size 10,485,761/3 ticks and origin (1/2, 7/4). This verifies
decoding/placement only. OpenType selection, license binding, shaping, rasterization
and native/PDF acceptance remain upstream or downstream gates. No font file is
copied into the repository and no display-list wire primitive is activated.

`mixed::MixedBatch` combines validated upstream traced primitives with explicitly
selected immutable CFF replacements. It preserves quadratic/cubic/rule distinctions,
paint order, stable primitive identity and attached source chains. Each primitive
retains the exact intersection of page, caller and source clips. Repeated identities,
stale project/revision/page context and attempted rule-to-glyph replacement fail.
CFF glyph selection is explicit; this API does not perform shaping or infer that
a replacement glyph represents the upstream logical text.

Construction enforces total command count and a strict serialized-byte cap through
a bounded writer, returning no partial batch on failure. `fixture_bytes()` uses
`flashtex-internal-mixed-v1`, a separate opt-in consumer fixture format. Rational
coordinates use decimal numerator/denominator strings so JSON consumers cannot
round large integers. These limits bound encoded payload, not total process RSS or
font-loader transient allocations. Legacy rendering wire and hinting flags remain
unchanged. Tests combine explicit synthetic quadratic geometry with parsed original
CFF fixture curves, plus exact byte-boundary, command-budget and culling checks.

`mixed_replay::ReplayBatch` validates the internal fixture and converts its paths
back to typed exact consumer geometry. It rejects duplicate keys recursively,
unknown fields/primitives, mismatched command kinds, noncanonical rationals,
invalid clips and altered total command counts. Full metadata remains available
for lossless canonical replay. The replay does not verify referenced font/source
bytes, and never marks a fixture paintable.

```sh
cargo run --manifest-path crates/rendering-core/Cargo.toml --example replay_mixed -- \
  crates/rendering-core/tests/fixtures/synthetic-mixed.json
```

The checked-in illustrative fixture has three primitives and five commands;
canonical output is 1,932 bytes with SHA256
`aaa78b395c8740a53bb4c363076b0b4263c159287097f943c61591def2f8e7ff`.
Tests also roundtrip numerators beyond 2^100 through typed geometry without float
conversion, retaining exact source metadata and primitive identity.

`CachedCffConsumer` shares the loader's immutable full-font cache behind a mutex.
Its identity retains containing font SHA256, CFF table SHA256, face and validated
range; the existing OpenType reader remains responsible for selecting that range.
Direct and cached providers share exact placement code. Cache status explicitly
distinguishes stored, hit and oversized bypass outcomes. Mixed batches retain the
optional full-font identity and reject provider results that change GID, hint policy
or claim applied hinting. Replay validates the optional identity while accepting
earlier fixtures without it. The merged loader also explicitly rejects stroked CFF
PaintType, which this fill-path consumer cannot implement.

`residency::MixedResidency` keeps immutable prepared pages under explicit total
page, encoded-byte and command budgets. `begin` verifies and captures source
snapshots plus resource/configuration identities in a generation-bound lease;
compile from `lease.snapshots()` and prepare through that same lease. Any changed
source, resource or configuration invalidates prior jobs, even at the same project
revision. Revision rollback is rejected. Preparation checks source UTF-8 boundaries
and requires every retained font identity in the declared resource set.

Installation rejects superseded work, conflicting outputs for one identity and
oversized pages without replacing a good frame. LRU eviction removes residency
while existing caller-held Arcs remain immutable. Encoded-byte and command limits
do not claim to bound all allocator overhead, captured source snapshots or external
Arcs. Source snapshots have their own 32 MiB input cap. A caller must capture its
lease before compilation; a fresh lease cannot prove old output used new sources.

`cff_run::CffRun` consumes the font loader's `BoundCffTfmFont` through the matching
immutable full-font cache. It retains the original 8-bit code, resolved glyph name
and original GID, TFM/encoding/full-font identities and input intervals. TFM widths
and kerns alone advance the pen; the exact transformed charstring advance remains
a separate field. Explicit scale/hint policies carry through rational cubic
placement. Missing mappings, .notdef, wrong cache identity and total glyph/command
overflow fail without returning a partial run. This is explicit encoding, not
Unicode shaping or TeX scaled-point rounding.

`CffRun::fixture_bytes` serializes exact TFM metrics, kerns, input intervals and
full cubic outline evidence through the same bounded writer as mixed batches.
The pinned STIX named-glyph harness checks the installed font and OFL license
hashes, the previously validated CFF range, explicit `A` name -> original GID 3,
and an original synthetic 10-point TFM with half-em advances.

```sh
cargo run --manifest-path crates/rendering-core/Cargo.toml --example cff_tfm_probe -- \
  /usr/share/fonts/stix-fonts/STIXTwoText-Regular.otf /usr/share/licenses/stix-fonts/OFL.txt
```

Two encoded A slots produce 48 exact commands and 6,346 evidence bytes, identical
for cold and warm runs. SHA256:
`f0210698b7d382171727f4768b3fb437a2fb6f2a63ccb492df603dae096e42e1`.
`tests/fixtures/stix-cff-tfm.json` pins the font/license/TFM and records reference
gaps. No matched distribution TFM/encoding pair or TeX rounding/native/PDF oracle
is claimed. The synthetic name-mapping test also compares direct and cached
resolved encodings, preserving independent TFM and outline advances.

`geometry_diff` compares two validated mixed fixtures or two display-list-v2
envelopes with an explicit capability offer. Reports retain raw input/offer hashes,
page and stable primitive identity, original values and exact right-minus-left
rational deltas where representable. Categories distinguish resources/GIDs,
advances/positions, baselines/rules, provenance and membership/order changes. It
does not align pages, normalize geometry, substitute fonts or apply tolerances.
Mixed and display formats are not assumed directly equivalent.

```sh
cargo run --manifest-path crates/rendering-core/Cargo.toml --example geometry_diff -- \
  left.json right.json [--offer capabilities.json]
```

CLI exit codes: 0 for complete equality, 1 for complete differences, 2 for an
incomplete/unsupported comparison. API limits bound traversal, difference count
and serialized report bytes. `equal` is null whenever truncated or unsupported,
including exact delta arithmetic that exceeds its i128/u128 representation budget.
Source/font bytes are not verified by comparison, and equality is not a visual
or reference-engine parity claim. Identical illustrative mixed inputs visit 141
comparison nodes and produce an empty complete difference report.

The Type2 arithmetic dependency checkpoint (`2f770fd`) preserves the pinned STIX
run SHA above. A synthetic add-operated curve matches literal-coordinate output
exactly through direct and cached placement, while retaining distinct input hashes.
Non-dyadic Type2 division stays an explicit unsupported result. This exercises the
new arithmetic without changing default hint, wire or device-grid policies.

`device_grid::DevicePathCache` is a separate opt-in cache for composites requiring
an explicit device grid. Keys retain font SHA/face/GID, ppem X/Y, tie rule, declared
outline-policy/build hash and transform order (declared offset transform before
grid rounding; child assembly before parent transform). Missing context fails;
changing context clears device entries. The size-independent unhinted cache and
its unsupported outcomes remain unchanged. Retained byte charges exclude map
overhead and external Arcs; entry and payload caps stay explicit.

Device paths serialize only to `flashtex-internal-device-v1` comparison fixtures.
The geometry-diff tool validates this opt-in format and reports device policy changes
as resource-context differences; it does not infer equivalence to mixed/display
formats. Fractional placement remains exact, and `hinting_applied` remains false.

```sh
cargo run --manifest-path crates/rendering-core/Cargo.toml --example device_grid_probe -- \
  /usr/share/fonts/liberation-sans-fonts/LiberationSans-Regular.ttf \
  /usr/share/licenses/liberation-sans-fonts/LICENSE
```

Pinned 16-ppem/AwayFromZero replay: 2,619 non-.notdef device glyphs, 63,782 commands,
2,619 direct/cache matches and warm hits. Default expansion remains 1,678 accepted
and 941 explicitly unsupported non-.notdef glyphs. Geometry SHA256:
`a33a8836d80b2bd9fa89ba017c60df0ef175d563d930b5620e0b7e75f9dfa3b5`.
The fixture records font/license/policy pins. Device policy is not TrueType
instruction execution or a hinted raster/native/PDF parity claim.

`shaped_run::PlacedShapedRun::prepare` consumes the original font engine's
`BoundShapedRun` and a current source snapshot into exact internal quadratic or
cubic paths. It retains the complete immutable shaping record, original GIDs,
engine face identity and distinct full-font SHA, absolute UTF-8 cluster ranges,
empty clusters, ligature counts and feature notes. Integer advances already
include shaping kerning; exact size/UPEM scales them separately from outline
advances. Glyph offsets flip the font y-axis once. The caller supplies the exact
clip and an item identity; no hit positions are invented inside a cluster.

The source digest/revision and outline resource identity must agree before
expansion. Glyph/command/charged-payload limits fail atomically without returning
a partial run. Charged payload includes hint records; it is not allocator RSS or
shared cache residency. TrueType uses the unchanged unhinted cache; device-grid
paths require the separate explicit device API. CFF requires an immutable cache
bound to the full font and an explicit hint policy. No wire/native activation.

`cargo run --offline --manifest-path crates/rendering-core/Cargo.toml --example
shaped_run_probe -- /path/STIXTwoText-Regular.otf /path/OFL.txt` checks existing
font/license pins and exact warm-cache placement for a Unicode source slice.
The observed pinned run has 11 glyphs/clusters, 165 commands and advance
9762243491/600 canonical ticks. This demonstrates consumer consistency, not
reference shaping completeness, TeX metrics, hinting or native visual parity.
