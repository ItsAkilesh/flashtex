import AppKit
import SwiftUI
import FlashTeXProtocol

// Experimental "v2 preview": paints a rendering-v2 display list (opened from a
// `flashtex-render --v2` JSON file) through `GlyphRunRenderer` and navigates
// from clusters to source bytes. Off by default; the runtime-v1 preview stays
// the product path. A list that fails validation or font resolution shows its
// diagnostic and NO page — never a partial frame.

/// What the shell holds for the v2 pane: a fully prepared frame or the refusal.
enum V2PreviewState {
    case loaded(V2Frame, URL)
    case failed(RenderingV2.ValidationError, URL)

    var url: URL { switch self { case .loaded(_, let u), .failed(_, let u): u } }
    var frame: V2Frame? { if case .loaded(let f, _) = self { f } else { nil } }
}

extension ShellModel {
    /// `File > Open Display List (v2)…`
    func openDisplayListV2Panel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.message = "Choose a rendering-v2 display_list JSON file (flashtex-render --v2)"
        if panel.runModal() == .OK, let url = panel.url { loadDisplayListV2(url: url) }
    }

    /// Decodes, validates and resolves fonts; on any failure the pane shows the
    /// diagnostic and the previous frame (if any) is dropped, not kept as current.
    func loadDisplayListV2(url: URL) {
        do {
            let envelope = try RenderingV2.decode(try Data(contentsOf: url))
            let frame = try V2Frame.prepare(envelope)
            displayListV2 = .loaded(frame, url)
            previewV2 = true
            captureNote = "Loaded display list \(url.lastPathComponent): \(frame.list.pages.count) page(s), \(frame.fonts.count) font(s) resolved by content hash"
        } catch let error as RenderingV2.ValidationError {
            displayListV2 = .failed(error, url)
            previewV2 = true
            captureNote = "Display list refused: \(error)"
        } catch {
            displayListV2 = .failed(RenderingV2.ValidationError(code: "io_error", message: error.localizedDescription), url)
            previewV2 = true
        }
    }

    /// Click in the v2 preview → source selection. The display list's document
    /// digest must match the current buffer (the list attests exactly which
    /// bytes it laid out); then `navigate(to:expectedText:)` applies the
    /// shell's own stale-revision refusal and rebase verification.
    func navigateV2(_ hit: V2Geometry.Hit) {
        if let reason = hit.syntheticReason {
            navigationNote = "Generated content (\(reason)) has no source range."
            return
        }
        guard let source = hit.sources.first else {
            navigationNote = "This item has no source mapping."
            return
        }
        guard let frame = displayListV2?.frame else { return }
        guard let declared = frame.list.documents.first(where: { $0.path == source.path }) else {
            navigationNote = "Display list does not declare document \(source.path)."
            return
        }
        guard let doc = documents.first(where: { $0.path == source.path }) else {
            navigationNote = "No open document named \(source.path)."
            return
        }
        guard SourceDigest.sha256Hex(doc.text) == declared.sha256 else {
            navigationNote = "Display list is for \(source.path) revision \(declared.revision) (sha256 \(declared.sha256.prefix(12))…), which differs from the current buffer; regenerate the display list to navigate."
            return
        }
        navigate(to: source, expectedText: hit.text)
        if hit.sources.count > 1, navigationNote?.hasPrefix("Selected") == true {
            navigationNote! += " (+\(hit.sources.count - 1) more source range(s) for this cluster)"
        }
    }

    /// `Export PDF (v2)…` from the v2 pane: the same draw routine as the preview.
    func exportPDFV2() {
        guard let frame = displayListV2?.frame else {
            captureNote = "Nothing to export: no display list loaded."
            return
        }
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.pdf]
        panel.nameFieldStringValue = "\(frame.list.projectId)-r\(frame.list.revision)-v2.pdf"
        panel.message = "Export the v2 display list as PDF through the preview's draw routine (experimental)"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try GlyphRunRenderer.pdfData(frame: frame).write(to: url, options: .atomic)
            captureNote = "Exported \(frame.list.pages.count) page(s) (v2) to \(url.path)"
        } catch {
            captureNote = "PDF export (v2) failed: \(error.localizedDescription)"
        }
    }
}

