# mac-vector-graphics handoff

Agent / task / branch: mac-vector-graphics (Claude Code subagent, parent
mac-claude-a, machine mac-m1max-a) / issue #2 vector graphics primitives (no FT
number, no assignment file — no ack possible) /
`agent/mac-vector-graphics/primitives`
State: ready for integration
Owned paths: `crates/vector-graphics/**`, `coordination/mac-vector-graphics.md`,
`coordination/agents/mac-vector-graphics.json`
Main integrated through: 254193c175534e78bfd9e5206b4850b86123cbe6
Ready behavior:
- `crates/vector-graphics` (edition 2024, zero dependencies): `Point`/`Rect`/
  `Transform` (translate/scale/rotate/skew, compose, invert), `Path`
  (move/line/quad/cubic/close, exact bounds, tolerance-bound flattening,
  nonzero/even-odd containment, stroke params width/cap/join/dash/miter),
  `Clip`/`ClipStack`, `Color` gray/RGB/CMYK + straight-alpha `Paint`.
- Primitives `Rule`, `PathFill`, `PathStroke`, `Image` (content hash + pt size
  + transform; no decoding), `Group` (transform + clip + opacity), all with
  stable `ItemId` and optional runtime-v1-style `SourceRange`.
- `DisplayList { page_size, items }`: `validate`, deterministic `flatten` to
  device space (transforms/clips applied, groups resolved, opacity folded,
  ancestors + inherited source), `bounds`, `hit`/`hit_with_tolerance`/
  `source_at` (topmost first, clips honoured, stroke tolerance).
- Serializers: PDF content-stream fragment (`q…Q`, `cm`, `re f`, `m l c h f f*
  S`, `re W n`/`W n`/`W* n`, `gs` with returned ExtGState names, `Do` with
  returned image names; y-flip via F·T·F⁻¹, numbers match crates/pdf `num()`),
  hand-written JSON writer/reader for the tree (exact round trip) and a
  write-only device-list JSON for a preview consumer.
- Diagram helpers: arrow, polyline, circle/ellipse (4 cubics), text anchor
  box, grid.
- README (conventions, unsupported list, integration note) and
  `docs/rendering-v2-integration.md` (field mapping to the Commander's v2
  draft; proposal only).
Incomplete behavior: none of the crate is wired into any consumer (by design
until rendering-v2 ABI is agreed). Not supported: gradients/patterns/blend
modes/soft masks, text as paths, stroke outlining, arcs, transparency groups
(group opacity is per-leaf), image decoding, colour management, non-affine
transforms. Device-list stroke width under anisotropic scale is approximate
(PDF path is exact).
Interface changes and required consumer actions: none. No runtime-v1 change;
no v2 schema change made — v2 extensions are proposed in the doc for the
Commander's decision.
Validation: `cd crates/vector-graphics && cargo test` -> 14 unit + 26
integration tests pass; `cargo clippy --all-targets` 0 warnings; `cargo fmt
--check` clean (rustc 1.99 nightly on mac-m1max-a).
Needs from others: Commander review/integration of the branch; an FT number/
assignment if tracking is wanted; ABI decisions listed in
`crates/vector-graphics/docs/rendering-v2-integration.md` §5.
Next action: none pending; adjust the v2 mapping when the ABI is decided.
Peer revisions reviewed and adaptations:
- origin/main b873340 then 254193c (both merged, no conflicts). 254193c adds
  `docs/contracts/rendering-v2-proposal.md`, `protocol/rendering-v2.schema.json`
  and `crates/rendering-core` (Tick(i64) bp_2pow20, from_tex_sp ties-to-even,
  hit_test::PageIndex). Adaptation: rewrote §4 of the integration doc so an
  unselected feature is an explicit render failure (error diagnostics), not
  a placeholder/approximation, as the contract requires; corrected the
  tick-rounding note (TeX sp needs the 7200/7227 factor via rendering-core,
  this crate is PDF points only); referenced PageIndex as the index to
  extend rather than a competing hit-tester.
- origin/agent/mac-pdf/pdf-output 5b5f7b5 `crates/pdf/src/writer.rs`: rules
  `x y w h re f` in bottom-left space, `num()` 3 decimals. Adopted the same
  formatting and flip; golden test `pdf_rule_golden_matches_pdf_crate_convention`.
- origin/agent/mac-claude-a/mac-shell PreviewView/PDFExport: top-left pt
  drawing, flip only at PDF export. Matches the crate's convention.
- origin/agent/commander-render-schema/rendering-v2-schema 41cacfc: v2 draft
  has `rule` with integer `bp_2pow20` ticks, RGBA sRGB paint, `sources` or
  `synthetic_reason`. Mapped in the proposal doc; no new v1 kinds proposed.
Resource: allocation claude-mac20x-vector-graphics (parent mac-claude-a's
Max 20x grant); quota unknown to this subagent.
Updated: 2026-09-12T06:18Z
