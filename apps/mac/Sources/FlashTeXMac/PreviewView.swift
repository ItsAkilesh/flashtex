import SwiftUI
import FlashTeXProtocol

/// Draws `compile_result` pages. Coordinates are points, origin top-left;
/// text items are positioned by baseline. Clicking a text item navigates to
/// its UTF-8 source range.
struct PreviewView: View {
    let result: RuntimeV1.CompileResult
    let dark: Bool
    /// Items under the editor caret, `page number -> item indices` (see `CaretSync`).
    var caretItems: [Int: Set<Int>] = [:]
    let onSelect: (RuntimeV1.SourceRange?, String?) -> Void

    /// First page holding a caret item, or nil; drives page-level auto-scroll.
    private var caretPage: Int? { caretItems.filter { !$0.value.isEmpty }.keys.min() }

    var body: some View {
        GeometryReader { geo in
        ScrollViewReader { proxy in
            let widest = result.pages.map(\.widthPt).max() ?? 612
            // Fit the widest page to the pane (never upscale past 100%).
            let scale = min(1, max(0.2, (geo.size.width - 48) / widest))
            ScrollView([.vertical, .horizontal]) {
                VStack(spacing: 24) {
                    ForEach(result.pages, id: \.number) { page in
                        PageView(page: page, dark: dark, caretItems: caretItems[page.number] ?? [], scale: scale, onSelect: onSelect)
                            .id(page.number)
                    }
                }
                .padding(24)
            }
            .onChange(of: caretPage) { _, page in
                // Page-level only: keeps the page under the caret in view when the
                // editor moves across pages; no scrolling within a page.
                if let page { withAnimation { proxy.scrollTo(page, anchor: .top) } }
            }
        }
        }
        .background(dark ? Color(white: 0.12) : Color(nsColor: .windowBackgroundColor))
    }
}

private struct PageView: View {
    let page: RuntimeV1.Page
    let dark: Bool
    var caretItems: Set<Int> = []
    /// Display scale (1 = 1pt per screen point); the preview fits pages to width.
    var scale: CGFloat = 1.0
    let onSelect: (RuntimeV1.SourceRange?, String?) -> Void

    var body: some View {
        let size = CGSize(width: page.widthPt * scale, height: page.heightPt * scale)
        HitTestCanvas(page: page, dark: dark, scale: scale, caretItems: caretItems, onSelect: onSelect)
            .frame(width: size.width, height: size.height)
            .background(dark ? Color(white: 0.16) : .white)
            .shadow(radius: 4)
            .overlay(alignment: .bottomTrailing) {
                Text("page \(page.number)")
                    .font(.caption2).foregroundStyle(.secondary).padding(4)
            }
    }
}

private struct HitTestCanvas: View {
    let page: RuntimeV1.Page
    let dark: Bool
    let scale: CGFloat
    /// Indices into `page.items` to mark as containing the editor caret.
    var caretItems: Set<Int> = []
    let onSelect: (RuntimeV1.SourceRange?, String?) -> Void

    /// Hit rects keyed by `page.items` index so hover, click, and caret
    /// highlights agree even when non-text items are interleaved.
    @State private var hitRects: [(index: Int, rect: CGRect, source: RuntimeV1.SourceRange?, text: String)] = []
    @State private var hover: Int?

    var body: some View {
        Canvas { context, _ in
            var rects: [(index: Int, rect: CGRect, source: RuntimeV1.SourceRange?, text: String)] = []
            for (index, item) in page.items.enumerated() {
                guard case .text(let t) = item else { continue }
                if let r = RuleConvention.rect(for: t) {
                    let rect = CGRect(x: r.x * scale, y: r.y * scale, width: r.width * scale, height: max(0.5, r.height * scale))
                    context.fill(Path(rect), with: .color(dark ? .white : .black))
                    rects.append((index, rect, t.source, t.text))
                    continue
                }
                // Times-Roman: the face the compiler measured with. `.serif` design
                // would be New York, which is wider and made words run together.
                let font = Font.custom("Times-Roman", size: t.fontSizePt * scale)
                var text = Text(t.text).font(font)
                text = text.foregroundColor(dark ? .white : .black)
                let resolved = context.resolve(text)
                let measured = resolved.measure(in: CGSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude))
                let baseline = resolved.firstBaseline(in: measured)
                let origin = CGPoint(x: t.xPt * scale, y: t.baselineYPt * scale - baseline)
                let rect = CGRect(origin: origin, size: measured)
                if caretItems.contains(index) {
                    // Secondary (caret) highlight: subtle fill plus an underline.
                    context.fill(Path(rect.insetBy(dx: -2, dy: -1)),
                                 with: .color(Color.accentColor.opacity(0.15)))
                    let y = rect.maxY + 1
                    var underline = Path()
                    underline.move(to: CGPoint(x: rect.minX, y: y))
                    underline.addLine(to: CGPoint(x: rect.maxX, y: y))
                    context.stroke(underline, with: .color(Color.accentColor), lineWidth: 1.5)
                }
                if hover == index {
                    context.fill(Path(rect.insetBy(dx: -2, dy: -1)),
                                 with: .color(Color.accentColor.opacity(0.25)))
                }
                context.draw(resolved, at: origin, anchor: .topLeading)
                rects.append((index, rect, t.source, t.text))
            }
            DispatchQueue.main.async { hitRects = rects }
        }
        .contentShape(Rectangle())
        .onContinuousHover { phase in
            switch phase {
            case .active(let p): hover = hitRects.first { $0.rect.contains(p) }?.index
            case .ended: hover = nil
            }
        }
        .onTapGesture { location in
            if let hit = hitRects.first(where: { $0.rect.contains(location) }) {
                onSelect(hit.source, hit.text)
            }
        }
    }
}
