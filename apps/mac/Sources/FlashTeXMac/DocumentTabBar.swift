import SwiftUI
import FlashTeXProtocol

/// Document tabs above the editor (mac-ui-redesign): one tab per open
/// project member in `ShellModel.documents` order (entry first), the active
/// one highlighted, a dot for unsaved edits, the helper's durable revision,
/// and a close control on non-entry members that detaches them for this
/// session (`ProjectDocuments.detachDocument`). Switching goes through
/// `ProjectDocuments.switchDocument` so each document keeps its caret and a
/// pending insertion is never applied to the wrong buffer. The Project menu
/// (open includes, save/detach, bibliography kinds) and the kind/byte
/// indicators sit at the trailing end, where the old header put them.
struct DocumentTabBar: View {
    @Environment(ShellModel.self) var model

    static let identifier = "document.tabs"

    var body: some View {
        HStack(spacing: 0) {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 2) {
                    ForEach(model.chrome.listing) { doc in // throttled, change-only copy (ShellChrome.swift): `project.listing` reads `documents` per keystroke
                        DocumentTab(doc: doc, active: doc.path == model.activePath, kind: model.documentKinds.kind(of: doc.path))
                    }
                }
                .padding(.horizontal, 6)
            }
            .accessibilityElement(children: .contain)
            .accessibilityLabel("Open documents")
            .accessibilityIdentifier(Self.identifier)
            Spacer(minLength: 8)
            ProjectMenu()
            DocumentKindIndicator() // DocumentKinds.swift: helper-reported bibliography kind, read-only
            if let url = model.documentURL {
                let dirty = model.project.isDirty(model.activePath)
                Text(dirty ? "edited" : "saved")
                    .font(.caption).foregroundStyle(dirty ? .orange : .secondary)
                    .help(model.activePath == model.project.entryPath ? url.path : url.deletingLastPathComponent().appendingPathComponent(model.activePath).path)
            } else {
                Text("unsaved buffer").font(.caption).foregroundStyle(.secondary)
            }
            Text("\(model.chrome.activeTextBytes) B · \(model.chrome.activeTextUTF16) u16")
                .font(.caption).foregroundStyle(.tertiary).monospacedDigit()
                .help("\(model.chrome.activeTextBytes) UTF-8 bytes · \(model.chrome.activeTextUTF16) UTF-16 units")
                .padding(.trailing, 8)
        }
        .frame(height: 30)
        .background(.bar)
    }
}

private struct DocumentTab: View {
    @Environment(ShellModel.self) var model
    let doc: ProjectDocument
    let active: Bool
    let kind: DocumentKind?
    @State private var hovering = false

