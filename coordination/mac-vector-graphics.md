# mac-vector-graphics handoff

Agent / task / branch: mac-vector-graphics (Claude Code subagent, parent
mac-claude-a, machine mac-m1max-a) / issue #2 vector graphics primitives (no FT
number, no assignment file yet — no ack possible) /
`agent/mac-vector-graphics/primitives`
State: in progress
Owned paths: `crates/vector-graphics/**`, `coordination/mac-vector-graphics.md`,
`coordination/agents/mac-vector-graphics.json`
Main integrated through: 1dd26c5e0dd5e04a39f0b8e55c90abebf635c863
Ready behavior: none yet
Incomplete behavior: everything (crate skeleton being written)
Interface changes and required consumer actions: none. The crate is an isolated
display-primitive model; runtime-v1 item kinds are untouched and no new v1 kinds
are proposed (rendering-v2 is under Commander review).
Validation: none yet
Needs from others: assignment file / FT number from Commander for ack.
Next action: geometry + primitives + display list + serializers + tests.
Peer revisions reviewed and adaptations:
- origin/main 1dd26c5: base.
- origin/agent/mac-pdf/pdf-output 5b5f7b5 `crates/pdf/src/writer.rs`: rules are
  `x y w h re f` in bottom-left PDF space (`y = height - baseline`), numbers via
  `num()` (3 decimals, trimmed, `-0` -> `0`). The PDF fragment serializer here
  follows the same number formatting and performs the same y-flip.
- origin/agent/mac-claude-a/mac-shell PreviewView/PDFExport: preview draws in
  top-left pt space scaled; PDF export flips `y = height - y - h`. Matches the
  crate's coordinate convention (top-left origin; flip only in PDF serializer).
Updated: 2026-09-12T05:36Z
