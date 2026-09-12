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
        guard isDirty else { openTex(at: url); return }
        let alert = NSAlert()
        alert.messageText = "Save changes to \(documentURL?.lastPathComponent ?? "the unsaved buffer") before opening \(url.lastPathComponent)?"
        alert.informativeText = "Discarded text stays recoverable this session via Edit > Restore Discarded Buffer."
        alert.addButton(withTitle: "Save")
        alert.addButton(withTitle: "Discard")
        alert.addButton(withTitle: "Cancel")
        switch alert.runModal() {
        case .alertFirstButtonReturn:
            if documentURL == nil, !saveTexAs() { return }
            openTex(at: url, dirty: .saveFirst)
        case .alertSecondButtonReturn: openTex(at: url, dirty: .discard)
        default: break
        }
    }

    /// What a caller authorized for the unsaved buffer before opening another file.
    enum DirtyDisposition: Equatable { case none, saveFirst, discard }

    enum OpenOutcome: Equatable { case opened, blockedByUnsavedEdits, saveFailed, readFailed }

    /// The last buffer replaced by an authorized open, kept so an accidental
    /// discard remains recoverable within the session.
    struct RecoverableBuffer: Equatable { var url: URL?; var text: String }

    /// Opens a `.tex` file as the entry document. When the current buffer is
    /// dirty, nothing is replaced unless the caller passes an explicit
    /// disposition: `.saveFirst` writes the current file (or refuses if it has
    /// no URL), `.discard` replaces it but keeps the text in `recoverableBuffer`.
    @discardableResult
    func openTex(at url: URL, dirty: DirtyDisposition = .none) -> OpenOutcome {
        if isDirty {
            switch dirty {
            case .none:
                captureNote = "\(documentURL?.lastPathComponent ?? "The unsaved buffer") has unsaved edits; save or discard before opening \(url.lastPathComponent)."
                return .blockedByUnsavedEdits
            case .saveFirst:
                guard documentURL != nil, saveTex() else {
                    captureNote = "Could not save the current buffer; \(url.lastPathComponent) was not opened."
                    return .saveFailed
                }
            case .discard:
                recoverableBuffer = RecoverableBuffer(url: documentURL, text: activeText)
            }
        }
        do {
            let text = try String(contentsOf: url, encoding: .utf8)
            replaceProject(entryText: text)
            documentURL = url
            savedText = text
            captureNote = "Opened \(url.lastPathComponent) (\(text.utf8.count) bytes)"
                + (recoverableBuffer == nil ? "" : "; previous unsaved buffer kept (Edit > Restore Discarded Buffer)")
            if workerAttached { compile() }
            return .opened
        } catch {
            captureNote = "Could not open \(url.lastPathComponent): \(error.localizedDescription)"
            return .readFailed
        }
    }

    /// Restores the buffer discarded by the last authorized open (undo of the
    /// discard decision); the currently open file is left untouched on disk.
    @discardableResult
    func restoreDiscardedBuffer() -> Bool {
        guard let kept = recoverableBuffer else { return false }
        if isDirty {
            captureNote = "Current buffer has unsaved edits; save it before restoring the discarded buffer."
            return false
        }
        replaceProject(entryText: kept.text)
        documentURL = kept.url
        // "Saved" is whatever is on disk now, so the restored text stays dirty
        // (it differs from disk) and cannot be lost again silently.
        savedText = kept.url.flatMap { try? String(contentsOf: $0, encoding: .utf8) } ?? ""
        recoverableBuffer = nil
        captureNote = "Restored the discarded buffer (\(kept.text.utf8.count) bytes, unsaved)."
        if workerAttached { compile() }
        return true
    }

    /// Dirty means the buffer differs from what was last opened/saved. A fresh
    /// fixture-seeded buffer (no file, never saved) counts as dirty only once edited.
    var isDirty: Bool {
        if let savedText { return savedText != activeText }
        return documentURL == nil && editorRevision > 1 && !activeText.isEmpty
    }

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
            bridgeSourceSaved(url: url, text: activeText)
            return true
        } catch {
            captureNote = "Save failed: \(error.localizedDescription)"
            return false
        }
    }
}
