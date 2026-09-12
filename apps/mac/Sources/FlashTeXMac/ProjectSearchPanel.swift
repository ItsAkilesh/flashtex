import AppKit
import SwiftUI
import FlashTeXProtocol

/// Native project search through the durable helper's `search_literal`
/// (crates/preview-controller STDIO.md): a case-sensitive raw UTF-8 literal
/// search over the helper's durable source (comments and verbatim text
/// included), no regex, no normalization. The helper is the only route: it
/// searches the durable ledger text at an exact `source_versions` map, so
/// results name the revision they were computed on and are shown with the
/// helper's explicit `termination` (`complete` / `match_limit` /
/// `work_limit`). Partial results are never labelled exhaustive.
///
/// Navigation to a match is refused, never guessed, when the durable
/// versions moved since the search (a durable edit anywhere in the project
/// — the helper's own stale-version rule), when the durable bytes at the
/// range no longer spell the literal, or when the open buffer's bytes at
/// the range differ from the durable text (`Navigation.rebaseExactly`: a
/// local edit overlapping the range is refused, an edit elsewhere is
/// rebased byte-exactly and re-verified — the `navigateExactly` rule). The
/// selection covers whole composed character sequences via
/// `ShellModel.selectIndexLocation` (Navigation.swift).
///
/// Pure helpers live in `ProjectSearch`; `ProjectSearchClient` is the
/// bounded request/reply client on top of `ShellModel.controllerRequest`;
/// `ProjectSearchPanel` is the ⌘⇧F window (`ProjectSearchWindow` scene +
/// `ProjectSearchCommands` menu item are added from `FlashTeXMacApp`).
enum ProjectSearch {
    /// `Window(id:)` of the panel.
    static let windowID = "project-search"
    /// The helper's admissible ranges (STDIO.md).
    static let matchLimitRange = 1...1000
    static let workLimitRange = 1...1_000_000
    static let defaultMaxMatches = 200
    static let defaultMaxWork = 1_000_000
    /// Characters of line context kept on each side of a match in a snippet.
    static let snippetContext = 48
    /// STDIO.md: literals are at most 64 KiB (the helper refuses longer ones).
    static let maxLiteralBytes = 64 * 1024

    /// The helper's explicit termination. Only `complete` is exhaustive.
    enum Termination: Equatable {
        case complete
        case matchLimit
        case workLimit
        case cancelled
        /// A value this client does not know: shown verbatim, never exhaustive.
        case unknown(String)

        init(wire: String) {
            switch wire {
            case "complete": self = .complete
            case "match_limit": self = .matchLimit
            case "work_limit": self = .workLimit
            case "cancelled": self = .cancelled
            default: self = .unknown(wire)
            }
        }

        var isExhaustive: Bool { self == .complete }

        /// Short badge text.
        var badge: String {
            switch self {
            case .complete: return "complete"
            case .matchLimit: return "match limit"
            case .workLimit: return "work limit"
            case .cancelled: return "cancelled"
            case .unknown(let s): return s
            }
        }

        /// What the badge means, for the panel and VoiceOver.
        func explanation(maxMatches: Int, maxWork: Int) -> String {
            switch self {
            case .complete: return "Exhaustive: every match in the durable project source is listed."
            case .matchLimit: return "Partial: the helper stopped at the match limit (\(maxMatches)); more matches may exist."
            case .workLimit: return "Partial: the helper stopped at the work budget (\(maxWork) byte comparisons); more matches may exist."
            case .cancelled: return "Partial: the search was cancelled; more matches may exist."
            case .unknown(let s): return "Partial: unknown termination “\(s)”; not treated as exhaustive."
            }
        }
    }

    /// A match's line, shown with the matched bytes distinguishable from its
    /// context. Line breaks inside a multi-line literal are shown as ⏎.
    struct Snippet: Equatable {
        var before: String
        var match: String
        var after: String
        /// True when `before`/`after` were clipped to `snippetContext` characters.
        var clippedBefore = false
        var clippedAfter = false

        var text: String { (clippedBefore ? "…" : "") + before + match + after + (clippedAfter ? "…" : "") }
    }

