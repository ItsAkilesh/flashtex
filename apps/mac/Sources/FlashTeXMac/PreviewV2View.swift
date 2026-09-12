import AppKit
import SwiftUI
import FlashTeXProtocol

// Experimental "v2 preview": paints a rendering-v2 display list (opened from a
// `flashtex-render --v2` JSON file) through `GlyphRunRenderer` and navigates
// from clusters to source bytes. Off by default; the runtime-v1 preview stays
// the product path. A list that fails validation or font resolution shows its
// diagnostic and NO page — never a partial frame.
//
// Threading: decoding, validation, font resolution and page preparation run
// on `V2Loader.queue` (serial, off-main); every result carries the load
// ticket it was started with and is dropped on the main thread if a newer
// load has since started (stale results never overwrite a newer state).
// Page bitmaps are rasterized off-main by `V2PageRasterizer` and installed
// only while their frame is still the one on screen.

/// What the shell holds for the v2 pane.
enum V2PreviewState {
    /// A load is in flight. `previous` is the frame still on screen, shown
    /// with an explicit stale indicator until the new one is verified
    /// (rendering-v2 proposal: retain the old frame, label it stale).
    case loading(URL, ticket: Int, previous: V2Frame?)
    case loaded(V2Frame, URL)
    case failed(RenderingV2.ValidationError, URL)

    var url: URL {
        switch self {
        case .loading(let u, _, _), .loaded(_, let u), .failed(_, let u): u
        }
    }
    /// The verified frame the pane may paint (a stale one while loading).
    var frame: V2Frame? {
        switch self {
        case .loaded(let f, _): f
        case .loading(_, _, let previous): previous
        case .failed: nil
        }
    }
    var isLoading: Bool { if case .loading = self { true } else { false } }
    var ticket: Int? { if case .loading(_, let t, _) = self { t } else { nil } }
}

/// Off-main preparation of display lists with monotonically increasing load
/// tickets. `ShellModel.loadDisplayListV2` starts a load; the result reaches
/// the main run loop (same delivery as `WorkerClient`: a run-loop block plus
/// wake-up, not a plain dispatch) and is published only if its ticket is
/// still the one the shell is waiting for.
enum V2Loader {
    static let queue = DispatchQueue(label: "flashtex.preview-v2.prepare", qos: .userInitiated)
    private static let lock = NSLock()
    private static var nextTicket = 1
    /// Results dropped because a newer load superseded them (tests/evidence).
    private(set) static var staleResultsDropped = 0
    private(set) static var resultsPublished = 0

    static func issueTicket() -> Int { lock.lock(); defer { lock.unlock() }; let t = nextTicket; nextTicket += 1; return t }
    static func noteDropped() { lock.lock(); staleResultsDropped += 1; lock.unlock() }
    static func notePublished() { lock.lock(); resultsPublished += 1; lock.unlock() }

    enum Outcome {
        case loaded(V2Frame)
        case failed(RenderingV2.ValidationError)
    }

    /// Pure preparation: file → decoded, validated, font-resolved, page-prepared frame.
    static func prepare(url: URL, store: V2FontStore = .shared) -> Outcome {
        do {
            let data = try Data(contentsOf: url)
            let envelope = try RenderingV2.decode(data)
            return .loaded(try V2Frame.prepare(envelope, store: store))
        } catch let error as RenderingV2.ValidationError {
            return .failed(error)
        } catch {
            return .failed(RenderingV2.ValidationError(code: "io_error", message: error.localizedDescription))
        }
    }

    /// Runs `block` at the head of the next main run-loop iteration.
    static func deliverOnMain(_ block: @escaping @Sendable () -> Void) {
        CFRunLoopPerformBlock(CFRunLoopGetMain(), CFRunLoopMode.commonModes.rawValue, block)
        CFRunLoopWakeUp(CFRunLoopGetMain())
    }
}

