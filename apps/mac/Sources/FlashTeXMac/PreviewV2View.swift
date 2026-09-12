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

/// Where a v2 display list came from.
enum V2Source: Equatable {
    /// `File > Open Display List (v2)…` / `FLASHTEX_V2_FILE`.
    case file(URL)
    /// The negotiated `display_list` sibling line of a `compile_result`
    /// (docs/contracts/runtime-v1-display-list-v2.md): request id, project,
    /// revision, and the line's bytes (kept for the current frame so tools that
    /// take a list file — the exact PDF export — can run on a live frame).
    case worker(requestID: String, projectId: String, revision: Int, line: Data)

    var label: String {
        switch self {
        case .file(let u): u.lastPathComponent
        case .worker(let id, _, let revision, _): "live \(id) (revision \(revision))"
        }
    }
    var url: URL? { if case .file(let u) = self { u } else { nil } }
    var isLive: Bool { if case .worker = self { true } else { false } }

    /// A file holding the list: the opened file, or the live line written to a
    /// temporary file named by request id and revision.
    func listFileURL() throws -> URL {
        switch self {
        case .file(let u): return u
        case .worker(let id, _, let revision, let line):
            let dir = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-v2-live", isDirectory: true)
            try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
            let url = dir.appendingPathComponent("\(id)-r\(revision).json")
            try line.write(to: url, options: .atomic)
            return url
        }
    }
}

/// The negotiated live route: `display-list-v2` in `layout_capabilities`
/// (docs/contracts/runtime-v1-display-list-v2.md). Requested only while the
/// v2 pane is visible; the sibling line is applied only for the currently
/// applied compile_result.
enum V2Live {
    static let capability = "display-list-v2"
    private static let lock = NSLock()
    private(set) static var staleLinesDropped = 0
    private(set) static var unsolicitedLinesDropped = 0
    private(set) static var linesAccepted = 0
    static func note(stale: Bool = false, unsolicited: Bool = false, accepted: Bool = false) {
        lock.lock(); defer { lock.unlock() }
        if stale { staleLinesDropped += 1 }
        if unsolicited { unsolicitedLinesDropped += 1 }
        if accepted { linesAccepted += 1 }
    }
}

/// The newest list that arrived while another was being prepared: it starts
/// when the in-flight preparation delivers. Lists that arrived in between
/// are dropped undecoded (coalesced), so visible progress keeps up with
/// preparation instead of being starved by strict supersession.
struct V2QueuedLoad {
    var source: V2Source
    var prepare: @Sendable () -> V2Loader.Outcome
    var completion: (() -> Void)?
}

/// What the shell holds for the v2 pane.
enum V2PreviewState {
    /// A load is in flight. `previous` is the frame still on screen, shown
    /// with an explicit stale indicator until the new one is verified
    /// (rendering-v2 proposal: retain the old frame, label it stale).
    /// `queued` is the newest list waiting for this preparation to finish.
    case loading(V2Source, ticket: Int, previous: V2Frame?, queued: V2QueuedLoad? = nil)
    case loaded(V2Frame, V2Source)
    case failed(RenderingV2.ValidationError, V2Source)

    var source: V2Source {
        switch self {
        case .loading(let s, _, _, _), .loaded(_, let s), .failed(_, let s): s
        }
    }
    var queued: V2QueuedLoad? { if case .loading(_, _, _, let q) = self { q } else { nil } }
    /// The file URL when the list came from a file (tests, evidence hook).
    var url: URL? { source.url }
    /// The verified frame the pane may paint (a stale one while loading).
    var frame: V2Frame? {
        switch self {
        case .loaded(let f, _): f
        case .loading(_, _, let previous, _): previous
        case .failed: nil
        }
    }
    var isLoading: Bool { if case .loading = self { true } else { false } }
    var ticket: Int? { if case .loading(_, let t, _, _) = self { t } else { nil } }
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
    /// Lists dropped undecoded because a newer one arrived while one was in flight.
    private(set) static var coalescedLoads = 0

    static func issueTicket() -> Int { lock.lock(); defer { lock.unlock() }; let t = nextTicket; nextTicket += 1; return t }
    static func noteDropped() { lock.lock(); staleResultsDropped += 1; lock.unlock() }
    static func notePublished() { lock.lock(); resultsPublished += 1; lock.unlock() }
    static func noteCoalesced() { lock.lock(); coalescedLoads += 1; lock.unlock() }

