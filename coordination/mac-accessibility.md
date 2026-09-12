# mac-accessibility (Claude Code subagent, parent mac-claude-a)

- Updated UTC: 2026-09-12T10:40Z
- Agent / parent / machine alias: mac-accessibility / mac-claude-a / mac-m1max-a
- Task / acceptance gate / owned paths: lane "keyboard/VoiceOver completeness
  for the shell": (1) Help > Accessibility Help window content and
  README/menu-wiring/focus-order parity tests, (2) preview rotor/landmarks
  tree with navigation order and a hosted read-back test, (3) diagnostics
  rows and completion popup labels verified against the accessibility
  announcements. Owned: `apps/mac/Sources/FlashTeXAccessibility/**`,
  `apps/mac/Tests/FlashTeXAccessibilityTests/**`, this handoff,
  `coordination/agents/mac-accessibility.json`. Parent retains
  ShellModel*/ContentView/PreviewView/FlashTeXMacApp (exact diffs below, not
  applied). Transferred crates untouched.
- Branch / code revision / main integrated through:
  `agent/mac-accessibility/help-and-rotor` from
  `origin/agent/mac-claude-a/mac-shell` 40d53b7, then merged mac-shell
  b898cfc (multi-file ProjectDocuments + Project menu in the editor header,
  bridge recovery, IME, vocabulary, helper navigation lanes) at 959a036;
  accessibility tests stayed green, the Editor pane description now names
  the Project menu. main 4248b49 fetched and inspected, not merged (its
  apps/mac content arrives through mac-shell).
  Worktree `.claude/worktrees/agent-a3cbe1d28258de76a`.
- State: ready for integration (parent review + the parent-file patch below).

## Ready behavior and evidence