    /// One helper match: the exact durable byte range plus its display data.
    struct Match: Identifiable, Equatable {
        var location: ShellModel.IndexLocation
        /// 1-based line of the match start in the durable text (LF-counted; a
        /// `\r\n` file counts its `\n`s), 0 when the text was unavailable.
        var line: Int
        /// 1-based byte column of the match start within its line.
        var column: Int
        var snippet: Snippet?

        var id: String { "\(location.path)@\(location.revision):\(location.start)..<\(location.end)" }
        var path: String { location.path }
    }

    /// The reply to one `search_literal`, bound to the versions it ran on.
    struct Results: Equatable {
        var literal: String
        var sourceVersions: [String: Int]
        /// The `documents` filter the search ran with; nil is the whole project.
        var documents: [String]?
        var matches: [Match]
        var termination: Termination
        var workUsed: Int
        var maxMatches: Int
        var maxWork: Int

        /// "3 matches (complete)" / "200 matches shown — partial (match limit)".
        var summary: String {
            let n = matches.count
            let count = "\(n) match\(n == 1 ? "" : "es")"
            return termination.isExhaustive ? "\(count) (complete)" : "\(count) shown — partial (\(termination.badge))"
        }
    }

    // MARK: - wire

    /// The `search_literal` payload; limits are clamped into the helper's
    /// admissible ranges (a value outside them would be refused as a whole).
    static func request(sourceVersions: [String: Int], literal: String, maxMatches: Int, maxWork: Int, documents: [String]? = nil) -> PreviewControllerClient.JSONObject {
        var payload: PreviewControllerClient.JSONObject = [
            "source_versions": sourceVersions,
            "literal": literal,
            "max_matches": clamp(maxMatches, to: matchLimitRange),
            "max_work": clamp(maxWork, to: workLimitRange),
        ]
        if let documents { payload["documents"] = documents }
        return payload
    }

    static func clamp(_ value: Int, to range: ClosedRange<Int>) -> Int { min(max(value, range.lowerBound), range.upperBound) }

    /// Raw locations from a `search_literal` reply (`{source_versions,
    /// matches:[{path,revision,start_byte,end_byte}], termination, work_used}`).
    struct RawReply: Equatable {
        var sourceVersions: [String: Int]
        var matches: [ShellModel.IndexLocation]
        var termination: Termination
        var workUsed: Int
    }

    static func parse(reply payload: [String: Any]) -> Result<RawReply, ControllerError> {
        guard let versions = payload["source_versions"] as? [String: Int] else { return .failure(.init(message: "search reply has no source_versions")) }
        guard let wire = payload["termination"] as? String else { return .failure(.init(message: "search reply has no termination")) }
        guard let raw = payload["matches"] as? [Any] else { return .failure(.init(message: "search reply has no matches")) }
        var matches: [ShellModel.IndexLocation] = []
        matches.reserveCapacity(raw.count)
        for item in raw {
            guard let loc = ShellModel.IndexLocation(item) else { return .failure(.init(message: "search reply match \(matches.count) is malformed")) }
            guard versions[loc.path] == loc.revision else {
                return .failure(.init(message: "search reply match \(matches.count) names \(loc.path) r\(loc.revision), not the reply's r\(versions[loc.path].map(String.init) ?? "?")"))
            }
            matches.append(loc)
        }
        return .success(RawReply(sourceVersions: versions, matches: matches, termination: Termination(wire: wire),
                                 workUsed: payload["work_used"] as? Int ?? 0))
    }

    // MARK: - text (pure, UTF-8 exact)

    /// Whether bytes `start..<end` of `text` are exactly the literal's bytes.
    static func bytesSpell(_ literal: String, in text: String, start: Int, end: Int) -> Bool {
        guard start >= 0, end >= start, end <= text.utf8.count, end - start == literal.utf8.count else { return false }
        var t = text, l = literal
        return t.withUTF8 { tb in
            l.withUTF8 { lb in
                lb.count == 0 || memcmp(tb.baseAddress! + start, lb.baseAddress!, lb.count) == 0
            }
        }
    }

