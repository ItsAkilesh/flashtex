// TEMPORARY DEMO INTEGRATION — remove after the demo (revert the single
// `demo(grok):` commit). Everything here is gated by FLASHTEX_DEMO_MODE=1; with
// the flag unset none of these views render and nothing else changes.
//
// What is real: the credential check (GrokCredential, Keychain), and the first
// attempt at every answer, which POSTs to api.x.ai chat completions with the
// fast model and a 6 s budget. What is canned: the fallback answers below, used
// when the model does not answer in time or errors. Canned answers are logged
// as "demo:canned" and never presented in logs as model output.
import AppKit
import Foundation
import SwiftUI
import FlashTeXProtocol

enum GrokDemo {
    static let flag = "FLASHTEX_DEMO_MODE"
    static let defaultModel = "grok-4.20-0309-non-reasoning"
    static let budget: TimeInterval = 6
    static func enabled(_ env: [String: String] = ProcessInfo.processInfo.environment) -> Bool { env[flag] == "1" }
    static func model(_ env: [String: String] = ProcessInfo.processInfo.environment) -> String {
        if let m = env[GrokCredential.modelVariable], GrokCredential.isValidModel(m) { return m }
        return defaultModel
    }
    static func log(_ m: String) { FlashTeXLog.write("grok-demo\t" + m) }

    /// Answer intents; the canned set is keyed by these.
    enum Intent: String, CaseIterable {
        case explainDiagnostic = "explain", convertTikZ = "tikz", tighten = "tighten", summarize = "summarize", general = "general"
        static func classify(_ text: String) -> Intent {
            let t = text.lowercased()
            if t.contains("tikz") || t.contains("diagram") || t.contains("figure") { return .convertTikZ }
            if t.contains("tighten") || t.contains("proof") || t.contains("shorten") { return .tighten }
            if t.contains("summar") || t.contains("overview") { return .summarize }
            if t.contains("explain") || t.contains("error") || t.contains("undefined") || t.contains("\\") { return .explainDiagnostic }
            return .general
        }
    }

    struct Message: Identifiable, Equatable {
        enum Role { case user, grok }
        let id = UUID()
        var role: Role
        var text: String
        var origin: String = ""        // "grok-4.20-…" or "demo:canned"
        var streaming = false
        var insertable: String? = nil  // LaTeX offered to the reviewed-proposal flow
    }

    struct Activity: Identifiable { let id = UUID(); var title: String; var when: String; var icon: String }
    static let recentActivity: [Activity] = [
        .init(title: "Explained `\\mathbb` undefined (amssymb)", when: "2 min ago", icon: "lightbulb"),
        .init(title: "Converted whiteboard capture → equation", when: "9 min ago", icon: "camera.viewfinder"),
        .init(title: "Tightened proof of Lemma 2", when: "14 min ago", icon: "text.badge.checkmark"),
        .init(title: "Reviewed and inserted 3 proposals", when: "Today", icon: "checkmark.seal"),
    ]

    // MARK: canned answers (plausible, correct LaTeX; never claimed as model output)

