import SwiftUI
import FlashTeXProtocol
import FlashTeXAccessibility

struct ContentView: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        @Bindable var model = model
        VStack(spacing: 0) {
            StatusBanner()
            Divider()
            HSplitView {
                EditorPane().frame(minWidth: 320)
                PreviewPane().frame(minWidth: 400)
            }
            Divider()
            Footer()
        }
        .toolbar {
            ToolbarItem {
                HStack(spacing: 4) {
                    Text("Dark preview").font(.caption)
                    Toggle("Dark preview", isOn: $model.darkPreview).toggleStyle(.switch).labelsHidden()
                }
            }
            ToolbarItem {
                HStack(spacing: 4) {
                    Text("Auto-compile").font(.caption)
                    Toggle("Auto-compile", isOn: $model.autoCompile).toggleStyle(.switch).labelsHidden()
                        .disabled(!model.workerAttached)
                }
            }
            ToolbarItem { HStack(spacing: 4) { Text("v2 preview").font(.caption); Toggle("v2 preview", isOn: $model.previewV2).toggleStyle(.switch).labelsHidden() }.help("Experimental display-list-v2 preview (File > Open Display List (v2)…)") }
            ToolbarItem { Button("Reload fixture") { model.reloadFixture() } }
            ToolbarItem {
                Button("Compile", systemImage: "hammer") { model.compile() }
                    .disabled(!model.workerAttached)
                    .help("Send the current buffers to the attached worker (⌘B)")
            }
        }
    }
}

// MARK: pieces
//
// Each pane is its own view reading the model from the environment, so
// `@Observable` tracking scopes invalidation: a keystroke (documents,
// editorRevision) re-evaluates EditorPane and Footer; a compile result the
// StatusBanner and PreviewPane; bridge traffic the bridge bar only.

private struct StatusBanner: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        HStack(spacing: 12) {
            sourceBadge
            if let r = model.result {
                Text("\(sourceName) · id \(model.resultID ?? "?") · project \(r.projectId) · revision \(r.revision)")
                Text("status: \(r.status.rawValue)")
                    .foregroundStyle(statusColor(r.status)).bold()
                let diags = model.displayedDiagnostics
                let errors = diags.filter { $0.severity == .error }.count
                let warnings = diags.count - errors
                if errors > 0 { Label("\(errors)", systemImage: "xmark.octagon.fill").foregroundStyle(.red) }
                if warnings > 0 { Label("\(warnings)", systemImage: "exclamationmark.triangle.fill").foregroundStyle(.orange) }
                if r.status == .recovered {
                    Text("recovered: preview shown with provisional rendering").foregroundStyle(.orange)
                }
                Text("pdf: \(r.pdfPath ?? "none")").foregroundStyle(.secondary)
                Text("layout: " + (model.negotiation.accepted.isEmpty ? "legacy" : model.negotiation.accepted.joined(separator: ", ")))
                    .font(.caption).foregroundStyle(.secondary)
                    .help(model.negotiation.accepted.isEmpty
                          ? "No layout capability accepted for this result: U+2500 fraction bars are an approximation."
                          : "Capabilities the worker accepted for this result (typed rules / explicit font hints).")
                ForEach(model.capabilityNotes, id: \.self) { note in
                    Text(note).font(.caption).foregroundStyle(.orange).lineLimit(1).help(note)
                }
                if model.inFlightRevision != nil {
                    ProgressView().controlSize(.small)
                }
                if let ms = model.lastLatencyMs, let med = model.medianLatencyMs {
                    Text(String(format: "latency %.0f ms (median %.0f over %d)", ms, med, model.latenciesMs.count))
                        .font(.caption).foregroundStyle(.secondary)
                }
                if let historical = model.historicalPreview {
                    Text(historical.label).foregroundStyle(.purple).bold()
                        .help("A completed older snapshot is shown while the helper compiles the newer revision; navigation, caret sync, capture destinations and export return with the current preview.")
                } else if model.previewIsStale {
                    Text(model.workerAttached
                         ? (model.autoCompile ? "editor at revision \(model.editorRevision) — compiling…" : "editor at revision \(model.editorRevision) — press ⌘B to compile")
                         : "editor at revision \(model.editorRevision) — preview not recompiled (no worker attached)")
                        .foregroundStyle(.orange)
                }
            } else if let err = model.loadError {
                Text(err).foregroundStyle(.red)
            }
            Spacer()
            Text(model.isFixture ? "Not a real compile." : model.workerStatus)
                .font(.caption).foregroundStyle(.secondary).lineLimit(1)
        }
        .font(.callout)
        .padding(.horizontal, 12).padding(.vertical, 6)
        .background(.bar)
    }

    private var sourceBadge: some View {
        let (label, color): (String, Color) = switch model.previewSource {
        case .none: ("NONE", .gray)
        case .fixture: ("FIXTURE", .orange)
        case .worker: model.historicalPreview != nil ? ("HISTORICAL", .purple) : ("WORKER", .green)
        }
        return Text(label)
            .font(.caption.bold())
            .padding(.horizontal, 6).padding(.vertical, 2)
            .background(color.opacity(0.25), in: Capsule())
    }

    private var sourceName: String {
        switch model.previewSource {
        case .none: "—"
        case .fixture: model.fixtureURL?.lastPathComponent ?? "fixture"
        case .worker(let name): name
        }
    }

    private func statusColor(_ s: RuntimeV1.Status) -> Color {
        switch s { case .ok: .green; case .recovered: .orange; case .failed: .red }
    }
}

