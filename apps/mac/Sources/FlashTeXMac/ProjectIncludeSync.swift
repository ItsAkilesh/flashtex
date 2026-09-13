import Foundation

/// Real file names and automatic included-file loading (issue #75).
///
/// The entry document's protocol path is its real file name (`HW1.tex`, not
/// `main.tex`), so the tab, sidebar, Problems rows, quit prompt, window
/// title and compiler diagnostics all name the same file; other members keep
/// their project-relative paths. The compiler accepts any project-relative
/// entry path, and the preview controller already required the entry path to
/// equal the file name to use the real project root. Include loading itself
/// lives in `ProjectDocuments` (automatic include loading).

extension ProjectIncludes {
    /// Protocol path for an entry document opened from `url`: its file name
    /// when that is a valid single rooted component, else `main.tex`.
    static func entryPath(for url: URL?) -> String {
        guard let name = url?.lastPathComponent, let path = try? normalize(name), !path.contains("/") else { return "main.tex" }
        return path
    }
}

extension ShellModel {
    /// Window title: the entry file name, with the active member when another one is edited.
    var windowTitle: String {
        guard let url = documentURL else { return "FlashTeX" }
        let entry = ProjectIncludes.entryPath(for: url)
        return activePath == entry ? url.lastPathComponent : "\(activePath) — \(url.lastPathComponent)"
    }

    /// True when the buffers differ from the ones the applied result was
    /// compiled from (an include opened, detached or reloaded from disk
    /// without an editor revision), so `compile()` must not skip them.
    var projectMembershipChangedSinceResult: Bool {
        documents.count != compiledDocuments.count
            || documents.contains { compiledDocuments[$0.path]?.sameBytes(as: $0.text) != true }
    }

    /// Save As of a single-document project renames the entry to the saved
    /// file name. Direct route only: the helper names its project at attach.
    func adoptEntryFileName() {
        guard let url = documentURL, documents.count == 1, !controllerAttached else { return }
        let name = ProjectIncludes.entryPath(for: url), old = documents[0].path
        guard name != old else { return }
        documents[0].path = name
        if activePath == old { activePath = name }
        if anchor?.path == old { anchor = nil }
        selection = nil
        bridgeDocumentReplaced()
        scheduleAutoCompile() // compiledDocuments still names the old path, so this recompiles
    }
}
