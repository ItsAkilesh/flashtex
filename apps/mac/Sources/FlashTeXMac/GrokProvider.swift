import Foundation
import FlashTeXProtocol

/// Live Grok (xAI) explanation transport for the review sheet: one
/// `flashtex-assistant-context --provider-session <fresh> <model>` child per
/// explanation request (crates/assistant-context README, "provider commands").
/// The Mac never talks to xAI for explanations itself; the helper (built with
/// `--features grok`) owns the HTTPS call, the request/response bounds and the
/// source-bound validation. This driver:
///
/// 1. launches the helper with the key ONLY in its environment
///    (`FLASHTEX_GROK_API_KEY`, from `GrokCredential`), never in argv/JSON;
/// 2. sends `provider/admit` with the exact `prepare` request the one-shot
///    helper already produced the context from (`user_requested:true`);
/// 3. polls `provider/poll` with the current shadow sources until the helper
///    hands back a `validated_proposal` (already validated against the fresh
///    snapshot, `applied:false`) or a terminal state;
/// 4. returns the proposal bytes as the provider stage's "stdout", so the
///    existing validate/review/approve stages run unchanged.
///
/// Failures map onto `OneShotProcess.Failure` (an `error` JSON as an exit) so
/// the sheet's wording and the recovery log are the same as for a local
/// provider command. The helper's session collapses provider errors into
/// `state:"failed"` without the HTTP class (see apps/mac/docs/grok-live.md, helper
/// requests); the text says so rather than guessing.
@MainActor
final class GrokProviderSession {
    typealias Failure = OneShotProcess.Failure
    typealias Output = OneShotProcess.Output

    /// What the session launched (tests inspect argv and environment keys).
    struct Launch: Equatable {
        var executable: URL
        var arguments: [String]
        var environmentKeys: [String]
        var sessionId: String
        var model: String
    }

    static let pollInterval: TimeInterval = 0.4
    static let commandTimeout: TimeInterval = 10

    let launch: Launch
    private let client: LineProcessClient
    private var requestId: String?
    private var finished = false
    private var cancelled = false
    private var pollTask: Task<Void, Never>?
    private let completion: (Result<Output, Failure>) -> Void
    private(set) var polls = 0
    private(set) var lastState: String?
    private var timeout: TimeInterval = 0

    /// `sessionId` must be fresh per request (`[A-Za-z0-9_-]{1,128}`, the
    /// registry's rule). `environment` is the sanitized helper environment;
    /// the credential is injected here and nowhere else.
    init(helper: URL, model: String, sessionId: String, credential: GrokCredential.Resolution,
         environment: [String: String], completion: @escaping (Result<Output, Failure>) -> Void) throws {
        var env = environment
        credential.inject(into: &env, as: GrokCredential.helperVariable)
        let arguments = ["--provider-session", sessionId, model]
        launch = Launch(executable: helper, arguments: arguments, environmentKeys: env.keys.sorted(),
                        sessionId: sessionId, model: model)
        self.completion = completion
        var events: ((LineProcessClient.Event) -> Void)?
        client = try LineProcessClient(executable: helper, arguments: arguments, label: "grok-session",
                                       environment: env, classify: Self.classify) { events?($0) }
        events = { [weak self] event in
            Task { @MainActor in self?.handle(event) }
        }
    }

    var isRunning: Bool { client.isRunning }
    var processIdentifier: Int32 { client.processIdentifier }

    /// Session replies: `{"id":..,"result":{"type":..}}` or `{"id":..,"error":".."}`.
    /// Both poll reply types map to one expected type so the poll loop can
    /// accept either.
    nonisolated static func classify(_ line: Data) -> LineProcessClient.Classified? {
        guard let obj = try? JSONSerialization.jsonObject(with: line) as? [String: Any] else { return nil }
        let id = obj["id"] as? String
        if let error = obj["error"] as? String {
            return .init(id: id, type: "error", error: .init(code: "provider_session", message: error))
        }
        guard let result = obj["result"] as? [String: Any], let type = result["type"] as? String else { return nil }
        let normalized = (type == "provider_status" || type == "validated_proposal") ? "provider_poll" : type
        return .init(id: id, type: normalized, error: nil)
    }

    /// Starts the request. `prepareRequest` is the exact one-shot `prepare`
    /// request (operation, binding, sources, compiler_result, user_instruction,
    /// selected_diagnostics, destinations); `currentSources` the ledger
    /// documents polled against. `timeout` bounds the provider call
    /// (`timeout_ms`) and, plus a grace period, the whole session.
    func run(prepareRequest: [String: Any], currentSources: [[String: Any]], timeout: TimeInterval, allocation: String) {
        self.timeout = timeout
        let admit: [String: Any] = [
            "id": "admit",
            "action": ["operation": "provider",
                       "command": ["operation": "admit", "input": prepareRequest,
                                   "timeout_ms": Int(timeout * 1000), "user_requested": true,
                                   "allocation": String(allocation.prefix(128))]],
        ]
        send(admit, expected: "provider_admitted") { [weak self] result in
            guard let self, !self.finished else { return }
            switch result {
            case .failure(let f): self.finish(.failure(f))
            case .success(let obj):
                guard let id = obj["request_id"] as? String, !id.isEmpty else {
                    return self.finish(.failure(Self.refusal("helper admitted the request without an id")))
                }
                self.requestId = id
                self.poll(currentSources: currentSources, deadline: Date().addingTimeInterval(timeout + 5))
            }
        }
    }

