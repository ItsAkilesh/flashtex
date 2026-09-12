import Foundation
import FlashTeXPadKit
import FlashTeXProtocol
import NearbyClient
import SwiftUI

/// App state for FlashTeXPad. Everything the Mac does not send over
/// nearby-v1 is labelled by `Provenance` so no panel can pass off a local
/// fixture as a Mac reply.
@MainActor
final class PadModel: ObservableObject {
    enum Provenance: Equatable {
        case mac(String)
        case localFixture(String)
        case notCarried
        var banner: String {
            switch self {
            case .mac(let s): return "from the Mac over nearby-v1: \(s)"
            case .localFixture(let s): return "local fixture — not carried by transfer-v1: \(s)"
            case .notCarried: return "not carried by transfer-v1 (nearby-v1 §6): stays on the Mac"
            }
        }
    }

    // Document
    @Published var document: PadDocument?
    @Published var documentTitle = "No document"
    @Published var caretUTF16 = 0
    @Published var openError: String?

    // Diagnostics (runtime-v1 compile_result)
    @Published var diagnostics: [DiagnosticItem] = []
    @Published var diagnosticsSource: DiagnosticsSource = .none

    // Completions (local, source-derived)
    @Published var completions: [LocalCompletion.Suggestion] = []

    // Reviewed proposal
    @Published var review: ReviewSession?
    @Published var reviewProvenance: Provenance = .notCarried
    @Published var lastReceipt: ReviewSession.Receipt?
    @Published var reviewError: String?
    @Published var insertionCount = 0

    // Mac link
    let link: MacLink
    @Published var transcript: [MacLink.TranscriptLine] = []
    @Published var linkStatus = "not paired"
    @Published var linkError: String?
    @Published var destination: NearbyWire.Destination?
    @Published var lastCaptureReceived: NearbyWire.CaptureReceived?
    @Published var pairedMac: PairedMac?

    init(link: MacLink = MacLink(store: try? PairFile(url: PairFile.defaultURL()))) {
        self.link = link
        link.onTranscript = { [weak self] line in Task { @MainActor in self?.transcript.append(line) } }
        if let p = link.store?.pairs.last { pairedMac = p; linkStatus = "stored pairing: \(p.macName) (\(p.pairId))" }
    }

    // MARK: documents

    static let bundledSample = "demo.tex"

    func openBundledSample() {
        guard let url = Bundle.main.url(forResource: "demo", withExtension: "tex") else { openError = "demo.tex missing from bundle"; return }
        open(url: url, title: PadModel.bundledSample)
    }

    func open(url: URL, title: String? = nil) {
        let scoped = url.startAccessingSecurityScopedResource()
        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
        do {
            let data = try Data(contentsOf: url)
            guard let text = String(data: data, encoding: .utf8) else { openError = "\(url.lastPathComponent) is not UTF-8"; return }
            setDocument(PadDocument(path: url.lastPathComponent, text: text), title: title ?? url.lastPathComponent)
        } catch { openError = error.localizedDescription }
    }

    /// The assistant-context recorded review fixture: loads its `main.tex`,
    /// its compiler diagnostics and its `proposal_review` as a pending review.
    func openReviewFixture() {
        do {
            let f = try loadWorkflowFixture()
            setDocument(PadDocument(path: f.sourcePath, text: f.sourceText, revision: f.compileResult.revision), title: "review-fixture/main.tex")
            diagnostics = DiagnosticsModel.items(from: f.compileResult, boundTo: document)
            diagnosticsSource = .fixture(name: "review-workflow.json compile_result", revision: f.compileResult.revision)
            attachFixtureReview(f)
        } catch { openError = "review fixture: \(error)" }
    }

    func loadWorkflowFixture() throws -> ReviewedProposal.WorkflowFixture {
        guard let url = Bundle.main.url(forResource: "review-workflow", withExtension: "json") else {
            throw ReviewedProposal.DecodeError.missingStep("bundled review-workflow.json")
        }
        return try ReviewedProposal.decodeWorkflowFixture(try Data(contentsOf: url))
    }

    func attachFixtureReview(_ f: ReviewedProposal.WorkflowFixture) {
        review = ReviewSession(review: f.review, approved: f.approved)
        reviewProvenance = .localFixture("crates/assistant-context/examples/review-workflow.json (provider_called:false)")
        reviewError = nil
    }

    private func setDocument(_ doc: PadDocument, title: String) {
        document = doc
        documentTitle = title
        caretUTF16 = 0
        openError = nil
        review = nil
        lastReceipt = nil
        diagnostics = []
        diagnosticsSource = .none
        refreshCompletions()
    }