private struct EditorPane: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        @Bindable var model = model
        VStack(spacing: 0) {
            HStack {
                // Switching goes through ProjectDocuments so each document's
                // caret/selection is kept and a pending insertion is never
                // applied to the wrong buffer (ProjectDocuments.swift).
                Picker("Document", selection: Binding(get: { model.activePath },
                                                      set: { model.project.switchDocument(to: $0) })) {
                    ForEach(model.project.listing) { doc in
                        Text(doc.path + (doc.isDirty ? " •" : "") + (doc.durableRevision.map { " r\($0)" } ?? "")).tag(doc.path)
                    }
                }
                .labelsHidden().frame(maxWidth: 260)
                ProjectMenu()
                if let url = model.documentURL {
                    let dirty = model.project.isDirty(model.activePath)
                    let name = model.activePath == model.project.entryPath ? url.lastPathComponent : model.activePath
                    Text(name + (dirty ? " — edited" : ""))
                        .font(.caption).foregroundStyle(dirty ? .orange : .secondary)
                        .help(model.activePath == model.project.entryPath ? url.path : url.deletingLastPathComponent().appendingPathComponent(model.activePath).path)
                } else {
                    Text("unsaved buffer").font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
                Text("\(model.activeText.utf8.count) UTF-8 bytes · \((model.activeText as NSString).length) UTF-16 units")
                    .font(.caption).foregroundStyle(.secondary)
            }
            .padding(8)
            SourceEditorView(
                text: Binding(get: { model.activeText }, set: { model.updateActiveText($0) }),
                selection: model.selection,
                pendingEdit: model.pendingEdit,
                marks: model.editorMarks,
                result: model.result,
                editorRevision: model.editorRevision,
                projectIndexMetadata: model.completionMetadata,
                onCaretChange: { model.caretUTF16 = $0 },
                onSelectionChange: { model.caretLengthUTF16 = $0.length },
                onEditApplied: { model.editApplied($0, newText: $1) }
            )
            CaptureBar()
            BridgeBar()
        }
    }
}

/// Project membership: open the entry document's `\input`/`\include`
/// targets, save or detach the active non-entry document. Discovery runs
/// when the menu opens (bounded lexical scan, ProjectDocuments.swift).
private struct ProjectMenu: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        Menu {
            let found = model.project.discoverIncludes()
            if found.isEmpty {
                Text("No \\input or \\include in \(model.project.entryPath)")
            }
            ForEach(Array(found.enumerated()), id: \.offset) { _, d in
                switch d.state {
                case .available:
                    Button("Open \(d.resolvedPath ?? d.reference.argument)") {
                        Task { await model.project.openInclude(d.reference.argument) }
                    }
                case .open:
                    Button("Show \(d.resolvedPath ?? d.reference.argument)") {
                        if let path = d.resolvedPath { model.project.switchDocument(to: path) }
                    }
                case .unresolvable(let why):
                    Text("\\\(d.reference.kind.rawValue){\(d.reference.argument)}: \(why)")
                }
            }
            if !found.isEmpty, found.contains(where: { $0.state == .available }) {
                Button("Open All Includes") { Task { await model.project.openDiscoveredIncludes() } }
            }
            if model.activePath != model.project.entryPath {
                Divider()
                Button("Save \(model.activePath)") { Task { await model.project.saveDocument(model.activePath) } }
                    .disabled(model.documentURL == nil)
                Button("Detach \(model.activePath)") {
                    Task {
                        if case .refused(let why) = await model.project.detachDocument(model.activePath) { model.captureNote = why }
                    }
                }
            }
        } label: {
            Label("Project", systemImage: "doc.on.doc")
        }
        .menuStyle(.borderlessButton).fixedSize()
        .help(model.project.status)
    }
}