    private func poll(currentSources: [[String: Any]], deadline: Date) {
        guard !finished, let requestId else { return }
        let command: [String: Any] = [
            "id": "poll-\(polls + 1)",
            "action": ["operation": "provider",
                       "command": ["operation": "poll", "request_id": requestId, "current_sources": currentSources]],
        ]
        polls += 1
        send(command, expected: "provider_poll") { [weak self] result in
            guard let self, !self.finished else { return }
            switch result {
            case .failure(let f): self.finish(.failure(f))
            case .success(let obj):
                if obj["type"] as? String == "validated_proposal" {
                    guard let payload = obj["payload"], JSONSerialization.isValidJSONObject(payload),
                          let bytes = try? JSONSerialization.data(withJSONObject: payload) else {
                        return self.finish(.failure(Self.refusal("helper returned an undecodable validated proposal")))
                    }
                    return self.finish(.success(Output(stdout: bytes, stderr: "")))
                }
                let state = (obj["state"] as? String) ?? "?"
                self.lastState = state
                switch state {
                case "queued", "running":
                    if Date() >= deadline {
                        self.sendCancelBestEffort()
                        return self.finish(.failure(.timeout(self.timeout)))
                    }
                    self.pollTask = Task { [weak self] in
                        try? await Task.sleep(nanoseconds: UInt64(Self.pollInterval * 1_000_000_000))
                        guard !Task.isCancelled else { return }
                        self?.poll(currentSources: currentSources, deadline: deadline)
                    }
                case "failed":
                    self.finish(.failure(Self.refusal("Grok request failed (helper state \"failed\"; the helper's provider session reports no HTTP class — a 401/403 key problem, 429 rate limit, timeout and transport failure all arrive here; Test Connection in Preferences (⌘,) distinguishes them)")))
                case "cancelled": self.finish(.failure(.cancelled))
                case "expired": self.finish(.failure(Self.refusal("Grok request expired in the helper before a reply arrived")))
                default: self.finish(.failure(Self.refusal("helper reported an unknown provider state \"\(state.prefix(32))\"")))
                }
            }
        }
    }

    /// Terminates the helper; the completion reports `.cancelled` (once).
    func cancel() {
        cancelled = true
        sendCancelBestEffort()
        finish(.failure(.cancelled))
    }

    private func sendCancelBestEffort() {
        guard let requestId, client.isRunning else { return }
        let command: [String: Any] = ["id": "cancel", "action": ["operation": "provider", "command": ["operation": "cancel", "request_id": requestId]]]
        send(command, expected: "provider_cancelled") { _ in }
    }

    private func finish(_ result: Result<Output, Failure>) {
        guard !finished else { return }
        finished = true
        pollTask?.cancel()
        pollTask = nil
        client.terminate()
        let delivered: Result<Output, Failure> = cancelled ? .failure(.cancelled) : result
        // Delivered on the next main-queue turn, exactly like `OneShotProcess`: a
        // cancellation's completion then finds the job already cleared and is
        // counted as a late reply rather than changing the displayed state.
        DispatchQueue.main.async { [completion] in completion(delivered) }
    }

    private func handle(_ event: LineProcessClient.Event) {
        guard !finished else { return }
        switch event {
        case .exited(let code):
            // A helper built without the `grok` feature prints usage and exits 2
            // (as does one launched without the key); say which.
            let why = code == 2
                ? "helper exited (2) at launch: it was built without the grok feature or received no credential — build crates/assistant-context with --features grok"
                : "helper exited (\(code)) during the Grok session"
            finish(.failure(Self.refusal(why)))
        case .protocolViolation(let m): finish(.failure(Self.refusal("helper protocol violation: \(m)")))
        case .stderr, .unsolicited: break
        }
    }

    private func send(_ command: [String: Any], expected: String, completion: @escaping (Result<[String: Any], Failure>) -> Void) {
        guard let id = command["id"] as? String, var line = try? JSONSerialization.data(withJSONObject: command) else {
            return completion(.failure(Self.refusal("could not encode a session command")))
        }
        line.append(0x0a)
        client.enqueue(id: id, line: line, expected: expected, timeout: Self.commandTimeout) { result in
            switch result {
            case .failure(let f):
                switch f {
                case .timeout(let t): completion(.failure(.timeout(t)))
                case .bridge(let e): completion(.failure(Self.refusal("helper refused \(id.split(separator: "-").first ?? ""): \(e.message)")))
                case .notRunning, .exited: completion(.failure(Self.refusal("helper is not running (\(f.text))")))
                default: completion(.failure(Self.refusal(f.text)))
                }
            case .success(let data):
                guard let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                      let result = obj["result"] as? [String: Any] else {
                    return completion(.failure(Self.refusal("undecodable session reply")))
                }
                completion(.success(result))
            }
        }
    }