    /// Line (1-based), byte column (1-based) and the clipped line snippet for
    /// bytes `start..<end` of `text`. Nil when the range is not a valid scalar-
    /// aligned range of `text`. Context is clipped on `Character` boundaries,
    /// so a snippet never begins or ends inside a composed character sequence.
    static func locate(start: Int, end: Int, in text: String, context: Int = snippetContext) -> (line: Int, column: Int, snippet: Snippet)? {
        guard let matchRange = text.rangeOfUTF8(start: start, end: end) else { return nil }
        var t = text
        let (line, lineStart, lineEnd): (Int, Int, Int) = t.withUTF8 { b in
            let n = b.count
            var line = 1, lineStart = 0
            for i in 0..<start where b[i] == UInt8(ascii: "\n") { line += 1; lineStart = i + 1 }
            var lineEnd = end
            while lineEnd < n, b[lineEnd] != UInt8(ascii: "\n") { lineEnd += 1 }
            if lineEnd > end, b[lineEnd - 1] == UInt8(ascii: "\r") { lineEnd -= 1 }
            return (line, lineStart, lineEnd)
        }
        guard let beforeRange = text.rangeOfUTF8(start: lineStart, end: start),
              let afterRange = text.rangeOfUTF8(start: end, end: lineEnd) else { return nil }
        var snippet = Snippet(before: String(text[beforeRange]), match: String(text[matchRange]), after: String(text[afterRange]))
        snippet.match = snippet.match.replacingOccurrences(of: "\r\n", with: "⏎").replacingOccurrences(of: "\n", with: "⏎")
        if snippet.before.count > context { snippet.before = String(snippet.before.suffix(context)); snippet.clippedBefore = true }
        if snippet.after.count > context { snippet.after = String(snippet.after.prefix(context)); snippet.clippedAfter = true }
        return (line, start - lineStart + 1, snippet)
    }

    /// Display matches for the raw locations, using the durable text of each
    /// document when the caller could read it (`texts[path]` at the reply's
    /// revision); a match whose text is unavailable keeps line 0 and no snippet.
    static func matches(from raw: [ShellModel.IndexLocation], texts: [String: String]) -> [Match] {
        raw.map { loc in
            guard let text = texts[loc.path], let at = locate(start: loc.start, end: loc.end, in: text) else {
                return Match(location: loc, line: 0, column: 0, snippet: nil)
            }
            return Match(location: loc, line: at.line, column: at.column, snippet: at.snippet)
        }
    }

    /// VoiceOver label of a row: "match n of m, path, line l, snippet".
    static func accessibilityLabel(index: Int, count: Int, match: Match) -> String {
        var parts = ["match \(index + 1) of \(count)", match.path]
        parts.append(match.line > 0 ? "line \(match.line)" : "line unknown")
        parts.append(match.snippet?.text ?? "text unavailable")
        return parts.joined(separator: ", ")
    }

    /// The paths whose durable version differs between two maps, for messages.
    static func changedVersions(from old: [String: Int], to new: [String: Int]) -> [String] {
        Set(old.keys).union(new.keys).filter { old[$0] != new[$0] }.sorted().map { path in
            "\(path) r\(old[path].map(String.init) ?? "–")→r\(new[path].map(String.init) ?? "–")"
        }
    }

    static let noHelperMessage = "Search requires the durable helper (flashtex-preview-controller) to be attached and ready."
}

// MARK: - client

/// Bounded client for one search panel: one `search_literal` in flight, one
/// bounded automatic retry after the helper's stale-version error, explicit
/// limits, and navigation that re-reads the helper's current versions before
/// selecting anything. Runs on the main actor with the shell model.
@MainActor
@Observable
final class ProjectSearchClient {
    /// Where to search: the whole helper project, or only the active document
    /// (the contract's optional `documents` filter).
    enum Scope: String, CaseIterable, Identifiable {
        case project = "Whole project"
        case activeDocument = "Active document"
        var id: String { rawValue }
    }

    private(set) var model: ShellModel
    var query = ""
    var scope: Scope = .project
    var maxMatches = ProjectSearch.defaultMaxMatches
    var maxWork = ProjectSearch.defaultMaxWork
    private(set) var results: ProjectSearch.Results?
    /// What the panel says under the field: helper state, errors, refusals, navigation notes.
    private(set) var status = ""
    private(set) var isSearching = false
    var selectedID: ProjectSearch.Match.ID?
    /// Bumped on every accepted navigation (tests observe it).
    private(set) var navigationCount = 0
    /// Bounded time to wait for a `document` read the helper answers asynchronously.
    var documentReadTimeout: TimeInterval = 2