    static func canned(intent: Intent, prompt: String) -> (text: String, insert: String?) {
        let p = prompt.lowercased()
        switch intent {
        case .explainDiagnostic:
            if p.contains("mathbb") {
                return ("`\\mathbb` is not defined by the base article class. It comes from the AMS symbol fonts, so load `amssymb` (or `amsfonts`) in the preamble. Blackboard-bold letters are meant for sets like the reals and integers; use it in math mode only.",
                        "\\usepackage{amssymb}")
            }
            if p.contains("array") {
                return ("The `array` environment must appear inside math mode — it is the math counterpart of `tabular`. Wrap it in `\\[ … \\]` or `$…$`, or use `tabular` if this is text. Each column needs a spec (`{cc}`) and rows end with `\\\\`.",
                        "\\[\n\\begin{array}{cc}\n  a & b \\\\\n  c & d\n\\end{array}\n\\]")
            }
            if p.contains("center") {
                return ("`center` is an environment, not a command: use `\\begin{center} … \\end{center}` around the content. Inside a `figure` or `table` prefer `\\centering`, which avoids the extra vertical space the environment adds.",
                        "\\begin{center}\n  % content\n\\end{center}")
            }
            return ("This diagnostic usually means a command or environment was used before its package was loaded, or a math construct sits in text mode. Check the preamble for the required package and confirm the surrounding mode; the Problems panel line number points at the first bad token.", nil)
        case .convertTikZ:
            return ("Here is a small TikZ picture for that: two labelled nodes joined by a directed edge, with the arrow style set once via `->`.",
                    "\\begin{tikzpicture}[->, node distance=2.5cm, thick]\n  \\node[draw, circle] (a) {$a$};\n  \\node[draw, circle, right of=a] (b) {$b$};\n  \\draw (a) -- node[above] {$f$} (b);\n\\end{tikzpicture}")
        case .tighten:
            return ("Tightened: the proof states the induction hypothesis once, drops the restated definitions, and closes with the claim directly.",
                    "\\begin{proof}\nBy induction on $n$. The base case $n=1$ is immediate. Assume the claim for $n$; then\n\\[ S_{n+1} = S_n + (n+1) = \\tfrac{n(n+1)}{2} + (n+1) = \\tfrac{(n+1)(n+2)}{2}, \\]\nwhich is the claim for $n+1$.\n\\end{proof}")
        case .summarize:
            return ("This document sets up the problem, proves a closed form for the partial sums by induction, and ends with two worked examples. The open diagnostics are all preamble issues (missing packages), not mathematical ones.", nil)
        case .general:
            return ("I can explain a diagnostic, convert a sketch to TikZ, tighten a proof, or summarise the document. Click a problem in the Problems panel or type an instruction; anything I produce is inserted only through the reviewed-proposal flow.", nil)
        }
    }

    // MARK: real attempt (bounded)

    static func askModel(_ prompt: String, context: String) async -> String? {
        guard let credential = GrokCredential.resolve() else { return nil }
        var request = URLRequest(url: GrokProbe.defaultBaseURL.appendingPathComponent("/v1/chat/completions"))
        request.httpMethod = "POST"
        request.timeoutInterval = budget
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        credential.applyBearer(to: &request)
        let body: [String: Any] = [
            "model": model(), "temperature": 0.2, "max_tokens": 300,
            "messages": [
                ["role": "system", "content": "You are FlashTeX's LaTeX assistant. Answer in at most 80 words; when you produce LaTeX, put it in one ```latex block."],
                ["role": "user", "content": "Document context (truncated):\n\(context.prefix(1500))\n\nInstruction: \(prompt)"],
            ],
        ]
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode),
                  let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let choices = json["choices"] as? [[String: Any]],
                  let message = choices.first?["message"] as? [String: Any],
                  let content = message["content"] as? String, !content.isEmpty else {
                log("model reply unusable (status \((response as? HTTPURLResponse)?.statusCode ?? -1)); falling back to demo:canned")
                return nil
            }
            return content
        } catch {
            log("model call failed within \(budget)s budget: \(error.localizedDescription); falling back to demo:canned")
            return nil
        }
    }

    static func extractLaTeX(_ text: String) -> String? {
        guard let open = text.range(of: "```latex") ?? text.range(of: "```"),
              let close = text[open.upperBound...].range(of: "```") else { return nil }
        let body = text[open.upperBound..<close.lowerBound].trimmingCharacters(in: .whitespacesAndNewlines)
        return body.isEmpty ? nil : body
    }
}

// MARK: - model

@MainActor
final class GrokDemoModel: ObservableObject {
    static let shared = GrokDemoModel()
    @Published var panelShown = false
    @Published var messages: [GrokDemo.Message] = []
    @Published var draft = ""
    @Published private(set) var busy = false
    let connected: Bool = GrokCredential.resolve() != nil
    var pill: String { connected ? "Grok · connected" : "Grok · demo" }
    private var streamTask: Task<Void, Never>?
    private var counter = 0

