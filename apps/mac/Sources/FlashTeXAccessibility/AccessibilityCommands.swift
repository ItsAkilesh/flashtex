import Foundation

/// The keyboard workflow as data: every app action a screen-reader user can
/// reach, its shortcut(s) exactly as the README table spells them, the menu it
/// lives in, and a discoverable description. The UI can render this as an
/// "Accessibility help" list; the test target checks it against the README.
public enum AccessibilityCommand: String, CaseIterable, Equatable {
    case openLaTeXFile, save, saveAs, openFixture, reloadFixture
    case attachBuiltCompiler, attachWorker, compile
    case exportPDF, exportPDFViaRust
    case pinInsertionPoint, openCaptureProposal, submitSampleCapture, convertCapture, nearbyCompanion
    case undo, completion
    case goToMatching, nextDiagnostic, previousDiagnostic, revealCaretInPreview
    case selectPreviewItemSource
    case nearbyCompanion

    public struct Entry: Equatable {
        public var command: AccessibilityCommand
        public var title: String
        /// Shortcut spellings as in the README ("⌘⇧]", "Esc", "⌃Space", "Click preview text").
        public var shortcuts: [String]
        public var menu: String
        public var description: String
        /// What must be true for the command to be enabled, or nil if always.
        public var requires: String?

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
                         description: "Opens a .tex file as the main.tex entry document and compiles it when a worker is attached.")
        case .save:
            return Entry(command: self, title: "Save", shortcuts: ["⌘S"], menu: "File",
                         description: "Saves the entry document as UTF-8; the editor header says “edited” while unsaved.")
        case .saveAs:
            return Entry(command: self, title: "Save As", shortcuts: ["⌘⇧S"], menu: "File",
                         description: "Saves the entry document under a new name.")
        case .openFixture:
            return Entry(command: self, title: "Open compile result fixture", shortcuts: ["⌘⇧O"], menu: "File",
                         description: "Loads a runtime v1 compile_result JSON into the preview; a sibling -request.json seeds the editor.")
        case .reloadFixture:
            return Entry(command: self, title: "Reload fixture", shortcuts: ["⌘R"], menu: "File",
                         description: "Reloads the current fixture from disk.")
        case .attachBuiltCompiler:
            return Entry(command: self, title: "Attach built compiler", shortcuts: ["⌘⇧K"], menu: "File",
                         description: "Attaches the FlashTeX compiler found via $FLASHTEX_COMPILER or crates/compiler/target.")
        case .attachWorker:
            return Entry(command: self, title: "Attach worker executable", shortcuts: ["⌘K"], menu: "File",
                         description: "Chooses any executable speaking runtime v1 JSON Lines and attaches it as the compiler.")
        case .compile:
            return Entry(command: self, title: "Compile now", shortcuts: ["⌘B"], menu: "File / toolbar",
                         description: "Sends the current buffers to the attached worker; auto-compile also runs 250 ms after edits.",
                         requires: "an attached worker")
        case .exportPDF:
            return Entry(command: self, title: "Export PDF", shortcuts: ["⌘⇧E"], menu: "File",
                         description: "Writes the current preview as a PDF with CoreGraphics (always white).",
                         requires: "a compile result")
        case .exportPDFViaRust:
            return Entry(command: self, title: "Export PDF via Rust writer", shortcuts: ["⌘⌥E"], menu: "File",
                         description: "Pipes the compile result to flashtex-pdf --verify (always white).",
                         requires: "a compile result")
        case .pinInsertionPoint:
            return Entry(command: self, title: "Pin insertion point", shortcuts: ["⌘⇧P"], menu: "Edit",
                         description: "Records the caret as the destination anchor for capture proposals; the capture bar reads it back.")
        case .openCaptureProposal:
            return Entry(command: self, title: "Open capture proposal", shortcuts: ["⌘⇧I"], menu: "Edit",
                         description: "Queues a capture_proposal file for review; Return in the sheet approves and inserts one undoable edit.")
        case .submitSampleCapture:
            return Entry(command: self, title: "Submit sample capture", shortcuts: ["⌘⇧U"], menu: "Edit",
                         description: "Sends a chosen PNG/JPEG as capture_submit through the attached bridge.",
                         requires: "an attached capture bridge")
        case .convertCapture:
            return Entry(command: self, title: "Convert capture", shortcuts: ["⌘⇧G"], menu: "Edit",
                         description: "Sends capture_convert for the latest received capture; the proposal opens for review.",
                         requires: "a received capture")
        case .nearbyCompanion:
            return Entry(command: self, title: "Nearby Companion", shortcuts: ["⌘⇧N"], menu: "Edit",
                         description: "Opens the window that advertises this Mac to a paired iPad/iPhone companion: pairing code, paired devices, received captures (nearby-v1 proposal).")
        case .undo:
            return Entry(command: self, title: "Undo", shortcuts: ["⌘Z"], menu: "Edit",
                         description: "Undoes the last edit, including an approved capture insertion.")
        case .completion:
            return Entry(command: self, title: "Completion popup", shortcuts: ["Esc", "⌃Space"], menu: "Editor",
                         description: "Lists supported commands, \\end{…} for open environments, labels, and document words; arrow keys choose, Return inserts.")
        case .goToMatching:
            return Entry(command: self, title: "Go to matching", shortcuts: ["⌘⇧D"], menu: "Navigate",
                         description: "Selects the matching \\begin/\\end or \\label/\\ref for the command under the caret; misses are explained in the footer.")
        case .nextDiagnostic:
            return Entry(command: self, title: "Next diagnostic", shortcuts: ["⌘⇧]"], menu: "Navigate",
                         description: "Selects the next diagnostic with a source in the active document (wrapping); refused if its span was edited since the compile.",
                         requires: "a compile result")
        case .previousDiagnostic:
            return Entry(command: self, title: "Previous diagnostic", shortcuts: ["⌘⇧["], menu: "Navigate",
                         description: "Selects the previous diagnostic with a source in the active document (wrapping).",
                         requires: "a compile result")
        case .revealCaretInPreview:
            return Entry(command: self, title: "Reveal caret in preview", shortcuts: ["⌘⇧J"], menu: "Navigate",
                         description: "Selects the source span of the preview item under the caret and names its page and item.",
                         requires: "a compile result")
        case .nearbyCompanion:
            return Entry(command: self, title: "Nearby Companion…", shortcuts: ["⌘⇧N"], menu: "Edit",
                         description: "Opens the window that advertises this Mac, shows the pairing code, lists paired devices and received captures.",
                         requires: "nothing")
        case .selectPreviewItemSource:
            return Entry(command: self, title: "Go to source of a preview item", shortcuts: ["Click preview text"], menu: "Preview",
                         description: "Selects the item's source in the editor; with VoiceOver, use the “Go to source” action on the item.",
                         requires: "a compile result")
        }
    }

    public static var entries: [Entry] { allCases.map(\.entry) }

    /// Every shortcut spelling in the table (README parity).
    public static var allShortcuts: Set<String> { Set(entries.flatMap(\.shortcuts)) }

    /// Lines for an "Accessibility help" list, in menu order.
    public static var helpLines: [String] { entries.map(\.helpLine) }
}