    init(model: ShellModel) { self.model = model }

    var helperAvailable: Bool { model.controllerAttached && model.controllerState.ready }

    var selectedIndex: Int? {
        guard let selectedID, let results else { return nil }
        return results.matches.firstIndex { $0.id == selectedID }
    }

    var selectedMatch: ProjectSearch.Match? { selectedIndex.map { results!.matches[$0] } }

    /// Whether the helper's versions moved since `results` were computed, as
    /// far as this session knows (durable receipts); the authoritative check
    /// is the fresh `snapshot` taken when navigating.
    var knownStaleNote: String? {
        guard let results else { return nil }
        let changed = results.sourceVersions.compactMap { path, rev -> String? in
            guard let durable = model.controllerState.durable[path]?.revision, durable != rev else { return nil }
            return "\(path) r\(rev)→r\(durable)"
        }.sorted()
        return changed.isEmpty ? nil : "Project changed since this search (\(changed.joined(separator: ", "))); search again."
    }

    /// Return in the field: search when the query is not what the list shows,
    /// otherwise go to the selected match.
    func submit() {
        if let results, results.literal == query, results.documents == (scope == .activeDocument ? [model.activePath] : nil), !results.matches.isEmpty {
            Task { await navigateToSelected() }
        } else {
            Task { await search() }
        }
    }

    func moveSelection(by delta: Int) {
        guard let results, !results.matches.isEmpty else { return }
        let next = min(max((selectedIndex ?? (delta > 0 ? -1 : results.matches.count)) + delta, 0), results.matches.count - 1)
        selectedID = results.matches[next].id
    }

    /// `snapshot` → `search_literal {source_versions, literal, max_matches,
    /// max_work}`; the reply's locations are shown with their durable line
    /// and snippet (durable text read through `document` when this session
    /// has not seen that revision). One automatic retry after the helper's
    /// stale-version error (the user was typing); the second refusal is shown.
    func search() async {
        let literal = query
        guard !isSearching else { status = "A search is already running."; return }
        guard helperAvailable else { results = nil; status = ProjectSearch.noHelperMessage; return }
        guard !literal.isEmpty else { results = nil; status = "Type a literal to search for (case-sensitive, no regex)."; return }
        guard literal.utf8.count <= ProjectSearch.maxLiteralBytes else {
            results = nil; status = "The literal is \(literal.utf8.count) bytes; the helper accepts at most \(ProjectSearch.maxLiteralBytes)."; return
        }
        isSearching = true
        status = "Searching the durable project source…"
        defer { isSearching = false }
        var attempt = 0
        while true {
            attempt += 1
            guard let versionsReply = await model.controllerSourceVersions() else { results = nil; status = ProjectSearch.noHelperMessage; return }
            let versions: [String: Int]
            switch versionsReply {
            case .failure(let e): results = nil; status = "Snapshot refused: \(e.message)"; return
            case .success(let v): versions = v
            }
            let documents: [String]? = scope == .activeDocument ? [model.activePath] : nil
            if let documents, let missing = documents.first(where: { versions[$0] == nil }) {
                results = nil
                status = "\(missing) is not part of the helper's project (indexed: \(versions.keys.sorted().joined(separator: ", ")))."
                return
            }
            let payload = ProjectSearch.request(sourceVersions: versions, literal: literal, maxMatches: maxMatches, maxWork: maxWork, documents: documents)
            guard let reply = await model.controllerRequest("search_literal", payload) else { results = nil; status = ProjectSearch.noHelperMessage; return }
            switch reply {
            case .failure(let e):
                if e.message.contains("source versions changed"), attempt == 1 { continue } // one bounded retry
                results = nil
                status = e.message.contains("source versions changed")
                    ? "Project changed while searching (\(e.message)); search again."
                    : "Search refused by the helper: \(e.message)"
                return
            case .success(let dict):
                switch ProjectSearch.parse(reply: dict) {
                case .failure(let why): results = nil; status = "Search reply refused: \(why.message)"; return
                case .success(let raw):
                    var texts: [String: String] = [:]
                    for path in Set(raw.matches.map(\.path)).sorted() {
                        if let rev = raw.sourceVersions[path], let text = await durableText(path: path, revision: rev) { texts[path] = text }
                    }
                    let matches = ProjectSearch.matches(from: raw.matches, texts: texts)
                    let out = ProjectSearch.Results(literal: literal, sourceVersions: raw.sourceVersions, documents: documents, matches: matches,
                                                    termination: raw.termination, workUsed: raw.workUsed,
                                                    maxMatches: ProjectSearch.clamp(maxMatches, to: ProjectSearch.matchLimitRange),
                                                    maxWork: ProjectSearch.clamp(maxWork, to: ProjectSearch.workLimitRange))
                    results = out
                    selectedID = matches.first?.id
                    let unread = matches.filter { $0.snippet == nil }.count
                    let scopeName: String = documents?.joined(separator: ", ") ?? "the project"
                    var line = "\(out.summary) for “\(literal)” in \(scopeName) (durable source, work \(raw.workUsed))"
                    if unread > 0 { line += "; \(unread) without text (durable revision not readable)" }
                    if attempt > 1 { line += "; retried once after the project changed" }
                    status = line
                    return
                }
            }
        }
    }

