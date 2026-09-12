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

## Delivered

- `apps/mac/Sources/FlashTeXMac/PreviewAnchor.swift` (new): `PreviewPageLayout`
  (page frames from page sizes, fit scale, 24 pt spacing/padding, centered widths),
  `PreviewAnchor` (page, fraction of page, horizontal fraction at the viewport top;
  `capture`/`restore`, vanished page → last page, clamped to the scroll range),
  `PreviewAnchorKeeper` (NSViewRepresentable placed as the page column's
  `.background`; observes the backing NSScrollView, captures on scroll, re-imposes
  the anchor through a 150 ms settle window after the layout changes), with a
  test-visible `corrections`/`trace`.
- `apps/mac/Sources/FlashTeXMac/PreviewV2View.swift`: builds the layout next to the
  fit scale and attaches `PreviewAnchorKeeper` (committed; not parent-retained).
- `apps/mac/Tests/FlashTeXMacTests/PreviewAnchoringTests.swift`: 9 tests (5 pure,
  hosted column mirror, wide-content horizontal, `PreviewV2View`, real-compiler
  `PreviewView` gated on `FLASHTEX_COMPILER` and asserting only when the hook is
  present).
- `docs/evidence/mac-preview-anchoring-2026-09-12.md`: measured table.

Results (details in the evidence file): resize 500→350 pt with the reader at
page 2 @ 0.3 drifts 252.4 pt on the unhooked `PreviewView` and 0.0 pt with the hook
(and on `PreviewV2View`); appended pages and dropped stale results move nothing
(0.0 pt) with or without the hook; a vanished anchored page clamps to the end of
the document (never the top). Under shared load (1-min load 8–25).

## Diff for the parent (PreviewView.swift is parent-retained — not committed here)

Applied and measured on `agent/mac-preview-anchoring/anchoring-applied` (5d1503e7,
pushed; do not merge that branch — apply this diff on the parent branch instead):

```diff
diff --git a/apps/mac/Sources/FlashTeXMac/PreviewView.swift b/apps/mac/Sources/FlashTeXMac/PreviewView.swift
index 871a1f28..3f83a4e8 100644
--- a/apps/mac/Sources/FlashTeXMac/PreviewView.swift
+++ b/apps/mac/Sources/FlashTeXMac/PreviewView.swift
@@ -25,6 +25,10 @@ struct PreviewView: View {
             let widest = result.pages.map(\.widthPt).max() ?? 612
             // Fit the widest page to the pane (never upscale past 100%).
             let scale = min(1, max(0.2, (geo.size.width - 48) / widest))
+            // Scroll anchoring (PreviewAnchor.swift): the (page, fraction) under the
+            // viewport's top edge survives a result with another page count and a
+            // pane resize; a result with the same page geometry never moves the scroll.
+            let layout = PreviewPageLayout(pages: result.pages.map { PreviewPageLayout.Page(number: $0.number, widthPt: $0.widthPt, heightPt: $0.heightPt) }, scale: scale)
             ScrollView([.vertical, .horizontal]) {
                 // Lazy: only pages near the viewport are laid out and drawn;
                 // `.equatable()`: a page whose items, caret set and scale did not
@@ -40,6 +44,7 @@ struct PreviewView: View {
                     }
                 }
                 .padding(24)
+                .background(PreviewAnchorKeeper(layout: layout))
             }
             .onChange(of: caretPage) { _, page in
                 // Page-level only: keeps the page under the caret in view when the
```

## Limitations

- Anchor = viewport TOP edge (page, fraction). A page-count change that shifts
  content ahead of the anchored page (text inserted on an earlier page) keeps the
  same page number/fraction — the content under the reader changes, by design
  of the brief ("visible page/fraction preserved").
- Enforcement window: a user scroll within 150 ms of a layout change is overridden.
- Horizontal restore only applies when the document is wider than the clip; with
  legacy (non-overlay) scrollers in the test process the clip is 16 pt narrower,
  which produced a harmless 8 pt x-adjustment in the harness.
- Full `swift test` not run (parent's heavy-build window). No app-level capture
  (harness is hosted off-screen; no UI scripting).

## Durable checkpoint

- Branch `agent/mac-preview-anchoring/anchoring` (lane, pushed) from
  `origin/agent/mac-claude-a/mac-shell` 5bc3fc0f (contains main dda0b62); applied
  branch `agent/mac-preview-anchoring/anchoring-applied` 5d1503e7 (pushed, local
  measurement only). Worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-adc47fb73ba7bb6e1`.
- Owned files: `PreviewAnchor.swift`, `PreviewAnchoringTests.swift`, the
  `PreviewV2View.swift` hook, `docs/evidence/mac-preview-anchoring-2026-09-12.md`,
  this handoff, `coordination/agents/mac-preview-anchoring.json`.
- Helpers: FLASHTEX_COMPILER=/Users/jay3332/Projects/flashtex/crates/compiler/target/release/flashtex-compiler
  (main checkout build 09:31Z; not rebuilt here).
- Verify: `cd apps/mac && swift build --build-tests && FLASHTEX_COMPILER=… swift test --filter PreviewAnchoringTests`.
- State: ready_for_integration; next action is the parent applying the diff above.
- Claude Max shared quota, no purchases.
