import SwiftUI
import FlashTeXProtocol

/// Draws `compile_result` pages. Coordinates are points, origin top-left;
/// text items are positioned by baseline. Clicking a text item navigates to
/// its UTF-8 source range.
struct PreviewView: View {
    let result: RuntimeV1.CompileResult
    let dark: Bool
    let onSelect: (RuntimeV1.SourceRange?) -> Void

    var body: some View {
        ScrollView([.vertical, .horizontal]) {
            VStack(spacing: 24) {
                ForEach(result.pages, id: \.number) { page in
                    PageView(page: page, dark: dark, onSelect: onSelect)
                }
            }
            .padding(24)
        }
        .background(dark ? Color(white: 0.12) : Color(nsColor: .windowBackgroundColor))
    }
}

private struct PageView: View {
    let page: RuntimeV1.Page
    let dark: Bool
    let onSelect: (RuntimeV1.SourceRange?) -> Void

    /// Fixed display scale: 1pt = 1 screen point at 100%.
    private let scale: CGFloat = 1.0

    var body: some View {
        let size = CGSize(width: page.widthPt * scale, height: page.heightPt * scale)
        HitTestCanvas(page: page, dark: dark, scale: scale, onSelect: onSelect)
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
    let onSelect: (RuntimeV1.SourceRange?) -> Void

    @State private var hitRects: [(CGRect, RuntimeV1.SourceRange?)] = []
    @State private var hover: Int?

    var body: some View {
        Canvas { context, _ in
            var rects: [(CGRect, RuntimeV1.SourceRange?)] = []
            for (index, item) in page.items.enumerated() {
                guard case .text(let t) = item else { continue }
                let font = Font.system(size: t.fontSizePt * scale, design: .serif)
                var text = Text(t.text).font(font)
                text = text.foregroundColor(dark ? .white : .black)
                let resolved = context.resolve(text)
                let measured = resolved.measure(in: CGSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude))
                let baseline = resolved.firstBaseline(in: measured)
                let origin = CGPoint(x: t.xPt * scale, y: t.baselineYPt * scale - baseline)
                let rect = CGRect(origin: origin, size: measured)
                if hover == index {
                    context.fill(Path(rect.insetBy(dx: -2, dy: -1)),
                                 with: .color(Color.accentColor.opacity(0.25)))
                }
                context.draw(resolved, at: origin, anchor: .topLeading)
                rects.append((rect, t.source))
            }
            DispatchQueue.main.async { hitRects = rects }
        }
        .contentShape(Rectangle())
        .onContinuousHover { phase in
            switch phase {
            case .active(let p): hover = hitRects.firstIndex { $0.0.contains(p) }
            case .ended: hover = nil
            }
        }
        .onTapGesture { location in
            if let hit = hitRects.first(where: { $0.0.contains(location) }) {
                onSelect(hit.1)
            }
        }
    }
}
