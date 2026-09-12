import AppKit
import SwiftUI
import FlashTeXProtocol

/// Invisible, hit-test-free accessibility elements laid over one preview
/// page: a container per page, one element per item, in the
/// `AccessibleDocumentModel` reading order. Items with a source expose a
/// "Go to source" action that calls the same closure as a mouse click.
/// The visible rendering and mouse behavior are untouched.
///
/// The elements are `NSAccessibilityElement`s built lazily inside one
/// `NSView` the first time an assistive client asks for the page's children
/// (and rebuilt after the page changes). A SwiftUI `ForEach` of ~1000
/// per-item views cost ~200 ms of diffing on every compile result; the lazy
/// tree costs nothing per keystroke unless VoiceOver is reading the page.
public struct AccessibilityOverlay: View {
    let page: RuntimeV1.Page
    let totalPages: Int
    let scale: CGFloat
    let fontName: (Double) -> String
    let onSelect: (RuntimeV1.SourceRange?, String?) -> Void

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
    ///   - fontName: PostScript name of the face the preview draws a given
    ///     point size with (default Times-Roman); used only to size AX frames.
    ///   - onSelect: the preview's click handler (source range, item text).
    public init(page: RuntimeV1.Page, totalPages: Int = 0, scale: CGFloat,
                fontName: @escaping (Double) -> String = { _ in "Times-Roman" },
                onSelect: @escaping (RuntimeV1.SourceRange?, String?) -> Void) {
        self.page = page
        self.totalPages = totalPages
        self.scale = scale
        self.fontName = fontName
        self.onSelect = onSelect
    }

    /// Reading-order lines of the page (computed on demand, not per render).
    var lines: [AccessibleDocumentModel.Line] {
        AccessibleDocumentModel.lines(of: page, documents: [:], compiledDocuments: nil)
    }

    /// Elements in reading order with their frames; higher sort priority reads first.
    var slots: [Slot] { Self.slots(page: page, scale: scale, fontName: fontName, lines: lines) }

    static func slots(page: RuntimeV1.Page, scale: CGFloat, fontName: (Double) -> String,
                      lines: [AccessibleDocumentModel.Line]) -> [Slot] {
        let ordered = lines.flatMap(\.elements)
        return ordered.enumerated().map { i, e in
            Slot(element: e, frame: frame(for: e, in: page, scale: scale, fontName: fontName),
                 priority: Double(ordered.count - i))
        }
    }

    public var body: some View {
        PageAccessibilityView(page: page, totalPages: totalPages, scale: scale, fontName: fontName, onSelect: onSelect)
            .frame(width: page.widthPt * scale, height: page.heightPt * scale, alignment: .topLeading)
            .allowsHitTesting(false)
    }

    var pageLabel: String {
        AccessibleDocumentModel.PageSummary(number: page.number, lines: lines, totalPages: totalPages).label
    }

    /// Bounding box of an element in view points: measured with the preview's
    /// face (Times-Roman unless told otherwise) or the rule rectangle.
    static func frame(for element: AccessibleDocumentModel.Element, in page: RuntimeV1.Page, scale: CGFloat,
                      fontName: (Double) -> String = { _ in "Times-Roman" }) -> CGRect {
        guard element.itemIndex < page.items.count, case .text(let item) = page.items[element.itemIndex] else { return .zero }
        if let r = RuleConvention.rect(for: item) {
            return CGRect(x: r.x * scale, y: r.y * scale, width: r.width * scale, height: max(1, r.height * scale))
        }
        let font = NSFont(name: fontName(item.fontSizePt), size: item.fontSizePt)
            ?? NSFont(name: "Times-Roman", size: item.fontSizePt) ?? NSFont.systemFont(ofSize: item.fontSizePt)
        let width = NSAttributedString(string: item.text, attributes: [.font: font]).size().width
        let top = item.baselineYPt - Double(font.ascender)
        let height = Double(font.ascender - font.descender)
        return CGRect(x: item.xPt * scale, y: top * scale, width: max(1, width * scale), height: max(1, height * scale))
    }
}

