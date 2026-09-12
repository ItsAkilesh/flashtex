import SwiftUI
import FlashTeXProtocol
import FlashTeXAccessibility

/// The main window (mac-ui-redesign): a `NavigationSplitView` whose sidebar
/// is the project/outline/problems navigator (WorkspaceSidebar.swift) and
/// whose detail is the document tabs + editor and the preview side by side,
/// the Problems panel underneath (ProblemsPanel.swift) and a status bar at
/// the bottom. The window toolbar carries the everyday commands with their
/// menu shortcuts in tooltips, and View > Command Palette… (⌘⇧P) lists every
/// command of the accessibility table (CommandPalette.swift). Every action
/// here is an existing model operation; the redesign moves and labels them.
struct ContentView: View {
    @Environment(ShellModel.self) var model
    @Environment(\.openWindow) private var openWindow
    @State private var columns: NavigationSplitViewVisibility = .all
    /// Height of the Problems panel; remembered across launches.
    @AppStorage("FlashTeX.workspace.problemsHeight") private var problemsHeight: Double = ProblemsPanel.idealHeight

    var body: some View {
        @Bindable var model = model
        NavigationSplitView(columnVisibility: $columns) {
            WorkspaceSidebar()
                .navigationSplitViewColumnWidth(min: 200, ideal: 240, max: 360)
        } detail: {
            GeometryReader { geo in
                VStack(spacing: 0) {
                    HSplitView {
                        EditorPane().frame(minWidth: 340, maxWidth: .infinity)
                        PreviewPane().frame(minWidth: 380, maxWidth: .infinity)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    if model.problemsVisible {
                        // Drag the handle to give the diagnostics list more or less
                        // room; the list scrolls within whatever height it has.
                        PanelResizeHandle(height: $problemsHeight,
                                          range: ProblemsPanel.minHeight...max(ProblemsPanel.minHeight, geo.size.height - 240))
                        ProblemsPanel().frame(height: min(problemsHeight, max(ProblemsPanel.minHeight, geo.size.height - 240)))
                    }
                    Divider()
                    StatusBar()
                }
            }
        }
        .navigationSplitViewStyle(.balanced)
        .toolbar { WorkspaceToolbar(openWindow: openWindow) }
        .sheet(isPresented: $model.commandPaletteShown) { CommandPalette().environment(model) }
    }
}

/// The divider above the Problems panel, draggable up and down (the cursor
/// shows the resize arrows on hover). Keyboard users size it with the split
/// of the window itself; the panel is never taller than the window allows.
private struct PanelResizeHandle: View {
    @Binding var height: Double
    let range: ClosedRange<Double>
    @State private var startHeight: Double?

    var body: some View {
        Rectangle().fill(.clear)
            .frame(height: 7)
            .overlay(Divider(), alignment: .center)
            .contentShape(Rectangle())
            .onHover { inside in if inside { NSCursor.resizeUpDown.push() } else { NSCursor.pop() } }
            .gesture(DragGesture(minimumDistance: 1)
                .onChanged { value in
                    let start = startHeight ?? height
                    startHeight = start
                    height = min(max(start - value.translation.height, range.lowerBound), range.upperBound)
                }
                .onEnded { _ in startHeight = nil })
            .accessibilityHidden(true)
    }
}

// MARK: toolbar

/// Labelled toolbar items; each tooltip names the menu shortcut so the
/// keyboard workflow is discoverable from the toolbar itself.
private struct WorkspaceToolbar: ToolbarContent {
    @Environment(ShellModel.self) var model
    let openWindow: OpenWindowAction