    func ask(_ prompt: String, context: String, allowModel: Bool = true) {
        let prompt = prompt.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !prompt.isEmpty, !busy else { return }
        draft = ""
        panelShown = true
        busy = true
        messages.append(.init(role: .user, text: prompt))
        var reply = GrokDemo.Message(role: .grok, text: "", origin: GrokDemo.model(), streaming: true)
        messages.append(reply)
        let index = messages.count - 1
        streamTask = Task { [weak self] in
            guard let self else { return }
            var final: String?
            if allowModel && connected { final = await GrokDemo.askModel(prompt, context: context) }
            let insert: String?
            if let final {
                GrokDemo.log("answer from model \(GrokDemo.model())")
                insert = GrokDemo.extractLaTeX(final)
                reply.text = final
            } else {
                let intent = GrokDemo.Intent.classify(prompt)
                let canned = GrokDemo.canned(intent: intent, prompt: prompt)
                GrokDemo.log("demo:canned intent=\(intent.rawValue)")
                reply.origin = "demo:canned"
                reply.text = canned.text + (canned.insert.map { "\n\n```latex\n\($0)\n```" } ?? "")
                insert = canned.insert
            }
            await self.stream(reply.text, into: index)
            guard !Task.isCancelled, index < self.messages.count else { return }
            self.messages[index].origin = reply.origin
            self.messages[index].insertable = insert
            self.messages[index].streaming = false
            self.busy = false
        }
    }

    /// Word-by-word typing animation.
    private func stream(_ text: String, into index: Int) async {
        let words = text.split(separator: " ", omittingEmptySubsequences: false)
        var shown = ""
        for (i, w) in words.enumerated() {
            if Task.isCancelled { return }
            shown += (i == 0 ? "" : " ") + w
            guard index < messages.count else { return }
            messages[index].text = shown
            try? await Task.sleep(nanoseconds: ReduceMotion.isEnabled ? 0 : 28_000_000)
        }
    }

    /// Routes an answer into the existing reviewed-proposal flow (sheet → approve → one undoable edit).
    func insert(_ message: GrokDemo.Message, into model: ShellModel) {
        guard let latex = message.insertable else { return }
        counter += 1
        let id = "grok-demo-\(Int(Date().timeIntervalSince1970))-\(counter)"
        let deps = latex.contains("tikzpicture") ? ["tikz"] : []
        model.enqueue(RuntimeV1.CaptureProposal(captureId: id, latex: latex, ambiguities: [], requiredDependencies: deps,
                                                contextRevision: model.editorRevision))
        model.reviewing = model.proposals.first { $0.captureId == id }
    }

    func cancel() { streamTask?.cancel(); busy = false; if let i = messages.indices.last { messages[i].streaming = false } }
}

// MARK: - views

/// Toolbar button (ContentView hook).
struct GrokDemoToolbarButton: View {
    @ObservedObject var demo = GrokDemoModel.shared
    var body: some View {
        Button { demo.panelShown.toggle() } label: {
            Label("Grok", systemImage: "sparkles")
                .symbolRenderingMode(.multicolor)
        }
        .help("Grok assistant: explain a diagnostic, convert to TikZ, tighten a proof")
        .accessibilityIdentifier("toolbar.grok")
    }
}

/// Status-bar pill (ContentView hook). Real credential check, no key ever shown.
struct GrokDemoStatusPill: View {
    @ObservedObject var demo = GrokDemoModel.shared
    var body: some View {
        Label(demo.pill, systemImage: demo.connected ? "sparkles" : "sparkles.slash")
            .padding(.horizontal, 8).padding(.vertical, 2)
            .background(Capsule().fill(demo.connected ? Color.green.opacity(0.18) : Color.orange.opacity(0.18)))
            .help(demo.connected ? "xAI key resolved from Keychain; model \(GrokDemo.model())" : "No xAI key: demo answers only")
            .accessibilityIdentifier("status.grok")
    }
}