    var body: some View {
        HStack(spacing: 5) {
            Image(systemName: kind == .bibliography ? "books.vertical" : (doc.role == .entry ? "doc.text.fill" : "doc.text"))
                .font(.caption).foregroundStyle(active ? Color.accentColor : Color.secondary)
            Text(doc.path).font(.callout).lineLimit(1)
                .foregroundStyle(active ? Color.primary : Color.secondary)
            if let r = doc.durableRevision {
                Text("r\(r)").font(.caption2).foregroundStyle(.tertiary).monospacedDigit()
            }
            if doc.isDirty {
                Circle().fill(.orange).frame(width: 6, height: 6).accessibilityHidden(true)
            }
            if doc.role != .entry {
                Button {
                    Task {
                        switch await model.project.detachDocument(doc.path) {
                        case .refused(let why): model.captureNote = why
                        case .detached(let path): model.captureNote = "Detached \(path) — " + ProjectDocuments.detachScopeNote
                        }
                    }
                } label: {
                    Image(systemName: "xmark").font(.caption2.bold())
                        .foregroundStyle(.secondary)
                        .frame(width: 14, height: 14)
                        .background(hovering ? Color.primary.opacity(0.1) : .clear, in: RoundedRectangle(cornerRadius: 3))
                }
                .buttonStyle(.plain)
                .opacity(hovering || active ? 1 : 0.35)
                .help("Detach \(doc.path) for this session (" + ProjectDocuments.detachScopeNote + ")")
                .accessibilityLabel("Detach \(doc.path)")
            }
        }
        .padding(.horizontal, 10).padding(.vertical, 5)
        .background(active ? Color.accentColor.opacity(0.14) : (hovering ? Color.primary.opacity(0.05) : .clear),
                    in: RoundedRectangle(cornerRadius: 6))
        .overlay(alignment: .bottom) {
            if active { Rectangle().fill(Color.accentColor).frame(height: 2).padding(.horizontal, 4) }
        }
        .contentShape(Rectangle())
        .onTapGesture { model.switchOrNote(doc.path) }
        .onHover { hovering = $0 }
        .contextMenu {
            Button("Show \(doc.path)") { model.switchOrNote(doc.path) }.disabled(active)
            if doc.role != .entry {
                Button("Save \(doc.path)") { Task { await model.project.saveDocument(doc.path) } }
                    .disabled(model.documentURL == nil)
            }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(doc.path)\(doc.role == .entry ? ", entry" : "")\(doc.isDirty ? ", edited" : "")")
        .accessibilityAddTraits(active ? [.isSelected, .isButton] : .isButton)
        .accessibilityAction { model.switchOrNote(doc.path) }
        .help(Self.tooltip(doc))
    }

    static func tooltip(_ doc: ProjectDocument) -> String {
        var s = doc.path
        switch doc.role {
        case .entry: s += " — entry document"
        case .included(let from): s += " — included from \(from)"
        case .opened: s += " — opened by path"
        }
        if let r = doc.durableRevision { s += " · durable r\(r)" }
        if doc.isDirty { s += " · edited" }
        return s
    }
}

/// Project membership: open the entry document's `\input`/`\include`
/// targets, save or detach the active non-entry document. Discovery runs
/// when the menu opens (bounded lexical scan, ProjectDocuments.swift).
struct ProjectMenu: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        Menu {
            // The transitive closure (chapter → section → …), depth-first in
            // source order, indented by depth; cycles and missing files are
            // listed with their reason. Bounded: 8 levels, 256 documents.
            let closure = model.project.discoverClosure()
            if closure.nodes.isEmpty {
                Text("No \\input or \\include in \(model.project.entryPath)")
            }
            ForEach(Array(closure.nodes.enumerated()), id: \.offset) { _, n in
                let indent = String(repeating: "    ", count: max(0, n.depth))
                let name = n.resolvedPath ?? n.reference.argument
                switch n.state {
                case .available:
                    Button(indent + "Open \(name)") { Task { await model.project.openDocument(name, role: .included(from: n.from)) } }
                case .open:
                    Button(indent + "Show \(name)") { model.project.switchDocument(to: name) }
                case .unresolvable(let why):
                    Text(indent + "\\\(n.reference.kind.rawValue){\(n.reference.argument)}: \(why)")
                }
            }
            if closure.truncated { Text("closure truncated at \(ProjectDocuments.maxClosureDocuments) documents") }
            if closure.nodes.contains(where: { $0.state == .available }) {
                Button("Open All Includes") { Task { await model.project.openDiscoveredIncludes() } }
                    .help("Opens the whole include closure in this order; unresolvable references are reported in the footer note")
            }
            if let report = model.project.lastOpenReport, !report.unresolvable.isEmpty {
                Divider()
                Text("Open All: \(report.unresolvable.count) unresolvable")
                ForEach(Array(report.unresolvable.enumerated()), id: \.offset) { _, line in Text(line) }
            }
            if model.activePath != model.project.entryPath {
                Divider()
                Button("Save \(model.activePath)") { Task { await model.project.saveDocument(model.activePath) } }
                    .disabled(model.documentURL == nil)
                Button("Detach \(model.activePath) (this session)") {
                    Task {
                        switch await model.project.detachDocument(model.activePath) {
                        case .refused(let why): model.captureNote = why
                        case .detached(let path): model.captureNote = "Detached \(path) — " + ProjectDocuments.detachScopeNote
                        }
                    }
                }
                .help("Session only: " + ProjectDocuments.detachScopeNote)
            }
            DocumentKindsMenuSection() // DocumentKinds.swift: declare/undeclare bibliography sources
        } label: {
            Label("Project", systemImage: "doc.on.doc")
        }
        .menuStyle(.borderlessButton).fixedSize()
        .help(model.project.status)
    }
}
