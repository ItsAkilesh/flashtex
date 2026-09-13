import AppKit

/// Signature help: while the caret sits inside an argument of `\cmd`, a small
/// non-activating panel shows the command's argument pattern (from
/// `Completion.Vocabulary`) with the current argument emphasised and the
/// one-line documentation (`EditorIntelligence.CommandDocs`, falling back to
/// the vocabulary description). The model (`info`) is pure and bounded to
/// the caret's line; the panel is opened by `CompletingTextView` after a `{`
/// or `[` that follows a command name, or on ⌘⇧Space, and dismissed by `}`,
/// Esc or the caret leaving the argument.
enum SignatureHelp {
    struct Info: Equatable {
        /// Command name without the backslash.
        let command: String
        /// Argument pattern, e.g. `{num}{den}` or `[options]{class}`; empty when unknown.
        let arguments: String
        /// 0-based index of the argument group the caret is in (`[…]` and `{…}` groups both count).
        let activeArgument: Int
        /// One-line documentation.
        let description: String
        /// UTF-16 index of the unmatched opener the caret is inside; the help
        /// is dismissed once the caret is at or before it.
        let openerUTF16: Int

        /// Argument groups of `arguments` in order, e.g. `["{num}", "{den}"]`.
        var groups: [String] { SignatureHelp.groups(of: arguments) }

        /// Display text `\cmd{num}{den}` (or `\cmd` when the pattern is unknown)
        /// and the UTF-16 range of the active group inside it (nil when the
        /// caret is past the pattern's last group).
        var display: (text: String, active: NSRange?) {
            var text = "\\" + command
            var active: NSRange?
            for (i, g) in groups.enumerated() {
                if i == activeArgument { active = NSRange(location: (text as NSString).length, length: (g as NSString).length) }
                text += g
            }
            return (text, active)
        }
    }

    /// Scan limit backwards from the caret (bytes of the caret's line).
    static let scanLimit = 2048

    static func groups(of arguments: String) -> [String] {
        var out: [String] = []
        var current = ""
        var depth = 0
        for c in arguments {
            if c == "{" || c == "[" { if depth == 0 { current = "" }; depth += 1 }
            current.append(c)
            if c == "}" || c == "]" { depth -= 1; if depth == 0 { out.append(current); current = "" } }
        }
        return out
    }

    /// The command whose argument the caret is in, or nil when the caret is
    /// not inside an unmatched `{`/`[` on its line, the group is not an
    /// argument of a control word, or nothing is known about that command.
    static func info(in text: String, caretUTF16: Int) -> Info? {
        let ns = text as NSString
        guard caretUTF16 >= 0, caretUTF16 <= ns.length else { return nil }
        var lineStart = caretUTF16
        while lineStart > 0, caretUTF16 - lineStart < scanLimit, ns.character(at: lineStart - 1) != 0x0A { lineStart -= 1 }
        func isEscaped(_ i: Int) -> Bool {
            var n = 0
            var j = i - 1
            while j >= lineStart, ns.character(at: j) == 0x5C { n += 1; j -= 1 } // `\`
            return n % 2 == 1
        }
        // 1. The unmatched opener before the caret.
        var depth = 0
        var opener: Int?
        var i = caretUTF16 - 1
        while i >= lineStart {
            let c = ns.character(at: i)
            if c == 0x25, !isEscaped(i) { return nil } // `%`: the caret is in a comment
            if (c == 0x7D || c == 0x5D), !isEscaped(i) { depth += 1 } // `}` `]`
            else if (c == 0x7B || c == 0x5B), !isEscaped(i) { // `{` `[`
                if depth == 0 { opener = i; break }
                depth -= 1
            }
            i -= 1
        }
        guard let opener else { return nil }
        for k in lineStart..<opener where ns.character(at: k) == 0x25 && !isEscaped(k) { return nil } // commented out
        // 2. Closed argument groups between the command name and this opener.
        var closed = 0
        var j = opener - 1
        while true {
            while j >= lineStart, ns.character(at: j) == 0x20 || ns.character(at: j) == 0x09 { j -= 1 }
            guard j >= lineStart, ns.character(at: j) == 0x7D || ns.character(at: j) == 0x5D, !isEscaped(j) else { break }
            var d = 0
            var k = j
            var matched = false
            while k >= lineStart {
                let c = ns.character(at: k)
                if (c == 0x7D || c == 0x5D), !isEscaped(k) { d += 1 }
                else if (c == 0x7B || c == 0x5B), !isEscaped(k) { d -= 1; if d == 0 { matched = true; break } }
                k -= 1
            }
            guard matched else { return nil }
            closed += 1
            j = k - 1
        }
        // 3. The control word: letters ending at j, preceded by `\`.
        let nameEnd = j + 1
        var nameStart = nameEnd
        while nameStart > lineStart {
            let c = ns.character(at: nameStart - 1)
            guard (c >= 0x41 && c <= 0x5A) || (c >= 0x61 && c <= 0x7A) else { break }
            nameStart -= 1
        }
        // `\begin{env}{…}`: the environment's own arguments are not documented here.
        guard nameStart < nameEnd, nameStart > lineStart, ns.character(at: nameStart - 1) == 0x5C, !isEscaped(nameStart - 1) else { return nil }
        let name = ns.substring(with: NSRange(location: nameStart, length: nameEnd - nameStart))
        let entry = Completion.Vocabulary.byName[name]
        let doc = EditorIntelligence.CommandDocs.documentation(for: name) ?? entry?.description
        guard entry != nil || doc != nil else { return nil }
        let arguments = entry?.arguments ?? ""
        return Info(command: name, arguments: arguments, activeArgument: closed,
                    description: doc ?? entry?.description ?? "", openerUTF16: opener)
    }
}

