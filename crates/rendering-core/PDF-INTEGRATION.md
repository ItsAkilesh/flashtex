# Exact PDF operator handoff

The PDF owner published the additive exact export API at
`d25a647e065dd57e66bd6e70e0366ff76d3da803` on
`agent/mac-pdf/exact-export`. Rendering-core now consumes that implementation
unchanged: `pdf_export::export` supplies exact page dimensions and validated
verbatim operator content to `exact::render_exact`. The original PDF crate remains
the sole container/xref writer. The legacy runtime text formatter is bypassed by
this explicit API. Coordination remains in
[issue25](https://github.com/flash-tex/flashtex/issues/25).

`pdf_stream::PdfCommandStream` is an opt-in operator handoff for mixed or shaped
fixtures. It retains original fixture bytes/hash, stable primitive operator spans,
original GIDs/font/source identities and all exact rational operands in an evidence
sidecar. The source bytes are validated structurally; this does not prove the
truth of externally supplied font outlines. The standalone operator stream is not a complete PDF; the new export adapter
wraps it in a real PDF. This outline route has no embedded fonts, searchable text,
ToUnicode or ActualText mapping. Those identities remain in the export evidence.

The adapter converts canonical top-left ticks to bottom-left PDF points exactly.
It elevates quadratic paths to cubic paths algebraically, preserves existing cubic
paths and rectangles, and emits nonzero clipping/filling with isolated graphics
state. Paper starts white; no preview-theme input exists. Explicit document
background primitives may paint over it. Alpha other than1 is unsupported pending
an exact graphics-state resource API.

`content_bytes` emits finite decimal operands without exponent notation or
rounding. Nonterminating rationals (including some exact quadratic-to-cubic
control points) and configured precision excess return explicit errors. A rational
operator/evidence stream can exist when decimal PDF emission is unsupported.
Byte/operator/digit limits are independent; byte limits bound serialized artifacts,
not total allocator RSS. Float64 RGB input is converted to its exact binary value;
unsupported subnormal/precision cases are rejected.

These operator semantics follow the [Adobe PDF Reference1.7, sections3.2.2 and4.4](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf).
PDF curves use cubic control points; the adapter does not flatten curves.

## Remaining consumer capabilities

The required exact page/content API now exists and is connected. The original
writer owns objects, streams, xref, trailer and resource dictionaries; no second
container implementation was introduced. Real output passes structure/content
readback checks and local Poppler raster smoke.

Searchable text/accessibility still needs an explicit original-glyph/font binding
adapter for the backend's CID font route or marked-content/ActualText contract.
The outline exporter maintains its provenance sidecar today. Alpha resource
support remains unsupported by this bounded operator set. Supporting arbitrary
nonterminating rationals would require an explicitly agreed approximation policy;
none is assumed. Native visual comparison remains a separate acceptance gate.

## Reproduction

`cargo run --offline --manifest-path crates/rendering-core/Cargo.toml --example
pdf_stream -- crates/rendering-core/tests/fixtures/synthetic-pdf-stream.json
/tmp/flashtex.content /tmp/flashtex.evidence.json`

The original synthetic input hash is
`abfb227dafe89962540936529bdd8244a6b6201160f7ed491529fc924797f1a5`.
It produces32 operators and694 content bytes. The test independently checks
operator arities, graphics-state balance, clips, white paper and exact cubic
control points. The older fractional fixture intentionally refuses decimal output.

The existing `shaped_run_probe` also checks pinned licensed STIX outlines at an
explicit integral-em placement. Observed245 operators/3645 content bytes after empty-glyph fill correction, SHA
`e67b45a267111a19dce157d3492b62ddc54ef357e1916b94e351ce2b8abebac3`.
Source fixture hash on this dependency revision:
`b5bc2c2d6134280eab832e8f9c34006db8afa4b44efcfe5814808fe0ba2cf782`.
The fixture includes the shaping build key, so dependency changes can alter that
hash independently of geometry. Standalone PDF export is described below; native paint parity is not claimed.


## Real PDF export

Run `cargo run --offline --manifest-path crates/rendering-core/Cargo.toml
--example pdf_export -- INPUT.json OUTPUT.pdf OUTPUT.evidence.json` (one shell
line). Optional width/height tick arguments select shaped replay input. Export
limits bound page count, aggregate content, final PDF bytes and evidence bytes;
no partial result is returned on budget or unsupported-number failure. Container
allocation is transient and final PDF size is checked before returning it.

The adapter verifies xref structure, page sizes and decoded content bytes after
serialization. Each evidence page retains original operators, primitive spans,
source fixture and font/GID/source identities; the top-level evidence binds the
actual PDF hash. External supplied outlines are not magically font-verified by
this exporter. Paper remains white and preview theme is never an input.

The real STIX probe found and fixed an empty-glyph fill bug: a space retains its
source/operator span but emits no `f` when it has no path. The synthetic PDF
fixture is1297 bytes, SHA
`83f7cc50f4e988acc01ec981eba31896a3e8b0866d8543caf3a33f5c44be4f6a`.
The pinned STIX Text probe resolves original fonts/shaping to11 glyphs and emits
245 PDF operators,3645 content bytes and a4219-byte actual PDF, SHA
`d9df3bf55c2b2dcb126717b9a479965b7a7fd71f39733ff1c52a10d4cd019714`.
Its content SHA is
`e67b45a267111a19dce157d3492b62ddc54ef357e1916b94e351ce2b8abebac3`.
The PDF/evidence fixtures support deterministic geometry replay. The original
font SHA/license pins are verified by `shaped_run_probe` before rendering;
pass an optional third argument (existing output directory) to save PDF/evidence.

Independent local Poppler `pdfinfo` and `pdftoppm` accept the actual STIX output.
That is parser/raster smoke evidence, not a reference-layout or visual-parity
comparison. Nonterminating exact rationals, alpha and backend-unsupported
operators remain explicit errors. Native integration is still opt-in.
