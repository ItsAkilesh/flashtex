# Preview scroll anchoring (gap 5) — measured 2026-09-12

Lane mac-preview-anchoring (Claude Code subagent of mac-claude-a, mac-m1max-a,
Xcode 26.3 / Swift 6.2.4). Branch `agent/mac-preview-anchoring/anchoring`;
v1 hook measured on `agent/mac-preview-anchoring/anchoring-applied` (5d1503e7).
All figures under shared load (1-min load 8–25 during 12 concurrent lanes;
`uptime` recorded beside each run) — not an isolated result. Positions are
document points, y down, at the viewport's top edge.

Harness: `apps/mac/Tests/FlashTeXMacTests/PreviewAnchoringTests.swift` hosts the
views in an `NSHostingView` inside an `NSWindow` at (-10000, -10000) that is
ordered but never made key (no focus change), finds the backing `NSScrollView`,
scrolls programmatically, swaps the root view / resizes the window, and reads
`documentVisibleRect`. Scenario: three Letter pages, pane 500×400 pt
(fit scale 0.7386), reader at page 2 @ 0.3 of the page = top 808.4 pt.

| case | view | before → after (pt) | expected (pt) | result |
|---|---|---|---|---|
| (c) pane 500→350 pt (scale 0.7386→0.4935) | column mirror (keeper) | 808.4 → 556.1 (Δ −252.4) | 556.1 | held |
| (c) pane 500→350 pt | `PreviewV2View` (keeper committed) | 808.4 → 556.1 (Δ −252.4) | 556.1 | held |
| (c) pane 500→350 pt | `PreviewView` WITHOUT hook (lane tree fabdfb83) | 808.4 → 808.5 | 556.1 | drift 252.4 (the bug) |
| (c) pane 500→350 pt | `PreviewView` WITH hook (applied 5d1503e7) | 808.4 → 556.1 | 556.1 | drift 0.0 |
| (a) 3→6 pages appended after the anchor | column / v2 | 556.1 → 556.1 | 556.1 | no correction issued |
| (a) real compiler 4→7 pages (60→120 paragraphs) | `PreviewView` with and without hook | 808.4 → 808.4 | 808.4 | drift 0.0 either way |
| (a) 6→2 pages, page 2 still present but shorter document | column | 556.1 → 470.6 | min(556.1, end 470.5) | clamped to end, not top |
| (a) 2→1 pages, anchored page vanished | column / v2 | → 55.9 | end 56.0 | clamped to end, not top |
| (a) real compiler 7→1 pages (3 paragraphs) | `PreviewView` | → 56.0/56.2 | end 56.0 | clamped to end |
| (b) stale revision 2 after revision 3 (`handleForTesting(.result(stale))`) | `PreviewView` + `ShellModel` | 808.4 → 808.4 | 808.4 | model drops it; no scroll |
| (b) identical-geometry v2 frame (new nonce) | `PreviewV2View` | 808.4 → 808.4 | 808.4 | no correction issued |
| horizontal, page 4000 pt wide at 20 % floor, x = 200 | column | (200, y) → (200, y) after 2→3 pages | same | held |

Mechanism finding: SwiftUI's macOS `ScrollView` re-imposes its own stored
content offset once during the layout pass that follows a content-size
change, so a single programmatic scroll is reverted (trace in the test:
`corrected 808.4→556.1`, then `bounds top=556.0` after the document resized,
then re-applied). `PreviewAnchorProbe` therefore enforces the anchor on every
clip-bounds / document-frame change inside a 150 ms settle window after a
layout change, then resumes capturing. A user scroll inside that window is
overridden (bounded by the window length).

Test counts: `swift test --filter PreviewAnchoringTests`: lane tree 9 tests,
8 pass + 1 XCTSkip without `FLASHTEX_COMPILER`; with the compiler
(`crates/compiler/target/release/flashtex-compiler`, main checkout build)
9/9 pass on both trees. Full `swift test` NOT run (parent's heavy-build
window: only `swift build` + own file allowed).