    var body: some ToolbarContent {
        @Bindable var model = model
        ToolbarItemGroup(placement: .principal) {
            Button {
                if !model.outputBoundExplicitRetry() { model.compile() }
            } label: { Label("Compile", systemImage: "hammer.fill") }
                .labelStyle(.titleAndIcon)
                .disabled(!model.workerAttached)
                .help("Send the current buffers to the attached producer (File > Compile, ⌘B)")
            Menu {
                Button("Attach Built Compiler") { _ = model.attachDiscoveredWorker() }
                    .help("⌘⇧K")
                Button("Attach Render Pipeline (Latin Modern)") { _ = model.attachDiscoveredRenderPipeline() }
                    .help("⌘⇧R")
                Button("Attach Worker Executable…") { model.attachWorkerPanel() }
                    .help("⌘K")
                Divider()
                Toggle("Auto-compile after edits", isOn: $model.autoCompile).disabled(!model.workerAttached)
                Button("Detach Worker") { model.detachWorker() }.disabled(!model.workerAttached)
                if model.isFixture { Divider(); Button("Reload Fixture") { model.reloadFixture() } }
            } label: {
                Label(model.workerAttached ? "Producer" : "Attach", systemImage: model.workerAttached ? "cpu.fill" : "cpu")
            }
            .help("Producer: " + (model.isFixture ? "fixture (not a real compile)" : model.workerStatus) + " — attach the built compiler (⌘⇧K), the Latin Modern render pipeline (⌘⇧R) or any executable (⌘K)")
        }
        ToolbarItemGroup(placement: .automatic) {
            Toggle(isOn: $model.previewV2) { Label("v2 pane", systemImage: "rectangle.on.rectangle") }
                .toggleStyle(.button)
                .help("Experimental display-list-v2 preview (File > Open Display List (v2)…)")
            Toggle(isOn: $model.darkPreview) { Label("Dark preview", systemImage: "moon") }
                .toggleStyle(.button)
                .help("Draw the preview pages dark (page and text colors only)")
            Button { openWindow(id: ProjectSearch.windowID) } label: { Label("Find in Project", systemImage: "magnifyingglass") }
                .help("Find in Project… (Edit, ⌘⇧F)")
            Button { openWindow(id: CitationRename.windowID) } label: { Label("Rename Citation", systemImage: "quote.bubble") }
                .help("Rename Citation… (Edit): reviewed plan across the project")
            Button { openWindow(id: EditHistoryPanel.windowID) } label: { Label("Durable History", systemImage: "clock.arrow.circlepath") }
                .help("Durable History… (Edit): undo/redo on the helper's edit ledger")
            Menu {
                Button("Export PDF…") { model.exportPDF() }.disabled(model.result == nil)
                Button("Export PDF via Rust Writer…") { model.exportPDFViaRust() }.disabled(model.result == nil)
                Button("Export PDF (exact, v2)…") { model.exportPDFExact() }.disabled(model.displayListV2?.frame == nil)
            } label: { Label("Export", systemImage: "square.and.arrow.up") }
                .help("Export PDF… (⌘⇧E), via Rust writer (⌘⌥E), or exact from the v2 display list (File menu)")
            Button { openWindow(id: "nearby") } label: { Label("Nearby", systemImage: "ipad.and.iphone") }
                .help("Nearby Companion… (Edit, ⌘⇧N): pair an iPad/iPhone to send captures")
            Toggle(isOn: $model.problemsVisible) {
                let n = model.displayedDiagnostics.count
                Label(n > 0 ? "Problems \(n)" : "Problems", systemImage: n > 0 ? "exclamationmark.triangle.fill" : "exclamationmark.triangle")
            }
            .toggleStyle(.button)
            .help("Show or hide the Problems panel (View, ⌘⇧M)")
            Button { model.commandPaletteShown = true } label: { Label("Commands", systemImage: "command") }
                .help("Command Palette… (View, ⌘⇧P): every command with its shortcut")
                .accessibilityIdentifier("toolbar.command-palette")
        }
    }
}

// MARK: pieces
//
// Each pane is its own view reading the model from the environment, so
// `@Observable` tracking scopes invalidation: a keystroke (documents,
// editorRevision) re-evaluates EditorPane and the status bar; a compile
// result the status bar, PreviewPane and ProblemsPanel; bridge traffic the
// bridge bar only.

private struct EditorPane: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        @Bindable var model = model
        VStack(spacing: 0) {
            // Switching goes through ProjectDocuments so each document's
            // caret/selection is kept and a pending insertion is never
            // applied to the wrong buffer (ProjectDocuments.swift).
            DocumentTabBar()
            Divider()
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
                onEditApplied: { model.editApplied($0, newText: $1) },
                onEditRefused: { model.editRefused($0, reason: $1) },
                autoClosePairs: EditorPreferences.shared.autoCloseBraces ? model.autoClosePairs : [] // EditorPreferences.swift gates the braces lane set
                ,
                syntaxHighlighting: true, // SyntaxHighlighter.swift / EditorIntelligence.swift (mac-syntax-highlight)
                showLineNumbers: true,
                onDefinitionRequest: { target in
                    switch target {
                    case .label, .citation, .environment: model.goToMatching() // caret already on the token
                    case .file(let path, _): Task { await model.project.openDocument(path, role: .opened) }
                    }
                }
            )
            CaptureBar()
            BridgeBar()
        }
    }
}