extension ShellModel {
    /// `File > Open Display List (v2)…`
    func openDisplayListV2Panel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.message = "Choose a rendering-v2 display_list JSON file (flashtex-render --v2)"
        if panel.runModal() == .OK, let url = panel.url { loadDisplayListV2(url: url) }
    }

    /// Starts an off-main load. The previous verified frame (if any) stays on
    /// screen with a stale indicator until this load finishes; a result whose
    /// ticket is no longer current is dropped. `completion` runs on the main
    /// thread after the state was (or was not) published.
    func loadDisplayListV2(url: URL, completion: (() -> Void)? = nil) {
        let ticket = V2Loader.issueTicket()
        displayListV2 = .loading(url, ticket: ticket, previous: displayListV2?.frame)
        previewV2 = true
        captureNote = "Loading display list \(url.lastPathComponent)…"
        V2Loader.queue.async {
            let outcome = V2Loader.prepare(url: url)
            V2Loader.deliverOnMain {
                MainActor.assumeIsolated {
                    self.deliverDisplayListV2(ticket: ticket, url: url, outcome: outcome)
                    completion?()
                }
            }
        }
    }

    /// Publishes a prepared result only if `ticket` is the load the shell is
    /// still waiting for. Anything else (a superseded load, or the pane was
    /// reset) is dropped: a stale frame never overwrites a newer state.
    /// Returns whether the result was published.
    @discardableResult
    func deliverDisplayListV2(ticket: Int, url: URL, outcome: V2Loader.Outcome) -> Bool {
        guard displayListV2?.ticket == ticket else {
            V2Loader.noteDropped()
            FlashTeXLog.write("preview-v2: dropped stale load result ticket \(ticket) for \(url.lastPathComponent) (current: \(displayListV2?.ticket.map(String.init) ?? "none"))")
            return false
        }
        V2Loader.notePublished()
        switch outcome {
        case .loaded(let frame):
            displayListV2 = .loaded(frame, url)
            captureNote = "Loaded display list \(url.lastPathComponent): \(frame.list.pages.count) page(s), \(frame.fonts.count) font(s) resolved by content hash, \(frame.prepared.reduce(0) { $0 + $1.glyphCount }) glyphs prepared"
            V2ParityEvidence.runIfRequested(frame: frame, url: url)
        case .failed(let error):
            // A refusal drops the previous frame: nothing verified is on screen.
            displayListV2 = .failed(error, url)
            captureNote = "Display list refused: \(error)"
        }
        return true
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
        guard case .loaded(let frame, _)? = displayListV2 else {
            captureNote = displayListV2?.isLoading == true ? "Nothing to export yet: a display list is still loading." : "Nothing to export: no display list loaded."
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

/// Off-main page bitmaps for the pane, keyed by frame identity, page, scale
/// and appearance. A page is rasterized once through
/// `GlyphRunRenderer.rasterize` (the same routine and bitmap configuration
/// the export parity check compares) and then only blitted, so hover/caret
/// repaints never re-run glyph drawing on the main thread. Bitmaps for a
/// frame that is no longer current are dropped when they arrive (stale paint
/// suppression) and evicted when the frame changes. Retention is bounded in
/// bytes; the least recently requested pages go first.
@MainActor
@Observable
final class V2PageRasterizer {
    struct Key: Hashable {
        var frameToken: String
        var page: Int
        /// Pixels per PDF point (display scale × backing scale).
        var pixelsPerPoint: Double
        var dark: Bool
    }

    static let shared = V2PageRasterizer()
    static let queue = DispatchQueue(label: "flashtex.preview-v2.raster", qos: .userInteractive)

    /// Ready bitmaps; reading this from a view body subscribes it to arrivals.
    private(set) var images: [Key: CGImage] = [:]
    @ObservationIgnored private var order: [Key] = []
    @ObservationIgnored private var inFlight: Set<Key> = []
    /// Observed: a page body that asked before the frame became current
    /// re-evaluates (and re-requests) when `setCurrent` runs.
    private(set) var currentFrameToken: String?
    @ObservationIgnored private(set) var retainedBytes = 0
    @ObservationIgnored let maxBytes: Int
    @ObservationIgnored private(set) var staleBitmapsDropped = 0
    @ObservationIgnored private(set) var rasterizations = 0

    init(maxBytes: Int = 192 << 20) { self.maxBytes = maxBytes }

    /// Marks `frame` as the one on screen; bitmaps of any other frame are
    /// evicted now and dropped if still in flight.
    func setCurrent(frameToken: String?) {
        guard currentFrameToken != frameToken else { return }
        currentFrameToken = frameToken
        for key in order where key.frameToken != frameToken { drop(key) }
        order.removeAll { $0.frameToken != frameToken }
    }

    /// The bitmap for `page` if ready; otherwise starts rasterizing it
    /// off-main and returns nil (the page paints its background until the
    /// bitmap arrives on the next run-loop turn).
    func image(for page: V2PreparedPage, frameToken: String, pixelsPerPoint: Double, dark: Bool) -> CGImage? {
        let key = Key(frameToken: frameToken, page: page.number, pixelsPerPoint: pixelsPerPoint, dark: dark)
        if let image = images[key] {
            touch(key)
            return image
        }
        guard frameToken == currentFrameToken, !inFlight.contains(key) else { return nil }
        inFlight.insert(key)
        Self.queue.async {
            let image = GlyphRunRenderer.rasterize(page, scale: pixelsPerPoint, dark: dark)
            V2Loader.deliverOnMain {
                MainActor.assumeIsolated { self.install(image, for: key) }
            }
        }
        return nil
    }

    /// Installs an arrived bitmap unless its frame is no longer current.
    func install(_ image: CGImage?, for key: Key) {
        inFlight.remove(key)
        rasterizations += 1
        guard key.frameToken == currentFrameToken, let image else {
            staleBitmapsDropped += 1
            FlashTeXLog.write("preview-v2: dropped stale page bitmap \(key.page) of frame \(key.frameToken.prefix(24)) (current: \(currentFrameToken?.prefix(24) ?? "none"))")
            return
        }
        images[key] = image
        order.removeAll { $0 == key }
        order.append(key)
        retainedBytes += image.bytesPerRow * image.height
        while retainedBytes > maxBytes, let oldest = order.first, oldest != key {
            order.removeFirst()
            drop(oldest)
        }
    }

    func clear() { for key in order { drop(key) }; order.removeAll() }

    private func touch(_ key: Key) {
        if let i = order.firstIndex(of: key) { order.remove(at: i); order.append(key) }
    }

    private func drop(_ key: Key) {
        if let image = images.removeValue(forKey: key) { retainedBytes -= image.bytesPerRow * image.height }
    }
}

/// `FLASHTEX_V2_PARITY_OUT=<dir>`: after every successful load, run the
/// zero-tolerance export-versus-preview comparison off-main and write
/// `parity.json`, `export.pdf` and per-page `page-N-preview.png` /
/// `page-N-export.png` there (evidence capture; never in the product path).
/// `FLASHTEX_V2_PARITY_SCALE` sets pixels per point (default 2).
enum V2ParityEvidence {
    static func runIfRequested(frame: V2Frame, url: URL) {
        guard let dir = ProcessInfo.processInfo.environment["FLASHTEX_V2_PARITY_OUT"], !dir.isEmpty else { return }
        let scale = Double(ProcessInfo.processInfo.environment["FLASHTEX_V2_PARITY_SCALE"] ?? "") ?? 2
        V2Loader.queue.async {
            let out = URL(fileURLWithPath: dir)
            try? FileManager.default.createDirectory(at: out, withIntermediateDirectories: true)
            let report = V2Parity.compare(frame: frame, scale: scale) { page in
                write(page.preview, to: out.appendingPathComponent("page-\(page.page)-preview.png"))
                write(page.export, to: out.appendingPathComponent("page-\(page.page)-export.png"))
            }
            try? GlyphRunRenderer.pdfData(frame: frame).write(to: out.appendingPathComponent("export.pdf"))
            let encoder = JSONEncoder(); encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            var object: [String: Any] = [:]
            if let data = try? encoder.encode(report), let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] { object = json }
            object["source"] = url.path
            object["identical"] = report.identical
            object["total_differing_pixels"] = report.totalDifferingPixels
            if let data = try? JSONSerialization.data(withJSONObject: object, options: [.prettyPrinted, .sortedKeys]) {
                try? data.write(to: out.appendingPathComponent("parity.json"))
            }
            FlashTeXLog.write("preview-v2: parity \(report.identical ? "identical" : "DIFFERS") — \(report.totalDifferingPixels) differing pixel(s) over \(report.pages.count) page(s) at \(scale)px/pt → \(out.path)")
        }
    }

    static func write(_ image: CGImage, to url: URL) {
        let rep = NSBitmapImageRep(cgImage: image)
        try? rep.representation(using: .png, properties: [:])?.write(to: url)
    }
}

/// The v2 preview pane: header, pages or the refusal, and the list's diagnostics.
struct PreviewV2Pane: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            switch model.displayListV2 {
            case .loaded(let frame, _):
                pages(frame, stale: false)
                diagnostics(frame)
            case .loading(_, _, let previous):
                if let previous {
                    pages(previous, stale: true)
                } else {
                    ContentUnavailableView("Loading display list…", systemImage: "hourglass")
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

    private func pages(_ frame: V2Frame, stale: Bool) -> some View {
        PreviewV2View(frame: frame, dark: model.darkPreview, stale: stale, caretPath: model.activePath, caretByte: model.caretByte) { hit in
            model.navigateV2(hit)
        }
    }

    @ViewBuilder
    private func diagnostics(_ frame: V2Frame) -> some View {
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
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 8) {
                Text("V2").font(.caption.bold()).padding(.horizontal, 6).padding(.vertical, 2).background(Color.purple.opacity(0.25), in: Capsule())
                Text("experimental display-list-v2 — not the default path").font(.caption).foregroundStyle(.secondary).lineLimit(1)
                if case .loading(let url, _, let previous) = model.displayListV2 {
                    ProgressView().controlSize(.small)
                    Text(previous == nil ? "loading \(url.lastPathComponent)…" : "STALE — showing the previous frame while \(url.lastPathComponent) is verified")
                        .font(.caption.bold()).foregroundStyle(.orange).lineLimit(1)
                        .accessibilityIdentifier("v2-stale")
                }
                Spacer()
                Button("Open…") { model.openDisplayListV2Panel() }.controlSize(.small).fixedSize()
                Button("Export PDF (v2)…") { model.exportPDFV2() }.controlSize(.small).fixedSize()
                    .disabled({ if case .loaded = model.displayListV2 { false } else { true } }())
            }
            if let frame = model.displayListV2?.frame {
                let fonts = frame.fonts.values.map { "\($0.resource.postscriptName) \($0.resource.sha256.prefix(8))" }.sorted().joined(separator: ", ")
                Text("\(model.displayListV2?.url.lastPathComponent ?? "") · id \(frame.id) · project \(frame.list.projectId) · revision \(frame.list.revision) · \(frame.list.pages.count) page(s) · fonts by hash: \(fonts)")
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
    var stale = false
    let caretPath: String
    let caretByte: Int?
    let onSelect: (V2Geometry.Hit) -> Void
    @Environment(\.displayScale) private var displayScale

    /// Unique per prepared frame instance: a reload of the same file makes a
    /// new frame whose bitmaps must not be confused with the old one's.
    private var frameToken: String { V2FrameIdentity.token(frame) }

    var body: some View {
        GeometryReader { geo in
            let widest = frame.list.pages.map(\.widthPt).max() ?? 612
            let scale = min(1, max(0.2, (geo.size.width - 48) / widest))
            ScrollView([.vertical, .horizontal]) {
                LazyVStack(spacing: 24) {
                    ForEach(frame.prepared, id: \.number) { prepared in
                        if let page = frame.page(number: prepared.number) {
                            PageV2View(page: page, prepared: prepared, frameToken: frameToken, dark: dark, stale: stale, scale: scale, displayScale: displayScale,
                                       caretMatches: caretByte.map { V2Geometry.clusters(containing: $0, path: caretPath, in: page) } ?? [],
                                       onSelect: onSelect)
                                .id(page.number)
                        }
                    }
                }
                .padding(24)
            }
        }
        .background(dark ? Color(white: 0.12) : Color(nsColor: .windowBackgroundColor))
        .onAppear { V2PageRasterizer.shared.setCurrent(frameToken: frameToken) }
        .onChange(of: frameToken) { _, new in V2PageRasterizer.shared.setCurrent(frameToken: new) }
    }
}

/// Identity of a prepared frame instance for bitmap keys: the envelope id
/// plus the per-preparation nonce, so a reload of the same file (a new
/// frame) never reuses the previous frame's bitmaps.
enum V2FrameIdentity {
    static func token(_ frame: V2Frame) -> String { "\(frame.id)#r\(frame.list.revision)#\(frame.preparedNonce)" }
}

private struct PageV2View: View {
    let page: RenderingV2.Page
    let prepared: V2PreparedPage
    let frameToken: String
    let dark: Bool
    let stale: Bool
    let scale: CGFloat
    let displayScale: CGFloat
    let caretMatches: [V2Geometry.CaretMatch]
    let onSelect: (V2Geometry.Hit) -> Void
    @State private var hover: V2Geometry.Hit?

    private func viewRect(_ r: RenderingV2.Rect) -> CGRect {
        CGRect(x: RenderingV2.points(r.x) * scale, y: RenderingV2.points(r.top) * scale,
               width: RenderingV2.points(r.width) * scale, height: RenderingV2.points(r.height) * scale)
    }

    var body: some View {
        let size = CGSize(width: page.widthPt * scale, height: page.heightPt * scale)
        let rasterizer = V2PageRasterizer.shared
        // Reading `images` through `image(for:)` subscribes this page to bitmap arrivals.
        let bitmap = rasterizer.image(for: prepared, frameToken: frameToken, pixelsPerPoint: Double(scale * displayScale), dark: dark)
        Canvas(rendersAsynchronously: false) { context, _ in
            if let bitmap {
                // The off-main raster of this page, blitted 1:1 onto device
                // pixels (no resampling): the same bitmap the parity check compares.
                context.withCGContext { cg in
                    // The canvas is y-down; CGImage drawing is y-up. Flip once.
                    cg.saveGState()
                    cg.translateBy(x: 0, y: size.height)
                    cg.scaleBy(x: 1, y: -1)
                    cg.interpolationQuality = .none
                    cg.draw(bitmap, in: CGRect(origin: .zero, size: size))
                    cg.restoreGState()
                }
            }
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
            if stale {
                context.fill(Path(CGRect(origin: .zero, size: size)), with: .color(Color.orange.opacity(0.08)))
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
            // Colored for the PAGE background (white or dark), not the window appearance.
            Text(bitmap == nil ? "page \(page.number) · v2 · rasterizing…" : (stale ? "page \(page.number) · v2 · STALE" : "page \(page.number) · v2"))
                .font(.caption2).foregroundStyle(stale ? Color.orange : (dark ? Color(white: 0.7) : Color(white: 0.35))).padding(4)
        }
        .help(hover.map { h in
            (h.text.map { "“\($0)” → " } ?? "rule → ") + (h.syntheticReason.map { "generated: \($0)" }
                ?? h.sources.map { "\($0.path) \($0.startByte)..<\($0.endByte)" }.joined(separator: ", "))
        } ?? "")
    }
}
