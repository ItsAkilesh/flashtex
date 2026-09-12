# mac-render-pipeline handoff

Agent / task / branch: mac-render-pipeline (Claude Code subagent, parent mac-claude-a) /
unified render pipeline (`crates/render-pipeline`, CLI `flashtex-render`) /
`agent/mac-render-pipeline/unified`
State: in progress (resumed after an accidental stop; WIP b2d4708 verified green)
Owned paths: `crates/render-pipeline/**`, `docs/proposals/rendering-abi.md`,
`coordination/mac-render-pipeline.md`, `coordination/agents/mac-render-pipeline.json`
Main integrated through: see `coordination/agents/mac-render-pipeline.json`

Ready behavior:
- `cargo build --release && cargo test --release` in `crates/render-pipeline` is green
  (25 tests: unit, golden v1 spans incl. multi-byte and included documents,
  v2/v1 consistency, capability negotiation, math rules, page-break determinism, CLI e2e).
- `flashtex-render` speaks runtime-v1 on stdin/stdout, honours
  `payload.layout_capabilities` (`rules-v1`, `font-hints-v1`), echoes the accepted
  subset, emits typed `rule` items / `font` hints only when accepted; falls back to
  the U+2500 approximation otherwise. Fails closed on unknown v2 version / resource.
- Default face Latin Modern (from the local TeX Live tree at run time), Times only when
  the document selects it.

Incomplete behavior:
- `src/pagebuild.rs` (TeX §980–1028 page builder with penalty costs) is written and unit
  tested but not yet wired into `typeset::build` (still `paragraph-layout::layout_pages`,
  which enforces club/widow as hard minimums).
- README / `docs/proposals/rendering-abi.md` deviation list, sibling pin refresh
  (font-engine f418238, paragraph-layout 70209e2, math-layout db90047, pdf 4bd8c2e).
- Evidence against the committed oracle rasters (reference side cannot be regenerated:
  no pdflatex on this machine).

Interface changes and required consumer actions: none on main yet; the CLI is a drop-in
`FLASHTEX_COMPILER` for the Mac app.
Validation: `cargo test --release` (25 passed) at the checkpoint SHA.
Needs from others: see README "requested sibling API changes" once written.
Next action: wire pagebuild, refresh pins, README/ABI doc, evidence table.
Peer revisions reviewed and adaptations: main a949f5b/cab39a2 (layout capabilities)
already implemented in `src/v1.rs`; compiler 3ae7d9b `Span.document` honoured.
Updated: 2026-09-12T07:20:00Z