private struct CaptureBar: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        HStack(spacing: 8) {
            Button("Pin insertion point") { model.pinAnchorAtCaret() }
                .controlSize(.small)
                .help("Use the caret as the destination for capture proposals (⌘⌥P)")
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
                .controlSize(.small)
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
            PreviewHeader()
            Divider()
            if model.previewV2 {
                PreviewV2Pane() // experimental v2 path (PreviewV2View.swift); v1 below stays the default
            } else if let result = model.result {
                PreviewView(result: result, dark: model.darkPreview, caretItems: model.caretItems) { source, text in
                    guard let source else { model.navigationNote = "This item has no source mapping."; return }
                    model.navigate(to: source, expectedText: text)
                }
            } else {
                ContentUnavailableView {
                    Label("No preview yet", systemImage: "doc.richtext")
                } description: {
                    Text("Attach a producer from the toolbar (⌘⇧K builds, ⌘⇧R Latin Modern) and compile (⌘B), or File > Open Compile Result Fixture… (⌘⇧O).")
                }
            }
        }
        .sheet(isPresented: Binding(get: { model.quickFix != nil }, set: { if !$0 { model.quickFix = nil } })) {
            if let p = model.quickFix {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Suggested fix").font(.headline)
                    Text(p.summary).font(.caption).foregroundStyle(.secondary)
                    Text("Before").font(.caption.bold())
                    Text(p.before).font(.system(.body, design: .monospaced)).textSelection(.enabled)
                    Text("After").font(.caption.bold())
                    Text(p.after).font(.system(.body, design: .monospaced)).textSelection(.enabled)
                    Text("Heuristic suggestion from the explanation catalogue; applied as one undoable edit only when you choose Apply.")
                        .font(.caption2).foregroundStyle(.tertiary)
                    HStack {
                        Spacer()
                        Button("Cancel") { model.quickFix = nil }.keyboardShortcut(.cancelAction)
                        Button("Apply") { model.applyQuickFix() }.keyboardShortcut(.defaultAction)
                    }
                }
                .padding(16).frame(minWidth: 480)
                .accessibilityElement(children: .contain).accessibilityLabel("Suggested fix preview")
            }
        }
    }
}

