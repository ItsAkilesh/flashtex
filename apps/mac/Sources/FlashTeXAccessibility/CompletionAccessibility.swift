import Foundation

/// What VoiceOver reads for the editor's completion popup, as pure text the
/// shell's `CompletionPopup` applies and the test target checks. The popup is
/// a non-activating panel (it never takes focus from the editor), so the
/// selection change is what a screen-reader user hears: one announcement per
/// selected candidate, "n of m" first so the list size is known.
public enum CompletionAccessibility {
    /// Mirrors the shell's `Completion.Kind`; raw values are the wire/category names.
    public enum Kind: String, CaseIterable, Equatable {
        case command, environment, reference, citation, word

        /// Spoken kind: "label" and "citation" are what the rows are, not
        /// the parser's names for them.
        public var spoken: String {
            switch self {
            case .command: return "command"
            case .environment: return "environment"
            case .reference: return "label"
            case .citation: return "citation"
            case .word: return "word"
            }
        }
    }

    /// Accessibility label of the popup's table.
    public static let listLabel = "Completions"

    /// Help text of the table (how to use it from the keyboard).
    public static let listHelp = "Completion candidates for the word at the caret. Up and Down arrows choose, Return inserts, Escape closes."

    /// One row: "\\section, command, supported by this compiler". The label
    /// is spelled out so VoiceOver does not swallow the backslash: "\\end{itemize}"
    /// reads as "backslash end, brace, itemize, close brace" only when the
    /// text is left as is, so the label text is unchanged and the kind and
    /// origin follow after commas.
    public static func rowLabel(label: String, kind: Kind, detail: String) -> String {
        "\(label), \(kind.spoken), \(detail)"
    }

    /// Announcement for a (re)selected candidate: "3 of 7: \\section, command, supported by this compiler".
    public static func selectionAnnouncement(index: Int, total: Int, label: String, kind: Kind, detail: String) -> String {
        "\(index + 1) of \(total): " + rowLabel(label: label, kind: kind, detail: detail)
    }

    /// Announcement when the popup opens with candidates: "7 completions".
    public static func openedAnnouncement(total: Int) -> String {
        "\(total) completion\(total == 1 ? "" : "s")"
    }
}
