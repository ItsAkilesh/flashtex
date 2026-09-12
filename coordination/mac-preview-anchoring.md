# mac-preview-anchoring — handoff

Lane: Claude Code subagent of parent mac-claude-a on mac-m1max-a. Gap 5: preview
scroll/zoom anchoring when the page count changes, stale helper candidates arrive,
and the pane is resized (v1 `PreviewView` and v2 `PreviewV2View`).

## Coverage audit (2026-09-12, base 5bc3fc0f = origin/agent/mac-claude-a/mac-shell)

Grepped `apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/Sources/FlashTeXMac/Preview*.swift`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*` for
scroll / anchor / zoom / resize / visibleRect:

- No test reads or asserts a preview scroll offset, visible rect, or fit-to-width
  scale after a result swap, a stale result, or a resize. `anchor` hits in tests are
  the bridge `TransferV1.Anchor` / `pinAnchorAtCaret` (BridgeClientTests,
  BridgeRecoveryTests, HistoricalPreviewTests:291, InsertionTests:6) — unrelated.
- `NavigationTests.swift`, `CaretSyncTests.swift`: source-range navigation and caret
  item sets only; no scroll geometry.
- `PreviewTextCacheTests.swift` (:271-324): CTLine cache metrics at 1x/2x scale — no
  scrolling.
- `PreviewV2Tests.swift`: `testLoadingRetainsThePreviousFrameAsStaleUntilTheNewOneIsVerified`
  (:351), `testStaleLoadResultNeverOverwritesANewerState` (:374),
  `testStaleUnsolicitedAndMismatchedLinesNeverApply` (:631) cover the MODEL state
  (`displayListV2`) for stale v2 candidates; nothing checks that the pane's scroll
  offset is untouched. `testPageRasterizerDeliversOffMainBitmapsAndDropsStaleOnes`
  covers bitmaps, not scroll.
- `ShellModel+Controller.swift:405` drops a controller preview whose editor revision
  is older than the applied result (`ignored stale controller preview`); covered by
  `PreviewControllerTests` at the model level (result never replaced). Not covered:
  the view's scroll offset across that event.
- Sources: `PreviewView.swift` fit-to-width `scale` from `GeometryReader`, plain
  `ScrollView([.vertical, .horizontal])` + `VStack`; the only programmatic scroll is
  `proxy.scrollTo(caretPage, anchor: .top)` on caret page change. `PreviewV2View.swift`:
  same layout with `LazyVStack`, no programmatic scroll at all. Neither captures or
  restores a (page, fraction) anchor across a result/frame swap or a scale change.
- Live report 20260912T110944Z.md: "anchor" rows are bridge anchor sha256; no
  scroll-anchoring row. `docs/evidence/`: no scroll evidence.

Conclusion: gap 5 is NOT covered. Implementing: pure `PreviewAnchor.swift` model +
`PreviewAnchorKeeper` (NSViewRepresentable probe on the enclosing NSScrollView) +
`PreviewAnchoringTests.swift` (pure model + hosted off-screen NSScrollView measurements).

## Durable checkpoint

- Branch `agent/mac-preview-anchoring/anchoring` from `origin/agent/mac-claude-a/mac-shell`
  5bc3fc0f (contains main dda0b62). Worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-adc47fb73ba7bb6e1`.
- Owned files: `apps/mac/Sources/FlashTeXMac/PreviewAnchor.swift`,
  `apps/mac/Tests/FlashTeXMacTests/PreviewAnchoringTests.swift`, `PreviewV2View.swift`
  hook (committable), this handoff, `coordination/agents/mac-preview-anchoring.json`.
  Parent-retained `PreviewView.swift`: diff only (in this handoff), applied locally on
  `agent/mac-preview-anchoring/anchoring-applied` for measurement.
- Helpers: FLASHTEX_COMPILER=/Users/jay3332/Projects/flashtex/crates/compiler/target/release/flashtex-compiler
  (main checkout build 09:31Z; not rebuilt here).
- Next commands: `cd apps/mac && swift build --build-tests && swift test --filter PreviewAnchoringTests`.
- Budget: ~75 min from 09:29 local; Claude Max shared quota, no purchases.