/// The preview column's header: source badge, compile status, freshness
/// (historical / stale / compiling) and the layout capabilities — the former
/// top banner, kept to one line with details in tooltips.
private struct PreviewHeader: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        HStack(spacing: 8) {
            sourceBadge
            if let r = model.result {
                Text(sourceName).font(.caption).lineLimit(1)
                    .help("result id \(model.resultID ?? "?") · project \(r.projectId) · revision \(r.revision) · pdf: \(r.pdfPath ?? "none")")
                Text(r.status.rawValue).font(.caption.bold()).foregroundStyle(statusColor(r.status))
                if r.status == .recovered {
                    Text("provisional rendering").font(.caption).foregroundStyle(.orange)
                        .help("recovered: preview shown with provisional rendering")
                }
                if model.inFlightRevision != nil { ProgressView().controlSize(.mini) }
                if let historical = model.historicalPreview {
                    Text(historical.label).font(.caption.bold()).foregroundStyle(.purple).lineLimit(1)
                        .help("A completed older snapshot is shown while the helper compiles the newer revision; navigation, caret sync, capture destinations and export return with the current preview.")
                } else if model.previewIsStale {
                    Text(model.outputBound?.banner // the reply exceeded a bound: nothing is compiling (ShellModel+OutputBounds.swift)
                         ?? (model.workerAttached
                         ? (model.autoCompile ? "editor at r\(model.editorRevision) — compiling…" : "editor at r\(model.editorRevision) — ⌘B to compile")
                         : "editor at r\(model.editorRevision) — no producer attached"))
                        .font(.caption).foregroundStyle(model.outputBound != nil || !model.workerAttached ? .orange : .secondary).lineLimit(1) // routine "compiling…" is quiet; only bounds/no-producer are highlighted
                }
            } else if let err = model.loadError {
                Text(err).font(.caption).foregroundStyle(.red).lineLimit(1).help(err)
            } else {
                Text("Preview").font(.caption.bold()).foregroundStyle(.secondary)
            }
            Spacer()
            ForEach(model.capabilityNotes, id: \.self) { note in
                Image(systemName: "exclamationmark.circle").foregroundStyle(.orange).help(note)
                    .accessibilityLabel(note)
            }
            Text(model.negotiation.accepted.isEmpty ? "legacy layout" : model.negotiation.accepted.joined(separator: ", "))
                .font(.caption2).foregroundStyle(.tertiary).lineLimit(1)
                .help(model.negotiation.accepted.isEmpty
                      ? "No layout capability accepted for this result: U+2500 fraction bars are an approximation."
                      : "Capabilities the worker accepted for this result (typed rules / explicit font hints).")
        }
        .padding(.horizontal, 10).padding(.vertical, 5)
        .background(.bar)
    }

    private var sourceBadge: some View {
        let (label, color): (String, Color) = switch model.previewSource {
        case .none: ("NONE", .gray)
        case .fixture: ("FIXTURE", .orange)
        case .worker: model.historicalPreview != nil ? ("HISTORICAL", .purple) : ("WORKER", .green)
        }
        return Text(label)
            .font(.caption2.bold())
            .padding(.horizontal, 6).padding(.vertical, 2)
            .background(color.opacity(0.25), in: Capsule())
            .help(model.isFixture ? "Not a real compile." : model.workerStatus)
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

/// Bottom status bar: editor revision, the helper's durable revision of the
/// active document, compile latency, the route (fixture / worker /
/// controller), then the last navigation, staleness or capture note, and an
/// exact-export progress control while one runs.
private struct StatusBar: View {
    @Environment(ShellModel.self) var model

    var body: some View {
        HStack(spacing: 12) {
            Label("r\(model.editorRevision)", systemImage: "pencil.line")
                .help("Editor revision (increments on every edit)")
            if let durable = model.controllerState.durable[model.activePath]?.revision {
                Label("durable r\(durable)", systemImage: "internaldrive")
                    .help("Durable revision of \(model.activePath) in the helper's edit ledger")
            }
            if let ms = model.lastLatencyMs, let med = model.medianLatencyMs {
                Label(String(format: "%.0f ms", ms), systemImage: "timer")
                    .help(String(format: "Last compile latency %.0f ms (median %.0f over %d)", ms, med, model.latenciesMs.count))
            }
            Label(route, systemImage: routeIcon)
                .help(model.isFixture ? "Not a real compile." : (model.controllerAttached ? model.controllerStatus : model.workerStatus))
            let diags = model.displayedDiagnostics
            if !diags.isEmpty {
                let errors = diags.filter { $0.severity == .error }.count
                Button {
                    model.problemsVisible.toggle()
                } label: {
                    HStack(spacing: 6) {
                        if errors > 0 { Label("\(errors)", systemImage: "xmark.octagon.fill").foregroundStyle(.red) }
                        if diags.count - errors > 0 { Label("\(diags.count - errors)", systemImage: "exclamationmark.triangle.fill").foregroundStyle(.orange) }
                    }
                }
                .buttonStyle(.plain)
                .help("Errors and warnings of the last result — click to show or hide the Problems panel (⌘⇧M)")
            }
            Divider().frame(height: 12)
            Text(model.navigationNote ?? model.editorMarkReport.staleNote ?? model.explanationStatus
                 ?? "Click text in the preview to select its source range.")
                .foregroundStyle(.secondary).lineLimit(1)
            Spacer()
            if case .running(let pid, _) = model.exportSession.state { // ShellModel+ExportSession.swift
                ProgressView().controlSize(.small)
                Text("Exporting exact PDF (flashtex-pdf-exact pid \(pid))…")
                    .foregroundStyle(.secondary).lineLimit(1)
                Button("Cancel") { model.cancelExactExport() }
                    .controlSize(.small)
                    .help("Terminate flashtex-pdf-exact; nothing is written to the destination")
                    .accessibilityIdentifier("export.cancel")
            } else if let note = model.captureNote {
                Text(note).foregroundStyle(.secondary).lineLimit(1).help(note)
            }
        }
        .font(.caption)
        .monospacedDigit()
        .padding(.horizontal, 12).padding(.vertical, 4)
        .background(.bar)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Status bar")
    }

    private var route: String {
        if model.isFixture { return "fixture" }
        if model.controllerAttached { return "controller" }
        if model.workerAttached { return "worker" }
        return "no producer"
    }

    private var routeIcon: String {
        if model.isFixture { return "doc.badge.gearshape" }
        return model.workerAttached ? "bolt.horizontal.circle.fill" : "bolt.horizontal.circle"
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