/// The signature-help panel: one line for the pattern, one for the doc.
@MainActor
final class SignatureHelpPanel: NSPanel {
    static let width: CGFloat = 420
    static let height: CGFloat = 44
    private let pattern = NSTextField(labelWithString: "")
    private let doc = NSTextField(labelWithString: "")
    private(set) var info: SignatureHelp.Info?

    init() {
        super.init(contentRect: NSRect(x: 0, y: 0, width: Self.width, height: Self.height),
                   styleMask: [.nonactivatingPanel, .borderless], backing: .buffered, defer: false)
        isFloatingPanel = true
        hidesOnDeactivate = false
        level = .popUpMenu
        hasShadow = true
        isReleasedWhenClosed = false
        isExcludedFromWindowsMenu = true
        animationBehavior = .none
        let content = NSVisualEffectView(frame: NSRect(x: 0, y: 0, width: Self.width, height: Self.height))
        content.material = .popover
        content.state = .active
        content.wantsLayer = true
        content.layer?.cornerRadius = 6
        pattern.font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        pattern.lineBreakMode = .byTruncatingTail
        pattern.frame = NSRect(x: 10, y: Self.height - 22, width: Self.width - 20, height: 17)
        pattern.autoresizingMask = [.width]
        pattern.setAccessibilityIdentifier("signature-help-pattern")
        content.addSubview(pattern)
        doc.font = NSFont.systemFont(ofSize: 11)
        doc.textColor = .secondaryLabelColor
        doc.lineBreakMode = .byTruncatingTail
        doc.frame = NSRect(x: 10, y: 4, width: Self.width - 20, height: 15)
        doc.autoresizingMask = [.width]
        doc.setAccessibilityIdentifier("signature-help-doc")
        content.addSubview(doc)
        contentView = content
        setAccessibilityLabel("Signature help")
    }

    /// Attributed pattern: the command bold, the active argument in the accent colour.
    static func attributed(_ info: SignatureHelp.Info) -> NSAttributedString {
        let (text, active) = info.display
        let s = NSMutableAttributedString(string: text, attributes: [
            .font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular), .foregroundColor: NSColor.labelColor,
        ])
        s.addAttribute(.font, value: NSFont.monospacedSystemFont(ofSize: 12, weight: .bold),
                       range: NSRange(location: 0, length: (info.command as NSString).length + 1))
        if let active {
            s.addAttributes([.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .bold),
                             .foregroundColor: NSColor.controlAccentColor,
                             .underlineStyle: NSUnderlineStyle.single.rawValue], range: active)
        }
        return s
    }

    var patternText: String { pattern.stringValue }
    var docText: String { doc.stringValue }

    func show(_ info: SignatureHelp.Info, above caretRect: NSRect, parent: NSWindow) {
        self.info = info
        pattern.attributedStringValue = Self.attributed(info)
        doc.stringValue = info.description
        var origin = NSPoint(x: caretRect.minX - 6, y: caretRect.maxY + 4)
        if let screen = parent.screen ?? NSScreen.main {
            let visible = screen.visibleFrame
            if origin.y + Self.height > visible.maxY { origin.y = caretRect.minY - Self.height - 4 } // flip below
            origin.x = min(max(origin.x, visible.minX), max(visible.minX, visible.maxX - Self.width))
        }
        let target = NSRect(origin: origin, size: NSSize(width: Self.width, height: Self.height))
        if frame != target { setFrame(target, display: false) }
        if self.parent !== parent {
            self.parent?.removeChildWindow(self)
            parent.addChildWindow(self, ordered: .above)
        }
        if !isVisible { orderFront(nil) }
    }

    func hide() {
        info = nil
        if parent != nil { parent?.removeChildWindow(self) }
        if isVisible { orderOut(nil) }
    }
}
