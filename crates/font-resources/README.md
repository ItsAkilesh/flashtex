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
all selected subtable groups/segments and resulting GIDs before answering. Each
lookup is bounded but currently rescans the selected table (no shaping cache).
`horizontal_metrics(gid)` returns original unsigned advance and signed bearing,
including the repeated final advance for trailing hmtx bearings.

Composite validation runs during metadata inspection and resource loading. It
bounds component record arguments/transforms/instruction extents, rejects absent
child GIDs and graph cycles, and caps dependency depth at 32 edges and aggregate
component references at 1,000,000. Shared acyclic subgraphs are permitted; cached
subtree heights cannot hide an over-depth ancestor chain. These are explicit
experimental resource limits, not a claim to accept every valid TrueType font.
Simple-outline point data and glyph instructions are still not interpreted.
