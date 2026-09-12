import SwiftUI
import FlashTeXProtocol

struct ContentView: View {
    @EnvironmentObject var model: ShellModel

    var body: some View {
        VStack(spacing: 0) {
            statusBanner
            Divider()
            HSplitView {
                editorPane.frame(minWidth: 320)
                previewPane.frame(minWidth: 400)
            }
            Divider()
            footer
        }
        .toolbar {
            ToolbarItem { Toggle("Dark preview", isOn: $model.darkPreview).toggleStyle(.switch) }
            ToolbarItem { Toggle("Auto-compile", isOn: $model.autoCompile).toggleStyle(.switch).disabled(!model.workerAttached) }
            ToolbarItem { Button("Reload fixture") { model.reloadFixture() } }
            ToolbarItem {
                Button("Compile", systemImage: "hammer") { model.compile() }
                    .disabled(!model.workerAttached)
                    .help("Send the current buffers to the attached worker (⌘B)")
            }
        }
    }

    // MARK: pieces

    private var statusBanner: some View {
        HStack(spacing: 12) {
            sourceBadge
            if let r = model.result {
                Text("\(sourceName) · id \(model.resultID ?? "?") · project \(r.projectId) · revision \(r.revision)")
                Text("status: \(r.status.rawValue)")
                    .foregroundStyle(statusColor(r.status)).bold()
                Text("pdf: \(r.pdfPath ?? "none")").foregroundStyle(.secondary)
                if model.inFlightRevision != nil {
                    ProgressView().controlSize(.small)
                }
                if let ms = model.lastLatencyMs, let med = model.medianLatencyMs {
                    Text(String(format: "latency %.0f ms (median %.0f over %d)", ms, med, model.latenciesMs.count))
                        .font(.caption).foregroundStyle(.secondary)
                }
                if model.previewIsStale {
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
        case .worker: ("WORKER", .green)
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

    private var editorPane: some View {
        VStack(spacing: 0) {
            HStack {
                Picker("Document", selection: $model.activePath) {
                    ForEach(model.documents, id: \.path) { Text($0.path).tag($0.path) }
                }
                .labelsHidden().frame(maxWidth: 220)
                Spacer()
                Text("\(model.activeText.utf8.count) UTF-8 bytes · \((model.activeText as NSString).length) UTF-16 units")
                    .font(.caption).foregroundStyle(.secondary)
            }
            .padding(8)
            SourceEditorView(
                text: Binding(get: { model.activeText }, set: { model.updateActiveText($0) }),
                selection: model.selection,
                pendingEdit: model.pendingEdit,
                onCaretChange: { model.caretUTF16 = $0 },
                onEditApplied: { model.editApplied($0, newText: $1) }
            )
            captureBar
        }
    }

    private var captureBar: some View {
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
        .sheet(item: Binding(get: { model.reviewing.map { ReviewItem(proposal: $0) } },
                             set: { model.reviewing = $0?.proposal })) { item in
            ProposalReviewSheet(proposal: item.proposal)
        }
    }

    private var previewPane: some View {
        VStack(spacing: 0) {
            if let result = model.result {
                PreviewView(result: result, dark: model.darkPreview, caretItems: model.caretItems) { source, text in
                    guard let source else { model.navigationNote = "This item has no source mapping."; return }
                    model.navigate(to: source, expectedText: text)
                }
                if !result.diagnostics.isEmpty {
                    Divider()
                    diagnosticsList(result.diagnostics)
                }
            } else {
                ContentUnavailableView("No compile result loaded", systemImage: "doc.richtext",
                                       description: Text("Use File > Open Compile Result Fixture…"))
            }
        }
    }

    private func diagnosticsList(_ diags: [RuntimeV1.Diagnostic]) -> some View {
        List(Array(diags.enumerated()), id: \.offset) { _, d in
            HStack(alignment: .top) {
                Image(systemName: d.severity == .error ? "xmark.octagon.fill" : "exclamationmark.triangle.fill")
                    .foregroundStyle(d.severity == .error ? .red : .yellow)
                VStack(alignment: .leading) {
                    Text(d.message)
                    if let rec = d.recovery { Text("recovery: \(rec)").font(.caption).foregroundStyle(.secondary) }
                }
                Spacer()
                if d.source != nil { Button("Go to source") { model.navigate(to: d.source) } }
            }
        }
        .frame(maxHeight: 140)
    }

    private var footer: some View {
        HStack {
            Text(model.navigationNote ?? "Click text in the preview to select its source range.")
                .font(.caption).foregroundStyle(.secondary).lineLimit(1)
            Spacer()
            if let note = model.captureNote {
                Text(note).font(.caption).foregroundStyle(.secondary).lineLimit(1)
            }
        }
        .padding(.horizontal, 12).padding(.vertical, 4)
    }

    private func statusColor(_ s: RuntimeV1.Status) -> Color {
        switch s { case .ok: .green; case .recovered: .orange; case .failed: .red }
    }
}


private struct ReviewItem: Identifiable {
    let proposal: RuntimeV1.CaptureProposal
    var id: String { proposal.captureId }
}

/// Review sheet: the reviewer sees ambiguities and dependencies, may edit the
/// LaTeX, and explicitly approves or rejects. Nothing is inserted otherwise.
private struct ProposalReviewSheet: View {
    @EnvironmentObject var model: ShellModel
    @Environment(\.dismiss) private var dismiss
    let proposal: RuntimeV1.CaptureProposal
    @State private var latex: String = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Review capture \(proposal.captureId)").font(.headline)
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
                Button("Approve and insert") {
                    if case .inserted = model.approveProposal(proposal, latex: latex) { dismiss() }
                }
                .keyboardShortcut(.defaultAction)
                .disabled(model.anchor == nil || latex.trimmingCharacters(in: .whitespaces).isEmpty)
            }
        }
        .padding(16)
        .frame(width: 520)
        .onAppear { latex = proposal.latex }
    }
}