    /// The durable text of `path` at `revision`: what this session recorded
    /// from `document`/`edit` results, else a `document` read (the parent's
    /// handler records it) awaited for a bounded time. Nil when unavailable
    /// or when the helper is at another revision by the time it answers.
    func durableText(path: String, revision: Int) async -> String? {
        if let text = model.controllerState.textByDurable[path]?[revision] { return text }
        guard let controller = model.controller, controller.isRunning else { return nil }
        _ = try? controller.document(path: path)
        let deadline = Date().addingTimeInterval(documentReadTimeout)
        while model.controllerState.textByDurable[path]?[revision] == nil, Date() < deadline, model.controllerAttached {
            try? await Task.sleep(nanoseconds: 5_000_000)
        }
        return model.controllerState.textByDurable[path]?[revision]
    }

    /// ⌘G in the panel: advance the selection (wrapping) and go there.
    func navigateNext() async {
        guard let results, !results.matches.isEmpty else { status = "No matches to step through."; return }
        let next = ((selectedIndex ?? -1) + 1) % results.matches.count
        selectedID = results.matches[next].id
        await navigate(to: next, of: results)
    }

    func navigateToSelected() async {
        guard let results, let index = selectedIndex else { status = "No match selected."; return }
        await navigate(to: index, of: results)
    }

    /// Goes to match `index` exactly or refuses with the reason:
    /// 1. the helper's current `snapshot` must equal the versions the search
    ///    ran on (a durable edit anywhere since is the stale-version rule);
    /// 2. the durable bytes at the range must still spell the literal;
    /// 3. `ShellModel.selectIndexLocation` maps the durable range onto the
    ///    open buffer with `Navigation.rebaseExactly` (refusing a range that
    ///    overlaps a local edit) and selects whole composed characters,
    ///    opening a project document that is not open in the window.
    func navigate(to index: Int, of results: ProjectSearch.Results) async {
        guard index >= 0, index < results.matches.count else { status = "No match \(index + 1)."; return }
        let match = results.matches[index]
        let label = "Match \(index + 1) of \(results.matches.count)\(results.termination.isExhaustive ? "" : "+") for “\(results.literal)”"
        guard helperAvailable, let versionsReply = await model.controllerSourceVersions() else { status = ProjectSearch.noHelperMessage; return }
        let fresh: [String: Int]
        switch versionsReply {
        case .failure(let e): status = "\(label): snapshot refused (\(e.message)); not navigated."; return
        case .success(let v): fresh = v
        }
        guard fresh == results.sourceVersions else {
            let changed = ProjectSearch.changedVersions(from: results.sourceVersions, to: fresh).joined(separator: ", ")
            status = "\(label): project changed since this search (\(changed)); search again."
            return
        }
        guard let durable = await durableText(path: match.path, revision: match.location.revision) else {
            status = "\(label): durable revision \(match.location.revision) of \(match.path) could not be read; not navigated."
            return
        }
        guard ProjectSearch.bytesSpell(results.literal, in: durable, start: match.location.start, end: match.location.end) else {
            status = "\(label): bytes \(match.location.start)..<\(match.location.end) of \(match.path) r\(match.location.revision) no longer spell the literal; search again."
            return
        }
        let before = model.selection?.token
        await model.selectIndexLocation(match.location, versions: fresh, label: label)
        if let sel = model.selection, sel.token != before, sel.path == match.path {
            navigationCount += 1
            status = model.navigationNote ?? label
        } else {
            status = model.navigationNote.map { "Not navigated — " + $0 } ?? "\(label): not navigated."
        }
    }
}

