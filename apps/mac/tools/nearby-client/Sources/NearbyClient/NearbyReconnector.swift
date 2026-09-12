import Foundation
import Network

/// Bounded reconnect policy (proposal §7.4–5, §8): exponential backoff with
/// jitter, a hard cap on connection attempts and on wall-clock time, and no
/// retry at all for terminal failures. Pure: `delay(beforeAttempt:random:)`
/// is deterministic given `random`, so tests pin the schedule.
public struct ReconnectPolicy: Equatable {
    /// Connection attempts per operation, the first one included. 1 = never retry.
    public var maxAttempts: Int
    /// Wait before attempt 2; doubles (× `multiplier`) up to `maxDelay`.
    public var initialDelay: TimeInterval
    public var maxDelay: TimeInterval
    public var multiplier: Double
    /// ± fraction of the computed delay (0.2 = 80–120 %), so a fleet of
    /// companions reconnecting to one Mac does not do so in lockstep.
    public var jitter: Double
    /// Whole-operation budget, attempts and waits included; exceeded → give up.
    public var overallDeadline: TimeInterval
    public var connectTimeout: TimeInterval
    public var requestTimeout: TimeInterval

    public init(maxAttempts: Int = 5, initialDelay: TimeInterval = 0.25, maxDelay: TimeInterval = 4, multiplier: Double = 2,
                jitter: Double = 0.2, overallDeadline: TimeInterval = 60, connectTimeout: TimeInterval = 5, requestTimeout: TimeInterval = 30) {
        self.maxAttempts = max(1, maxAttempts); self.initialDelay = max(0, initialDelay); self.maxDelay = max(0, maxDelay)
        self.multiplier = max(1, multiplier); self.jitter = min(max(0, jitter), 1); self.overallDeadline = max(0, overallDeadline)
        self.connectTimeout = max(0.01, connectTimeout); self.requestTimeout = max(0.01, requestTimeout)
    }

    /// No waiting, one attempt after another: for tests and for "try once more now".
    public static let immediate = ReconnectPolicy(initialDelay: 0, maxDelay: 0, jitter: 0)

    /// Delay before attempt `n` (n ≥ 2). `random` draws a uniform value in the
    /// given range (jitter); pass a constant for a deterministic schedule.
    public func delay(beforeAttempt n: Int, random: (ClosedRange<Double>) -> Double = { Double.random(in: $0) }) -> TimeInterval {
        guard n >= 2 else { return 0 }
        let base = min(maxDelay, initialDelay * pow(multiplier, Double(n - 2)))
        guard base > 0, jitter > 0 else { return base }
        return max(0, base * random((1 - jitter)...(1 + jitter)))
    }
}

