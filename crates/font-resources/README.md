# Deterministic font resources

Original Rust resource loader for rendering-v2 schema revision `41cacfc`. Its
`FontDescriptor` has exactly the eight wire fields defined there. Paths and
license records belong to a separate resource manifest (`schema_version: 1`),
not to the display-list descriptor. No platform font discovery or fallback occurs.

`FontCollection::load(root, &manifest)` reads explicitly named relative resources,
checks bounded sizes, SHA256 for font and license bytes, declared metrics and
PostScript name, then retains immutable shared bytes. Duplicate IDs, traversal,
escaping symlinks, unsupported formats, missing files, digest mismatches and
metadata mismatches are errors. IDs iterate lexically. Existing resources remain
unchanged if the underlying files or caller buffers later change.

`FontResource::from_bytes` performs the same digest/metadata checks for already
provided bytes. `inspect_static_truetype(&[u8]) -> Result<FontMetadata>` exposes
structural metadata inspection for rendering-core's FontValidator adapter; that
inspection alone does not establish expected identity or license provenance.
`bytes()`, `shared_bytes()` and `table(tag)` expose immutable data for consumers.

The current profile accepts single-face static TrueType sfnt version 1 only,
face index zero. TTC, variable fonts, CFF and WOFF are explicitly unsupported.
Checks cover directory bounds/alignment/overlap, required table presence, head
magic and units, maxp glyph counts, loca offsets and glyph header extents, hmtx
bounds and name record bounds/UTF16 decoding. Raw OS/2 embedding flags are exposed.
This is not complete glyph-program, composite-outline or cmap validation; name
format-1 language-tag metadata is not interpreted. Downstream shaping/rasterizers
must retain their own validation. No shaping or font metrics substitution occurs.

License metadata must identify source, copyright, license identifier, license
text path/hash and operator-declared embedding permission. The loader preserves
this provenance and actual text; it does not adjudicate legal rights, approve
embedding, or override OS/2 restrictions. Export consumers must enforce their
embedding policy before using bytes.

Run `cargo test --offline --manifest-path crates/font-resources/Cargo.toml`.
Tests generate structural fonts without claiming typographic fidelity. The
explicit-path `inspect` example is a smoke tool, not a fallback resolver. On this
Linux host, LiberationSans-Regular.ttf SHA256
`76d04c18ea243f426b7de1f3ad208e927008f961dc5945e5aad352d0dfde8ee8`
(410712 bytes) was accepted with 2048 units/em, 2620 glyphs and name
`LiberationSans`; no installed font is vendored or implicitly selected.

Integration handoff: rendering-core `fe29901` owns display-list cross references
and glyph-ID bounds; adapt its FontValidator using `inspect_static_truetype` and
return units_per_em/glyph_count. No dependency on rendering-core exists here.
Visual identity, PDF byte identity and typing-to-visible latency remain separate,
unmeasured integration gates. No reference LaTeX engine is used in this crate.

Unicode access: `glyph_id(char)` returns the original GID or `None` for .notdef.
It deterministically prefers Unicode format 12 over format 4, then first record
in directory order; unsupported-only cmaps return an explicit error. It checks
all selected subtable groups/segments and resulting GIDs before answering. The validated representation is initialized once per immutable resource and shared
by clones, including cached validation failures. Lookup uses binary search; no
process-global cache or filename-based identity exists. cmap_cache_key() exposes
the verified SHA256/face tuple. A maximum of 65536 ranges retains at most 768KiB
of mapping payload per resource (plus small allocation metadata).
`horizontal_metrics(gid)` returns original unsigned advance and signed bearing,
including the repeated final advance for trailing hmtx bearings.

Composite validation runs during metadata inspection and resource loading. It
bounds component record arguments/transforms/instruction extents, rejects absent
child GIDs and graph cycles, and caps dependency depth at 32 edges and aggregate
component references at 1,000,000. Shared acyclic subgraphs are permitted; cached
subtree heights cannot hide an over-depth ancestor chain. These are explicit
experimental resource limits, not a claim to accept every valid TrueType font.
Simple-outline point data and glyph instructions are still not interpreted.

Cache evidence: 23 normal tests pass; an explicitly run release benchmark on this
Linux host measured 100000 synthetic format-12 lookups at 9.672ms rebuilding
validation versus 0.358ms cached. This is one tiny synthetic font and excludes
initialization, shaping, rendering and UI latency. Reproduce with
`cargo test --offline --release --manifest-path crates/font-resources/Cargo.toml repeated_lookup_benchmark -- --ignored --nocapture`.

