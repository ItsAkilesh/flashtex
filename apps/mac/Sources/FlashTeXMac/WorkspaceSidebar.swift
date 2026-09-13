import SwiftUI
import FlashTeXProtocol

/// The main window's sidebar (mac-ui-redesign): the project's members with
/// their kind and state, the active buffer's Outline (sections,
/// environments, labels — DocumentOutline.swift) and a Problems summary.
/// Every row drives an existing model operation: switching goes through
/// `ProjectDocuments.switchDocument`, includes open through
/// `openDocument`, outline rows select through `reveal(outlineItem:)`, and
/// the Problems rows show/filter the bottom panel. Nothing here is a new
/// source of truth.
struct WorkspaceSidebar: View {
    @Environment(ShellModel.self) var model
    /// Outline of the active buffer, rescanned ~150 ms after edits settle so
    /// a keystroke never pays for a scan on its own frame (TypingBench).
    @State private var outline: [DocumentOutline.Item] = []
    @State private var outlineFor: (path: String, revision: Int) = ("", -1)
    @State private var expanded: Set<DocumentOutline.Kind> = [.section, .environment, .label]

    static let identifier = "workspace.sidebar"

    var body: some View {
        List {
            ProjectSection()
            OutlineSection(outline: outline, expanded: $expanded, stale: outlineFor.revision != model.editorRevision)
            ProblemsSection()
        }
        .listStyle(.sidebar)
        .accessibilityIdentifier(Self.identifier)
        .modifier(ProjectScaffoldSheets()) // New Project / New File / Rename / Delete (ProjectScaffoldViews.swift)
        .task(id: "\(model.activePath)@\(model.editorRevision)") {
            // Rescan after a short quiet period; the previous scan is cancelled.
            let revision = model.editorRevision, path = model.activePath
            if outlineFor.revision >= 0 { try? await Task.sleep(for: .milliseconds(150)) }
            guard !Task.isCancelled else { return }
            outline = model.outline
            outlineFor = (path, revision)
        }
    }
}

// MARK: - Project