private struct CaptureBar: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        HStack(spacing: 8) {
            Button("Pin insertion point") { model.pinAnchorAtCaret() }
                .help("Use the caret as the destination for capture proposals (⌘⇧P)")
            if let a = model.anchor {
                Text("anchor \(a.id) · \(a.path) byte \(a.byteOffset) @ rev \(a.revision)")
                    .font(.caption).foregroundStyle(.secondary)
            } else {
                Text("no insertion point pinned").font(.caption).foregroundStyle(.secondary)
            }
            Spacer()
            if !model.proposals.isEmpty {
                Button("Review \(model.proposals.count) proposal\(model.proposals.count == 1 ? "" : "s")") {
                    model.reviewing = model.proposals.first
                }
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 4)
        .background(.bar)
        .accessibleCaptureBar(anchor: model.anchor.map { "\($0.id) at \($0.path) byte \($0.byteOffset), revision \($0.revision)" }, proposals: model.proposals.count) // FlashTeXAccessibility
        .sheet(item: Binding(get: { model.reviewing.map { ReviewItem(proposal: $0) } },
                             set: { model.reviewing = $0?.proposal })) { item in
            ProposalReviewSheet(proposal: item.proposal)
        }
    }
}

/// Bridge lifecycle line: attached/error status, the pinned bridge
/// destination, and the latest capture's state (plain text, never a prompt).
private struct BridgeBar: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        HStack(spacing: 8) {
            Text("bridge:").font(.caption.bold())
            Text(model.bridgeStatus).font(.caption)
                .foregroundStyle(model.bridgeAttached ? Color.secondary : Color.orange).lineLimit(1)
            if let d = model.bridgeDestination {
                Text("· destination \(d.destinationId) bytes \(d.startByte)..<\(d.endByte) @ rev \(d.pinnedRevision)\(d.valid ? "" : " (invalid)")")
                    .font(.caption).foregroundStyle(.secondary).lineLimit(1)
            }
            Spacer()
            if let c = model.bridgeCaptures.last {
                Text("\(c.captureId): \(c.state.rawValue) — \(c.note)").font(.caption).foregroundStyle(.secondary).lineLimit(1)
                    .help(c.note)
            }
            if model.latestConvertibleCapture?.state == .received {
                Button("Convert") { model.convertLatestCapture() }.controlSize(.small)
                    .help("capture_convert for the latest received capture (Edit > Convert Capture)")
            }
        }
        .padding(.horizontal, 8).padding(.vertical, 3)
        .background(.bar)
    }
}

private struct PreviewPane: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        VStack(spacing: 0) {
            if model.previewV2 {
                PreviewV2Pane() // experimental v2 path (PreviewV2View.swift); v1 below stays the default
            } else if let result = model.result {
                PreviewView(result: result, dark: model.darkPreview, caretItems: model.caretItems) { source, text in
                    guard let source else { model.navigationNote = "This item has no source mapping."; return }
                    model.navigate(to: source, expectedText: text)
                }
                let diags = model.displayedDiagnostics
                if !diags.isEmpty {
                    Divider()
                    diagnosticsList(diags)
                }
            } else {
                ContentUnavailableView("No compile result loaded", systemImage: "doc.richtext",
                                       description: Text("Use File > Open Compile Result Fixture…"))
            }
        }
    }

    private func diagnosticsList(_ diags: [RuntimeV1.Diagnostic]) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Diagnostics (\(diags.count)) — the preview above is still shown; errors are not hidden")
                .font(.caption.bold()).padding(.horizontal, 8).padding(.vertical, 4)
            List(Array(diags.enumerated()), id: \.offset) { i, d in
                HStack(alignment: .top) {
                    Image(systemName: d.severity == .error ? "xmark.octagon.fill" : "exclamationmark.triangle.fill")
                        .foregroundStyle(d.severity == .error ? .red : .orange)
                    VStack(alignment: .leading) {
                        Text(d.message)
                        if let line = EditorDiagnostics.recoveryLine(recovery: d.recovery, status: model.result?.status ?? .ok) {
                            Text("↳ \(line)").font(.caption).foregroundStyle(d.recovery == nil ? .tertiary : .secondary)
                        }
                        if let explain = model.explanations.explanation(resultID: model.resultID, index: i)?.line {
                            Text("↳ \(explain)").font(.caption).foregroundStyle(.secondary)
                        }
                        if let result = model.result,
                           let id = EditorDiagnostics.identity(resultID: model.resultID, index: i, in: result),
                           model.editorMarkReport.staleIdentities.contains(id) {
                            Text("underline withheld: span edited since the compile").font(.caption2).foregroundStyle(.orange)
                        }
                        if let src = d.source {
                            Text("\(src.path) bytes \(src.startByte)..<\(src.endByte)").font(.caption2).foregroundStyle(.tertiary)
                        } else {
                            Text("no source mapping").font(.caption2).foregroundStyle(.tertiary)
                        }
                    }
                    Spacer()
                    if d.source != nil { Button("Go to source") { model.navigate(to: d.source) } }
                }
                .accessibleDiagnostic(d, index: i, total: diags.count) { model.navigate(to: d.source) } // FlashTeXAccessibility
            }
            .frame(minHeight: 80, maxHeight: 180)
        }
    }
}