// MARK: - view

/// The ⌘⇧F window: a literal field, the result list with explicit
/// termination, and the refusal/navigation note. Return searches (new query)
/// or goes to the selected match; ↑/↓ move the selection from the field or
/// the list; Esc closes the window.
struct ProjectSearchPanel: View {
    static let windowID = ProjectSearch.windowID
    @Environment(ShellModel.self) private var model
    @Environment(\.dismissWindow) private var dismissWindow
    @State private var client: ProjectSearchClient?
    @FocusState private var fieldFocused: Bool

    var body: some View {
        Group {
            if let client {
                ProjectSearchBody(client: client, fieldFocused: $fieldFocused)
            } else {
                ProgressView().onAppear {
                    let c = ProjectSearchClient(model: model)
                    client = c
                    // Automation evidence only (launch-check screenshots): seed a
                    // query and run it once the helper is ready (bounded wait).
                    if let seed = ProcessInfo.processInfo.environment["FLASHTEX_SEARCH_QUERY"], !seed.isEmpty {
                        c.query = seed
                        if let n = ProcessInfo.processInfo.environment["FLASHTEX_SEARCH_MAX_MATCHES"].flatMap(Int.init) { c.maxMatches = n }
                        Task {
                            let deadline = Date().addingTimeInterval(10)
                            while !c.helperAvailable, Date() < deadline { try? await Task.sleep(nanoseconds: 50_000_000) }
                            await c.search()
                        }
                    }
                }
            }
        }
        .frame(minWidth: 520, minHeight: 320)
        .onAppear { fieldFocused = true }
        .background {
            // Esc anywhere in the window closes it (the cancel action).
            Button("Close") { dismissWindow(id: Self.windowID) }
                .keyboardShortcut(.cancelAction)
                .hidden()
        }
    }
}