/// Hosts `PageAXView`; updates hand it the new page and drop the cached tree.
private struct PageAccessibilityView: NSViewRepresentable {
    let page: RuntimeV1.Page
    let totalPages: Int
    let scale: CGFloat
    let fontName: (Double) -> String
    let onSelect: (RuntimeV1.SourceRange?, String?) -> Void

    func makeNSView(context: Context) -> PageAXView { PageAXView() }

    func updateNSView(_ view: PageAXView, context: Context) {
        view.update(page: page, totalPages: totalPages, scale: scale, fontName: fontName, onSelect: onSelect)
    }
}

/// The page container: a group whose children are one static-text element
/// per item, created only when asked for. Never hit-tested, never drawn.
final class PageAXView: NSView {
    private var page: RuntimeV1.Page?
    private var totalPages = 0
    private var scale: CGFloat = 1
    private var fontName: (Double) -> String = { _ in "Times-Roman" }
    private var onSelect: (RuntimeV1.SourceRange?, String?) -> Void = { _, _ in }
    private var cached: [NSAccessibilityElement]?
    private var cachedLabel: String?

    override var isFlipped: Bool { true }
    override func hitTest(_ point: NSPoint) -> NSView? { nil }
    override var isOpaque: Bool { false }

    func update(page: RuntimeV1.Page, totalPages: Int, scale: CGFloat,
                fontName: @escaping (Double) -> String, onSelect: @escaping (RuntimeV1.SourceRange?, String?) -> Void) {
        let changed = self.page != page || self.totalPages != totalPages || self.scale != scale
        self.page = page; self.totalPages = totalPages; self.scale = scale
        self.fontName = fontName; self.onSelect = onSelect
        guard changed else { return }
        let hadTree = cached != nil
        cached = nil; cachedLabel = nil
        // Only a client that already read this page needs to hear about the change.
        if hadTree { NSAccessibility.post(element: self, notification: .layoutChanged) }
    }

    override func isAccessibilityElement() -> Bool { true }
    override func accessibilityRole() -> NSAccessibility.Role? { .group }
    override func accessibilityLabel() -> String? {
        if let cachedLabel { return cachedLabel }
        guard let page else { return nil }
        let lines = AccessibleDocumentModel.lines(of: page, documents: [:], compiledDocuments: nil)
        let label = AccessibleDocumentModel.PageSummary(number: page.number, lines: lines, totalPages: totalPages).label
        cachedLabel = label
        return label
    }

    override func accessibilityChildren() -> [Any]? { elements() }

    /// Builds the per-item elements on first request; `AccessibilityOverlay.slots`
    /// defines the order and frames so the two stay identical.
    private func elements() -> [NSAccessibilityElement] {
        if let cached { return cached }
        guard let page else { return [] }
        let lines = AccessibleDocumentModel.lines(of: page, documents: [:], compiledDocuments: nil)
        let slots = AccessibilityOverlay.slots(page: page, scale: scale, fontName: fontName, lines: lines)
        let onSelect = self.onSelect
        let out: [NSAccessibilityElement] = slots.map { slot in
            let e = slot.element
            let ax = NSAccessibilityElement.element(withRole: .staticText, frame: slot.frame, label: e.label, parent: self)
                as! NSAccessibilityElement
            ax.setAccessibilityValue(e.value)
            ax.setAccessibilityHelp("Page \(e.page), line \(e.line)")
            if let source = e.source {
                ax.setAccessibilityCustomActions([
                    NSAccessibilityCustomAction(name: "Go to source") { onSelect(source, e.text); return true },
                ])
            }
            return ax
        }
        cached = out
        return out
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
            recovery: diagnostic.recovery, source: diagnostic.source, utf16Range: nil, textKnown: false, lines: [])
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
