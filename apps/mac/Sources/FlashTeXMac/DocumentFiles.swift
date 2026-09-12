import AppKit
import UniformTypeIdentifiers
import FlashTeXProtocol

/// Open/save of the active `.tex` document. The project is single-entry for
/// now: the opened file becomes `main.tex` in the compile request (the compiler
/// only compiles the entry document), and its URL is remembered for saving.
extension ShellModel {
    static let texType = UTType(filenameExtension: "tex") ?? .plainText

    func openTexPanel() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [Self.texType, .plainText]
        panel.message = "Open a LaTeX source file as the entry document"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        openTex(at: url)
    }

    func openTex(at url: URL) {
        do {
            let text = try String(contentsOf: url, encoding: .utf8)
            replaceProject(entryText: text)
            documentURL = url
            savedText = text
            captureNote = "Opened \(url.lastPathComponent) (\(text.utf8.count) bytes)"
            if workerAttached { compile() }
        } catch {
            captureNote = "Could not open \(url.lastPathComponent): \(error.localizedDescription)"
        }
    }

    var isDirty: Bool { savedText != activeText }

    @discardableResult
    func saveTex() -> Bool {
        guard let url = documentURL else { return saveTexAs() }
        return write(to: url)
    }

    @discardableResult
    func saveTexAs() -> Bool {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [Self.texType]
        panel.nameFieldStringValue = documentURL?.lastPathComponent ?? "main.tex"
        guard panel.runModal() == .OK, let url = panel.url else { return false }
        return write(to: url)
    }

    private func write(to url: URL) -> Bool {
        do {
            try activeText.write(to: url, atomically: true, encoding: .utf8)
            documentURL = url
            savedText = activeText
            captureNote = "Saved \(url.lastPathComponent)"
            return true
        } catch {
            captureNote = "Save failed: \(error.localizedDescription)"
            return false
        }
    }
}
