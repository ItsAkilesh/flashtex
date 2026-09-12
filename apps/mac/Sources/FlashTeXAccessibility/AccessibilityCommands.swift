import Foundation

/// The keyboard workflow as data: every app action a screen-reader user can
/// reach, its shortcut(s) exactly as the README table spells them, the menu it
/// lives in, and a discoverable description. The UI can render this as an
/// "Accessibility help" list; the test target checks it against the README.
public enum AccessibilityCommand: String, CaseIterable, Equatable {
    case editorPreferences
    case openLaTeXFile, save, saveAs, openFixture, reloadFixture
    case attachBuiltCompiler, attachRenderPipeline, attachWorker, compile
    case exportPDF, exportPDFViaRust, exportPDFExact
    case pinInsertionPoint, openCaptureProposal, submitSampleCapture, convertCapture, nearbyCompanion
    case restoreDiscardedBuffer
    case undo, completion, completionList
    case goToMatching, nextDiagnostic, previousDiagnostic, revealCaretInPreview
    case selectPreviewItemSource
    case accessibilityHelp
    case durableHistory, findInProject, nextSearchMatch

    public struct Entry: Equatable {
        public var command: AccessibilityCommand
        public var title: String
        /// Shortcut spellings as in the README ("⌘⇧]", "Esc", "⌃Space", "Click preview text").
        public var shortcuts: [String]
        public var menu: String
        public var description: String
        /// What must be true for the command to be enabled, or nil if always.
        public var requires: String?
        /// The menu item's title exactly as `FlashTeXMacApp`/`NavigationCommands`
        /// wire it (`Button("…")`), or nil for commands that are not menu items
        /// (editor keys, a preview click). The test target reads the shell
        /// source and checks the title and its `keyboardShortcut` agree.
        public var menuItem: String?

        public init(command: AccessibilityCommand, title: String, shortcuts: [String], menu: String,
                    description: String, requires: String? = nil, menuItem: String? = nil) {
            self.command = command; self.title = title; self.shortcuts = shortcuts; self.menu = menu
            self.description = description; self.requires = requires; self.menuItem = menuItem
        }

        /// One line for a help list: "Next diagnostic — ⌘⇧] (Navigate): …".
        public var helpLine: String {
            "\(title) — \(shortcuts.joined(separator: " or ")) (\(menu)): \(description)"
                + (requires.map { " Requires \($0)." } ?? "")
        }
    }

