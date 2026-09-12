import XCTest
@testable import FlashTeXAccessibility

/// The command table must match the README's "Keyboard shortcuts" table exactly
/// (both directions), the menu wiring in the shell source, and the focus order
/// must describe the pane order `ContentView.swift` actually builds.
final class CommandTableTests: XCTestCase {
    static let macRoot = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
    static let readme = macRoot.appendingPathComponent("README.md")
    static let shellSources = macRoot.appendingPathComponent("Sources/FlashTeXMac")

    struct ReadmeRow: Equatable {
        var shortcuts: [String]
        var action: String
    }

    /// Rows of the README table: shortcut cell split on " / ", action cell.
    func readmeRows() throws -> [ReadmeRow] {
        let text = try String(contentsOf: Self.readme, encoding: .utf8)
        guard let section = text.range(of: "## Keyboard shortcuts") else { throw XCTSkip("README has no shortcuts table") }
        var out: [ReadmeRow] = []
        for line in text[section.upperBound...].split(separator: "\n", omittingEmptySubsequences: false) {
            if line.hasPrefix("## ") { break }
            guard line.hasPrefix("|") else { continue }
            let cells = line.split(separator: "|", omittingEmptySubsequences: false).map { $0.trimmingCharacters(in: .whitespaces) }
            guard cells.count >= 3, cells[1] != "Shortcut", !cells[1].hasPrefix("---") else { continue }
            out.append(ReadmeRow(shortcuts: cells[1].components(separatedBy: " / "), action: cells[2]))
        }
        return out
    }

    func readmeShortcuts() throws -> [String] { try readmeRows().flatMap(\.shortcuts) }

    func testCommandTableMatchesREADMEShortcuts() throws {
        let readme = try readmeShortcuts()
        XCTAssertGreaterThan(readme.count, 15, "table parsed")
        XCTAssertEqual(Set(readme).count, readme.count, "README shortcuts are unique")
        let table = AccessibilityCommand.allShortcuts
        XCTAssertEqual(Set(readme).subtracting(table), [], "README shortcuts missing from the command table")
        XCTAssertEqual(table.subtracting(readme), [], "command-table shortcuts not documented in the README")
        // Each shortcut belongs to exactly one command.
        let owners = AccessibilityCommand.entries.flatMap { e in e.shortcuts.map { ($0, e.command) } }
        XCTAssertEqual(owners.count, Set(owners.map(\.0)).count)
    }

    /// Every command is one README row (all of its shortcuts in that row) and
    /// every README row is claimed by at least one command: a new menu item
    /// documented in the README without a table entry fails here, and so does
    /// a table entry nobody documented.
    func testEveryCommandHasAREADMERowAndEveryRowACommand() throws {
        let rows = try readmeRows()
        var rowOf: [AccessibilityCommand: Int] = [:]
        for e in AccessibilityCommand.entries {
            let matching = rows.indices.filter { i in e.shortcuts.allSatisfy { rows[i].shortcuts.contains($0) } }
            XCTAssertEqual(matching.count, 1, "\(e.command) \(e.shortcuts) should sit in exactly one README row")
            if let i = matching.first { rowOf[e.command] = i }
        }
        let claimed = Set(rowOf.values)
        for (i, row) in rows.enumerated() {
            XCTAssertTrue(claimed.contains(i), "README row \(row.shortcuts) has no AccessibilityCommand")
        }
        // The row must name what the help calls the command (its title's first
        // word appears in the row's shortcut or action cell), so the help text
        // and the README agree on what each shortcut is for.
        for e in AccessibilityCommand.entries {
            guard let i = rowOf[e.command] else { continue }
            let first = e.title.split(separator: " ").first.map(String.init)?.lowercased() ?? ""
            let haystack = (rows[i].shortcuts.joined(separator: " ") + " " + rows[i].action).lowercased()
            XCTAssertTrue(haystack.contains(first), "\(e.command): README says “\(rows[i].action)”, help says “\(e.title)”")
        }
    }

    // MARK: menu wiring

    struct WiredItem: Equatable {
        var title: String
        var menu: String
        /// README spelling ("⌘⇧K") or nil when the item has no key equivalent.
        var shortcut: String?
    }