private struct Footer: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        HStack {
            Text(model.navigationNote ?? model.editorMarkReport.staleNote ?? model.explanationStatus
                 ?? "Click text in the preview to select its source range.")
                .font(.caption).foregroundStyle(.secondary).lineLimit(1)
            Spacer()
            if let note = model.captureNote {
                Text(note).font(.caption).foregroundStyle(.secondary).lineLimit(1)
            }
        }
        .padding(.horizontal, 12).padding(.vertical, 4)
    }
}

private struct ReviewItem: Identifiable {
    let proposal: RuntimeV1.CaptureProposal
    var id: String { proposal.captureId }
}

/// Review sheet: the reviewer sees ambiguities and dependencies, may edit the
/// LaTeX, and explicitly approves or rejects. Nothing is inserted otherwise.
private struct ProposalReviewSheet: View {
    @Environment(ShellModel.self) var model
    @Environment(\.dismiss) private var dismiss
    let proposal: RuntimeV1.CaptureProposal
    @State private var latex: String = ""
    @StateObject private var preview = ProposalPreview(executable: ShellModel.locateCompiler()) // shadow compile (ProposalPreview.swift)

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Review capture \(proposal.captureId)").font(.headline)
            if model.isBridgeCapture(proposal.captureId) {
                Text("Bridge capture: approval asks the bridge for a prepared edit, verifies revision, SHA-256 and removed text, then inserts once. The LaTeX must stay as proposed.")
                    .font(.caption).foregroundStyle(.secondary)
                if let d = model.bridgeDestination {
                    Text("Bridge destination \(d.destinationId): \(d.path) bytes \(d.startByte)..<\(d.endByte)").font(.caption).foregroundStyle(.secondary)
                }
                if let cr = proposal.contextRevision {
                    Text("Context revision \(cr)\(cr == model.editorRevision ? "" : " (editor is at \(model.editorRevision))")").font(.caption).foregroundStyle(cr == model.editorRevision ? Color.secondary : Color.orange)
                }
            }
            if let a = model.anchor {
                Text("Inserts at \(a.path) byte \(a.byteOffset) (anchor \(a.id))").font(.caption).foregroundStyle(.secondary)
            } else {
                Label("No insertion point pinned — approve will fail until you pin one.", systemImage: "exclamationmark.triangle")
                    .font(.caption).foregroundStyle(.orange)
            }
            TextEditor(text: $latex)
                .font(.system(.body, design: .monospaced))
                .frame(minHeight: 140)
                .border(.separator)
            ProposalPreviewView(preview: preview)
            if !proposal.ambiguities.isEmpty {
                Text("Ambiguities").font(.subheadline.bold())
                ForEach(proposal.ambiguities, id: \.self) { Text("• \($0)").font(.caption) }
            }
            if !proposal.requiredDependencies.isEmpty {
                Text("Required packages: " + proposal.requiredDependencies.joined(separator: ", ")).font(.caption)
            }
            HStack {
                Button("Reject", role: .destructive) { model.rejectProposal(proposal); dismiss() }
                Spacer()
                ProposalApproveWarning(preview: preview)
                Button("Approve and insert") {
                    if model.isBridgeCapture(proposal.captureId) {
                        Task { if case .inserted = await model.approveBridgeProposal(proposal, latex: latex) { dismiss() } }
                    } else if case .inserted = model.approveProposal(proposal, latex: latex) { dismiss() }
                }
                .keyboardShortcut(.defaultAction)
                .disabled((model.anchor == nil && !model.isBridgeCapture(proposal.captureId)) || latex.trimmingCharacters(in: .whitespaces).isEmpty)
            }
        }
        .padding(16)
        .frame(width: 520)
        .onAppear { latex = proposal.latex; preview.update(from: model, latex: proposal.latex) }
        .onChange(of: latex) { _, new in preview.update(from: model, latex: new) }
        .onChange(of: model.editorRevision) { _, _ in preview.update(from: model, latex: latex) }
        .onChange(of: model.anchor) { _, _ in preview.update(from: model, latex: latex) }
        // A reviewer-approved assistant amendment replaces the DRAFT only;
        // insertion still requires "Approve and insert" (mac-ai-review).
        .onChange(of: preview.amendedProposalLatex) { _, new in if let new { latex = new } }
        .onDisappear { preview.close() }
    }
}