/// Keeps one authenticated session to a paired Mac alive across drops with a
/// bounded reconnect, and delivers captures at-least-once with the *same*
/// `capture_id` (the Mac de-duplicates; proposal §7.4).
///
/// What is bounded: connection attempts and wall-clock per operation
/// (`ReconnectPolicy`), one session at a time (a dropped one is discarded,
/// never queued), no pending-capture queue (callers hold their own capture
/// and await `submit`), and the connection's own in-flight/inbound limits.
///
/// What is terminal (thrown at once, no retry): a refused key
/// (`handshakeFailed` → re-pair), any `error` reply, a destination the Mac no
/// longer reports (`destinationChanged` → reselect on the Mac), bad input,
/// a protocol violation, and — after the budget — `attemptsExhausted`.
public actor NearbyReconnector {
    public enum Event: Equatable {
        /// About to dial: `number` counts every dial of this reconnector, `of` is the per-operation cap.
        case attempt(number: Int, of: Int)
        case connected(macName: String, attempt: Int, destination: NearbyWire.Destination?)
        /// A retryable failure; `retryIn` is nil when the budget is spent.
        case failed(attempt: Int, error: String, retryIn: TimeInterval?)
        /// A live session ended (the next operation reconnects).
        case disconnected(reason: String)
        case gaveUp(error: String)
    }

    /// Produces the endpoint for the next attempt. A Bonjour-based source
    /// re-browses by `fp` every time, so a Mac that came back on another port
    /// is still found; a fixed `--host/--port` source just returns it.
    public typealias EndpointSource = @Sendable () async throws -> NWEndpoint

    public let pair: PairedMac
    public let policy: ReconnectPolicy
    private let endpoints: EndpointSource
    private let onEvent: (@Sendable (Event) -> Void)?
    private let onLine: ((NearbyConnection.Direction, Data) -> Void)?
    private let sleep: @Sendable (TimeInterval) async throws -> Void
    private let random: @Sendable (ClosedRange<Double>) -> Double
    private let clock: @Sendable () -> TimeInterval
    private var session: NearbySession?
    private var closedByUser = false
    /// Connection attempts made so far (for evidence/tests).
    public private(set) var attemptsMade = 0
    public private(set) var reconnects = 0

    public init(pair: PairedMac, policy: ReconnectPolicy = ReconnectPolicy(), endpoints: @escaping EndpointSource,
                onEvent: (@Sendable (Event) -> Void)? = nil,
                onLine: ((NearbyConnection.Direction, Data) -> Void)? = nil,
                sleep: @escaping @Sendable (TimeInterval) async throws -> Void = { try await Task.sleep(nanoseconds: UInt64(max(0, $0) * 1_000_000_000)) },
                random: @escaping @Sendable (ClosedRange<Double>) -> Double = { Double.random(in: $0) },
                clock: @escaping @Sendable () -> TimeInterval = { Date().timeIntervalSinceReferenceDate }) {
        self.pair = pair
        self.policy = policy
        self.endpoints = endpoints
        self.onEvent = onEvent
        self.onLine = onLine
        self.sleep = sleep
        self.random = random
        self.clock = clock
    }

    public init(pair: PairedMac, endpoint: NWEndpoint, policy: ReconnectPolicy = ReconnectPolicy(),
                            onEvent: (@Sendable (Event) -> Void)? = nil,
                            onLine: ((NearbyConnection.Direction, Data) -> Void)? = nil) {
        self.init(pair: pair, policy: policy, endpoints: { endpoint }, onEvent: onEvent, onLine: onLine)
    }

    /// The live session, if the last one is still open.
    public var currentSession: NearbySession? {
        if let s = session, s.connection.isOpen { return s }
        return nil
    }

    /// Closes the live session; later operations reconnect (the budget is per operation).
    public func close() {
        session?.close()
        session = nil
    }

    /// Closes and refuses further operations.
    public func shutdown() {
        close()
        closedByUser = true
    }

    /// Connects (with the bounded policy) unless a session is already open.
    /// Returns the session and whether it was freshly established.
    @discardableResult
    public func connect() async throws -> NearbySession {
        try await withRetries { budget in try await self.openSession(budget: budget).session }
    }

    /// Delivers one capture, reconnecting on retryable failures and re-sending
    /// the identical `capture_id`/payload. With `requireCurrentDestination`
    /// (default) every delivery — first or retry — is checked against the
    /// destination the Mac reports *now*; a mismatch is terminal
    /// (`destinationChanged`), so a capture never lands on a re-pinned or
    /// unpinned anchor without the user knowing. A fresh connection's
    /// `hello_ack` supplies that destination for free; a reused session pays
    /// one `destination_query`.
    public func submit(_ capture: NearbyWire.CaptureSubmit, requireCurrentDestination: Bool = true,
                       requestID: String? = nil) async throws -> NearbyWire.CaptureReceived {
        guard NearbyWire.isValidID(capture.captureId) else { throw NearbyError.invalidInput("capture_id must be 1–128 ASCII [A-Za-z0-9_-]") }
        return try await withRetries { budget in
            let (session, fresh) = try await self.openSession(budget: budget)
            if requireCurrentDestination {
                let now = fresh ? session.destination : try await session.connection.destinationQuery(timeout: self.policy.requestTimeout)
                guard let now, now.destinationId == capture.destinationId, now.baseRevision == capture.baseRevision else {
                    throw NearbyError.destinationChanged(
                        captureDestination: "\(capture.destinationId) @ rev \(capture.baseRevision)",
                        current: now.map { "\($0.destinationId) @ rev \($0.baseRevision)" })
                }
            }
            return try await session.connection.submitCapture(capture, requestID: requestID, timeout: self.policy.requestTimeout)
        }
    }

    // MARK: retry loop

    /// Per-operation budget: `tries` counts delivery tries (a reused session
    /// that drops counts too, though it dialed nothing); the policy caps them
    /// at `maxAttempts` and the backoff index is the number of failures so far.
    private final class Budget { let started: TimeInterval; var tries = 0; init(started: TimeInterval) { self.started = started } }

    private func withRetries<T>(_ body: (Budget) async throws -> T) async throws -> T {
        guard !closedByUser else { throw NearbyError.closed("reconnector shut down") }
        let budget = Budget(started: clock())
        var last: NearbyError?
        while true {
            if let l = last {
                // A retry: decide whether the budget allows it and wait.
                let elapsed = clock() - budget.started
                guard budget.tries < policy.maxAttempts else {
                    let e = NearbyError.attemptsExhausted(attempts: budget.tries, last: l.description)
                    onEvent?(.gaveUp(error: e.description)); throw e
                }
                let wait = policy.delay(beforeAttempt: budget.tries + 1, random: random)
                guard elapsed + wait <= policy.overallDeadline else {
                    let e = NearbyError.attemptsExhausted(attempts: budget.tries, last: "\(l.description) (deadline \(policy.overallDeadline)s reached)")
                    onEvent?(.gaveUp(error: e.description)); throw e
                }
                onEvent?(.failed(attempt: attemptsMade, error: l.description, retryIn: wait))
                do { try await sleep(wait) } catch { throw NearbyError.cancelled }
            }
            budget.tries += 1
            do {
                return try await body(budget)
            } catch let e as NearbyError where e.isRetryable {
                dropSession(reason: e.description)
                last = e
            } catch let e as NearbyError {
                if case .cancelled = e { throw e }
                onEvent?(.gaveUp(error: e.description))
                throw e
            } catch is CancellationError {
                throw NearbyError.cancelled
            }
        }
    }

    private func dropSession(reason: String) {
        guard let s = session else { return }
        s.close()
        session = nil
        onEvent?(.disconnected(reason: reason))
    }

    /// Reuses the open session or dials once. Event attempt numbers count
    /// every dial since this reconnector was created.
    private func openSession(budget: Budget) async throws -> (session: NearbySession, fresh: Bool) {
        if let s = currentSession { return (s, false) }
        if let s = session { session = nil; onEvent?(.disconnected(reason: s.connection.closeReason ?? "closed")) }
        attemptsMade += 1
        if attemptsMade > 1 { reconnects += 1 }
        try Task.checkCancellation()
        onEvent?(.attempt(number: attemptsMade, of: policy.maxAttempts))
        let endpoint = try await endpoints()
        let s = try await NearbyClient.connect(endpoint: endpoint, pair: pair, connectTimeout: policy.connectTimeout,
                                               helloTimeout: policy.requestTimeout, onLine: onLine)
        session = s
        onEvent?(.connected(macName: s.macName, attempt: attemptsMade, destination: s.destination))
        return (s, true)
    }
}