    /// `Button("Title")` items with their `keyboardShortcut` and enclosing
    /// menu from `FlashTeXMacApp.swift` (File = `replacing: .newItem`,
    /// Edit = `after: .pasteboard`) and `Navigation.swift` (`CommandMenu("Navigate")`).
    func wiredItems() throws -> [WiredItem] {
        var out: [WiredItem] = []
        for file in ["FlashTeXMacApp.swift", "Navigation.swift"] {
            let text = try String(contentsOf: Self.shellSources.appendingPathComponent(file), encoding: .utf8)
            var menu = "?"
            let lines = text.components(separatedBy: "\n")
            var i = 0
            while i < lines.count {
                let line = lines[i]
                if line.contains("CommandGroup(replacing: .newItem)") { menu = "File" }
                else if line.contains("CommandGroup(after: .pasteboard)") { menu = "Edit" }
                else if line.contains("CommandMenu(\"Navigate\")") { menu = "Navigate" }
                else if line.contains("CommandGroup(replacing: .help)") || line.contains("CommandGroup(after: .help)") { menu = "Help" }
                else if line.contains(" Window(\"") || line.contains("WindowGroup(\"") { menu = "?" }
                if let r = line.range(of: "Button(\""), let end = line.range(of: "\")", range: r.upperBound..<line.endIndex) {
                    let raw = String(line[r.upperBound..<end.lowerBound])
                    let title = raw.replacingOccurrences(of: "\\\\", with: "\\")
                    // Modifiers follow on the next lines until the next Button/Divider.
                    var shortcut: String?
                    var j = i + 1
                    while j < lines.count, !lines[j].contains("Button("), !lines[j].contains("Divider()"), !lines[j].contains("CommandGroup"), !lines[j].contains("CommandMenu") {
                        if let s = Self.spell(keyboardShortcutLine: lines[j]) { shortcut = s }
                        j += 1
                    }
                    out.append(WiredItem(title: title, menu: menu, shortcut: shortcut))
                }
                i += 1
            }
        }
        return out
    }

    /// `.keyboardShortcut("k", modifiers: [.command, .shift])` → "⌘⇧K"; `.keyboardShortcut("o")` → "⌘O".
    static func spell(keyboardShortcutLine line: String) -> String? {
        guard let r = line.range(of: ".keyboardShortcut(\"") else { return nil }
        let rest = line[r.upperBound...]
        guard let q = rest.firstIndex(of: "\"") else { return nil }
        let key = String(rest[..<q]).uppercased()
        var mods = "⌘"
        if let m = rest.range(of: "modifiers: [") {
            let list = rest[m.upperBound...]
            mods = ""
            if list.contains(".control") { mods += "⌃" }
            if list.contains(".command") { mods += "⌘" }
            if list.contains(".option") { mods += "⌥" }
            if list.contains(".shift") { mods += "⇧" }
        }
        return mods + key
    }

    func testMenuItemsMatchTheShellWiring() throws {
        let wired = try wiredItems()
        XCTAssertGreaterThan(wired.count, 20, "menu items parsed from the shell source")
        for e in AccessibilityCommand.entries {
            guard let item = e.menuItem else {
                // Undo is the system Edit item; completion and the preview click are not menu items.
                XCTAssertTrue([.undo, .completion, .selectPreviewItemSource].contains(e.command), "\(e.command) has no menu item")
                continue
            }
            let matches = wired.filter { $0.title == item }
            XCTAssertEqual(matches.count, 1, "\(e.command): Button(\"\(item)\") wired once in the shell")
            guard let w = matches.first else { continue }
            XCTAssertTrue(e.menu.hasPrefix(w.menu), "\(e.command): table says menu \(e.menu), shell wires it under \(w.menu)")
            if let key = w.shortcut {
                XCTAssertEqual(e.shortcuts, [key], "\(e.command): shell key equivalent")
            } else {
                XCTAssertEqual(e.shortcuts, ["\(w.menu) > \(item)"], "\(e.command): no key equivalent, so the README names the menu path")
            }
            // "Requires" claims a .disabled() modifier and vice versa.
            let file = w.menu == "Navigate" ? "Navigation.swift" : "FlashTeXMacApp.swift"
            let text = try String(contentsOf: Self.shellSources.appendingPathComponent(file), encoding: .utf8)
            let escaped = item.replacingOccurrences(of: "\\", with: "\\\\")
            let start = try XCTUnwrap(text.range(of: "Button(\"\(escaped)\")"))
            let tail = text[start.upperBound...]
            let next = tail.range(of: "Button(")?.lowerBound ?? tail.endIndex
            let disabled = tail[..<next].contains(".disabled(")
            XCTAssertEqual(e.requires != nil, disabled, "\(e.command): help says requires \(e.requires ?? "nothing"), wiring \(disabled ? "has" : "has no") .disabled()")
        }
        // Every shell item with a key equivalent is in the table (menu items
        // without one, e.g. Detach Worker, are not keyboard workflow steps).
        let tableItems = Set(AccessibilityCommand.entries.compactMap(\.menuItem))
        for w in wired where w.shortcut != nil {
            XCTAssertTrue(tableItems.contains(w.title), "shell item “\(w.title)” (\(w.shortcut!)) is missing from AccessibilityCommand")
        }
    }

