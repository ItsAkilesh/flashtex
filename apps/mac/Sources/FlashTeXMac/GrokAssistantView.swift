import AppKit
import SwiftUI

/// The Ask Grok sheet (GrokAssistant.swift): instruction, context line, the
/// provider state, Ask/Cancel with the live elapsed counter, then the
/// explanation, the helper's notes, the proposed edits as a before/after diff
/// with Apply (one undoable edit) and Copy. Errors are shown verbatim.
struct GrokAssistantView: View {
    @Environment(ShellModel.self) private var model
    @ObservedObject var assistant: GrokAssistant
    @FocusState private var instructionFocused: Bool

    static let identifier = "grok.assistant"

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 6) {
                Image(systemName: "sparkles").foregroundStyle(.purple)
                Text("Ask Grok").font(.headline)
                Spacer()
                Text(assistant.providerStatusText)
                    .font(.caption).monospacedDigit()
                    .padding(.horizontal, 6).padding(.vertical, 2)
                    .background((assistant.grokLive ? Color.green : Color.secondary).opacity(0.15)).cornerRadius(4)
                    .help(assistant.providerStatusHelp)
                    .accessibilityIdentifier("grok.assistant.status")
            }
            TextField(GrokAssistant.instructionPlaceholder, text: $assistant.instruction, axis: .vertical)
                .lineLimit(2...6)
                .textFieldStyle(.roundedBorder)
                .focused($instructionFocused)
                .disabled(assistant.inFlight)
                .accessibilityLabel("Instruction for Grok")
                .accessibilityIdentifier("grok.assistant.instruction")
                .onSubmit { if assistant.canAsk { model.askGrokNow() } }
            HStack(spacing: 8) {
                Text(contextLine).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                    .accessibilityIdentifier("grok.assistant.context")
                if !diagnosticsLine.isEmpty { Text("·").foregroundStyle(.tertiary); Text(diagnosticsLine).font(.caption).foregroundStyle(.secondary) }
                Spacer()
                if assistant.inFlight {
                    ProgressView().controlSize(.small)
                    if let started = assistant.providerStartedAt, let grokModel = assistant.model {
                        TimelineView(.periodic(from: started, by: 1)) { context in
                            let elapsed = max(0, Int(context.date.timeIntervalSince(started)))
                            Text("Asking Grok (\(grokModel))… \(elapsed) s").font(.caption).monospacedDigit().foregroundStyle(.secondary)
                                .accessibilityLabel("Asking Grok, \(elapsed) seconds elapsed of at most \(Int(assistant.configuration.providerTimeout))")
                        }
                    }
                    Button("Cancel") { assistant.cancel() }
                        .help("Stops waiting and terminates the helper's provider session; nothing is applied")
                        .accessibilityIdentifier("grok.assistant.cancel")
                } else {
                    Button("Ask") { model.askGrokNow() }
                        .keyboardShortcut(.defaultAction)
                        .disabled(!assistant.canAsk)
                        .help("Binds the last compile of this document, sends the selection (or the document head) and your instruction to Grok through the assistant helper; nothing is applied until you choose Apply")
                        .accessibilityIdentifier("grok.assistant.ask")
                }
            }
            if let note = assistant.coverageNote {
                Label(note, systemImage: "exclamationmark.triangle").font(.caption).foregroundStyle(.orange)
                    .accessibilityIdentifier("grok.assistant.coverage")
            }
            Text(assistant.statusText).font(.caption2).foregroundStyle(statusColor).lineLimit(4).textSelection(.enabled)
                .accessibilityIdentifier("grok.assistant.statusline")
            switch assistant.state {
            case .ready(let reply), .approving(let reply): replyBody(reply)
            default: EmptyView()
            }
            HStack {
                Text("Model output — never applied automatically. The document is written only by Apply, as one undoable edit.")
                    .font(.caption2).foregroundStyle(.tertiary)
                Spacer()
                Button("Close") { assistant.shown = false }.keyboardShortcut(.cancelAction)
            }
        }
        .padding(16)
        .frame(minWidth: 560, idealWidth: 640, minHeight: 240)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Ask Grok")
        .accessibilityIdentifier(Self.identifier)
        .onAppear { instructionFocused = true }
    }

    private var contextLine: String {
        if let snapshot = model.grokSnapshot() {
            if let sel = snapshot.selection, !sel.isEmpty {
                let (a, b) = GrokAssistant.lines(of: sel, in: snapshot.activeText)
                return "Selection: line\(a == b ? " \(a)" : "s \(a)–\(b)") of \(snapshot.activePath)"
            }
            return "Whole document: \(snapshot.activePath)" + (model.previewIsStale ? " (buffer ahead of the compile)" : "")
        }
        return "No compile result yet — compile (⌘B) first"
    }

    private var diagnosticsLine: String {
        guard let snapshot = model.grokSnapshot() else { return "" }
        var s = snapshot
        s.pinnedDiagnostic = assistant.pinnedDiagnostic
        let n = GrokAssistant.selectedDiagnostics(s).count
        if s.selection.map({ !$0.isEmpty }) == true || n > 0 { return "\(n) diagnostic\(n == 1 ? "" : "s") in range" }
        return ""
    }

    private var statusColor: Color {
        switch assistant.state {
        case .failed, .unavailable: return .red
        case .applied: return .green
        default: return .secondary
        }
    }

    @ViewBuilder private func replyBody(_ reply: GrokAssistant.Reply) -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 6) {
                Text(reply.text).font(.body).textSelection(.enabled)
                    .padding(8).frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color.purple.opacity(0.08)).cornerRadius(6)
                    .accessibilityLabel("Grok's explanation, not applied")
                    .accessibilityIdentifier("grok.assistant.explanation")
                ForEach(reply.notes, id: \.self) { note in
                    Label(note, systemImage: "minus.circle").font(.caption).foregroundStyle(.orange)
                }
                if let preview = assistant.applicationPreview {
                    Text("Proposed edit — \(reply.edits.count) change\(reply.edits.count == 1 ? "" : "s")" + (reply.edits.contains { $0.relocated } ? " (relocated by removed text)" : ""))
                        .font(.caption.bold())
                    diff(preview)
                } else {
                    ForEach(reply.edits) { edit in
                        VStack(alignment: .leading, spacing: 1) {
                            Text("Edit at bytes \(edit.range.startByte)..<\(edit.range.endByte)" + (edit.relocated ? " (relocated)" : "")).font(.caption2).foregroundStyle(.secondary)
                            Text("− " + edit.removedText).font(.system(.caption, design: .monospaced)).foregroundStyle(.red)
                            Text("+ " + edit.replacement).font(.system(.caption, design: .monospaced)).foregroundStyle(.green)
                        }
                    }
                    if let refusal = assistant.applicationRefusal {
                        Label("Cannot apply: \(refusal)", systemImage: "xmark.octagon").font(.caption).foregroundStyle(.red)
                            .accessibilityIdentifier("grok.assistant.refusal")
                    }
                }
                HStack {
                    Spacer()
                    Button("Copy") { copy(reply) }
                        .help("Copies the explanation and the proposed replacement text")
                        .accessibilityIdentifier("grok.assistant.copy")
                    if reply.reviewId != nil {
                        Button("Apply") { model.applyGrokEdit() }
                            .disabled(!assistant.canApply || assistant.inFlight)
                            .help("Helper approval of this exact review, then one undoable edit in the editor (⌘Z undoes it)")
                            .accessibilityIdentifier("grok.assistant.apply")
                    }
                }
            }
        }
        .frame(maxHeight: 360)
    }

    @ViewBuilder private func diff(_ p: EditorDiagnostics.QuickFix.Preview) -> some View {
        Text("Before").font(.caption2).foregroundStyle(.secondary)
        Text(p.before).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
            .padding(6).frame(maxWidth: .infinity, alignment: .leading).background(Color.red.opacity(0.06)).cornerRadius(4)
            .accessibilityIdentifier("grok.assistant.before")
        Text("After").font(.caption2).foregroundStyle(.secondary)
        Text(p.after).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
            .padding(6).frame(maxWidth: .infinity, alignment: .leading).background(Color.green.opacity(0.06)).cornerRadius(4)
            .accessibilityIdentifier("grok.assistant.after")
    }

    private func copy(_ reply: GrokAssistant.Reply) {
        var text = reply.text
        if let p = assistant.applicationPreview { text += "\n\n" + p.after }
        else { for e in reply.edits { text += "\n\n" + e.replacement } }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(text, forType: .string)
    }
}

/// Presents the Ask Grok sheet over any view of the shell; `ContentView`
/// applies it in one line (`.grokAssistantSheet()`).
struct GrokAssistantSheet: ViewModifier {
    @Environment(ShellModel.self) private var model

    func body(content: Content) -> some View {
        content.modifier(Presenter(assistant: model.grokAssistant, model: model))
    }

    private struct Presenter: ViewModifier {
        @ObservedObject var assistant: GrokAssistant
        var model: ShellModel
        func body(content: Content) -> some View {
            content.sheet(isPresented: $assistant.shown) { GrokAssistantView(assistant: assistant).environment(model) }
        }
    }
}

extension View {
    func grokAssistantSheet() -> some View { modifier(GrokAssistantSheet()) }
}