/// Keyboard focus order of the main window's panes and why it is that way.
public enum FocusOrder {
    public struct Pane: Equatable {
        public var name: String
        public var contents: String
        public var rationale: String
    }

    public static let panes: [Pane] = [
        Pane(name: "Editor",
             contents: "Source text view (document picker and byte/UTF-16 counts above it).",
             rationale: "Editing is the primary task; the caret drives caret sync, diagnostics at caret, and every Navigate command."),
        Pane(name: "Preview",
             contents: "Pages in reading order; each line and item is an accessibility element with a “Go to source” action.",
             rationale: "Follows the editor so a user can check what the last edit produced, page by page, without leaving the keyboard."),
        Pane(name: "Diagnostics",
             contents: "List of the compile result's diagnostics: severity, message, recovery note, source bytes, “Go to source”.",
             rationale: "Comes after the preview because the preview is still shown when errors exist; diagnostics refine, not replace, it."),
        Pane(name: "Capture bar",
             contents: "Pin insertion point, the pinned anchor, and the review button for queued proposals.",
             rationale: "Last because capture review is an occasional, explicit workflow that begins from the editor caret."),
    ]

    /// "Editor → Preview → Diagnostics → Capture bar"
    public static var description: String { panes.map(\.name).joined(separator: " → ") }

    /// Full help text: order plus rationale for each pane.
    public static var helpLines: [String] {
        panes.enumerated().map { i, p in "\(i + 1). \(p.name): \(p.contents) \(p.rationale)" }
    }
}
