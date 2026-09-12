# mac-preview-v2 handoff

Agent / task / branch: mac-preview-v2 (Claude Code subagent, parent mac-claude-a) /
experimental rendering-v2 display-list consumer for the Mac preview (glyph runs by
original GID from content-addressed fonts, rules, exact cluster hit/caret geometry,
shared preview/export draw routine) / `agent/mac-claude-a/preview-v2`
State: in progress
Owned paths: `apps/mac/Sources/FlashTeXProtocol/RenderingV2.swift`,
`apps/mac/Sources/FlashTeXMac/PreviewV2View.swift`,
`apps/mac/Sources/FlashTeXMac/GlyphRunRenderer.swift`,
`apps/mac/Tests/FlashTeXMacTests/RenderingV2Tests.swift`,
`apps/mac/Tests/FlashTeXMacTests/PreviewV2Tests.swift`, `coordination/mac-preview-v2.md`;
≤8 hook lines in `ContentView.swift`/`ShellModel.swift`, one menu item in `FlashTeXMacApp.swift`,
a README section.
Main integrated through: base f1bf50a (origin/agent/mac-claude-a/mac-shell tip)
Ready behavior: none yet
Incomplete behavior: everything listed under owned paths
Interface changes and required consumer actions: none. Consumes the pipeline's `--v2`
display list (origin/agent/mac-render-pipeline/unified 7094ef7, built from a scratch
export) validated against protocol/rendering-v2.schema.json on main; deviations are
documented in the model file and README. v1 stays the default preview.
Validation: pending
Needs from others: nothing blocking. Pipeline note: `sha256`/`font_id` on the wire is
SHA-256(font bytes ‖ face_index as u32 BE), not SHA-256(bytes) — see model notes.
Next action: model + validator, renderer, view, tests, README, screenshot.
Peer revisions reviewed and adaptations: origin/main (rendering-v2 proposal + schema,
rendering-core README) → model shapes follow the schema; pipeline 7094ef7 display.rs →
`opentype-cff`/`core14-afm` formats and the hash convention accepted as documented deviations.
Updated: 2026-09-12T12:05:00Z