    public var entry: Entry {
        switch self {
        case .openLaTeXFile:
            return Entry(command: self, title: "Open LaTeX file", shortcuts: ["⌘O"], menu: "File",
                         description: "Opens a .tex file as the main.tex entry document and compiles it when a worker is attached.",
                         menuItem: "Open LaTeX File…")
        case .save:
            return Entry(command: self, title: "Save", shortcuts: ["⌘S"], menu: "File",
                         description: "Saves the entry document as UTF-8; the editor header says “edited” while unsaved.",
                         menuItem: "Save")
        case .saveAs:
            return Entry(command: self, title: "Save As", shortcuts: ["⌘⇧S"], menu: "File",
                         description: "Saves the entry document under a new name.",
                         menuItem: "Save As…")
        case .openFixture:
            return Entry(command: self, title: "Open compile result fixture", shortcuts: ["⌘⇧O"], menu: "File",
                         description: "Loads a runtime v1 compile_result JSON into the preview; a sibling -request.json seeds the editor.",
                         menuItem: "Open Compile Result Fixture…")
        case .reloadFixture:
            return Entry(command: self, title: "Reload fixture", shortcuts: ["⌘R"], menu: "File",
                         description: "Reloads the current fixture from disk.",
                         menuItem: "Reload Fixture")
        case .attachBuiltCompiler:
            return Entry(command: self, title: "Attach built compiler", shortcuts: ["⌘⇧K"], menu: "File",
                         description: "Attaches the FlashTeX compiler found via $FLASHTEX_COMPILER or crates/compiler/target.",
                         menuItem: "Attach Built Compiler")
        case .attachRenderPipeline:
            return Entry(command: self, title: "Attach render pipeline", shortcuts: ["⌘⇧R"], menu: "File",
                         description: "Attaches flashtex-render (crates/render-pipeline) found via $FLASHTEX_RENDER, the app bundle, or crates/render-pipeline/target: the producer measured with Latin Modern metrics, so the preview shows Computer Modern-style text.",
                         menuItem: "Attach Render Pipeline (Latin Modern)")
        case .attachWorker:
            return Entry(command: self, title: "Attach worker executable", shortcuts: ["⌘K"], menu: "File",
                         description: "Chooses any executable speaking runtime v1 JSON Lines and attaches it as the compiler.",
                         menuItem: "Attach Worker Executable…")
        case .compile:
            return Entry(command: self, title: "Compile now", shortcuts: ["⌘B"], menu: "File / toolbar",
                         description: "Sends the current buffers to the attached worker; auto-compile also runs 250 ms after edits.",
                         requires: "an attached worker",
                         menuItem: "Compile")
        case .exportPDF:
            return Entry(command: self, title: "Export PDF", shortcuts: ["⌘⇧E"], menu: "File",
                         description: "Writes the current preview as a PDF with CoreGraphics (always white).",
                         requires: "a compile result",
                         menuItem: "Export PDF…")
        case .exportPDFViaRust:
            return Entry(command: self, title: "Export PDF via Rust writer", shortcuts: ["⌘⌥E"], menu: "File",
                         description: "Pipes the compile result to flashtex-pdf --verify (always white).",
                         requires: "a compile result",
                         menuItem: "Export PDF via Rust Writer…")
        case .exportPDFExact:
            return Entry(command: self, title: "Export PDF (exact, v2)", shortcuts: ["File > Export PDF (exact, v2)…"], menu: "File",
                         description: "Hands the loaded v2 display list to flashtex-pdf-exact from-v2: glyphs by original GID, embedded font programs, typed rules; refuses what it cannot express exactly.",
                         requires: "a loaded v2 display list and a built flashtex-pdf-exact",
                         menuItem: "Export PDF (exact, v2)…")
        case .pinInsertionPoint:
            return Entry(command: self, title: "Pin insertion point", shortcuts: ["⌘⇧P"], menu: "Edit",
                         description: "Records the caret as the destination anchor for capture proposals; the capture bar reads it back.",
                         menuItem: "Pin Insertion Point")
        case .openCaptureProposal:
            return Entry(command: self, title: "Open capture proposal", shortcuts: ["⌘⇧I"], menu: "Edit",
                         description: "Queues a capture_proposal file for review; Return in the sheet approves and inserts one undoable edit.",
                         menuItem: "Open Capture Proposal…")
        case .submitSampleCapture:
            return Entry(command: self, title: "Submit sample capture", shortcuts: ["⌘⇧U"], menu: "Edit",
                         description: "Sends a chosen PNG/JPEG as capture_submit through the attached bridge.",
                         requires: "an attached capture bridge",
                         menuItem: "Submit Sample Capture…")
        case .convertCapture:
            return Entry(command: self, title: "Convert capture", shortcuts: ["⌘⇧G"], menu: "Edit",
                         description: "Sends capture_convert for the latest received capture; the proposal opens for review.",
                         requires: "a received capture",
                         menuItem: "Convert Capture")
        case .nearbyCompanion:
            return Entry(command: self, title: "Nearby Companion", shortcuts: ["⌘⇧N"], menu: "Edit",
                         description: "Opens the window that advertises this Mac to a paired iPad/iPhone companion: pairing code (also as a QR image; Copy code or ⌘C on the code copies the digits), paired devices with a per-companion permission pop-up (Captures allowed / View only), received captures (nearby-v1 proposal). Return shows or resumes a pairing code, Esc cancels it or dismisses a banner; the status row, step indicator and every announcement are VoiceOver text.",
                         menuItem: "Nearby Companion…")
        case .undo:
            return Entry(command: self, title: "Undo", shortcuts: ["⌘Z"], menu: "Edit",
                         description: "Undoes the last edit, including an approved capture insertion.")
        case .completion:
            return Entry(command: self, title: "Completion popup", shortcuts: ["Esc", "⌃Space"], menu: "Editor",
                         description: "Lists supported commands, \\end{…} for open environments, labels, citation keys and document words for the token at the caret; the list never takes the keyboard from the editor.")
        case .completionList:
            return Entry(command: self, title: "Completion list keys", shortcuts: ["↑", "↓", "Tab", "⇧Tab", "Return"], menu: "Editor",
                         description: "While the completion list is open: ↑/↓ or Tab/⇧Tab choose the candidate (wrapping; VoiceOver announces “n of m: candidate, kind, origin”), Return or Enter inserts it over the typed token, Esc closes without inserting; typing narrows the list and any other caret move closes it.",
                         requires: "an open completion list")
        case .goToMatching:
            return Entry(command: self, title: "Go to matching", shortcuts: ["⌘⇧D"], menu: "Navigate",
                         description: "Selects the matching \\begin/\\end or \\label/\\ref for the command under the caret; misses are explained in the footer.",
                         menuItem: "Go to Matching \\begin/\\end or \\label/\\ref")
        case .nextDiagnostic:
            return Entry(command: self, title: "Next diagnostic", shortcuts: ["⌘⇧]"], menu: "Navigate",
                         description: "Selects the next diagnostic with a source in the active document (wrapping); refused if its span was edited since the compile.",
                         requires: "a compile result",
                         menuItem: "Next Diagnostic")
        case .previousDiagnostic:
            return Entry(command: self, title: "Previous diagnostic", shortcuts: ["⌘⇧["], menu: "Navigate",
                         description: "Selects the previous diagnostic with a source in the active document (wrapping).",
                         requires: "a compile result",
                         menuItem: "Previous Diagnostic")
        case .revealCaretInPreview:
            return Entry(command: self, title: "Reveal caret in preview", shortcuts: ["⌘⇧J"], menu: "Navigate",
                         description: "Selects the source span of the preview item under the caret and names its page and item.",
                         requires: "a compile result",
                         menuItem: "Reveal Caret in Preview")
        case .restoreDiscardedBuffer:
            return Entry(command: self, title: "Restore Discarded Buffer", shortcuts: ["Edit > Restore Discarded Buffer"], menu: "Edit",
                         description: "Brings back the unsaved text replaced by a Discard decision when another file was opened; the restored buffer stays unsaved.",
                         requires: "a discarded buffer from this session",
                         menuItem: "Restore Discarded Buffer")
        case .accessibilityHelp:
            return Entry(command: self, title: "Help window", shortcuts: ["Help > FlashTeX Accessibility Help"], menu: "Help",
                         description: "Opens the Accessibility Help window: focus order, what VoiceOver reads in each pane, and every command in this table.",
                         menuItem: "FlashTeX Accessibility Help")
        case .selectPreviewItemSource:
            return Entry(command: self, title: "Select source of a preview item", shortcuts: ["Click preview text"], menu: "Preview",
                         description: "Selects the item's source in the editor; with VoiceOver, use the “Go to source” action on the item.",
                         requires: "a compile result")
        case .editorPreferences:
            return Entry(command: self, title: "Settings window", shortcuts: ["⌘,"], menu: "FlashTeX",
                         description: "Opens the editor preferences (the system Settings item): font family and size, wrapping, tab width and indent style, appearance, auto-close braces, completion list, Restore Defaults; Tab walks the controls top to bottom, ⌘W closes and the editor keeps the keyboard.")
        case .durableHistory:
            return Entry(command: self, title: "Durable History window", shortcuts: ["Edit > Durable History…"], menu: "Edit",
                         description: "Opens the durable undo/redo history on the helper's edit ledger: Refresh, Undo, Redo (Retry/Discard after an uncertain reply), retention gauge, then the undo and redo stacks as a list; ⌘W closes and the editor keeps the keyboard.",
                         menuItem: "Durable History…")
        case .findInProject:
            return Entry(command: self, title: "Find in Project window", shortcuts: ["⌘⇧F"], menu: "Edit",
                         description: "Opens the project search: the literal field takes the keyboard, Return searches or goes to the selected match, ↑/↓ move the selection, then Search, Go to Match, Next Match, scope, match limit, results, the replacement field, Plan Replacement and Apply; Esc closes and the editor keeps the keyboard.",
                         menuItem: "Find in Project…")
        case .nextSearchMatch:
            return Entry(command: self, title: "Next match", shortcuts: ["⌘G"], menu: "Find in Project window",
                         description: "Selects the next search match (wrapping) and goes to it in the editor; only while the Find in Project window is key.",
                         requires: "a search with matches")
        }
    }

