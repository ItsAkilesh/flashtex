import AppKit
import Combine
import FlashTeXProtocol

/// State for the editor/preview shell. The preview is fixture-backed: nothing
/// here compiles LaTeX. `isFixture` is surfaced in the UI so the shell never
/// implies a real compiler ran.
@MainActor
final class ShellModel: ObservableObject {
    struct Selection: Equatable {
        var path: String
        var nsRange: NSRange
        var token = 0 // bump so the same range re-applies
    }

    @Published var documents: [RuntimeV1.Document] = []
    @Published var activePath: String = "main.tex"
    @Published var result: RuntimeV1.CompileResult?
    @Published var resultID: String?
    @Published var fixtureURL: URL?
    @Published var loadError: String?
    @Published var selection: Selection?
    @Published var navigationNote: String?
    @Published var darkPreview = false
    /// Revision the editor buffer corresponds to. Bumps on every edit so the
    /// UI can say when the preview's source ranges no longer match the buffer.
    @Published private(set) var editorRevision = 1

    var isFixture: Bool { true }
    var previewIsStale: Bool { (result?.revision ?? editorRevision) != editorRevision }

    var activeText: String {
        get { documents.first { $0.path == activePath }?.text ?? "" }
    }

    init() {
        if let root = Self.locateRepoRoot() {
            let fixtures = root.appendingPathComponent("protocol/fixtures")
            loadFixtures(request: fixtures.appendingPathComponent("compile-request.json"),
                         result: fixtures.appendingPathComponent("compile-result.json"))
        } else {
            loadError = "Could not locate protocol/fixtures (set FLASHTEX_REPO or use File > Open)."
            documents = [.init(path: "main.tex", text: "")]
        }
    }

    // MARK: loading

    func loadFixtures(request: URL?, result: URL) {
        loadError = nil
        do {
            let res = try RuntimeV1.decodeCompileResult(Data(contentsOf: result))
            self.result = res.payload
            self.resultID = res.id
            self.fixtureURL = result
            if let request, let data = try? Data(contentsOf: request),
               let req = try? RuntimeV1.decodeCompileRequest(data) {
                documents = req.payload.documents
                activePath = req.payload.entryPath
                editorRevision = req.payload.revision
            } else if documents.isEmpty {
                documents = [.init(path: "main.tex", text: "")]
            }
            selection = nil
            navigationNote = nil
        } catch {
            loadError = "Failed to load \(result.lastPathComponent): \(error)"
        }
    }

    func reloadFixture() {
        guard let url = fixtureURL else { return }
        let request = url.deletingLastPathComponent().appendingPathComponent("compile-request.json")
        loadFixtures(request: request, result: url)
    }

    func openFixturePanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.message = "Choose a runtime v1 compile_result JSON file"
        if panel.runModal() == .OK, let url = panel.url {
            let request = url.deletingLastPathComponent().appendingPathComponent("compile-request.json")
            loadFixtures(request: request, result: url)
        }
    }

    // MARK: editing

    func updateActiveText(_ text: String) {
        guard let i = documents.firstIndex(where: { $0.path == activePath }) else { return }
        guard documents[i].text != text else { return }
        documents[i].text = text
        editorRevision += 1
    }

    // MARK: navigation (preview -> source)

    /// Converts the contract's UTF-8 byte range to a UTF-16 selection in the
    /// matching document and asks the editor to select it.
    func navigate(to source: RuntimeV1.SourceRange?) {
        guard let source else {
            navigationNote = "This item has no source mapping."
            return
        }
        guard let doc = documents.first(where: { $0.path == source.path }) else {
            navigationNote = "No open document named \(source.path)."
            return
        }
        guard let ns = doc.text.nsRange(utf8Bytes: source) else {
            navigationNote = "Bytes \(source.startByte)..<\(source.endByte) are not a valid range in \(source.path) (buffer is \(doc.text.utf8.count) bytes)."
            return
        }
        activePath = source.path
        selection = .init(path: source.path, nsRange: ns, token: (selection?.token ?? 0) + 1)
        navigationNote = "Selected \(source.path) bytes \(source.startByte)..<\(source.endByte) → UTF-16 \(ns.location)..<\(ns.location + ns.length)"
            + (previewIsStale ? " (buffer edited since revision \(result?.revision ?? 0); mapping may be off)" : "")
    }

    // MARK: repo discovery

    static func locateRepoRoot() -> URL? {
        var candidates: [URL] = []
        if let env = ProcessInfo.processInfo.environment["FLASHTEX_REPO"] {
            candidates.append(URL(fileURLWithPath: env))
        }
        candidates.append(URL(fileURLWithPath: FileManager.default.currentDirectoryPath))
        candidates.append(Bundle.main.bundleURL)
        candidates.append(URL(fileURLWithPath: #filePath))
        for start in candidates {
            var url = start
            for _ in 0..<8 {
                if FileManager.default.fileExists(atPath: url.appendingPathComponent("protocol/fixtures/compile-result.json").path) {
                    return url
                }
                let parent = url.deletingLastPathComponent()
                if parent == url { break }
                url = parent
            }
        }
        return nil
    }
}