    func testEntriesAreDiscoverable() {
        for e in AccessibilityCommand.entries {
            XCTAssertFalse(e.title.isEmpty)
            XCTAssertFalse(e.shortcuts.isEmpty, e.title)
            XCTAssertFalse(e.menu.isEmpty, e.title)
            XCTAssertGreaterThan(e.description.count, 20, e.title)
            XCTAssertTrue(e.helpLine.hasPrefix(e.title + " — "), e.helpLine)
        }
        XCTAssertEqual(AccessibilityCommand.helpLines.count, AccessibilityCommand.allCases.count)
        XCTAssertEqual(AccessibilityCommand.nextDiagnostic.entry.helpLine,
                       "Next diagnostic — ⌘⇧] (Navigate): Selects the next diagnostic with a source in the active document (wrapping); refused if its span was edited since the compile. Requires a compile result.")
        XCTAssertEqual(AccessibilityCommand.completion.entry.shortcuts, ["Esc", "⌃Space"])
        XCTAssertEqual(AccessibilityCommand.helpLines, AccessibilityCommand.helpLines, "deterministic")
    }

    // MARK: focus order

    /// Body text of `struct Name` in `ContentView.swift`, up to the next top-level struct.
    static func structBody(_ name: String, in text: String) throws -> Substring {
        let start = try XCTUnwrap(text.range(of: "struct \(name): View"), "\(name) exists in ContentView.swift")
        let rest = text[start.upperBound...]
        let end = rest.range(of: "\nprivate struct ")?.lowerBound ?? rest.range(of: "\nstruct ")?.lowerBound ?? rest.endIndex
        return rest[..<end]
    }

    func testFocusOrderMatchesContentViewPaneOrder() throws {
        let text = try String(contentsOf: Self.shellSources.appendingPathComponent("ContentView.swift"), encoding: .utf8)
        // 1. The split view places the editor column before the preview column.
        let root = try Self.structBody("ContentView", in: text)
        let editor = try XCTUnwrap(root.range(of: "EditorPane()"))
        let preview = try XCTUnwrap(root.range(of: "PreviewPane()"))
        XCTAssertLessThan(editor.lowerBound, preview.lowerBound)
        XCTAssertTrue(root[..<editor.lowerBound].contains("HSplitView"))
        // 2. Within each column the markers appear in the table's order, and
        //    the table lists the columns in the split view's order.
        let containers = ["EditorPane", "PreviewPane"]
        var flattened: [String] = []
        for c in containers {
            let body = try Self.structBody(c, in: text)
            let panes = FocusOrder.panes.filter { $0.container == c }
            XCTAssertFalse(panes.isEmpty, c)
            var cursor = body.startIndex
            for p in panes {
                let r = try XCTUnwrap(body.range(of: p.sourceMarker, range: cursor..<body.endIndex),
                                      "\(p.name): \(p.sourceMarker) after the previous pane in \(c)")
                cursor = r.upperBound
                flattened.append(p.name)
            }
        }
        XCTAssertEqual(flattened, FocusOrder.panes.map(\.name), "table order is the view order")
        XCTAssertEqual(Set(FocusOrder.panes.map(\.container)), Set(containers))
        // 3. No focusable pane in the source is missing from the table.
        for marker in ["SourceEditorView(", "CaptureBar()", "BridgeBar()", "PreviewView(", "diagnosticsList("] {
            XCTAssertTrue(FocusOrder.panes.contains { $0.sourceMarker == marker }, marker)
        }
        XCTAssertEqual(FocusOrder.description, "Editor → Capture bar → Bridge bar → Preview → Diagnostics")
    }

    func testFocusOrderHelpText() {
        XCTAssertEqual(FocusOrder.panes.count, 5)
        for p in FocusOrder.panes {
            XCTAssertGreaterThan(p.rationale.count, 30, p.name)
            XCTAssertGreaterThan(p.contents.count, 20, p.name)
        }
        XCTAssertTrue(FocusOrder.helpLines[0].hasPrefix("1. Editor: "))
        XCTAssertTrue(FocusOrder.helpLines[4].hasPrefix("5. Diagnostics: "))
        XCTAssertEqual(FocusOrder.statusLines.count, 3)
    }

    // MARK: help window

    func testHelpViewCoversEveryCommandAndMenu() {
        let menus = AccessibilityHelpView.menus
        XCTAssertEqual(menus.map(\.menu), ["File", "File / toolbar", "Edit", "Editor", "Navigate", "Preview", "Help"])
        XCTAssertEqual(menus.flatMap(\.entries).count, AccessibilityCommand.allCases.count)
        XCTAssertEqual(AccessibilityHelpView.windowID, "a11y-help")
        XCTAssertEqual(AccessibilityHelpView.menuItem, "FlashTeX Accessibility Help")
        // The VoiceOver notes cover each focusable pane and the completion popup.
        let notes = AccessibilityHelpView.voiceOverNotes.joined(separator: "\n")
        for p in FocusOrder.panes where p.name != "Bridge bar" { XCTAssertTrue(notes.contains(p.name + ":"), p.name) }
        XCTAssertTrue(notes.contains("Completion popup"))
        XCTAssertTrue(notes.contains(CompletionAccessibility.listLabel))
        XCTAssertTrue(notes.contains(PreviewAccessibility.goToSourceAction))
        XCTAssertTrue(notes.contains(DiagnosticRowAccessibility.noSourceHint))
    }
}
