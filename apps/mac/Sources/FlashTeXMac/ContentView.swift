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
            ToolbarItem { Button("Reload fixture") { model.reloadFixture() } }
        }
    }

    // MARK: pieces

    private var statusBanner: some View {
        HStack(spacing: 12) {
            Label("FIXTURE", systemImage: "doc.text.magnifyingglass")
                .font(.caption.bold())
                .padding(.horizontal, 6).padding(.vertical, 2)
                .background(Color.orange.opacity(0.25), in: Capsule())
            if let r = model.result {
                Text("\(model.fixtureURL?.lastPathComponent ?? "?") · id \(model.resultID ?? "?") · project \(r.projectId) · revision \(r.revision)")
                Text("status: \(r.status.rawValue)")
                    .foregroundStyle(statusColor(r.status)).bold()
                Text("pdf: \(r.pdfPath ?? "none")").foregroundStyle(.secondary)
                if model.previewIsStale {
                    Text("editor at revision \(model.editorRevision) — preview not recompiled (no compiler attached)")
                        .foregroundStyle(.orange)
                }
            } else if let err = model.loadError {
                Text(err).foregroundStyle(.red)
            }
            Spacer()
            Text("Not a real compile.").font(.caption).foregroundStyle(.secondary)
        }
        .font(.callout)
        .padding(.horizontal, 12).padding(.vertical, 6)
        .background(.bar)
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
                selection: model.selection
            )
        }
    }

    private var previewPane: some View {
        VStack(spacing: 0) {
            if let result = model.result {
                PreviewView(result: result, dark: model.darkPreview) { model.navigate(to: $0) }
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
        }
        .padding(.horizontal, 12).padding(.vertical, 4)
    }

    private func statusColor(_ s: RuntimeV1.Status) -> Color {
        switch s { case .ok: .green; case .recovered: .orange; case .failed: .red }
    }
}