`simple_outline(gid)` decodes closed simple-glyph contours into original points
and inclusive contour endpoints. Coordinates are signed 16.16 fixed-point design
units; no scaling or rounding is performed. On/off-curve flags, raw bounding box,
overlap flag and unexecuted instruction bytes are preserved. Limits come from
signed contour counts and 16-bit point endpoints (at most 65536 points), with
checked flag repetitions, instruction/coordinate extents and signed coordinate
accumulation. Missing bytes or invalid endpoints return errors. Composite glyphs
explicitly return unsupported; implied quadratic midpoint expansion, hinting,
composite transforms, shaping and rasterization are not implemented here.

Real-font smoke decoded all 1544 simple/empty glyphs of the LiberationSans digest
above, explicitly skipping 1076 composites. This proves decoder acceptance, not
visual/byte parity. Reproduce by setting FLASHTEX_SMOKE_FONT to that explicit font
path and running the ignored installed_simple_glyph_smoke test with --nocapture.

`expanded_outline(gid)` recursively expands supported composite affine transforms
and explicit XY translations, retaining root font SHA/face/ID and each original
component GID plus point range. Coordinates are normalized exact dyadic rationals
(numerator / 2^shift, accessed through methods), checked within i128 and at most
96 fractional bits. No floating rounding occurs. Expansion caps 4096 visited
instances, 1000000 leaf points and 32 dependency edges. Nonzero grid-rounded offsets and transformed nonzero offsets without an explicit
scaled/unscaled policy return unsupported. Instructions remain unexecuted.

Composite smoke on the pinned LiberationSans font accepted 1679 glyphs and
explicitly rejected 941 requesting nonzero grid-rounded offsets. These are
unsupported pending a hinting policy, not substituted or counted as complete.
`ExpandedOutline::quadratic_path()` yields deterministic MoveTo/LineTo/QuadTo/Close
commands, inserts exact implied midpoints between consecutive off-curve points,
and handles contours whose first/last points are off-curve. Original root/font
identity remains on the owning ExpandedOutline. The iterator validates contour
coverage and materializes a bounded command stream; it does not execute hints,
choose fill/rasterization rules or assert output parity.

