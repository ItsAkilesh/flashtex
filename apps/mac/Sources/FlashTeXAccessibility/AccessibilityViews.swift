import AppKit
import SwiftUI
import FlashTeXProtocol

/// Invisible, hit-test-free accessibility elements laid over one preview
/// page: a container per page, one element per line, one per item, in the
/// `AccessibleDocumentModel` reading order. Items with a source expose a
/// "Go to source" action that calls the same closure as a mouse click.
/// The visible rendering and mouse behavior are untouched.
public struct AccessibilityOverlay: View {
    let page: RuntimeV1.Page
    let totalPages: Int
    let scale: CGFloat
    let onSelect: (RuntimeV1.SourceRange?, String?) -> Void
    let lines: [AccessibleDocumentModel.Line]
    /// Elements in reading order with their frames; higher sort priority reads first.
    let slots: [Slot]

    struct Slot: Identifiable {
        let element: AccessibleDocumentModel.Element
        let frame: CGRect
        let priority: Double
        var id: Int { element.itemIndex }
    }

    /// - Parameters:
    ///   - page: the page being drawn.
    ///   - totalPages: page count for "Page 1 of 2" (0 = unknown).
    ///   - scale: display scale (1 = 1 pt per screen point).
    ///   - onSelect: the preview's click handler (source range, item text).
    public init(page: RuntimeV1.Page, totalPages: Int = 0, scale: CGFloat,
                onSelect: @escaping (RuntimeV1.SourceRange?, String?) -> Void) {
        self.page = page
        self.totalPages = totalPages
        self.scale = scale
        self.onSelect = onSelect
        let lines = AccessibleDocumentModel.lines(of: page, documents: [:], compiledDocuments: nil)
        self.lines = lines
        let ordered = lines.flatMap(\.elements)
        self.slots = ordered.enumerated().map { i, e in
            Slot(element: e, frame: Self.frame(for: e, in: page, scale: scale), priority: Double(ordered.count - i))
        }
    }

    public var body: some View {
        ZStack(alignment: .topLeading) {
            ForEach(slots) { slot in
                ElementView(element: slot.element, onSelect: onSelect)
                    .frame(width: slot.frame.width, height: slot.frame.height)
                    .position(x: slot.frame.midX, y: slot.frame.midY)
                    .accessibilitySortPriority(slot.priority)
            }
        }
        .frame(width: page.widthPt * scale, height: page.heightPt * scale, alignment: .topLeading)
        .allowsHitTesting(false)
        .accessibilityElement(children: .contain)
        .accessibilityLabel(pageLabel)
    }

    var pageLabel: String {
        AccessibleDocumentModel.PageSummary(number: page.number, lines: lines, totalPages: totalPages).label
    }

    /// Bounding box of an element in view points: measured with Times-Roman
    /// (the face the preview draws with) or the rule rectangle.
    static func frame(for element: AccessibleDocumentModel.Element, in page: RuntimeV1.Page, scale: CGFloat) -> CGRect {
        guard element.itemIndex < page.items.count, case .text(let item) = page.items[element.itemIndex] else { return .zero }
        if let r = RuleConvention.rect(for: item) {
            return CGRect(x: r.x * scale, y: r.y * scale, width: r.width * scale, height: max(1, r.height * scale))
        }
        let font = NSFont(name: "Times-Roman", size: item.fontSizePt) ?? NSFont.systemFont(ofSize: item.fontSizePt)
        let width = NSAttributedString(string: item.text, attributes: [.font: font]).size().width
        let top = item.baselineYPt - Double(font.ascender)
        let height = Double(font.ascender - font.descender)
        return CGRect(x: item.xPt * scale, y: top * scale, width: max(1, width * scale), height: max(1, height * scale))
    }

    private struct ElementView: View {
        let element: AccessibleDocumentModel.Element
        let onSelect: (RuntimeV1.SourceRange?, String?) -> Void

        var body: some View {
            let base = Color.clear
                .accessibilityElement(children: .ignore)
                .accessibilityLabel(element.label)
                .accessibilityValue(element.value)
                .accessibilityHint("Page \(element.page), line \(element.line)")
                .accessibilityAddTraits(.isStaticText)
            if let source = element.source {
                base.accessibilityAction(named: "Go to source") { onSelect(source, element.text) }
            } else {
                base
            }
        }
    }
}

public extension View {
    /// Makes a diagnostics-list row one element: "Diagnostic 1 of 2: Error:
    /// message" with the recovery/source value and a "Go to source" action
    /// when the diagnostic has a source. The visible row is unchanged.
    func accessibleDiagnostic(_ diagnostic: RuntimeV1.Diagnostic, index: Int, total: Int,
                              goToSource: @escaping () -> Void) -> some View {
        let element = AccessibleDocumentModel.DiagnosticElement(
            index: index, severity: diagnostic.severity, message: diagnostic.message,
            recovery: diagnostic.recovery, source: diagnostic.source, utf16Range: nil, lines: [])
        let base = accessibilityElement(children: .ignore)
            .accessibilityLabel("Diagnostic \(index + 1) of \(total): \(element.label)")
            .accessibilityValue(element.recovery.map { "recovery: \($0)" } ?? "no provisional rendering")
        return Group {
            if diagnostic.source != nil {
                base.accessibilityAction(named: "Go to source", goToSource)
            } else {
                base.accessibilityHint("No source mapping; listed only.")
            }
        }
    }

    /// Labels the capture bar as a group whose value reads the pinned
    /// insertion point and queued proposals; its buttons stay reachable.
    func accessibleCaptureBar(anchor: String?, proposals: Int) -> some View {
        accessibilityElement(children: .contain)
            .accessibilityLabel("Capture bar")
            .accessibilityValue((anchor.map { "Insertion point pinned: \($0)" } ?? "No insertion point pinned")
                                + "; \(proposals) proposal\(proposals == 1 ? "" : "s") to review")
    }
}

/// Renders the command table and focus order as a plain list, for an
/// "Accessibility help" window or sheet. Not attached by this target; a menu
/// item in `FlashTeXMacApp` can present it.
public struct AccessibilityHelpView: View {
    public init() {}

    public var body: some View {
        List {
            Section("Focus order: \(FocusOrder.description)") {
                ForEach(FocusOrder.helpLines, id: \.self) { Text($0).font(.callout) }
            }
            Section("Commands") {
                ForEach(AccessibilityCommand.entries, id: \.command) { e in
                    VStack(alignment: .leading, spacing: 2) {
                        Text("\(e.title) — \(e.shortcuts.joined(separator: " or "))").bold()
                        Text(e.description + (e.requires.map { " Requires \($0)." } ?? "")).font(.callout)
                    }
                    .accessibilityElement(children: .combine)
                }
            }
        }
    }
}