    func textChanged(_ text: String) {
        guard var d = document, !d.text.sameBytes(as: text) else { return }
        d.text = text
        d.revision += 1
        document = d
        // A buffer edit invalidates a pending review's binding (sha/revision); keep it
        // pending so approve() refuses it with the exact reason rather than hiding it.
        refreshCompletions()
    }

    func caretMoved(_ utf16: Int) { caretUTF16 = utf16; refreshCompletions() }

    func refreshCompletions() {
        guard let d = document, let byte = d.byteOffset(ofUTF16: caretUTF16) else { completions = []; return }
        completions = LocalCompletion.suggestions(in: d.text, caretByte: byte)
    }

    /// Applies a completion at its byte range (the same offset discipline as an edit).
    func accept(_ s: LocalCompletion.Suggestion) {
        guard var d = document, let r = d.text.rangeOfUTF8(start: s.replaceStart, end: s.replaceEnd) else { return }
        d.text.replaceSubrange(r, with: s.text)
        d.revision += 1
        document = d
        caretUTF16 = NSRange(d.text.startIndex..<d.text.index(r.lowerBound, offsetBy: s.text.count), in: d.text).length
        refreshCompletions()
    }

    // MARK: diagnostics from a compile_result file

    func openCompileResult(url: URL) {
        let scoped = url.startAccessingSecurityScopedResource()
        defer { if scoped { url.stopAccessingSecurityScopedResource() } }
        do {
            let r = try DiagnosticsModel.decode(try Data(contentsOf: url))
            diagnostics = DiagnosticsModel.items(from: r, boundTo: document)
            diagnosticsSource = .file(name: url.lastPathComponent, revision: r.revision)
        } catch { openError = "compile_result: \(error)" }
    }

    func openBundledCompileResult() {
        guard let url = Bundle.main.url(forResource: "compile-result", withExtension: "json") else { return }
        do {
            let r = try DiagnosticsModel.decode(try Data(contentsOf: url))
            diagnostics = DiagnosticsModel.items(from: r, boundTo: document)
            diagnosticsSource = .fixture(name: "protocol/fixtures/compile-result.json", revision: r.revision)
        } catch { openError = "compile_result: \(error)" }
    }

    // MARK: review gate

    func cancelReview() {
        review?.cancel()
    }

    func approveReview() {
        guard var r = review, var d = document else { return }
        do {
            let receipt = try r.approve(applyingTo: &d)
            document = d
            lastReceipt = receipt
            insertionCount += 1
            reviewError = nil
            refreshCompletions()
        } catch {
            reviewError = "\(error)"
        }
        review = r
    }

    // MARK: Mac link

    func pair(host: String, port: String, saltHex: String, fingerprint: String, macName: String, code: String) async {
        guard let p = UInt16(port) else { linkError = "port must be a number"; return }
        linkError = nil
        linkStatus = "pairing…"
        do {
            let pair = try await link.pair(host: host, port: p, saltHex: saltHex, fingerprint: fingerprint, macName: macName,
                                           code: code, companionName: "FlashTeXPad (\(UIDevice.current.name))")
            pairedMac = pair
            destination = link.destination
            linkStatus = "paired with \(pair.macName) (\(pair.pairId)); connected"
        } catch { linkError = "\(error)"; linkStatus = "pairing failed" }
    }

    func reconnect(host: String, port: String) async {
        guard let pair = pairedMac else { linkError = "no stored pairing"; return }
        guard let p = UInt16(port) else { linkError = "port must be a number"; return }
        linkError = nil
        do {
            try await link.connect(host: host, port: p, pair: pair)
            destination = link.destination
            linkStatus = "connected to \(pair.macName)"
        } catch { linkError = "\(error)"; linkStatus = "connect failed" }
    }

    func refreshDestination() async {
        do { destination = try await link.destinationQuery() } catch { linkError = "\(error)" }
    }

    /// Sends the bundled 1×1 PNG (protocol/fixtures/capture-submission.json) as a
    /// capture with the given instructions and keeps the Mac's receipt.
    @discardableResult
    func sendTestCapture(instructions: String) async -> NearbyWire.CaptureReceived? {
        linkError = nil
        do {
            let r = try await link.submitCapture(image: PadModel.png1x1, mimeType: "image/png", instructions: instructions)
            lastCaptureReceived = r
            return r
        } catch { linkError = "\(error)"; return nil }
    }

    func disconnect() { link.disconnect(); linkStatus = "disconnected" }

    static let png1x1 = Data(base64Encoded: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAADElEQVR4nGP4//8/AAX+Av4N70a4AAAAAElFTkSuQmCC")!
}