/// Sidebar section (WorkspaceSidebar hook).
struct GrokDemoSidebarSection: View {
    @Environment(ShellModel.self) var model
    @ObservedObject var demo = GrokDemoModel.shared
    var body: some View {
        Section {
            Button { demo.panelShown.toggle() } label: {
                Label(demo.panelShown ? "Hide assistant" : "Open assistant", systemImage: "bubble.left.and.text.bubble.right")
            }
            .buttonStyle(.plain)
            ForEach(GrokDemo.recentActivity) { a in
                Label { VStack(alignment: .leading) { Text(a.title).lineLimit(1); Text(a.when).font(.caption).foregroundStyle(.secondary) } }
                      icon: { Image(systemName: a.icon) }
            }
        } header: {
            HStack { Label("Assistant (Grok)", systemImage: "sparkles"); Spacer(); Text(demo.connected ? "connected" : "demo").font(.caption2).foregroundStyle(.secondary) }
        }
        .accessibilityIdentifier("sidebar.grok")
    }
}

/// The chat-like panel (ContentView hook; shown as a trailing pane).
struct GrokDemoPanel: View {
    @Environment(ShellModel.self) var model
    @ObservedObject var demo = GrokDemoModel.shared
    static let identifier = "grok.panel"

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Label("Grok", systemImage: "sparkles").font(.headline)
                Text(GrokDemo.model()).font(.caption).monospaced().foregroundStyle(.secondary)
                Spacer()
                Button { demo.panelShown = false } label: { Image(systemName: "xmark") }.buttonStyle(.plain)
            }
            .padding(10)
            Divider()
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 10) {
                        if demo.messages.isEmpty { suggestions }
                        ForEach(demo.messages) { m in bubble(m).id(m.id) }
                    }
                    .padding(10)
                }
                .onChange(of: demo.messages.last?.text) { _, _ in
                    if let id = demo.messages.last?.id { proxy.scrollTo(id, anchor: .bottom) }
                }
            }
            Divider()
            HStack {
                TextField("Ask Grok… (explain this error, convert to TikZ, tighten this proof)", text: $demo.draft)
                    .textFieldStyle(.roundedBorder)
                    .onSubmit { demo.ask(demo.draft, context: model.activeText) }
                    .accessibilityIdentifier("grok.input")
                if demo.busy {
                    Button("Stop") { demo.cancel() }
                } else {
                    Button { demo.ask(demo.draft, context: model.activeText) } label: { Image(systemName: "paperplane.fill") }
                        .disabled(demo.draft.trimmingCharacters(in: .whitespaces).isEmpty)
                }
            }
            .padding(10)
        }
        .frame(minWidth: 300, idealWidth: 360)
        .background(.background)
        .accessibilityIdentifier(Self.identifier)
    }

    private var suggestions: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Try one of these").font(.caption).foregroundStyle(.secondary)
            ForEach(Array(model.displayedDiagnostics.prefix(4).enumerated()), id: \.offset) { _, d in
                Button { demo.ask("Explain this error: \(d.message)", context: model.activeText) } label: {
                    Label(d.message, systemImage: "exclamationmark.triangle").lineLimit(1)
                }.buttonStyle(.link)
            }
            ForEach(["Convert this sketch to TikZ", "Tighten this proof", "Summarize this document"], id: \.self) { s in
                Button(s) { demo.ask(s, context: model.activeText) }.buttonStyle(.link)
            }
        }
    }

    @ViewBuilder private func bubble(_ m: GrokDemo.Message) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                if m.role == .grok {
                    Label("Grok", systemImage: "sparkles").font(.caption.bold())
                    Text(m.origin).font(.caption2).monospaced().foregroundStyle(.secondary)
                    if m.streaming { ProgressView().controlSize(.mini) }
                } else { Text("You").font(.caption.bold()) }
            }
            Text(m.text.isEmpty && m.streaming ? "…" : m.text).textSelection(.enabled)
            if m.insertable != nil, !m.streaming {
                Button { demo.insert(m, into: model) } label: { Label("Insert (review first)", systemImage: "text.insert") }
                    .help("Opens the reviewed-proposal sheet; approving makes one undoable edit")
                    .accessibilityIdentifier("grok.insert")
            }
        }
        .padding(8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 8).fill(m.role == .grok ? Color.accentColor.opacity(0.08) : Color.secondary.opacity(0.08)))
    }
}