/// Open members (entry first) plus every `\input`/`\include` the entry
/// references that is not open yet (bounded discovery, ProjectDocuments.swift).
private struct ProjectSection: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        let listing = model.project.listing
        let kinds = model.documentKinds
        let closure = model.project.discoverClosure()
        Section {
            ForEach(listing) { doc in
                SidebarRow(selected: doc.path == model.activePath) {
                    model.switchOrNote(doc.path)
                } label: {
                    Label {
                        HStack(spacing: 4) {
                            Text(doc.path).lineLimit(1).truncationMode(.middle)
                            if doc.isDirty { Circle().fill(.orange).frame(width: 6, height: 6).accessibilityLabel("edited") }
                            Spacer(minLength: 0)
                            if let r = doc.durableRevision { Text("r\(r)").font(.caption2).foregroundStyle(.tertiary).monospacedDigit() }
                        }
                    } icon: {
                        Image(systemName: Self.icon(for: doc, kind: kinds.kind(of: doc.path)))
                            .foregroundStyle(doc.path == model.activePath ? Color.accentColor : Color.secondary)
                    }
                }
                .help(Self.tooltip(for: doc, kind: kinds.kind(of: doc.path)))
                .accessibilityLabel(Self.spoken(for: doc, kind: kinds.kind(of: doc.path), active: doc.path == model.activePath))
                .contextMenu { // ProjectScaffoldViews.swift
                    Button("New File…") { model.scaffold.presentNewFile() }
                    if doc.role != .entry {
                        Divider()
                        Button("Rename…") { model.scaffold.presentRename(doc.path) }
                        Button("Delete…") { model.scaffold.presentDelete(doc.path) }
                    }
                }
            }
            let closed = closure.nodes.filter { $0.state == .available }
            ForEach(Array(closed.enumerated()), id: \.offset) { _, n in
                let name = n.resolvedPath ?? n.reference.argument
                SidebarRow(selected: false) {
                    Task { await model.project.openDocument(name, role: .included(from: n.from)) }
                } label: {
                    Label {
                        Text(name).lineLimit(1).truncationMode(.middle).foregroundStyle(.secondary)
                    } icon: { Image(systemName: "doc.badge.plus").foregroundStyle(.tertiary) }
                }
                .help("\\\(n.reference.kind.rawValue){\(n.reference.argument)} from \(n.from) — click to open")
                .accessibilityLabel("\(name), not open, included from \(n.from); activate to open")
            }
            // A literal, rooted reference with no file behind it: one click creates it (ProjectScaffold.swift).
            let missing = closure.nodes.filter { if case .unresolvable(let why) = $0.state { return why.hasPrefix("no such file") } else { return false } }
            ForEach(Array(missing.enumerated()), id: \.offset) { _, n in
                let name = MissingIncludeFix.path(for: n.reference.argument) ?? n.reference.argument
                SidebarRow(selected: false) {
                    Task { await model.project.createMissingInclude(n.reference.argument, from: n.from); model.navigationNote = model.project.status }
                } label: {
                    Label {
                        HStack(spacing: 4) {
                            Text(name).lineLimit(1).truncationMode(.middle).foregroundStyle(.secondary)
                            Text("missing — create").font(.caption2).foregroundStyle(.orange)
                        }
                    } icon: { Image(systemName: "doc.badge.plus").foregroundStyle(.orange) }
                }
                .help("\\\(n.reference.kind.rawValue){\(n.reference.argument)} from \(n.from) has no file — click to create \(name)")
                .accessibilityLabel("\(name), missing, included from \(n.from); activate to create it")
            }
        } header: {
            HStack {
                Label("Project", systemImage: "folder")
                Spacer()
                Text("\(listing.count)").font(.caption2).foregroundStyle(.tertiary).monospacedDigit()
                Button { model.scaffold.presentNewFile() } label: { Image(systemName: "plus") } // ProjectScaffoldViews.swift
                    .buttonStyle(.plain).foregroundStyle(.secondary)
                    .disabled(model.project.projectRoot == nil)
                    .help("New File… (⌘N): a rooted .tex file in this project, opened in a tab")
                    .accessibilityLabel("New file")
                    .accessibilityIdentifier("project.newfile")
            }
        }
    }

    static func icon(for doc: ProjectDocument, kind: DocumentKind?) -> String {
        if kind == .bibliography { return "books.vertical" }
        if doc.role == .entry { return "doc.text.fill" }
        return "doc.text"
    }

    static func tooltip(for doc: ProjectDocument, kind: DocumentKind?) -> String {
        var parts = [doc.path]
        switch doc.role {
        case .entry: parts.append("entry document")
        case .included(let from): parts.append("included from \(from)")
        case .opened: parts.append("opened by path")
        }
        if kind == .bibliography { parts.append("bibliography source (declared)") }
        switch doc.origin {
        case .buffer: parts.append("buffer")
        case .helper: parts.append("via the helper's durable ledger")
        case .disk: parts.append("read from disk")
        }
        if let r = doc.durableRevision { parts.append("durable revision \(r)") }
        if doc.isDirty { parts.append("edited since last save") }
        return parts.joined(separator: " · ")
    }

    static func spoken(for doc: ProjectDocument, kind: DocumentKind?, active: Bool) -> String {
        var s = doc.path
        if doc.role == .entry { s += ", entry" }
        if kind == .bibliography { s += ", bibliography" }
        if doc.isDirty { s += ", edited" }
        if active { s += ", active" }
        return s
    }
}

// MARK: - Outline

private struct OutlineSection: View {
    @Environment(ShellModel.self) var model
    let outline: [DocumentOutline.Item]
    @Binding var expanded: Set<DocumentOutline.Kind>
    let stale: Bool

    var body: some View {
        let counts = DocumentOutline.counts(outline)
        Section {
            if outline.isEmpty {
                Text(stale ? "Scanning…" : "No sections, environments or labels in \(model.activePath)")
                    .font(.caption).foregroundStyle(.secondary)
            }
            ForEach(DocumentOutline.Kind.allCases, id: \.self) { kind in
                let items = DocumentOutline.items(kind, in: outline)
                if !items.isEmpty {
                    DisclosureGroup(isExpanded: Binding(get: { expanded.contains(kind) },
                                                        set: { if $0 { expanded.insert(kind) } else { expanded.remove(kind) } })) {
                        ForEach(items) { item in
                            SidebarRow(selected: false) {
                                model.reveal(outlineItem: item)
                            } label: {
                                HStack(spacing: 4) {
                                    Image(systemName: Self.icon(item)).foregroundStyle(.secondary).font(.caption)
                                    Text(item.title.isEmpty ? "(untitled)" : item.title).lineLimit(1)
                                    Spacer(minLength: 0)
                                    Text("\(item.line)").font(.caption2).foregroundStyle(.tertiary).monospacedDigit()
                                }
                                .padding(.leading, CGFloat(min(item.level, 4)) * 10)
                            }
                            .help(Self.tooltip(item))
                            .accessibilityLabel(Self.spoken(item))
                        }
                    } label: {
                        HStack {
                            Text(kind.title)
                            Spacer()
                            Text("\(counts[kind] ?? 0)").font(.caption2).foregroundStyle(.tertiary).monospacedDigit()
                        }
                    }
                }
            }
        } header: {
            HStack {
                Label("Outline", systemImage: "list.bullet.indent")
                Spacer()
                if stale { ProgressView().controlSize(.mini) }
            }
        }
    }