Point-attachment placement now supports existing parent/child contour-point
indices, decoded as unsigned byte/word values. The child affine transform is
applied before computing the exact translation that makes the points coincide.
First-component attachment, missing indices and phantom-point references fail;
this API does not synthesize phantom points or execute child/parent instructions.
The semantics are explicitly unhinted design-space geometry, not the rasterizer's
post-hint placement. Source: [OpenType glyf specification](https://learn.microsoft.com/en-us/typography/opentype/spec/glyf).
Acceptance tests cover transformed attachment, byte/word indices, malformed
indices, exact scaled versus unscaled offsets and refusal to guess grid rounding.
No ppem, device grid or hint state is represented here, so grid-rounded nonzero
offsets remain unsupported. Real-font smoke remains 1679 accepted / 941 unsupported.

## Exact TeX font metrics

`tfm::Tfm::parse` is an original bounded parser for the documented TFM format.
It checks exact file/table lengths, dimensions, character indices, next-larger
cycles, extensible pieces and ligature/kern program targets and actions. Checksum
is retained as an external font identity value, not recomputed from TFM contents;
source_sha256 binds the actual bytes. `FixWord(i32)` preserves signed 12.20 values;
`at_design_size` returns an exact numerator over 2^40 in TeX points, without
pretending that this implements TeX's separate scaled-point rounding algorithm.
`char_metrics`, one-based `parameter`, and bounded `pair_action` expose typed
values and ligature retention/advance semantics. TFM codes are 8-bit encoding
slots, not Unicode or TrueType GIDs; consumers need an explicit encoding binding.
Specification reference: [TeX Live tex.web TFM format documentation](https://github.com/TeX-Live/texlive-source/blob/trunk/texk/web2c/tex.web).
No existing TeX engine is linked or invoked. No installed TFM oracle was found;
current tests use declared synthetic format fixtures, not measured font parity.

`apply_ligatures_kerns` interprets already encoded runs with exact kern FixWords
and ligature keep-left/keep-right/advance semantics. Each output glyph retains
its contributing input index interval; inserted kerns stay separate typed items.
Input is limited to 4096 bytes, output to 8192 items and execution to 65536 steps;
cyclic ligature programs fail rather than hanging. Fonts declaring boundary
programs are explicitly unsupported by this run interpreter (pair inspection
remains available). It does not perform Unicode encoding, hyphenation,
discretionaries, TeX scaled-point rounding or TrueType glyph selection.

## Explicit TFM encoding adapter

`encoding::EncodingManifest` is a typed JSON declaration (not a PostScript `.enc`
interpreter). It binds exact TFM SHA256, font SHA256 and face index to at most 256
code/name entries and 65536 declared name/original-GID entries. Names are literal
JSON strings; JSON escapes decode normally, while PostScript/PDF escape syntax is
not guessed. Duplicate codes/names, absent declarations, invalid GIDs and hash or
face mismatches fail. Distinct encoding slots may intentionally share a glyph.
`.notdef` yields the explicit GlyphIdentity::Notdef value; other names cannot map
to GID zero. A declaration is not proof that a font's name table uses those names.

`BoundTfmFont::new(tfm,font,manifest)` retains immutable borrowed resources and a
validated map. `map_code` returns original glyph identity plus exact TFM metrics;
`map_run` applies the bounded TFM interpreter then resolves all output slots,
retaining input intervals and separate exact kerns. No Unicode casting, font
fallback, hidden `.notdef` drawing, or TrueType metric substitution occurs.
The caller must handle Notdef explicitly and establish the declared encoding's
provenance. This adapter does not establish TFM-to-outline visual equivalence.

## Virtual font packets

`vf::VirtualFont::parse` preserves VF checksum/design size, raw font names/areas,
local font definitions, short/long character packets, exact signed movement/rule
units and typed glyph/font/register/stack commands. Names never trigger disk or
network resolution. Missing font references, malformed packet lengths, duplicate
IDs/codes, prohibited opcodes and unbalanced/deep stacks fail. Bounds: 16MiB file,
4096 fonts, 65536 packets, 1MiB/100000 commands per packet, 1000000 commands total,
and 64 stack entries. Specials remain explicit typed byte payloads; parsing does
not execute or silently discard them. [VF format documentation](https://github.com/TeX-Live/texlive-source/blob/trunk/texk/web2c/vftovp.web)
was consulted; no existing VF/DVI implementation is used in production.

`VirtualFont::expand_packet(code, virtual_tfm, physical_bindings)` validates the
virtual TFM checksum/design-size/packet width and each local TFM definition against
explicit BoundTfmFont resources. It emits original GID/font+TFM hashes, local font
ID and exact dyadic placement coordinates relative to the virtual font's size.
Glyph scale is a 12.20 fraction of that size. DVI y is positive downward; rule
position is its lower-left reference point and positive height extends upward.
Set/put advances, w/x/y/z registers and push/pop positions are interpreted without
float/device rounding. Font selection is not a pushed register. Specials, missing
bindings, explicit Notdef and physical codes above the supported 8-bit encoding
fail. No special-handler execution, nested VF resolver, font discovery, ligature
reshaping inside packets or silent substitution occurs. Current packet limits
bound execution and output; actual nested resource recursion is not implemented.
Synthetic integration tests establish arithmetic/binding behavior only, not
real-VF/font visual parity.

## Explicit nested VF graph

`vf_graph::ResourceGraph` extends the single-packet API with recursive virtual
resources and explicit local-font edges. Physical keys bind font SHA/face plus
TFM SHA; virtual keys bind VF SHA plus TFM SHA. Duplicate identities and malformed
or undeclared local edges fail; no pathname lookup or recursive fallback occurs.
One explicit encoding binding is accepted per physical key (conflicting duplicate
bindings are rejected). Each output preserves the root-to-leaf resource/code and
command-index chain, original physical GID, exact dyadic position and composed
scale. Child glyph advances use the child's TFM packet width at the declared
local scale; rules scale and translate through the same exact arithmetic.

Graph bounds are 4096 resources, 4096 expanded nodes, 32 nested edges, 1000000
executed commands and 100000 output placements across the whole expansion.
Repeated (resource,character) on the active stack is a cycle; acyclic reuse is
allowed. Precision/overflow errors propagate without partial output. Specials
remain explicitly unsupported. Flat-versus-nested synthetic glyph and rule
fixtures compare exact geometry and retain intentionally distinct provenance;
cycle, missing-resource, depth, node and global-output cap tests also pass.
Real licensed VF/TFM/outline oracle agreement remains pending.

## Staged CFF1 support (not production activated)

`cff::Cff::parse` accepts raw CFF table bytes from the existing public
font-engine `TrueTypeFace::cff_table()` or PDF `TrueTypeFont::cff_table()` accessor.
Reviewed peers: font-engine `2d6954b923340c788cd31f663fa8e7a845326dec`, PDF
`52b371171ee497a52ce0529dbe6bf22cda4bfe04`. Their private numeric readers are not
re-exported; existing local bounded readers are reused. No second OpenType font
selector, shaper or sfnt directory parser is introduced.

This first stage validates bounded CFF1 INDEX/DICT syntax, one-font Name/Top DICT,
CharStrings, custom charset/encoding and Private/Local/Global Subrs offsets.
Integer and decimal DICT values remain exact (decimals are preserved strings).
CID-keyed/CFF2/Expert predefined charsets are explicitly unsupported. Semantic
validation of every optional DICT operator is not claimed. Metadata acceptance
alone does not prove charstring safety or rendering support. Production font
resource schema remains static-truetype; no fallback or wire activation occurs.
References: [CFF specification](https://adobe-type-tools.github.io/font-tech-notes/pdfs/5176.CFF.pdf)
and [Type 2 specification](https://adobe-type-tools.github.io/font-tech-notes/pdfs/5177.Type2.pdf).

`Cff::cubic_outline(gid)` adds staged exact Type2 charstring-space MoveTo, LineTo,
CurveTo(control1,control2,end) and Close commands. It handles integer/16.16
operands, width extraction, relative/alternating line and curve forms, local/global
subroutine bias/calls/returns, and explicit closure. Limits: 48 operands, 10 active
subroutine calls, 100000 instructions and 100000 commands; recursion, malformed
arity and out-of-range calls fail. Hints/masks, escaped flex/arithmetic and seac
are explicitly unsupported. FontMatrix stays in Top DICT and is not silently
applied; cubic commands are not reinterpreted as quadratic rendering-v2 paths.

Real CFF smoke used the unchanged font-engine accessor at `2d6954b` in an isolated
/tmp harness, against installed STIXTwoText-Regular.otf font SHA
`c4864ca6ec071c2d31d0d8309001faa1ee3517fffb53a31a405a697b71f52ca1`.
CFF SHA `c5d11bab6a95e75a568e1b72fd30fdd5e4c95abe68a72f02c0c4329ee948b532`:
2221 glyphs, 824 global and 704 local subroutines; 16 glyphs accepted and 2205
explicitly unsupported due to hints/masks. No invalid-font failures occurred.
This does not establish useful visible coverage, Latin Modern support or parity;
no font file was copied into the repository and no schema activation occurred.

## Explicit unhinted CFF geometry and FontMatrix

`cubic_outline_with_policy(gid, HintPolicy::Unhinted)` validates/records stem
operands and hint/counter masks without applying grid fitting. It enforces the
96-stem limit, exact ceil(stems/8) mask payload length, zero unused bits, stem
ordering and operand counts. HintMetadata records policy, raw stem deltas/widths,
mask bytes and flex depths. The default `cubic_outline` retains Reject policy.
All four flex forms emit their two exact cubic curves under Unhinted policy;
no device-dependent flattening is performed. Arithmetic/seac/CID/CFF2 remain
explicitly unsupported. This supersedes the earlier stage's blanket hint/flex
rejection only when Unhinted is deliberately selected.

`matrix_outline(gid, policy)` applies the exact six-term FontMatrix and returns
separate MatrixCommand/MatrixOutline types. Coordinates use normalized checked
i128 Rational values with positive denominators, so decimal 0.001 is exactly
1/1000, not a binary approximation. The default matrix is applied explicitly.
Translation affects points but not the advance vector. Decimal scale/mantissa
budgets and arithmetic overflow return errors. Output is CFF font/text space,
not raw charstring units or rendering-v2 page ticks. Rasterizers must make their
size/grid policy separately; no schema or production font-profile switch occurs.

Under Unhinted plus matrix application, the same pinned STIX font above now
accepts all 2221 glyphs, with zero unsupported/invalid decoder outcomes. The old
Reject-policy results remain valid and unchanged in meaning. This proves bounded
parser/geometry acceptance only, not outline equality against an oracle, hinted
raster fidelity, Latin Modern coverage or PDF parity. Synthetic tests cover exact
masks, malformed operands, all flex forms, decimal matrices and overflow.

## Pinned CFF regression and immutable cache

`fixtures/stix-cff.json` records the installed font/OFL hashes, exact CFF table
range verified through the existing font-engine accessor, and a canonical exact
geometry regression hash. No font/license bytes are vendored. Run the opt-in
`cff_regression` test in release mode with `--ignored --nocapture`; optional
FLASHTEX_STIX_FONT and FLASHTEX_STIX_LICENSE override paths, never expected hashes.
Missing resources mean this gate remains pending, not that fidelity passed.
The canonical v1 hash streams GID u16be, advance rationals (signed numerator and
positive denominator i128be), command count u32be, then command tag 0/1/2/3 and
point rationals for move/line/curve/close. It is an implementation regression
baseline, not an independent visual oracle. Concrete hardening also rejects
stroked PaintType in the filled-outline API; mask/subroutine/matrix overflow
regressions run through the matrix-applied API.

`CffOutlineCache::from_font_table(full_bytes, face0, validated_table_range, limits)`
computes the full-font and CFF hashes, reparses immutable table bytes and retains
an internal Cff value. The existing OpenType accessor remains range/selection
authority. Keys include that fixed full identity, original GID and HintPolicy.
`lookup` returns CacheOutcome { result: Result<Arc<MatrixOutline>>, status };
status is Hit, Stored or BypassedOversize, including explicit negative results.
LRU entry/byte limits are enforced, with 512 conservative bookkeeping bytes per
entry plus actual retained Vec/String capacities and object sizes. The budget
covers cache-owned references/payload, not external Arc holders or allocator-wide
accounting. Source CFF storage has its separate 64MiB input cap. Oversized entries
are returned without retention; identities and decoded state cannot mutate through
the cache API. Max configured limits are 4096 entries and 256MiB.

Measured replay evidence is in `fixtures/stix-cff-replay.json`, with tested source
hashes. All 2221 STIX glyphs produce 55206 commands; cached/direct outputs match.
One Linux release observation: direct decoding 103.106ms, cache cold 108.647ms,
full warm replay 0.251ms, 20641359 charged retained bytes within a 32MiB budget.
These measurements exclude native rendering and are not typing-visible latency
or hinted/visual/PDF parity claims.

## CFF names and exact TFM encoding bindings

`Cff::glyph_names()` validates every glyph's charset SID against the 391 standard
SID strings or the custom String INDEX, then rejects duplicate or absent names.
The standard SID registry is format data transcribed from Adobe CFF Appendix A;
no third-party name resolver implementation is imported. Glyph names are literal
bounded ASCII PostScript names (127 bytes here), not Unicode or escape-decoded
aliases. SID strings used only as other metadata are not treated as glyph names.
GID0 resolves explicitly to Notdef. The immutable name index is shared by the CFF
outline cache and has its own conservative 16MiB metadata budget, separate from
decoded-outline retention.

`BoundCffTfmFont::new(tfm, outline_cache, CffEncodingManifest)` resolves explicit
TFM slot/name entries against actual CFF charset names, checking full-font/CFF/TFM
hashes and face. It returns exact TFM metrics and original GIDs/Notdef through
map_code/map_run, retaining input intervals. `validate_cache` checks the immutable
identity before using another outline cache. No invented named-GID declaration
is needed for CFF. Stroked/hinted/unsupported outline policies remain independent.

Opt-in `cff_names` regression verifies every pinned STIX name round-trips to its
original GID. The 2221-name canonical hash and resource hashes are recorded in
`fixtures/stix-cff-names.json`; A is GID3 and the literal name fi is absent.
A missing alias fails rather than being guessed. This is charset/name evidence,
not TeX encoding correctness or visual/raster parity.

`CffEncodingCache::new(&outline_cache, CacheLimits)` pins an immutable full-font,
CFF table range and face identity. `lookup(tfm, manifest)` validates identity before
reuse and returns `EncodingCacheOutcome { encoding: Arc<ResolvedCffEncoding>,
status }`. Canonical sorted slot/name declarations share entries regardless of
input order; different TFM hashes or declarations do not. Missing names and invalid
identities are errors, not negative fallback entries. `BoundCffTfmFont::from_resolved`
consumes the shared binding; `validate_cache` checks the eventual outline consumer.
LRU retention permits at most 128 entries and 8MiB of conservatively charged entry
payload/bookkeeping, with explicit oversize bypass and zero-budget operation.
The shared name index has its separate 16MiB bound; caller-retained Arc references
and allocator-wide memory are outside cache ownership. No shaping or native render
performance claim follows from an encoding cache hit.

Type2 arithmetic follows Adobe Technical Note5177 sections4.4–4.6:
https://adobe-type-tools.github.io/font-tech-notes/pdfs/5177.Type2.pdf
The original decoder now supports exact add/subtract/multiply, absolute/negation,
logic/comparison/ifelse, dup/exch/index/roll/drop, and 32 transient slots shared only
within one glyph and its subroutines. Reads before writes fail. Division accepts
exact dyadic quotients and sqrt accepts exact dyadic roots; non-dyadic quotients,
irrational roots and random return `UnsupportedFont`, never approximate geometry.
The existing Coordinate profile remains unchanged. Checked numeric overflow,
invalid indices, zero division, operand underflow, 48-stack and 100000-instruction
budgets fail explicitly. Subroutine cycle/depth limits remain active. This is not
an emulation of device arithmetic rounding, hinting, or a raster-fidelity claim.