    public static var entries: [Entry] { allCases.map(\.entry) }

    /// Every shortcut spelling in the table (README parity).
    public static var allShortcuts: Set<String> { Set(entries.flatMap(\.shortcuts)) }

    /// Lines for an "Accessibility help" list, in menu order.
    public static var helpLines: [String] { entries.map(\.helpLine) }
}

/// Keyboard focus order of the main window's panes and why it is that way.
///
/// The order is the view order of `ContentView.swift`: an `HSplitView` of
/// `EditorPane` (source text view, then the capture bar, then the bridge bar)
/// and `PreviewPane` (pages, then the diagnostics list). SwiftUI's Tab cycle
/// and VoiceOver's VO-Right follow that tree, so each pane names the source
/// marker that places it; the test target reads `ContentView.swift` and fails
/// when the panes are reordered without updating this table.
public enum FocusOrder {
    public struct Pane: Equatable {
        public var name: String
        public var contents: String
        public var rationale: String
        /// The `ContentView.swift` type the pane lives in (`EditorPane` / `PreviewPane`).
        public var container: String
        /// Text that places the pane inside `container`'s body, in order.
        public var sourceMarker: String
    }

    public static let panes: [Pane] = [
        Pane(name: "Editor",
             contents: "Source text view, labelled “LaTeX source”; the document picker, the Project menu (open \\input/\\include targets, save or detach a member) and the byte/UTF-16 counts sit above it. Caret moves announce line and column.",
             rationale: "Editing is the primary task; the caret drives caret sync, diagnostics at caret, and every Navigate command.",
             container: "EditorPane", sourceMarker: "SourceEditorView("),
        Pane(name: "Capture bar",
             contents: "Pin insertion point, the pinned anchor, and the review button for queued proposals; one group whose value reads the anchor and proposal count.",
             rationale: "Directly under the editor because pinning starts from the caret; the bar's value is what a capture proposal will insert against.",
             container: "EditorPane", sourceMarker: "CaptureBar()"),
        Pane(name: "Bridge bar",
             contents: "Capture bridge status, the pinned bridge destination, the latest capture's state, and a Convert button when a capture is received.",
             rationale: "Status text with at most one button; it follows the capture bar because a received capture is converted, then reviewed, from the same place.",
             container: "EditorPane", sourceMarker: "BridgeBar()"),
        Pane(name: "Preview",
             contents: "Pages in reading order; each page is a landmark (“Page n of m, k lines”), each line a group, each item static text with a “Go to source” action.",
             rationale: "Follows the editor column so a user can check what the last edit produced, page by page, without leaving the keyboard.",
             container: "PreviewPane", sourceMarker: "PreviewView("),
        Pane(name: "Diagnostics",
             contents: "List of the compile result's diagnostics: “Diagnostic n of m: Error/Warning: message”, the recovery note as the value, “Go to source” when it has a source.",
             rationale: "Comes after the preview because the preview is still shown when errors exist; diagnostics refine, not replace, it.",
             container: "PreviewPane", sourceMarker: "diagnosticsList("),
    ]