    enum Outcome {
        case loaded(V2Frame)
        case failed(RenderingV2.ValidationError)
    }

    /// Bitmaps rasterized in the same off-main job as the preparation, at the
    /// pane's last requested pixels-per-point/appearance, so the publish and
    /// the first blit happen in one main-thread pass instead of three.
    struct Prerastered {
        var pixelsPerPoint: Double
        var dark: Bool
        var images: [(page: Int, image: CGImage)]
    }

    static func preraster(_ frame: V2Frame, pixelsPerPoint: Double, dark: Bool) -> Prerastered {
        Prerastered(pixelsPerPoint: pixelsPerPoint, dark: dark,
                    images: frame.prepared.compactMap { page in GlyphRunRenderer.rasterize(page, scale: pixelsPerPoint, dark: dark).map { (page.number, $0) } })
    }

    /// Pure preparation: file → decoded, validated, font-resolved, page-prepared frame.
    static func prepare(url: URL, store: V2FontStore = .shared) -> Outcome {
        do {
            return prepare(data: try Data(contentsOf: url), store: store)
        } catch {
            return .failed(RenderingV2.ValidationError(code: "io_error", message: error.localizedDescription))
        }
    }

    /// Pure preparation of one `display_list` envelope's bytes (a file or the
    /// worker's sibling line).
    static func prepare(data: Data, store: V2FontStore = .shared) -> Outcome {
        do {
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
        previewV2 = true
        startDisplayListV2(source: .file(url), completion: completion) { V2Loader.prepare(url: url) }
    }

    /// Requests (or stops requesting) the negotiated live route while the v2
    /// pane is visible: `display-list-v2` joins `requestedLayoutCapabilities`,
    /// whose change re-requests the current revision under auto-compile. The
    /// v1 pages of every result keep painting the product preview.
    func setLiveV2(_ on: Bool) {
        let has = requestedLayoutCapabilities.contains(V2Live.capability)
        if on, !has { requestedLayoutCapabilities.append(V2Live.capability) }
        if !on, has { requestedLayoutCapabilities.removeAll { $0 == V2Live.capability } }
    }

    /// Whether the applied result negotiated the live route.
    var liveV2Accepted: Bool { negotiation.accepted.contains(V2Live.capability) }

    /// The worker's `display_list` sibling line for request `id`
    /// (`WorkerClient.Event.displayList`). Applied only when `id` is the id of
    /// the currently applied `compile_result` and that result accepted
    /// `display-list-v2`; anything else is stale or unsolicited and is dropped
    /// — a late v2 frame never replaces a newer one. Preparation is off-main;
    /// the payload's project/revision must match the applied result.
    func receiveDisplayListV2(id: String, line: Data, completion: (() -> Void)? = nil) {
        guard let result, resultID == id, previewSource != .fixture else {
            V2Live.note(stale: true)
            log("ignored stale display_list \(id) (\(line.count) B): applied result is \(resultID ?? "none")")
            completion?()
            return
        }
        guard liveV2Accepted else {
            V2Live.note(unsolicited: true)
            let msg = "display_list \(id) arrived but \(V2Live.capability) was not accepted for that result (accepted: \(negotiation.accepted.joined(separator: ", ")))"
            log("rejected unsolicited " + msg)
            workerStatus = "protocol violation: " + msg
            completion?()
            return
        }
        V2Live.note(accepted: true)
        let expectedProject = result.projectId, expectedRevision = result.revision
        startDisplayListV2(source: .worker(requestID: id, projectId: expectedProject, revision: expectedRevision, line: line), completion: completion) {
            switch V2Loader.prepare(data: line) {
            case .loaded(let frame) where frame.list.projectId != expectedProject || frame.list.revision != expectedRevision:
                return .failed(RenderingV2.ValidationError(code: "correlation_mismatch",
                                                           message: "display_list \(id) is for project \(frame.list.projectId) revision \(frame.list.revision); the compile_result is project \(expectedProject) revision \(expectedRevision)"))
            case let outcome: return outcome
            }
        }
    }

    /// Starts an off-main preparation. The previous verified frame (if any)
    /// stays on screen with a stale indicator until it finishes; a result
    /// whose ticket is no longer current is dropped. `completion` runs on the
    /// main thread after the state was (or was not) published.
    private func startDisplayListV2(source: V2Source, completion: (() -> Void)?, prepare: @escaping @Sendable () -> V2Loader.Outcome) {
        // One preparation in flight; the newest arrival waits, older waiting ones are dropped undecoded.
        if case .loading(let inFlight, let ticket, let previous, let queued) = displayListV2 {
            if let queued {
                V2Loader.noteCoalesced()
                if TypingBench.isBenchActive { FlashTeXLog.write("preview-v2: coalesced \(queued.source.label) behind \(source.label) at \(MonotonicClock.nowNs())") }
                queued.completion?()
            }
            displayListV2 = .loading(inFlight, ticket: ticket, previous: previous, queued: V2QueuedLoad(source: source, prepare: prepare, completion: completion))
            return
        }
        let ticket = V2Loader.issueTicket()
        displayListV2 = .loading(source, ticket: ticket, previous: displayListV2?.frame)
        if !source.isLive { captureNote = "Loading display list \(source.label)…" }
        let t0 = MonotonicClock.nowNs()
        if TypingBench.isBenchActive { FlashTeXLog.write("preview-v2: preparing \(source.label) ticket \(ticket) at \(t0)") }
        let rasterHint = V2PageRasterizer.shared.lastRequest
        V2Loader.queue.async {
            let outcome = prepare()
            let t1 = MonotonicClock.nowNs()
            var prerastered: V2Loader.Prerastered?
            if case .loaded(let frame) = outcome, let hint = rasterHint {
                prerastered = V2Loader.preraster(frame, pixelsPerPoint: hint.pixelsPerPoint, dark: hint.dark)
            }
            let t2 = MonotonicClock.nowNs()
            let prerasteredResult = prerastered // immutable copy for the Sendable delivery closure
            V2Loader.deliverOnMain {
                MainActor.assumeIsolated {
                    if TypingBench.isBenchActive { FlashTeXLog.write("preview-v2: prepared \(source.label) in \(Double(t1 &- t0) / 1e6) ms, prerastered \(prerasteredResult?.images.count ?? 0) page(s) in \(Double(t2 &- t1) / 1e6) ms, delivered \(Double(MonotonicClock.nowNs() &- t2) / 1e6) ms later") }
                    self.deliverDisplayListV2(ticket: ticket, source: source, outcome: outcome, prerastered: prerasteredResult)
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
    func deliverDisplayListV2(ticket: Int, source: V2Source, outcome: V2Loader.Outcome, prerastered: V2Loader.Prerastered? = nil) -> Bool {
        guard displayListV2?.ticket == ticket else {
            V2Loader.noteDropped()
            FlashTeXLog.write("preview-v2: dropped stale load result ticket \(ticket) for \(source.label) (current: \(displayListV2?.ticket.map(String.init) ?? "none"))")
            return false
        }
        let queued = displayListV2?.queued
        defer {
            // The newest list that arrived meanwhile starts now, over the frame just published.
            if let queued { startDisplayListV2(source: queued.source, completion: queued.completion, prepare: queued.prepare) }
        }
        V2Loader.notePublished()
        switch outcome {
        case .loaded(let frame):
            // Bitmaps first, so the render pass this publish triggers blits them.
            if let prerastered { V2PageRasterizer.shared.preinstall(prerastered, frame: frame) }
            displayListV2 = .loaded(frame, source)
            if TypingBench.isBenchActive { FlashTeXLog.write("preview-v2: published \(source.label) revision \(frame.list.revision) at \(MonotonicClock.nowNs())") }
            captureNote = "\(source.isLive ? "Live display list" : "Loaded display list") \(source.label): \(frame.list.pages.count) page(s), \(frame.fonts.count) font(s) resolved by content hash, \(frame.prepared.reduce(0) { $0 + $1.glyphCount }) glyphs prepared"
            V2ParityEvidence.runIfRequested(frame: frame, source: source)
        case .failed(let error):
            // A refusal drops the previous frame: nothing verified is on screen.
            displayListV2 = .failed(error, source)
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

    /// One observable slot per key: a page body reads its own slot's `image`,
    /// so a bitmap arriving for page 3 re-evaluates page 3 only.
    @Observable final class Slot { var image: CGImage? }
    @ObservationIgnored private var slots: [Key: Slot] = [:]
    /// Ready bitmaps (bookkeeping/tests; views observe their slot, not this).
    @ObservationIgnored private(set) var images: [Key: CGImage] = [:]
    @ObservationIgnored private var order: [Key] = []
    @ObservationIgnored private var inFlight: Set<Key> = []
    /// Observed: a page body that asked before the frame became current
    /// re-evaluates (and re-requests) when `setCurrent` runs.
    private(set) var currentFrameToken: String?
    @ObservationIgnored private(set) var retainedBytes = 0
    @ObservationIgnored let maxBytes: Int
    @ObservationIgnored private(set) var staleBitmapsDropped = 0
    @ObservationIgnored private(set) var rasterizations = 0
    /// The pane's most recent pixels-per-point/appearance: the loader
    /// pre-rasterizes new frames with it off-main.
    @ObservationIgnored private(set) var lastRequest: (pixelsPerPoint: Double, dark: Bool)?
    @ObservationIgnored private(set) var preinstalled = 0

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
        lastRequest = (pixelsPerPoint, dark)
        let slot: Slot
        if let existing = slots[key] { slot = existing } else { slot = Slot(); slots[key] = slot }
        if let image = slot.image { // observed read: this body follows this page's bitmap only
            touch(key)
            return image
        }
        guard frameToken == currentFrameToken, !inFlight.contains(key) else { return nil }
        inFlight.insert(key)
        let t0 = MonotonicClock.nowNs()
        Self.queue.async {
            let t1 = MonotonicClock.nowNs()
            let image = GlyphRunRenderer.rasterize(page, scale: pixelsPerPoint, dark: dark)
            let t2 = MonotonicClock.nowNs()
            V2Loader.deliverOnMain {
                MainActor.assumeIsolated {
                    if TypingBench.isBenchActive {
                        FlashTeXLog.write("preview-v2: raster page \(key.page) of \(key.frameToken.prefix(24)): queued \(Double(t1 &- t0) / 1e6) ms, drew \(Double(t2 &- t1) / 1e6) ms, delivered \(Double(MonotonicClock.nowNs() &- t2) / 1e6) ms later at \(MonotonicClock.nowNs())")
                    }
                    self.install(image, for: key)
                }
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
        (slots[key] ?? { let s = Slot(); slots[key] = s; return s }()).image = image
        order.removeAll { $0 == key }
        order.append(key)
        retainedBytes += image.bytesPerRow * image.height
        while retainedBytes > maxBytes, let oldest = order.first, oldest != key {
            order.removeFirst()
            drop(oldest)
        }
    }

    /// Installs bitmaps rasterized off-main together with `frame` and makes it
    /// current (evicting the previous frame's), immediately before the frame is
    /// published — the pass that shows the frame finds its bitmaps ready.
    func preinstall(_ prerastered: V2Loader.Prerastered, frame: V2Frame) {
        let token = V2FrameIdentity.token(frame)
        setCurrent(frameToken: token)
        for (page, image) in prerastered.images {
            install(image, for: Key(frameToken: token, page: page, pixelsPerPoint: prerastered.pixelsPerPoint, dark: prerastered.dark))
            preinstalled += 1
        }
    }

    func clear() { for key in order { drop(key) }; order.removeAll() }

    private func touch(_ key: Key) {
        if let i = order.firstIndex(of: key) { order.remove(at: i); order.append(key) }
    }

    private func drop(_ key: Key) {
        if let image = images.removeValue(forKey: key) { retainedBytes -= image.bytesPerRow * image.height }
        slots[key]?.image = nil
        slots.removeValue(forKey: key)
    }
}

/// `FLASHTEX_V2_PARITY_OUT=<dir>`: after every successful load, run the
/// zero-tolerance export-versus-preview comparison off-main and write
/// `parity.json`, `export.pdf` and per-page `page-N-preview.png` /
/// `page-N-export.png` there (evidence capture; never in the product path).
/// `FLASHTEX_V2_PARITY_SCALE` sets pixels per point (default 2).
enum V2ParityEvidence {
    static func runIfRequested(frame: V2Frame, source: V2Source) {
        guard let dir = ProcessInfo.processInfo.environment["FLASHTEX_V2_PARITY_OUT"], !dir.isEmpty else { return }
        let scale = Double(ProcessInfo.processInfo.environment["FLASHTEX_V2_PARITY_SCALE"] ?? "") ?? 2
        V2Loader.queue.async {
            var out = URL(fileURLWithPath: dir)
            if case .worker(let id, _, let revision, _) = source { out.appendPathComponent("live-\(id)-r\(revision)") }
            try? FileManager.default.createDirectory(at: out, withIntermediateDirectories: true)
            let report = V2Parity.compare(frame: frame, scale: scale) { page in
                write(page.preview, to: out.appendingPathComponent("page-\(page.page)-preview.png"))
                write(page.export, to: out.appendingPathComponent("page-\(page.page)-export.png"))
            }
            try? GlyphRunRenderer.pdfData(frame: frame).write(to: out.appendingPathComponent("export.pdf"))
            let encoder = JSONEncoder(); encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            var object: [String: Any] = [:]
            if let data = try? encoder.encode(report), let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] { object = json }
            object["source"] = source.url?.path ?? source.label
            object["identical"] = report.identical
            object["total_differing_pixels"] = report.totalDifferingPixels
            if let data = try? JSONSerialization.data(withJSONObject: object, options: [.prettyPrinted, .sortedKeys]) {
                try? data.write(to: out.appendingPathComponent("parity.json"))
            }
            FlashTeXLog.write("preview-v2: parity \(report.identical ? "identical" : "DIFFERS") — \(report.totalDifferingPixels) differing pixel(s) over \(report.pages.count) page(s) at \(scale)px/pt for \(source.label) → \(out.path)")
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
            case .loading(_, _, let previous, _):
                if let previous {
                    pages(previous, stale: true)
                } else {
                    ContentUnavailableView("Loading display list…", systemImage: "hourglass")
                }
            case .failed(let error, let source):
                ContentUnavailableView {
                    Label("Display list refused — nothing rendered", systemImage: "xmark.octagon")
                } description: {
                    Text("\(source.label)\n[\(error.code)] \(error.message)")
                        .textSelection(.enabled)
                }
                .accessibilityIdentifier("v2-refusal")
            case nil:
                ContentUnavailableView("No v2 display list yet", systemImage: "doc.richtext",
                                       description: Text(model.workerAttached
                                                         ? "Requesting display-list-v2 from the attached worker (\(model.liveV2Accepted ? "accepted" : "not accepted yet")); or use File > Open Display List (v2)…"
                                                         : "Attach a producer that accepts display-list-v2, or use File > Open Display List (v2)… with a flashtex-render --v2 JSON file."))
            }
        }
        .onAppear {
            // While visible, the pane asks the producer for the live route.
            model.setLiveV2(true)
            // Automation hook: FLASHTEX_V2_FILE seeds the pane at launch.
            if model.displayListV2 == nil, let path = ProcessInfo.processInfo.environment["FLASHTEX_V2_FILE"] {
                model.loadDisplayListV2(url: URL(fileURLWithPath: path))
            }
        }
        .onDisappear { model.setLiveV2(false) }
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

    private var header: some View { V2PaneHeader() }
}

/// The pane header is its own view: it reads the applied result, worker and
/// negotiation state, so those changes re-evaluate the header, not the pages.
private struct V2PaneHeader: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack(spacing: 8) {
                Text("V2").font(.caption.bold()).padding(.horizontal, 6).padding(.vertical, 2).background(Color.purple.opacity(0.25), in: Capsule())
                Text("experimental display-list-v2 — not the default path").font(.caption).foregroundStyle(.secondary).lineLimit(1)
                if model.workerAttached {
                    Text(model.liveV2Accepted ? "LIVE" : "v1 only")
                        .font(.caption.bold()).padding(.horizontal, 6).padding(.vertical, 2)
                        .background((model.liveV2Accepted ? Color.green : Color.gray).opacity(0.25), in: Capsule())
                        .help(model.liveV2Accepted ? "The applied compile_result accepted display-list-v2; frames arrive with each compile."
                                                   : "The applied compile_result did not accept display-list-v2 (producer without the capability, or declined for this request).")
                        .accessibilityIdentifier("v2-live")
                }
                if let frame = model.displayListV2?.frame, model.displayListV2?.source.isLive == true,
                   let applied = model.result?.revision, applied != frame.list.revision {
                    Text("frame revision \(frame.list.revision) — applied result is revision \(applied) (no v2 frame for it)")
                        .font(.caption.bold()).foregroundStyle(.orange).lineLimit(1)
                        .accessibilityIdentifier("v2-behind")
                }
                if case .loading(let source, _, let previous, _) = model.displayListV2 {
                    ProgressView().controlSize(.small)
                    Text(previous == nil ? "loading \(source.label)…" : "STALE — showing the previous frame while \(source.label) is verified")
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
                Text("\(model.displayListV2?.source.label ?? "") · id \(frame.id) · project \(frame.list.projectId) · revision \(frame.list.revision) · \(frame.list.pages.count) page(s) · fonts by hash: \(fonts)")
                    .font(.caption).foregroundStyle(.secondary).lineLimit(1).truncationMode(.middle)
                    .help(frame.fonts.values.map { "\($0.resource.postscriptName): \($0.resource.sha256) → \($0.file.url.lastPathComponent)" }.sorted().joined(separator: "\n"))
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
            // Scroll anchoring (PreviewAnchor.swift): the (page, fraction) under the
            // viewport's top edge survives a frame with another page count and a
            // pane resize; a frame with the same page geometry never moves the scroll.
            let layout = PreviewPageLayout(pages: frame.prepared.map { PreviewPageLayout.Page(number: $0.number, widthPt: $0.widthPt, heightPt: $0.heightPt) }, scale: scale)
            ScrollView([.vertical, .horizontal]) {
                LazyVStack(spacing: 24) {
                    ForEach(frame.prepared, id: \.number) { prepared in
                        if let page = frame.page(number: prepared.number) {
                            PageV2View(page: page, prepared: prepared, frameToken: frameToken, frameRevision: frame.list.revision, pageCount: frame.prepared.count,
                                       dark: dark, stale: stale, scale: scale, displayScale: displayScale,
                                       caretMatches: caretByte.map { V2Geometry.clusters(containing: $0, path: caretPath, in: page) } ?? [],
                                       onSelect: onSelect)
                                .equatable()
                                .id(page.number)
                        }
                    }
                }
                .padding(24)
                .background(PreviewAnchorKeeper(layout: layout))
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

private struct PageV2View: View, Equatable {
    let page: RenderingV2.Page
    let prepared: V2PreparedPage
    let frameToken: String
    /// Display-list revision and page count, for the typing bench's paint point.
    var frameRevision = 0
    var pageCount = 1
    let dark: Bool
    let stale: Bool
    let scale: CGFloat
    let displayScale: CGFloat
    let caretMatches: [V2Geometry.CaretMatch]
    let onSelect: (V2Geometry.Hit) -> Void
    @State private var hover: V2Geometry.Hit?

    /// Everything that affects the drawing except the bitmap (observed through
    /// its slot) and hover (local state): a pane re-evaluation for another
    /// page's bitmap, a caret move elsewhere or a header change skips this page.
    static func == (a: PageV2View, b: PageV2View) -> Bool {
        a.frameToken == b.frameToken && a.page.number == b.page.number && a.frameRevision == b.frameRevision && a.pageCount == b.pageCount
            && a.dark == b.dark && a.stale == b.stale && a.scale == b.scale && a.displayScale == b.displayScale && a.caretMatches == b.caretMatches
    }

    private func viewRect(_ r: RenderingV2.Rect) -> CGRect {
        CGRect(x: RenderingV2.points(r.x) * scale, y: RenderingV2.points(r.top) * scale,
               width: RenderingV2.points(r.width) * scale, height: RenderingV2.points(r.height) * scale)
    }

    var body: some View {
        let size = CGSize(width: page.widthPt * scale, height: page.heightPt * scale)
        let rasterizer = V2PageRasterizer.shared
        // Reading `images` through `image(for:)` subscribes this page to bitmap arrivals.
        let bitmap = rasterizer.image(for: prepared, frameToken: frameToken, pixelsPerPoint: Double(scale * displayScale), dark: dark)
        // Paint instrumentation (TypingBench.swift): the v2 paint point is the render
        // pass that blits a page bitmap of the frame's revision; pages whose bitmap is
        // still rasterizing do not count as drawn (finishPaint logs drew n/expected).
        let _ = bitmap == nil ? () : TypingBench.shared.willRender(revision: frameRevision, pages: pageCount)
        Canvas(rendersAsynchronously: false) { context, _ in
            if let bitmap {
                TypingBench.shared.didDraw(page: page.number) // paint instrumentation
                if TypingBench.isBenchActive { FlashTeXLog.write("preview-v2: blit page \(page.number) of \(frameToken.prefix(24)) at \(MonotonicClock.nowNs())") }
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