    /// An `error` JSON as a nonzero exit: `ProposalPreview.describe` then
    /// renders "provider refused (provider): <message>".
    nonisolated static func refusal(_ message: String) -> Failure {
        let json = (try? JSONSerialization.data(withJSONObject: ["type": "error", "message": message])) ?? Data()
        return .exited(1, output: json, stderr: "")
    }
}

// MARK: - Connection probe (Preferences "Test connection")

/// The one network call the Mac app makes itself, only when the user clicks
/// "Test connection" in Preferences: `GET <base>/v1/models` with the key as a
/// bearer token, reported as an HTTP class only (no body is read or shown).
/// The helper has no probe command yet (see apps/mac/docs/grok-live.md, helper
/// requests); once it does, this moves behind the helper. `FLASHTEX_GROK_BASE_URL`
/// (loopback `http://` or any `https://`) redirects the probe to a local stub in
/// tests; the helper does not honour it (also requested).
enum GrokProbe {
    static let defaultBaseURL = URL(string: "https://api.x.ai")!
    static let baseURLVariable = "FLASHTEX_GROK_BASE_URL"
    static let path = "/v1/models"

    enum Outcome: Equatable {
        case accepted(Int)
        case unauthorized(Int)
        case rateLimited
        case serverError(Int)
        case unexpected(Int)
        case timeout
        case transport(String)

        var text: String {
            switch self {
            case .accepted(let s): return "connection OK (HTTP \(s)): the key was accepted"
            case .unauthorized(let s): return "authentication failed (HTTP \(s)): the key was rejected"
            case .rateLimited: return "rate limited (HTTP 429): the key was accepted but xAI refused the request right now"
            case .serverError(let s): return "xAI server error (HTTP \(s)); try again later"
            case .unexpected(let s): return "unexpected reply (HTTP \(s))"
            case .timeout: return "timed out: no reply from xAI"
            case .transport(let why): return "no connection: \(why)"
            }
        }
        var keyAccepted: Bool {
            switch self {
            case .accepted, .rateLimited: return true
            default: return false
            }
        }
    }

    static func classify(status: Int) -> Outcome {
        switch status {
        case 200..<300: return .accepted(status)
        case 401, 403: return .unauthorized(status)
        case 429: return .rateLimited
        case 500..<600: return .serverError(status)
        default: return .unexpected(status)
        }
    }

    /// The override is honoured only for loopback `http://` or `https://`.
    static func baseURL(environment: [String: String] = ProcessInfo.processInfo.environment) -> URL {
        guard let raw = environment[baseURLVariable], let url = URL(string: raw), let host = url.host, let scheme = url.scheme else {
            return defaultBaseURL
        }
        let loopback = host == "127.0.0.1" || host == "localhost" || host == "::1"
        if scheme == "https" || (scheme == "http" && loopback) { return url }
        return defaultBaseURL
    }

    /// Never logs; `completion` runs on the main queue with the class only.
    /// `models` receives the model ids listed by a 2xx reply (`data[].id`, the
    /// only part of the body read; no user data) so Preferences and the live
    /// evidence can name valid ids.
    static func probe(credential: GrokCredential.Resolution, baseURL: URL = baseURL(), timeout: TimeInterval = 10,
                      models: (@MainActor ([String]) -> Void)? = nil,
                      completion: @escaping @MainActor (Outcome) -> Void) {
        var request = URLRequest(url: baseURL.appendingPathComponent(path))
        request.httpMethod = "GET"
        credential.applyBearer(to: &request)
        request.timeoutInterval = timeout
        let configuration = URLSessionConfiguration.ephemeral
        configuration.timeoutIntervalForRequest = timeout
        configuration.timeoutIntervalForResource = timeout
        configuration.httpShouldSetCookies = false
        configuration.urlCache = nil
        let session = URLSession(configuration: configuration, delegate: NoRedirects.shared, delegateQueue: nil)
        let task = session.dataTask(with: request) { data, response, error in
            let outcome: Outcome
            var ids: [String] = []
            if let error = error as NSError? {
                outcome = error.code == NSURLErrorTimedOut ? .timeout : .transport(error.localizedDescription)
            } else if let http = response as? HTTPURLResponse {
                outcome = classify(status: http.statusCode)
                if (200..<300).contains(http.statusCode), let data, data.count <= 256 * 1024,
                   let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                   let list = obj["data"] as? [[String: Any]] {
                    ids = list.compactMap { $0["id"] as? String }.filter(GrokCredential.isValidModel).sorted()
                }
            } else {
                outcome = .transport("no HTTP response")
            }
            session.finishTasksAndInvalidate()
            Task { @MainActor in
                if let models, !ids.isEmpty { models(ids) }
                completion(outcome)
            }
        }
        task.resume()
    }

    private final class NoRedirects: NSObject, URLSessionTaskDelegate {
        static let shared = NoRedirects()
        func urlSession(_ session: URLSession, task: URLSessionTask, willPerformHTTPRedirection response: HTTPURLResponse,
                        newRequest request: URLRequest, completionHandler: @escaping (URLRequest?) -> Void) {
            completionHandler(nil)
        }
    }
}
