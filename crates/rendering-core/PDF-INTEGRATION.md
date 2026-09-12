# Exact PDF operator handoff

The original PDF backend at `4bd8c2e79f66c161b7fb6438f3262f8d980eb8c9`
(owner branch `agent/mac-pdf/pdf-output`) exposes text and rule items, with f64
positions. It has no public path or original-GID input. `writer::num` at line822
formats operands to three decimal places. Its internal object/xref writer remains
private. Rendering-core therefore does not substitute Unicode text or send exact
path coordinates through this API. Integration is tracked in
[issue25](https://github.com/flash-tex/flashtex/issues/25).

`pdf_stream::PdfCommandStream` is an opt-in operator handoff for mixed or shaped
fixtures. It retains original fixture bytes/hash, stable primitive operator spans,
original GIDs/font/source identities and all exact rational operands in an evidence
sidecar. The source bytes are validated structurally; this does not prove the
truth of externally supplied font outlines. The stream is not a complete PDF and
has no embedded fonts, searchable text, ToUnicode or ActualText mapping.

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

## Required PDF-owner API

Add a bounded typed page/content operator input to the existing container writer,
with decimal operands preserved verbatim after validation (or an equivalent exact
number type). Preserve page dimensions, per-primitive clip/fill/color and operator
order. The existing writer should own all objects, streams, xref, trailer and
resource dictionaries. No second PDF container implementation was introduced.

For text accessibility/source fidelity, define how original glyph/source metadata
is carried into marked content/ActualText or a maintained sidecar. Any embedding
path must bind the same immutable full-font/table/face identity. Add alpha resource
support explicitly; do not silently drop it. An agreed approximation budget would
be necessary to support arbitrary nonterminating rationals; none is assumed here.

Acceptance: consume the exact stream through the original writer; independently
validate produced PDF xref/stream structure, exact operand sequences and page
geometry; then run the native visual comparison. Until that API exists, this
checkpoint claims operator-stream structural/geometry checks only.

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
explicit integral-em placement. Observed247 operators/3649 content bytes, SHA
`b5c42ae77008a3823076688513d636925c8c86dfdc5223f00ad481083b87bb42`.
Source fixture hash on this dependency revision:
`d19dd3064ec6d8bf91b590e344017384f9d6e38b3040221ebae361fd0449bcc7`.
The fixture includes the shaping build key, so dependency changes can alter that
hash independently of geometry. No standalone PDF or native paint parity claimed.