private struct ProjectSearchBody: View {
    @Bindable var client: ProjectSearchClient
    var fieldFocused: FocusState<Bool>.Binding

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                TextField("Find in project (case-sensitive literal, no regex)", text: $client.query)
                    .textFieldStyle(.roundedBorder)
                    .focused(fieldFocused)
                    .onSubmit { client.submit() }
                    .onKeyPress(.upArrow) { client.moveSelection(by: -1); return .handled }
                    .onKeyPress(.downArrow) { client.moveSelection(by: 1); return .handled }
                    .accessibilityLabel("Literal to find in the project, case-sensitive")
                    .accessibilityHint("Return searches the durable project source or goes to the selected match; up and down arrows move the selection")
                Button("Search") { Task { await client.search() } }
                    .disabled(client.isSearching || client.query.isEmpty)
                Button("Go to Match") { Task { await client.navigateToSelected() } }
                    .disabled(client.selectedMatch == nil)
                Button("Next Match") { Task { await client.navigateNext() } }
                    .keyboardShortcut("g", modifiers: .command)
                    .disabled(client.results?.matches.isEmpty ?? true)
                    .help("Select the next match (wrapping) and go there (⌘G while this window is key)")
            }
            HStack(spacing: 12) {
                Text("Case-sensitive literal; the helper has no regex or normalization.").font(.caption).foregroundStyle(.secondary)
                Picker("Scope", selection: $client.scope) {
                    ForEach(ProjectSearchClient.Scope.allCases) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.menu).font(.caption).fixedSize()
                .accessibilityLabel("Search scope")
                Spacer()
                Stepper("Max matches: \(client.maxMatches)", value: $client.maxMatches, in: ProjectSearch.matchLimitRange, step: 50)
                    .font(.caption)
                    .help("The helper's match limit (1–1000); a search stopped here is labelled partial")
            }
            if !client.helperAvailable {
                Label(ProjectSearch.noHelperMessage, systemImage: "exclamationmark.triangle")
                    .foregroundStyle(.orange)
                    .accessibilityLabel(ProjectSearch.noHelperMessage)
            }
            if let results = client.results {
                HStack(spacing: 8) {
                    Text(results.summary).bold()
                    Text(results.termination.badge)
                        .font(.caption).padding(.horizontal, 6).padding(.vertical, 2)
                        .background(results.termination.isExhaustive ? Color.green.opacity(0.2) : Color.orange.opacity(0.25), in: Capsule())
                        .help(results.termination.explanation(maxMatches: results.maxMatches, maxWork: results.maxWork))
                    if !results.termination.isExhaustive {
                        Text("not exhaustive").font(.caption).foregroundStyle(.orange)
                    }
                    Spacer()
                    Text("durable " + results.sourceVersions.keys.sorted().map { "\($0) r\(results.sourceVersions[$0]!)" }.joined(separator: ", "))
                        .font(.caption).foregroundStyle(.secondary).lineLimit(1)
                }
                .accessibilityElement(children: .combine)
                .accessibilityLabel(results.summary + ". " + results.termination.explanation(maxMatches: results.maxMatches, maxWork: results.maxWork))
                if let stale = client.knownStaleNote {
                    Label(stale, systemImage: "clock.arrow.circlepath").font(.caption).foregroundStyle(.orange)
                }
                List(selection: $client.selectedID) {
                    ForEach(Array(results.matches.enumerated()), id: \.element.id) { (index: Int, match: ProjectSearch.Match) in
                        ProjectSearchRow(index: index, count: results.matches.count, match: match)
                            .tag(match.id)
                    }
                }
                .onKeyPress(.return) { client.submit(); return .handled }
                .accessibilityLabel("Search results, \(results.summary)")
            } else {
                Spacer()
            }
            Text(client.status)
                .font(.caption)
                .foregroundStyle(client.status.hasPrefix("Not navigated") || client.status.contains("refused") ? .orange : .secondary)
                .lineLimit(3)
                .textSelection(.enabled)
                .accessibilityLabel("Search status: \(client.status)")
        }
        .padding(12)
    }
}

private struct ProjectSearchRow: View {
    var index: Int
    var count: Int
    var match: ProjectSearch.Match

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            Text(match.line > 0 ? "\(match.path):\(match.line)" : match.path)
                .font(.caption.monospaced()).foregroundStyle(.secondary)
                .frame(width: 140, alignment: .leading).lineLimit(1)
            if let s = match.snippet {
                (Text(s.clippedBefore ? "…" : "") + Text(s.before) + Text(s.match).bold().foregroundColor(.accentColor) + Text(s.after) + Text(s.clippedAfter ? "…" : ""))
                    .font(.body.monospaced()).lineLimit(1)
            } else {
                Text("bytes \(match.location.start)..<\(match.location.end) (text unavailable)").font(.caption).foregroundStyle(.secondary)
            }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(ProjectSearch.accessibilityLabel(index: index, count: count, match: match))
    }
}

// MARK: - scene and menu (added from FlashTeXMacApp with one line each)

/// `ProjectSearchWindow(model: model)` in the app's scene body.
struct ProjectSearchWindow: Scene {
    var model: ShellModel

    var body: some Scene {
        Window("Find in Project", id: ProjectSearch.windowID) {
            ProjectSearchPanel().environment(model)
        }
        .defaultSize(width: 720, height: 440)
    }
}

/// `ProjectSearchCommands(openWindow: openWindow)` in the app's `.commands`:
/// ⌘⇧F opens (or brings forward) the search window.
struct ProjectSearchCommands: Commands {
    var openWindow: OpenWindowAction

    var body: some Commands {
        CommandGroup(after: .textEditing) {
            Button("Find in Project…") { openWindow(id: ProjectSearch.windowID) }
                .keyboardShortcut("f", modifiers: [.command, .shift])
        }
    }
}
