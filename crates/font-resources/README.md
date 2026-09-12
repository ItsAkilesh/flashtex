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