    /// Non-focusable status text around the panes, in view order, so the help
    /// can say what VoiceOver reads when it walks the whole window.
    public static let statusLines: [String] = [
        "Toolbar: Dark preview, Auto-compile and v2 preview switches, Reload fixture, Compile (⌘B).",
        "Status banner (top): preview source badge, compile status, error and warning counts, latency, and whether the editor is ahead of the preview.",
        "Footer (bottom): the last navigation note, the stale-diagnostics note, or the preview-click hint; capture notes on the right.",
    ]

    /// "Editor → Capture bar → Bridge bar → Preview → Diagnostics"
    public static var description: String { panes.map(\.name).joined(separator: " → ") }

    /// Full help text: order plus rationale for each pane.
    public static var helpLines: [String] {
        panes.enumerated().map { i, p in "\(i + 1). \(p.name): \(p.contents) \(p.rationale)" }
    }
}

/// Keyboard focus order inside each secondary panel window. SwiftUI's Tab
/// cycle follows the view tree, so each control names the source text that
/// declares it (in `sourceFile`, in this order); the test target reads the
/// panel source and fails when a control is added, removed or reordered
/// without updating this table, and when a control has no spoken name.
public enum PanelFocusOrder {
    public struct Control: Equatable {
        /// What VoiceOver says when Tab lands on the control.
        public var name: String
        /// Text that declares the control in `Panel.sourceFile`, in order.
        public var sourceMarker: String
        /// When the control is present/enabled, or nil if always.
        public var when: String?