    static func icon(_ item: DocumentOutline.Item) -> String {
        switch item.kind {
        case .section: return item.level <= 1 ? "number" : "number.square"
        case .environment: return "curlybraces"
        case .label: return "tag"
        }
    }

    static func tooltip(_ item: DocumentOutline.Item) -> String {
        switch item.kind {
        case .section: return "\\\(item.command){\(item.title)} — line \(item.line)"
        case .environment: return "\\begin{\(item.title)} — line \(item.line)"
        case .label: return "\\label{\(item.title)} — line \(item.line)"
        }
    }

    static func spoken(_ item: DocumentOutline.Item) -> String {
        switch item.kind {
        case .section: return "\(item.command) \(item.title), line \(item.line)"
        case .environment: return "environment \(item.title), line \(item.line)"
        case .label: return "label \(item.title), line \(item.line)"
        }
    }
}

// MARK: - Problems

/// Counts by severity; activating a row shows the Problems panel filtered to
/// that severity (ProblemsPanel.swift), the "all" row clears the filter.
private struct ProblemsSection: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        let diags = model.displayedDiagnostics
        let (errors, warnings, gaps) = EditorDiagnostics.counts(diags)
        Section {
            if diags.isEmpty {
                Label { Text("No problems").foregroundStyle(.secondary) } icon: { Image(systemName: "checkmark.circle").foregroundStyle(.green) }
                    .font(.caption)
            } else {
                row("\(errors) error\(errors == 1 ? "" : "s")", icon: "xmark.octagon.fill", tint: .red, filter: .error, enabled: errors > 0)
                row("\(warnings) warning\(warnings == 1 ? "" : "s")", icon: "exclamationmark.triangle.fill", tint: .orange, filter: .warning, enabled: warnings > 0)
                if gaps > 0 { row("\(gaps) not implemented", icon: "puzzlepiece.extension", tint: .secondary, filter: nil, enabled: true) }
                row("All \(diags.count)", icon: "list.bullet.rectangle", tint: .secondary, filter: nil, enabled: true)
            }
        } header: {
            Label("Problems", systemImage: "exclamationmark.triangle")
        }
    }

    private func row(_ title: String, icon: String, tint: Color, filter: RuntimeV1.Severity?, enabled: Bool) -> some View {
        SidebarRow(selected: model.problemsVisible && model.problemsSeverityFilter == filter && filter != nil) {
            model.problemsSeverityFilter = filter
            model.problemsVisible = true
        } label: {
            Label { Text(title) } icon: { Image(systemName: icon).foregroundStyle(enabled ? tint : Color.secondary) }
        }
        .disabled(!enabled)
        .help("Show the Problems panel (⌘⇧M)\(filter.map { " filtered to \($0.rawValue)s" } ?? "")")
    }
}

// MARK: - Row

/// A sidebar row that is a button (keyboard + VoiceOver activation) and
/// draws the selected state like a `List` selection.
struct SidebarRow<Label: View>: View {
    let selected: Bool
    let action: () -> Void
    @ViewBuilder let label: () -> Label

    var body: some View {
        Button(action: action) {
            label()
                .frame(maxWidth: .infinity, alignment: .leading)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 6).padding(.vertical, 2)
        .background(selected ? Color.accentColor.opacity(0.18) : Color.clear, in: RoundedRectangle(cornerRadius: 5))
        .listRowInsets(EdgeInsets(top: 1, leading: 4, bottom: 1, trailing: 4))
        .accessibilityAddTraits(selected ? .isSelected : [])
    }
}

extension ShellModel {
    /// Sidebar/tab switching: a refused switch (pending capture insertion,
    /// unknown member) lands in the footer note instead of silently failing.
    func switchOrNote(_ path: String) {
        if case .refused(let why) = project.switchDocument(to: path) { navigationNote = why }
    }
}