/// The v2 preview pane: header, pages or the refusal, and the list's diagnostics.
struct PreviewV2Pane: View {
    @EnvironmentObject var model: ShellModel

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            switch model.displayListV2 {
            case .loaded(let frame, _):
                PreviewV2View(frame: frame, dark: model.darkPreview, caretPath: model.activePath, caretByte: model.caretByte) { hit in
                    model.navigateV2(hit)
                }
                let diags = frame.list.diagnostics
                if !diags.isEmpty {
                    Divider()
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Display list diagnostics (\(diags.count))").font(.caption.bold())
                        ForEach(Array(diags.enumerated()), id: \.offset) { _, d in
                            HStack(alignment: .top) {
                                Image(systemName: d.severity == .error ? "xmark.octagon.fill" : "exclamationmark.triangle.fill")
                                    .foregroundStyle(d.severity == .error ? .red : .orange)
                                Text("[\(d.code)] \(d.message)").font(.caption)
                                if let s = d.sources.first { Button("Go to source") { model.navigate(to: s) }.controlSize(.mini) }
                            }
                        }
                    }
                    .padding(8).frame(maxWidth: .infinity, alignment: .leading)
                }
            case .failed(let error, let url):
                ContentUnavailableView {
                    Label("Display list refused — nothing rendered", systemImage: "xmark.octagon")
                } description: {
                    Text("\(url.lastPathComponent)\n[\(error.code)] \(error.message)")
                        .textSelection(.enabled)
                }
                .accessibilityIdentifier("v2-refusal")
            case nil:
                ContentUnavailableView("No v2 display list loaded", systemImage: "doc.richtext",
                                       description: Text("Use File > Open Display List (v2)… with a flashtex-render --v2 JSON file."))
            }
        }
        .onAppear {
            // Automation hook: FLASHTEX_V2_FILE seeds the pane at launch.
            if model.displayListV2 == nil, let path = ProcessInfo.processInfo.environment["FLASHTEX_V2_FILE"] {
                model.loadDisplayListV2(url: URL(fileURLWithPath: path))
            }
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 8) {
                Text("V2").font(.caption.bold()).padding(.horizontal, 6).padding(.vertical, 2).background(Color.purple.opacity(0.25), in: Capsule())
                Text("experimental display-list-v2 — not the default path").font(.caption).foregroundStyle(.secondary).lineLimit(1)
                Spacer()
                Button("Open…") { model.openDisplayListV2Panel() }.controlSize(.small).fixedSize()
                Button("Export PDF (v2)…") { model.exportPDFV2() }.controlSize(.small).fixedSize().disabled(model.displayListV2?.frame == nil)
            }
            if case .loaded(let frame, let url) = model.displayListV2 {
                let fonts = frame.fonts.values.map { "\($0.resource.postscriptName) \($0.resource.sha256.prefix(8))" }.sorted().joined(separator: ", ")
                Text("\(url.lastPathComponent) · id \(frame.id) · project \(frame.list.projectId) · revision \(frame.list.revision) · \(frame.list.pages.count) page(s) · fonts by hash: \(fonts)")
                    .font(.caption).foregroundStyle(.secondary).lineLimit(1).truncationMode(.middle)
                    .help(frame.fonts.values.map { "\($0.resource.postscriptName): \($0.resource.sha256) → \($0.file.url.lastPathComponent) (\($0.hashConvention))" }.sorted().joined(separator: "\n"))
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 4)
        .background(.bar)
    }
}

/// Scrollable pages of a prepared frame, fit to the pane width.
struct PreviewV2View: View {
    let frame: V2Frame
    let dark: Bool
    let caretPath: String
    let caretByte: Int?
    let onSelect: (V2Geometry.Hit) -> Void