        public init(name: String, sourceMarker: String, when: String? = nil) {
            self.name = name; self.sourceMarker = sourceMarker; self.when = when
        }
    }

    public struct Panel: Equatable {
        public var name: String
        public var windowTitle: String
        public var command: AccessibilityCommand
        /// Where keyboard focus lands when the window opens.
        public var initialFocus: String
        /// How the window closes from the keyboard and where focus returns.
        public var closing: String
        public var controls: [Control]
        /// File under `apps/mac/Sources/FlashTeXMac` that declares the controls.
        public var sourceFile: String

        public var helpLine: String {
            "\(name) (\(command.entry.shortcuts.joined(separator: " or "))): opens with focus on \(initialFocus). Tab order: "
                + controls.map { $0.name + ($0.when.map { " (\($0))" } ?? "") }.joined(separator: " → ") + ". \(closing)"
        }
    }

    public static let panels: [Panel] = [
        Panel(name: "Settings", windowTitle: "Editor Preferences", command: .editorPreferences,
              initialFocus: "the font family pop-up",
              closing: "⌘W closes the window; the editor text view is first responder again.",
              controls: [
                Control(name: "Editor font family", sourceMarker: "Picker(\"Family\""),
                Control(name: "Editor font size (slider)", sourceMarker: "Slider(value: $prefs.fontSize"),
                Control(name: "Editor font size stepper", sourceMarker: "Stepper(value: $prefs.fontSize"),
                Control(name: "Wrap long lines", sourceMarker: "Toggle(\"Wrap long lines\""),
                Control(name: "Tab width", sourceMarker: "Stepper(value: $prefs.tabWidth"),
                Control(name: "Indent style (radio group)", sourceMarker: "Picker(\"Indent with\""),
                Control(name: "Editor appearance (segments)", sourceMarker: "Picker(\"Editor appearance\""),
                Control(name: "Auto-close braces", sourceMarker: "Toggle(\"Auto-close braces\""),
                Control(name: "Show completion list", sourceMarker: "Toggle(\"Show completion list\""),
                Control(name: "Restore Defaults", sourceMarker: "Button(\"Restore Defaults\""),
              ],
              sourceFile: "EditorPreferences.swift"),
        Panel(name: "Durable History", windowTitle: "Durable History", command: .durableHistory,
              initialFocus: "the undo/redo list (every button is disabled until a preview controller is attached)",
              closing: "⌘W closes the window; the editor text view is first responder again.",
              controls: [
                Control(name: "Refresh history", sourceMarker: "accessibilityIdentifier(\"history.refresh\")", when: "controller attached"),
                Control(name: "Undo, n steps available", sourceMarker: "accessibilityIdentifier(\"history.undo\")", when: "controller attached"),
                Control(name: "Redo, n steps available", sourceMarker: "accessibilityIdentifier(\"history.redo\")", when: "controller attached"),
                Control(name: "Retry the uncertain undo/redo", sourceMarker: "accessibilityIdentifier(\"history.retry\")", when: "after an uncertain reply"),
                Control(name: "Discard the uncertain undo/redo", sourceMarker: "accessibilityIdentifier(\"history.discard\")", when: "after an uncertain reply"),
                Control(name: "Undo and redo stacks (list)", sourceMarker: "accessibilityIdentifier(\"history.stacks\")"),
              ],
              sourceFile: "EditHistoryPanel.swift"),
        Panel(name: "Find in Project", windowTitle: "Find in Project", command: .findInProject,
              initialFocus: "the literal field",
              closing: "Esc or ⌘W closes the window; the editor text view is first responder again.",
              controls: [
                Control(name: "Literal to find in the project, case-sensitive", sourceMarker: "TextField(\"Find in project"),
                Control(name: "Search", sourceMarker: "Button(\"Search\")", when: "non-empty literal"),
                Control(name: "Go to Match", sourceMarker: "Button(\"Go to Match\")", when: "a match is selected"),
                Control(name: "Next Match", sourceMarker: "Button(\"Next Match\")", when: "matches"),
                Control(name: "Search scope", sourceMarker: "Picker(\"Scope\""),
                Control(name: "Max matches", sourceMarker: "Stepper(\"Max matches"),
                Control(name: "Search results, n matches (list)", sourceMarker: "List(selection: $client.selectedID)", when: "after a search"),
                Control(name: "Replacement text", sourceMarker: "TextField(\"Replace with"),
                Control(name: "Plan Replacement", sourceMarker: "Button(\"Plan Replacement\")", when: "complete search with matches"),
                Control(name: "Apply n replacements", sourceMarker: "Button(\"Apply \\(plan.summary)\")", when: "a planned proposal"),
                Control(name: "Retry path", sourceMarker: "Button(\"Retry \\(outcome.path)\")", when: "an uncertain apply"),
              ],
              sourceFile: "ProjectSearchPanel.swift"),
        Panel(name: "Nearby Companion", windowTitle: "Nearby Companion", command: .nearbyCompanion,
              initialFocus: "Show Pairing Code (Return); focus follows the pairing state: the code while it is shown or verified, Resume when interrupted, Dismiss on paired/error, Cancel while receiving",
              closing: "Esc cancels the current pairing step (or dismisses a banner); ⌘W closes the window; the editor text view is first responder again. The whole pairing is keyboard-only: Return shows or resumes a code, Esc cancels it.",
              controls: [
                Control(name: "Advertise on the local network (switch)", sourceMarker: "Toggle(\"Advertise\""),
                Control(name: "Dismiss", sourceMarker: "accessibilityIdentifier(\"nearby.pairing.dismiss\")", when: "after an error"),
                Control(name: "Show New Code", sourceMarker: "showCodeButton(title: \"Show New Code\")", when: "after an error"),
                Control(name: "Dismiss", sourceMarker: "accessibilityIdentifier(\"nearby.pairing.dismiss\")", when: "paired banner"),
                Control(name: "Cancel receiving", sourceMarker: "accessibilityLabel(\"Cancel receiving\")", when: "receiving a capture"),
                Control(name: "Show Pairing Code / Show New Code", sourceMarker: "Button(title) { controller.showCode() }", when: "idle, paired, error or expired"),
                Control(name: "Pairing code (spoken as digits in pairs; ⌘C copies it; a QR image of the same payload sits beside it)", sourceMarker: "accessibilityIdentifier(\"nearby.pairing.code\")", when: "code shown or verifying"),
                Control(name: "Copy code", sourceMarker: "Button(\"Copy code\")", when: "code shown or verifying"),
                Control(name: "Cancel pairing", sourceMarker: "accessibilityLabel(\"Cancel pairing\")", when: "code shown or verifying"),
                Control(name: "Resume", sourceMarker: "accessibilityIdentifier(\"nearby.pairing.resume\")", when: "interrupted, code still valid"),
                Control(name: "Cancel interrupted pairing", sourceMarker: "accessibilityLabel(\"Cancel interrupted pairing\")", when: "interrupted, code still valid"),
                Control(name: "Dismiss", sourceMarker: "accessibilityIdentifier(\"nearby.pairing.dismiss\")", when: "interrupted, code expired"),
                Control(name: "Show New Code", sourceMarker: "showCodeButton(title: \"Show New Code\")", when: "interrupted, code expired"),
                Control(name: "Permission for <companion> (pop-up: Captures allowed / View only)", sourceMarker: "Picker(\"Permission\"", when: "one per paired companion"),
                Control(name: "Forget <companion>", sourceMarker: "Button(\"Forget\")", when: "one per paired companion"),
                Control(name: "Clear refused captures", sourceMarker: "Button(\"Clear\")", when: "after a refused capture"),
              ],
              sourceFile: "NearbyView.swift"),
    ]

    public static var helpLines: [String] { panels.map(\.helpLine) }
}