Preview accessibility tree (`AccessibilityViews.swift`, `PageAXView`):
- The page container is a landmark: role group, subrole `AXLandmarkRegion`
  (the WebKit/AppKit spelling; AppKit has no Swift constant), role
  description "page", label "Page n of m, k lines" (m = `totalPages`, "Page
  n, k lines" when unknown). Its children are one group per reading-order
  line (role description "line", label "Page n, line k: <summary>"), each
  holding one static-text element per item (label = spoken text, value =
  size/source availability, help "Page n, line k") with the "Go to source"
  custom action only when the item has a source; the action calls the
  preview's click closure with the same (source, text) as a mouse click.
- `accessibilityChildrenInNavigationOrder` is the children array at both
  levels. `NSAccessibilityElement` does not declare the `NSAccessibilityElement`
  protocol at runtime (`e is NSAccessibilityElementProtocol` is false), so
  `PreviewAXElement` adopts it once with `class_addProtocol`; without that the
  typed navigation-order array cannot hold the elements.
- Frames: items keep the overlay's view-space frame (flipped page view) and
  answer `accessibilityFrame()` through `NSAccessibility.screenRect(fromView:rect:)`
  when hosted; the old code handed flipped-view rects to
  `accessibilityFrameInParentSpace` unconverted. Line frames are the union of
  their items; parent-space frames are still set for window-less clients.
- Still lazy: the tree is built on the first `accessibilityChildren` request
  and dropped on a page/scale/totalPages change (`layoutChanged` posted only if
  a client had read it). `hasBuiltTree` exposes that for tests.

Help window (`AccessibilityHelpView`): focus order + rationale, the
non-focusable status lines (toolbar, banner, footer), "What VoiceOver reads"
per pane (editor announcements, completion popup, preview landmarks/lines/
items, diagnostics rows, capture bar), and the command table grouped by menu
in menu-bar order with each row one combined element labelled by `helpLine`.
`windowID` "a11y-help", `windowTitle` "Accessibility Help", `menuItem`
"FlashTeX Accessibility Help". Not attached: needs the FlashTeXMacApp diff.

Command table (`AccessibilityCommands.swift`): every menu command now names
its exact `Button` title (`menuItem`); `FocusOrder` now states the order
`ContentView.swift` really builds, Editor → Capture bar → Bridge bar →
Preview → Diagnostics (the old table said Editor → Preview → Diagnostics →
Capture bar, which is not the view order: the capture and bridge bars sit
inside `EditorPane` under the text view), each pane with its container and
source marker.

Diagnostics rows: `DiagnosticRowAccessibility` (label "Diagnostic n of m:
Error/Warning: message", value "recovery line; path bytes a to b", hint "No
source mapping; listed only." / action "Go to source") and
`accessibleDiagnostic(_:index:total:status:goToSource:)`. The recovery line
follows the shell's `EditorDiagnostics.recoveryLine` rule (note, else "no
provisional rendering" only for a `recovered` result). The old call shape
without `status` still compiles and reads as `recovered` (previous behavior).

Completion popup: `CompletionAccessibility` — list label "Completions", list
help, row label "<candidate>, <kind>, <origin>" (kind spoken as command /
environment / label / citation / word), "n of m: …" selection announcement,
"k completions" opened announcement. Not applied in `Completion.swift` (not
my path): diff below.

Tests (`FlashTeXAccessibilityTests`, 41 tests, 1 documented skip):
- `CommandTableTests`: README shortcuts ↔ table (kept); every command maps to
  exactly one README row and every row to ≥1 command, with the row naming
  the command's first title word; `Button("…")` titles, `.keyboardShortcut`
  spellings, enclosing menu (`replacing: .newItem` = File, `after: .pasteboard`
  = Edit, `CommandMenu("Navigate")`) and `.disabled(` ↔ `requires` parsed from
  `FlashTeXMacApp.swift` and `Navigation.swift`; every shell item with a key
  equivalent is in the table; focus order parsed from `ContentView.swift`
  (HSplitView order, then each pane's markers in order inside its container);
  help view covers every command/menu and every pane note.
- `OverlayTests`: hosted `PageAXView` read back through the NSAccessibility
  protocol — landmark subrole/role description/label, line groups, item
  roles/labels/values/help/parents, same order as the flat overlay slots,
  navigation order === children at both levels, runtime protocol adoption,
  "Go to source" only where `Element.actions` says so and it forwards
  (source, text), screen frames equal `screenRect(fromView:)` of the slot
  frames (and differ from the raw flipped rects), line frame = union, tree
  kept on identical update / dropped on page change / relabelled on
  totalPages change.
- `CompletionAccessibilityTests`: labels/announcements; an `NSTableView`
  configured like the shell's popup read back through the AX attribute API
  (table role/label/help, 3 `AXRow`s with index, `AXCell` description = the
  row label, static-text child value = visible text, selected row index).
  Finding: `NSTableView.accessibilityRows()` traps ("NSArray element failed
  to match the Swift Array Element type") bridging AppKit's private
  `NSTableRow` proxies, and those proxies answer only the legacy
  `accessibilityAttributeValue` API, not the Swift protocol getters.
- `EditorDiagnosticsAccessibilityTests`: row label/value/actions per status;
  rows vs `EditorDiagnosticNavigation.Step.announcement` vs
  `AccessibleDocumentModel.DiagnosticElement` agree on severity word, message
  and recovery line while stepping through every diagnostic once.
- Skip (documented in the test): SwiftUI's `NSHostingView` exposes no
  accessibility children in-process (`accessibilityChildren`,
  `…InNavigationOrder`, legacy children attribute all empty, no subviews) so
  the `accessibleDiagnostic` modifier cannot be read back without the system
  AX server; its text is pinned through `DiagnosticRowAccessibility`.

Real VoiceOver was not driven: this machine has no Accessibility/UI-scripting
permission, so rotor membership (Landmarks) and the spoken output are
asserted at the NSAccessibility protocol level only.

## Diffs needed in parent-retained / other-lane files (not applied)

All hunks below were applied locally on top of the merged tip (959a036,
mac-shell b898cfc merged; tip now 1d75acb), built, run (`FlashTeXAccessibilityTests` +
`CompletionTests`: 62 tests, 0 failures, the 1 documented skip) and then
reverted with a checkout, so this is the verified text (`git apply --check`
passes on the branch tip). The FlashTeXMacApp hunk also lets
`FLASHTEX_OPEN_WINDOW=a11y-help` open the help window at launch for
evidence captures (same mechanism as "nearby").
The first four belong together (README row + `accessibilityHelp` entry + the
Help menu item/window + the test's expected menu list): applying only some
of them fails the parity tests by design. `ContentView.swift`
(status-aware recovery line), `PreviewView.swift` ("Page n of m") and
`Completion.swift` (mac-completion lane: spoken rows, "n of m" selection
announcement, list help) are independent.

```diff
diff --git a/apps/mac/README.md b/apps/mac/README.md
index d53090b..09c3115 100644
--- a/apps/mac/README.md
+++ b/apps/mac/README.md
@@ -679,6 +679,7 @@ explain that nothing is loaded.
 | ⌘⇧] / ⌘⇧[ | Next / previous diagnostic (refused if its span was edited since the compile) |
 | ⌘⇧J | Reveal caret in preview (selects the item's source span) |
 | Click preview text | Select its source (UTF-8 span → UTF-16; refused if edited since compile) |
+| Help > FlashTeX Accessibility Help | Help window: focus order, what VoiceOver reads in each pane, every command above |
 
 The compiler rejects request lines over 8 MiB with an `error` envelope, which the
 banner shows; the shell rejects response lines over 16 MiB.
diff --git a/apps/mac/Sources/FlashTeXAccessibility/AccessibilityCommands.swift b/apps/mac/Sources/FlashTeXAccessibility/AccessibilityCommands.swift
index cd2cb79..471321e 100644
--- a/apps/mac/Sources/FlashTeXAccessibility/AccessibilityCommands.swift
+++ b/apps/mac/Sources/FlashTeXAccessibility/AccessibilityCommands.swift
@@ -13,6 +13,7 @@ public enum AccessibilityCommand: String, CaseIterable, Equatable {
     case undo, completion
     case goToMatching, nextDiagnostic, previousDiagnostic, revealCaretInPreview
     case selectPreviewItemSource
+    case accessibilityHelp
 
     public struct Entry: Equatable {
         public var command: AccessibilityCommand
@@ -148,6 +149,10 @@ public enum AccessibilityCommand: String, CaseIterable, Equatable {
                          description: "Brings back the unsaved text replaced by a Discard decision when another file was opened; the restored buffer stays unsaved.",
                          requires: "a discarded buffer from this session",
                          menuItem: "Restore Discarded Buffer")
+        case .accessibilityHelp:
+            return Entry(command: self, title: "Help window", shortcuts: ["Help > FlashTeX Accessibility Help"], menu: "Help",
+                         description: "Opens the Accessibility Help window: focus order, what VoiceOver reads in each pane, and every command in this table.",
+                         menuItem: "FlashTeX Accessibility Help")
         case .selectPreviewItemSource:
             return Entry(command: self, title: "Select source of a preview item", shortcuts: ["Click preview text"], menu: "Preview",
                          description: "Selects the item's source in the editor; with VoiceOver, use the “Go to source” action on the item.",
diff --git a/apps/mac/Sources/FlashTeXMac/ContentView.swift b/apps/mac/Sources/FlashTeXMac/ContentView.swift
index 7656bae..71211b6 100644
--- a/apps/mac/Sources/FlashTeXMac/ContentView.swift
+++ b/apps/mac/Sources/FlashTeXMac/ContentView.swift
@@ -277,7 +277,7 @@ private struct PreviewPane: View {
                     Spacer()
                     if d.source != nil { Button("Go to source") { model.navigate(to: d.source) } }
                 }
-                .accessibleDiagnostic(d, index: i, total: diags.count) { model.navigate(to: d.source) } // FlashTeXAccessibility
+                .accessibleDiagnostic(d, index: i, total: diags.count, status: model.result?.status ?? .ok) { model.navigate(to: d.source) } // FlashTeXAccessibility
             }
             .frame(minHeight: 80, maxHeight: 180)
         }
diff --git a/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift b/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
index b0d90d0..d06b9d6 100644
--- a/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
+++ b/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
@@ -1,5 +1,6 @@
 import AppKit
 import SwiftUI
+import FlashTeXAccessibility
 
 /// A bare SwiftPM executable has no bundle, so AppKit defaults to an
 /// accessory-style process with no Dock icon and, when launched from a
@@ -88,11 +89,14 @@ struct FlashTeXMacApp: App {
                 .onAppear {
                     appDelegate.model = model; nearby.attach(sink: model, destinations: model); TypingBench.shared.install(model: model)
                     // Automation: open a secondary window at launch for evidence captures.
-                    if ProcessInfo.processInfo.environment["FLASHTEX_OPEN_WINDOW"] == "nearby" { openWindow(id: "nearby") }
+                    if let id = ProcessInfo.processInfo.environment["FLASHTEX_OPEN_WINDOW"], ["nearby", AccessibilityHelpView.windowID].contains(id) { openWindow(id: id) }
                 }
         }
         .commands {
             NavigationCommands(model: model) // Navigation.swift
+            CommandGroup(after: .help) {
+                Button("FlashTeX Accessibility Help") { openWindow(id: AccessibilityHelpView.windowID) }
+            }
             CommandGroup(after: .pasteboard) {
                 Divider()
                 Button("Pin Insertion Point") { model.pinAnchorAtCaret() }
@@ -163,5 +167,8 @@ struct FlashTeXMacApp: App {
             NearbyView().environmentObject(nearby).environment(model)
         }
         .windowResizability(.contentSize)
+        Window("Accessibility Help", id: AccessibilityHelpView.windowID) {
+            AccessibilityHelpView() // FlashTeXAccessibility: focus order, VoiceOver notes, command table
+        }
     }
 }
diff --git a/apps/mac/Sources/FlashTeXMac/PreviewView.swift b/apps/mac/Sources/FlashTeXMac/PreviewView.swift
index b11ff96..871a1f2 100644
--- a/apps/mac/Sources/FlashTeXMac/PreviewView.swift
+++ b/apps/mac/Sources/FlashTeXMac/PreviewView.swift
@@ -32,7 +32,7 @@ struct PreviewView: View {
                 // pages whose layout (or source offsets) actually moved.
                 VStack(spacing: 24) {
                     ForEach(result.pages, id: \.number) { page in
-                        PageView(page: page, dark: dark, caretItems: caretItems[page.number] ?? [], scale: scale,
+                        PageView(page: page, totalPages: result.pages.count, dark: dark, caretItems: caretItems[page.number] ?? [], scale: scale,
                                  rulesNegotiated: result.layoutCapabilities?.contains(RuntimeV1.LayoutCapabilities.rulesV1) == true,
                                  onSelect: onSelect)
                             .equatable()
@@ -54,6 +54,7 @@ struct PreviewView: View {
 
 private struct PageView: View, Equatable {
     let page: RuntimeV1.Page
+    let totalPages: Int
     let dark: Bool
     var caretItems: Set<Int> = []
     /// Display scale (1 = 1pt per screen point); the preview fits pages to width.
@@ -64,14 +65,14 @@ private struct PageView: View, Equatable {
     /// Everything that affects the drawing; `onSelect` is the same closure for
     /// every page and revision, so it is not part of identity.
     static func == (a: PageView, b: PageView) -> Bool {
-        a.page == b.page && a.dark == b.dark && a.caretItems == b.caretItems && a.scale == b.scale && a.rulesNegotiated == b.rulesNegotiated
+        a.page == b.page && a.totalPages == b.totalPages && a.dark == b.dark && a.caretItems == b.caretItems && a.scale == b.scale && a.rulesNegotiated == b.rulesNegotiated
     }
 
     var body: some View {
         let size = CGSize(width: page.widthPt * scale, height: page.heightPt * scale)
         HitTestCanvas(page: page, dark: dark, scale: scale, caretItems: caretItems, rulesNegotiated: rulesNegotiated, onSelect: onSelect)
             .frame(width: size.width, height: size.height)
-            .overlay(alignment: .topLeading) { AccessibilityOverlay(page: page, scale: scale, fontName: { PreviewFonts.postScriptName(size: $0) }, onSelect: onSelect) } // FlashTeXAccessibility
+            .overlay(alignment: .topLeading) { AccessibilityOverlay(page: page, totalPages: totalPages, scale: scale, fontName: { PreviewFonts.postScriptName(size: $0) }, onSelect: onSelect) } // FlashTeXAccessibility
             .background(dark ? Color(white: 0.16) : .white)
             .shadow(radius: 4)
             .overlay(alignment: .bottomTrailing) {
diff --git a/apps/mac/Tests/FlashTeXAccessibilityTests/CommandTableTests.swift b/apps/mac/Tests/FlashTeXAccessibilityTests/CommandTableTests.swift
index 1c10d5b..b5a042f 100644
--- a/apps/mac/Tests/FlashTeXAccessibilityTests/CommandTableTests.swift
+++ b/apps/mac/Tests/FlashTeXAccessibilityTests/CommandTableTests.swift
@@ -242,7 +242,7 @@ final class CommandTableTests: XCTestCase {
 
     func testHelpViewCoversEveryCommandAndMenu() {
         let menus = AccessibilityHelpView.menus
-        XCTAssertEqual(menus.map(\.menu), ["File", "File / toolbar", "Edit", "Editor", "Navigate", "Preview"])
+        XCTAssertEqual(menus.map(\.menu), ["File", "File / toolbar", "Edit", "Editor", "Navigate", "Preview", "Help"])
         XCTAssertEqual(menus.flatMap(\.entries).count, AccessibilityCommand.allCases.count)
         XCTAssertEqual(AccessibilityHelpView.windowID, "a11y-help")
         XCTAssertEqual(AccessibilityHelpView.menuItem, "FlashTeX Accessibility Help")
diff --git a/apps/mac/Sources/FlashTeXMac/Completion.swift b/apps/mac/Sources/FlashTeXMac/Completion.swift
index a3ca1a5..d6123f3 100644
--- a/apps/mac/Sources/FlashTeXMac/Completion.swift
+++ b/apps/mac/Sources/FlashTeXMac/Completion.swift
@@ -1,4 +1,5 @@
 import AppKit
+import FlashTeXAccessibility
 import FlashTeXProtocol
 import os
 
@@ -981,7 +982,8 @@ final class CompletionPopup: NSPanel, NSTableViewDataSource, NSTableViewDelegate
         table.target = self
         table.action = #selector(rowClicked(_:))
         table.doubleAction = #selector(rowDoubleClicked(_:))
-        table.setAccessibilityLabel("Completions")
+        table.setAccessibilityLabel(CompletionAccessibility.listLabel) // FlashTeXAccessibility
+        table.setAccessibilityHelp(CompletionAccessibility.listHelp)
         let scroll = NSScrollView(frame: contentView!.bounds)
         scroll.documentView = table
         scroll.hasVerticalScroller = true
@@ -1023,6 +1025,7 @@ final class CompletionPopup: NSPanel, NSTableViewDataSource, NSTableViewDelegate
         if items.indices.contains(selected) {
             table.selectRowIndexes(IndexSet(integer: selected), byExtendingSelection: false)
             table.scrollRowToVisible(selected)
+            announceSelection(items[selected], index: selected, total: items.count)
         }
         updatingSelection = false
     }
@@ -1077,9 +1080,24 @@ final class CompletionPopup: NSPanel, NSTableViewDataSource, NSTableViewDelegate
             return f
         }()
         field.attributedStringValue = Self.attributed(items[row])
+        field.setAccessibilityLabel(Self.spokenLabel(items[row])) // FlashTeXAccessibility
         return field
     }
 
+    /// "\section, command, supported by this compiler" — what VoiceOver reads for a row.
+    static func spokenLabel(_ s: Completion.Suggestion) -> String {
+        CompletionAccessibility.rowLabel(label: s.label, kind: s.kind.accessibilityKind, detail: s.detail)
+    }
+
+    /// "n of m: …" posted when the selection moves (the panel never takes focus, so this is the only cue).
+    private func announceSelection(_ s: Completion.Suggestion, index: Int, total: Int) {
+        NSAccessibility.post(element: table, notification: .announcementRequested, userInfo: [
+            .announcement: CompletionAccessibility.selectionAnnouncement(index: index, total: total, label: s.label,
+                                                                        kind: s.kind.accessibilityKind, detail: s.detail),
+            .priority: NSAccessibilityPriorityLevel.medium.rawValue,
+        ])
+    }
+
     /// `\section  cmd · supported by this compiler` — label in the editor's
     /// monospaced font, kind and detail in the secondary colour.
     static func attributed(_ s: Completion.Suggestion) -> NSAttributedString {
@@ -1094,6 +1112,17 @@ final class CompletionPopup: NSPanel, NSTableViewDataSource, NSTableViewDelegate
 }
 
 extension Completion.Kind {
+    /// The accessibility layer's spelling of the kind (spoken "label" for `.reference`).
+    var accessibilityKind: CompletionAccessibility.Kind {
+        switch self {
+        case .command: return .command
+        case .environment: return .environment
+        case .reference: return .reference
+        case .citation: return .citation
+        case .word: return .word
+        }
+    }
+
     var badge: String {
         switch self {
         case .command: return "cmd"
```

## Validation

- `swift build` and `swift test --filter FlashTeXAccessibilityTests`: 41
  tests, 0 failures, 1 skip (the SwiftUI-hosted row test, see above).
- Full `swift test` (apps/mac) at a434916 with FLASHTEX_COMPILER/PDF/BRIDGE/
  EDIT_LEDGER pointed at the release binaries in the main checkout and
  FLASHTEX_NO_ACTIVATE=1: 396 tests, 0 failures, 13 skips — 12 pre-existing
  environment skips (CompletionLiveHelper, DocumentFiles helper x3,
  ExactPDFExport, NearbyReferenceClient, NearbyView screenshots,
  PreviewController x2, ProposalPreview x3) plus this lane's documented
  SwiftUI-hosting skip. Log: scratchpad `fullsuite1.log` (not committed).
- Full `swift test` on the merged tip (1d75acb, mac-shell b898cfc merged),
  same environment: 437 tests, 19 skips, 4 failures, all in the preview-v2
  lane's tests and all caused by mac-shell b898cfc vendoring
  `Fonts/latinmodern-math.otf`: `PreviewV2ShellTests.testRefusedDisplayListShowsNoFrame`,
  `…testLoadingRetainsThePreviousFrameAsStaleUntilTheNewOneIsVerified`,
  `…testStaleLoadResultNeverOverwritesANewerState` and
  `RenderingV2Tests.testMathFixtureFailsClosedWithoutTheMathFontBundled`
  expect `display-list-v2-math.json` to be refused with
  `font_resource_unavailable`, and the math font now resolves. Not this
  lane's files (PreviewV2Tests/RenderingV2Tests, owner mac-preview-v2 /
  parent); nothing in FlashTeXAccessibility is involved. Reported, not
  fixed. Log: scratchpad `fullsuite2.log`.
- The parent-file diffs were applied locally, built and tested, then
  reverted, so the patch above is the verified text.
- Help window evidence: with the patch applied, the debug app was launched
  with FLASHTEX_NO_ACTIVATE=1 FLASHTEX_AUTOATTACH=0 FLASHTEX_OPEN_WINDOW=a11y-help
  (never activated, no focus taken) and the "Accessibility Help" window
  (CGWindowListCopyWindowInfo id, 900x450) captured with `screencapture -l`:
  scratchpad `evidence/accessibility-help.png` shows the focus-order section,
  the status lines and the start of "What VoiceOver reads". Not committed
  (evidence paths are not in this lane's owned tree); the parent may copy it
  into its evidence directory.

## Limitations / decisions

- Landmark subrole: `AXLandmarkRegion` is what WebKit sets for `<section>`
  landmarks and what VoiceOver's Landmarks rotor lists; AppKit exposes no
  constant, so it is a raw `NSAccessibility.Subrole`. Verified only that the
  view answers it; rotor membership was not observed with VoiceOver.
- The row value now also carries "path bytes a to b" (the visible row shows
  the same bytes); the old value was only the recovery text.
- `helpLine` (used as the help row label) still reads "Title — shortcut
  (menu): description", unchanged.

## Resources / rules

- Resource pool: parent mac-claude-a's shared Claude Max 20x quota
  (allocation alias `claude-mac20x-shared`); no purchases, overages or paid
  network calls; usage figure unknown from this session.
- Commit identity jay3332 (Mac primary-author rule); trailers name this
  subagent and "git via Claude Code".

## Dirty files / running jobs / next action

- Dirty: none after the handoff commit. Running jobs: none.
- Next: parent applies the patch (Completion.swift hunk to the completion
  lane), merges this branch into mac-shell, reruns `swift test`.

## Resume reading list

AGENTS.md, this file, `apps/mac/Sources/FlashTeXAccessibility/*.swift`,
`apps/mac/Tests/FlashTeXAccessibilityTests/*.swift`, `apps/mac/README.md`
(Keyboard shortcuts), `apps/mac/Sources/FlashTeXMac/{FlashTeXMacApp,Navigation,ContentView,PreviewView,Completion}.swift`.