    var body: some View {
        GeometryReader { geo in
            let widest = frame.list.pages.map(\.widthPt).max() ?? 612
            let scale = min(1, max(0.2, (geo.size.width - 48) / widest))
            ScrollView([.vertical, .horizontal]) {
                VStack(spacing: 24) {
                    ForEach(frame.list.pages, id: \.number) { page in
                        PageV2View(page: page, frame: frame, dark: dark, scale: scale,
                                   caretMatches: caretByte.map { V2Geometry.clusters(containing: $0, path: caretPath, in: page) } ?? [],
                                   onSelect: onSelect)
                            .id(page.number)
                    }
                }
                .padding(24)
            }
        }
        .background(dark ? Color(white: 0.12) : Color(nsColor: .windowBackgroundColor))
    }
}

private struct PageV2View: View {
    let page: RenderingV2.Page
    let frame: V2Frame
    let dark: Bool
    let scale: CGFloat
    let caretMatches: [V2Geometry.CaretMatch]
    let onSelect: (V2Geometry.Hit) -> Void
    @State private var hover: V2Geometry.Hit?

    private func viewRect(_ r: RenderingV2.Rect) -> CGRect {
        CGRect(x: RenderingV2.points(r.x) * scale, y: RenderingV2.points(r.top) * scale,
               width: RenderingV2.points(r.width) * scale, height: RenderingV2.points(r.height) * scale)
    }

    var body: some View {
        let size = CGSize(width: page.widthPt * scale, height: page.heightPt * scale)
        Canvas(rendersAsynchronously: false) { context, _ in
            // Caret highlight: exact caret bar when the compiler supplied one for
            // that byte, else the whole cluster's hit rectangles (documented fallback).
            for m in caretMatches {
                if let k = m.caret {
                    let bar = CGRect(x: RenderingV2.points(k.x) * scale - 0.75, y: RenderingV2.points(k.top) * scale, width: 1.5, height: RenderingV2.points(k.height) * scale)
                    context.fill(Path(bar), with: .color(Color.accentColor))
                    for r in m.hitRects { context.fill(Path(viewRect(r)), with: .color(Color.accentColor.opacity(0.10))) }
                } else {
                    for r in m.hitRects { context.fill(Path(viewRect(r)), with: .color(Color.accentColor.opacity(0.22))) }
                }
            }
            if let hover { context.fill(Path(viewRect(hover.rect).insetBy(dx: -1, dy: -1)), with: .color(Color.accentColor.opacity(0.25))) }
            // Flip the canvas into PDF space (y up, 1 unit = 1 pt) and paint through
            // the same routine the PDF export uses.
            context.withCGContext { cg in
                cg.saveGState()
                cg.translateBy(x: 0, y: size.height)
                cg.scaleBy(x: scale, y: -scale)
                GlyphRunRenderer.draw(page: page, frame: frame, in: cg, dark: dark)
                cg.restoreGState()
            }
        }
        .frame(width: size.width, height: size.height)
        .background(dark ? Color(white: 0.16) : .white)
        .shadow(radius: 4)
        .contentShape(Rectangle())
        .onContinuousHover { phase in
            switch phase {
            case .active(let p): hover = V2Geometry.hit(page: page, atPointX: p.x / scale, y: p.y / scale)
            case .ended: hover = nil
            }
        }
        .onTapGesture { location in
            if let hit = V2Geometry.hit(page: page, atPointX: location.x / scale, y: location.y / scale) { onSelect(hit) }
        }
        .overlay(alignment: .bottomTrailing) {
            Text("page \(page.number) · v2").font(.caption2).foregroundStyle(.secondary).padding(4)
        }
        .help(hover.map { h in
            (h.text.map { "“\($0)” → " } ?? "rule → ") + (h.syntheticReason.map { "generated: \($0)" }
                ?? h.sources.map { "\($0.path) \($0.startByte)..<\($0.endByte)" }.joined(separator: ", "))
        } ?? "")
    }
}
