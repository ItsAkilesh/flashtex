import AppKit
import Combine
import SwiftUI

// MARK: - flow controller (NearbyState → PairingFlow.Machine → effects)

/// Drives `PairingFlow.Machine` from `NearbyState`'s published values, runs
/// the machine's effects against the transport and the `PairingJournal`, and
/// posts accessibility announcements. One controller per `NearbyState`, kept
/// alive across window open/close so an attempt is tracked (and journaled)
/// even while the window is closed.
///
/// Inputs the transport does not publish yet (a bootstrap session opening,
/// per-connection close reasons, byte progress) can be fed through
/// `observe(_:)`; the machine and its tests already cover them.
@MainActor
final class PairingFlowController: ObservableObject {
    @Published private(set) var machine: PairingFlow.Machine
    /// Everything announced so far (VoiceOver text; also test evidence).
    @Published private(set) var announcements: [String] = []
    /// Inputs dropped because they belonged to an older attempt or another pairing.
    @Published private(set) var staleInputs: [PairingFlow.Input] = []
    let journal: PairingJournal
    private(set) weak var nearby: NearbyState?
    /// Posts `text` to assistive technology; tests replace it.
    var announcer: (String) -> Void = { text in
        if #available(macOS 14.0, *) { AccessibilityNotification.Announcement(text).post() }
    }
    private var cancellables: Set<AnyCancellable> = []
    private var expiry: DispatchWorkItem?
    /// `NearbyState.pairingCode` as last observed (nil when the transport's
    /// own code is not being shown, including for resumed attempts).
    private var transportCode: String?
    /// Set right before we ask the transport to withdraw its code, so the
    /// resulting `pairingCode = nil` is not mistaken for expiry.
    private var expectingWithdrawal = false
    private var knownPairIds: Set<String>
    private var lastStatus: String?
    private var lastCaptureId: String?

    private static let shared = NSMapTable<NearbyState, PairingFlowController>.weakToStrongObjects()

    /// The controller for `nearby`, created on first use with the default journal.
    static func controller(for nearby: NearbyState, journal: PairingJournal? = nil) -> PairingFlowController {
        if let c = shared.object(forKey: nearby) { return c }
        let c = PairingFlowController(nearby: nearby, journal: journal ?? PairingJournal(url: PairingJournal.defaultURL()))
        shared.setObject(c, forKey: nearby)
        return c
    }

    init(nearby: NearbyState, journal: PairingJournal, now: Date = Date(), announcer: ((String) -> Void)? = nil) {
        self.nearby = nearby
        self.journal = journal
        if let announcer { self.announcer = announcer }
        machine = PairingFlow.Machine(phase: nearby.isAdvertising ? .advertising : .off,
                                      generation: journal.generation, isAdvertising: nearby.isAdvertising)
        knownPairIds = Set(nearby.pairs.map(\.pairId))
        lastStatus = nearby.status
        lastCaptureId = nearby.lastReceivedCaptureId
        if let e = journal.loadError { announce(e) }
        subscribe(nearby)
        if let pending = journal.pending {
            apply(.restored(pending), now: now)
        }
    }

    var phase: PairingFlow.Phase { machine.phase }

    // MARK: user actions

    func showCode() {
        guard phase.canShowCode(), let nearby else { return }
        nearby.beginPairing() // → $pairingCode → .codeIssued
    }

    func cancel() { apply(.cancel) }
    func resume() { apply(.resume) }
    func dismiss() { apply(.dismiss) }

    func setAdvertising(_ on: Bool) {
        guard let nearby else { return }
        if on {
            nearby.startAdvertising()
        } else {
            if transportCode != nil { expectingWithdrawal = true }
            nearby.stopAdvertising()
        }
    }

    func forget(pairId: String) {
        nearby?.forget(pairId: pairId) // → $pairs → .forgotten
    }

    /// Listener events (when the transport owner wires them through).
    func observe(_ event: NearbyListener.Event) {
        let g = machine.generation
        switch event {
        case .connectionOpened: apply(.bootstrapSessionOpened(generation: g))
        case .hello(let id, let name, let bootstrap):
            apply(bootstrap ? .confirmed(pairId: id, companionName: name, generation: g) : .otherCompanionConnected(generation: g))
        case .connectionClosed(let id, let reason): apply(.peerGone(pairId: id, reason: reason, generation: g))
        case .capture(let id): apply(.captureReceived(pairId: nil, captureId: id))
        case .failed(let why): apply(.listenerFailed(why))
        case .ready, .stopped: break
        }
    }

    /// Byte progress for an in-flight line (no transport source yet; see handoff).
    func observeProgress(pairId: String, companionName: String?, captureId: String?, bytes: Int, total: Int?) {
        apply(.receiving(pairId: pairId, companionName: companionName, captureId: captureId, bytes: bytes, total: total))
    }

    // MARK: transport observation

    private func subscribe(_ nearby: NearbyState) {
        // `@Published` publishers fire on willSet with the new value; the
        // store and coordinator are already current at that point.
        nearby.$isAdvertising.dropFirst().removeDuplicates().sink { [weak self] on in
            self?.apply(.advertising(on))
        }.store(in: &cancellables)

        nearby.$pairingCode.dropFirst().sink { [weak self, weak nearby] code in
            guard let self, let nearby else { return }
            self.transportCodeChanged(to: code, nearby: nearby)
        }.store(in: &cancellables)

        nearby.$pairs.dropFirst().sink { [weak self] pairs in
            guard let self else { return }
            let ids = Set(pairs.map(\.pairId))
            if let a = self.machine.attempt, let r = pairs.first(where: { $0.pairId == a.pairId }), !self.knownPairIds.contains(a.pairId) {
                self.apply(.confirmed(pairId: r.pairId, companionName: r.companionName, generation: a.generation))
            }
            for gone in self.knownPairIds.subtracting(ids) { self.apply(.forgotten(pairId: gone)) }
            self.knownPairIds = ids
        }.store(in: &cancellables)

        nearby.$status.dropFirst().sink { [weak self] status in
            guard let self, status != self.lastStatus else { return }
            self.lastStatus = status
            if status.hasPrefix("listener error") || status.hasPrefix("listener failed") {
                self.apply(.listenerFailed(status))
            }
        }.store(in: &cancellables)

        nearby.$lastReceivedCaptureId.dropFirst().sink { [weak self] id in
            guard let self, let id, id != self.lastCaptureId else { return }
            self.lastCaptureId = id
            self.apply(.captureReceived(pairId: nil, captureId: id))
        }.store(in: &cancellables)
    }

    private func transportCodeChanged(to code: String?, nearby: NearbyState) {
        guard code != transportCode else { return }
        let previous = transportCode
        transportCode = code
        if let code {
            let now = Date()
            let generation = journal.nextGeneration()
            let attempt = PairingFlow.Attempt(
                generation: generation, code: code,
                pairId: Pairing.derive(code: code, salt: nearby.store.salt).pairId,
                startedAt: now,
                expiresAt: nearby.coordinator.current?.expiresAt ?? now.addingTimeInterval(Pairing.codeLifetime))
            apply(.codeIssued(attempt), now: now)
            return
        }
        guard previous != nil, let a = machine.attempt else { expectingWithdrawal = false; return }
        if let r = nearby.store.pair(id: a.pairId) {
            apply(.confirmed(pairId: r.pairId, companionName: r.companionName, generation: a.generation))
        } else if expectingWithdrawal {
            expectingWithdrawal = false // our own cancel or advertising-off; the machine already moved
        } else if let until = nearby.codeExpiresAt, until.timeIntervalSinceNow > 1 {
            apply(.withdrawn(generation: a.generation, reason: "the transport withdrew the code"))
        } else {
            apply(.codeExpired(generation: a.generation))
        }
    }

    // MARK: machine + effects

    @discardableResult
    func apply(_ input: PairingFlow.Input, now: Date = Date()) -> PairingFlow.Outcome {
        var m = machine
        let outcome = m.apply(input, now: now)
        machine = m
        if outcome.stale { staleInputs.append(input) }
        for effect in outcome.effects { perform(effect, now: now) }
        return outcome
    }

    private func perform(_ effect: PairingFlow.Effect, now: Date) {
        switch effect {
        case .persist(let a):
            journal.setPending(a)
            scheduleExpiry(a)
        case .clearJournal:
            journal.setPending(nil)
            expiry?.cancel()
            expiry = nil
        case .cancelTransport(let a):
            expiry?.cancel()
            expiry = nil
            guard let nearby else { return }
            if nearby.pairingCode == a.code {
                expectingWithdrawal = true
                nearby.cancelPairing() // restarts the listener without the bootstrap key
            } else if nearby.coordinator.current?.code == a.code {
                // Resumed attempt: the transport's own code state is nil.
                nearby.coordinator.cancel()
                if nearby.isAdvertising { nearby.startAdvertising() }
            }
        case .resumeTransport(let a):
            guard let nearby else { return }
            // `NearbyState.beginPairing()` only mints fresh codes; resume through
            // the coordinator it exposes and rebuild the listener's key table.
            _ = nearby.coordinator.begin(code: a.code, lifetime: a.remaining(at: now))
            nearby.startAdvertising()
            journal.setPending(a)
            scheduleExpiry(a)
        case .announce(let text):
            announce(text)
        }
    }

    private func announce(_ text: String) {
        announcements.append(text)
        if announcements.count > 50 { announcements.removeFirst(announcements.count - 50) }
        announcer(text)
    }

    private func scheduleExpiry(_ a: PairingFlow.Attempt) {
        expiry?.cancel()
        let item = DispatchWorkItem { [weak self] in self?.apply(.codeExpired(generation: a.generation)) }
        expiry = item
        DispatchQueue.main.asyncAfter(deadline: .now() + a.remaining(at: Date()) + 0.25, execute: item)
    }
}

// MARK: - window

/// `Edit > Nearby Companion…` (⌘⇧N): advertising, the pairing flow, paired
/// devices and received captures. Everything shown is the real listener state
/// as seen through `PairingFlowController`; captures land in an in-memory
/// inbox until the bridge client takes over.
struct NearbyView: View {
    @EnvironmentObject private var nearby: NearbyState
    @Environment(ShellModel.self) private var model
    @State private var controller: PairingFlowController?

    var body: some View {
        Group {
            if let controller {
                NearbyFlowView(controller: controller, nearby: nearby, model: model)
            } else {
                ProgressView().padding()
            }
        }
        .onAppear { if controller == nil { controller = PairingFlowController.controller(for: nearby) } }
    }
}

struct NearbyFlowView: View {
    @ObservedObject var controller: PairingFlowController
    @ObservedObject var nearby: NearbyState
    var model: ShellModel

    enum Focus: Hashable { case showCode, code, resume, dismiss, cancel }
    @FocusState private var focus: Focus?

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            advertisingSection
            Divider()
            pairingSection
            Divider()
            pairedSection
            Divider()
            CapturesSection(inbox: model.nearbyInbox, bridgeCaptures: model.bridgeCaptures,
                            lastCaptureId: nearby.lastReceivedCaptureId)
            Divider()
            logSection
        }
        .padding(16)
        .frame(minWidth: 460, idealWidth: 500, minHeight: 560)
        .onChange(of: controller.phase) { _, phase in moveFocus(for: phase) }
    }

    private func moveFocus(for phase: PairingFlow.Phase) {
        switch phase {
        case .codeShown, .verifying: focus = .code
        case .interrupted(let a, _, _): focus = a.isExpired(at: Date()) ? .dismiss : .resume
        case .failed, .paired: focus = .dismiss
        case .off, .advertising, .receiving: focus = .showCode
        }
    }

    // MARK: advertising

    private var advertisingSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("Nearby companion").font(.headline)
                Spacer()
                Toggle("Advertise", isOn: Binding(
                    get: { nearby.isAdvertising },
                    set: { controller.setAdvertising($0) }))
                .toggleStyle(.switch)
                .accessibilityLabel("Advertise on the local network")
                .accessibilityValue(nearby.isAdvertising ? "on, port \(nearby.port.map(String.init) ?? "unknown")" : "off")
                .accessibilityIdentifier("nearby.advertise")
            }
            Text(nearby.status).font(.caption).foregroundStyle(.secondary)
                .accessibilityLabel("Listener status")
                .accessibilityValue(nearby.status)
                .accessibilityIdentifier("nearby.listener.status")
            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 2) {
                GridRow { Text("Bonjour").foregroundStyle(.secondary); Text("\(nearby.serviceType)  “\(nearby.macName)”") }
                GridRow { Text("Port").foregroundStyle(.secondary); Text(nearby.port.map(String.init) ?? "—") }
                GridRow { Text("Mac id (fp)").foregroundStyle(.secondary); Text(nearby.fingerprint).font(.system(.body, design: .monospaced)) }
                GridRow { Text("TLS").foregroundStyle(.secondary); Text("1.2, \(nearby.cipherSuite), no resumption") }
            }
            .font(.callout)
            .accessibilityElement(children: .combine)
            .accessibilityLabel("Service details")
            .accessibilityValue("Bonjour \(nearby.serviceType) named \(nearby.macName); port \(nearby.port.map(String.init) ?? "none"); Mac id \(nearby.fingerprint); TLS 1.2 \(nearby.cipherSuite)")
            Text("Proposal nearby-v1 — not yet a published contract. Captures are kept in memory only (durable: false).")
                .font(.caption2).foregroundStyle(.secondary)
        }
    }

    // MARK: pairing flow

    private var pairingSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Pair a companion").font(.headline)
            statusRow
            switch controller.phase {
            case .codeShown(let a), .verifying(let a):
                codeRow(a, verifying: { if case .verifying = controller.phase { return true }; return false }())
                Text("Enter this code on the companion while “\(nearby.macName)” is selected. The code is valid for one pairing.")
                    .font(.caption).foregroundStyle(.secondary)
            case .interrupted(let a, _, _):
                interruptedRow(a)
            case .failed:
                HStack {
                    Button("Dismiss") { controller.dismiss() }
                        .keyboardShortcut(.cancelAction)
                        .focused($focus, equals: .dismiss)
                        .accessibilityIdentifier("nearby.pairing.dismiss")
                    showCodeButton(title: "Show New Code")
                }
            case .paired(let p):
                HStack {
                    Label("Paired with \(p.companionName)", systemImage: "checkmark.circle.fill").foregroundStyle(.green)
                    Spacer()
                    Button("Dismiss") { controller.dismiss() }
                        .keyboardShortcut(.cancelAction)
                        .focused($focus, equals: .dismiss)
                        .accessibilityIdentifier("nearby.pairing.dismiss")
                }
            case .receiving(let r):
                HStack(spacing: 10) {
                    if let total = r.total, total > 0 {
                        ProgressView(value: Double(r.bytes), total: Double(total))
                    } else {
                        ProgressView()
                    }
                    Text(byteProgress(r)).font(.callout).monospacedDigit()
                }
                .accessibilityElement(children: .combine)
                .accessibilityLabel("Receiving capture")
                .accessibilityValue(controller.phase.accessibilityValue())
                .accessibilityIdentifier("nearby.pairing.receiving")
                Text("Receiving cannot be cancelled from the Mac in v1; the companion's send controls it.")
                    .font(.caption2).foregroundStyle(.secondary)
            case .off, .advertising:
                HStack {
                    showCodeButton(title: "Show Pairing Code")
                    Text("A 6-digit code, valid for \(Int(Pairing.codeLifetime)) s; starts advertising if needed.")
                        .font(.caption).foregroundStyle(.secondary)
                }
            }
        }
    }

    /// One line that always names the state precisely (advertising, code
    /// shown, verifying, paired, receiving n/m bytes, interrupted, error).
    private var statusRow: some View {
        TimelineView(.periodic(from: .now, by: 1)) { ctx in
            let phase = controller.phase
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(phase.title).font(.subheadline.weight(.semibold))
                    .foregroundStyle(statusColor(phase))
                Text(phase.detail(now: ctx.date)).font(.caption).foregroundStyle(.secondary)
            }
            .accessibilityElement(children: .ignore)
            .accessibilityLabel("Pairing state: \(phase.title)")
            .accessibilityValue(phase.accessibilityValue(now: ctx.date))
            .accessibilityIdentifier("nearby.pairing.state")
        }
    }

    private func byteProgress(_ r: PairingFlow.Receiving) -> String {
        let f = ByteCountFormatter()
        f.countStyle = .file
        let got = f.string(fromByteCount: Int64(r.bytes))
        return r.total.map { "\(got) of \(f.string(fromByteCount: Int64($0)))" } ?? "\(got) so far"
    }

    private func statusColor(_ phase: PairingFlow.Phase) -> Color {
        switch phase {
        case .failed: return .red
        case .interrupted: return .orange
        case .paired: return .green
        default: return .primary
        }
    }

    private func showCodeButton(title: String) -> some View {
        Button(title) { controller.showCode() }
            .keyboardShortcut(.defaultAction)
            .focused($focus, equals: .showCode)
            .disabled(!controller.phase.canShowCode())
            .accessibilityHint("Shows a six-digit code to type on the companion; starts advertising if needed.")
            .accessibilityIdentifier("nearby.pairing.show")
    }

    private func codeRow(_ a: PairingFlow.Attempt, verifying: Bool) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 16) {
            Text(Pairing.displayCode(a.code))
                .font(.system(size: 34, weight: .semibold, design: .monospaced))
                .textSelection(.enabled)
                .focusable()
                .focused($focus, equals: .code)
                .accessibilityLabel("Pairing code")
                .accessibilityValue(Pairing.spokenCode(a.code))
                .accessibilityHint("Type these six digits on the companion.")
                .accessibilityIdentifier("nearby.pairing.code")
            TimelineView(.periodic(from: .now, by: 1)) { ctx in
                let left = Int(a.remaining(at: ctx.date).rounded(.up))
                Text("expires in \(left) s").font(.callout).foregroundStyle(left <= 15 ? .red : .secondary)
                    .accessibilityLabel("Code expiry")
                    .accessibilityValue("\(left) seconds left")
                    .accessibilityAddTraits(.updatesFrequently)
                    .accessibilityIdentifier("nearby.pairing.expiry")
            }
            if verifying {
                ProgressView().controlSize(.small)
                    .accessibilityLabel("Verifying companion")
            }
            Spacer()
            Button("Cancel") { controller.cancel() }
                .keyboardShortcut(.cancelAction)
                .focused($focus, equals: .cancel)
                .accessibilityLabel("Cancel pairing")
                .accessibilityIdentifier("nearby.pairing.cancel")
        }
    }

    private func interruptedRow(_ a: PairingFlow.Attempt) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 16) {
            if !a.isExpired(at: Date()) {
                Text(Pairing.displayCode(a.code))
                    .font(.system(size: 34, weight: .semibold, design: .monospaced))
                    .foregroundStyle(.secondary)
                    .accessibilityLabel("Interrupted pairing code")
                    .accessibilityValue(Pairing.spokenCode(a.code))
                    .accessibilityIdentifier("nearby.pairing.code")
                Spacer()
                Button("Resume") { controller.resume() }
                    .keyboardShortcut(.defaultAction)
                    .focused($focus, equals: .resume)
                    .accessibilityHint("Serves this code again for its remaining time.")
                    .accessibilityIdentifier("nearby.pairing.resume")
                Button("Cancel") { controller.cancel() }
                    .keyboardShortcut(.cancelAction)
                    .focused($focus, equals: .cancel)
                    .accessibilityLabel("Cancel interrupted pairing")
                    .accessibilityIdentifier("nearby.pairing.cancel")
            } else {
                Button("Dismiss") { controller.dismiss() }
                    .keyboardShortcut(.cancelAction)
                    .focused($focus, equals: .dismiss)
                    .accessibilityIdentifier("nearby.pairing.dismiss")
                showCodeButton(title: "Show New Code")
            }
        }
    }

    // MARK: paired devices

    private var pairedSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Paired companions").font(.headline)
            if nearby.pairs.isEmpty {
                Text("None yet.").font(.callout).foregroundStyle(.secondary)
                    .accessibilityLabel("Paired companions: none yet")
            } else {
                ForEach(nearby.pairs) { pair in
                    let connected = nearby.connectedPairIds.contains(pair.pairId)
                    let a11y = PairingAccessibility.deviceRow(pair, connected: connected)
                    HStack {
                        VStack(alignment: .leading, spacing: 1) {
                            HStack(spacing: 6) {
                                Text(pair.companionName).font(.callout)
                                if connected {
                                    Text("connected").font(.caption2).padding(.horizontal, 5).padding(.vertical, 1)
                                        .background(Color.green.opacity(0.2), in: Capsule())
                                }
                            }
                            Text("\(pair.pairId) · paired \(pair.createdAt.formatted(date: .abbreviated, time: .shortened))"
                                 + (pair.lastSeenAt.map { " · seen \($0.formatted(date: .abbreviated, time: .shortened))" } ?? ""))
                                .font(.caption).foregroundStyle(.secondary)
                        }
                        .accessibilityElement(children: .ignore)
                        .accessibilityLabel(a11y.label)
                        .accessibilityValue(a11y.value)
                        .accessibilityIdentifier("nearby.device.\(pair.pairId)")
                        Spacer()
                        Button("Forget") { controller.forget(pairId: pair.pairId) }
                            .accessibilityLabel("Forget \(pair.companionName)")
                            .accessibilityHint("Removes the pairing and closes its connection.")
                            .accessibilityIdentifier("nearby.device.\(pair.pairId).forget")
                    }
                }
            }
            Text("Keys live in \(nearby.store.url.path) (mode 0600), not the Keychain.")
                .font(.caption2).foregroundStyle(.secondary)
        }
    }

    // MARK: activity

    private var logSection: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Activity").font(.headline)
            ScrollView {
                VStack(alignment: .leading, spacing: 1) {
                    ForEach(Array(nearby.log.suffix(30).enumerated()), id: \.offset) { _, line in
                        Text(line).font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .frame(minHeight: 60, maxHeight: 120)
            .accessibilityLabel("Activity log")
            .accessibilityValue(nearby.log.suffix(5).joined(separator: ". "))
            .accessibilityIdentifier("nearby.log")
        }
    }
}

/// Received captures, one row per capture with its state: in-memory inbox
/// entries (never durable) and captures the attached bridge holds.
struct CapturesSection: View {
    @ObservedObject var inbox: NearbyInbox
    var bridgeCaptures: [BridgeSession.Capture]
    var lastCaptureId: String?

    var rows: [PairingAccessibility.CaptureRow] {
        var out = inbox.received.map { c in
            PairingAccessibility.CaptureRow(captureId: c.captureId, source: .inbox, state: "received",
                                            durable: false, note: "\(c.image.mimeType), \(c.image.dataBase64.utf8.count) base64 bytes")
        }
        out += bridgeCaptures.map {
            PairingAccessibility.CaptureRow(captureId: $0.captureId, source: .bridge, state: $0.state.rawValue,
                                            durable: nil, note: $0.note)
        }
        return out
    }

    private var lastId: String { lastCaptureId ?? inbox.lastCaptureId ?? "—" }

    var body: some View {
        let rows = rows
        VStack(alignment: .leading, spacing: 4) {
            Text("Received captures").font(.headline)
            Text("Last capture id: \(lastId)")
                .font(.system(.callout, design: .monospaced)).textSelection(.enabled)
                .accessibilityLabel("Last capture id")
                .accessibilityValue(lastId)
                .accessibilityIdentifier("nearby.captures.last")
            if rows.isEmpty {
                Text(inbox.lastNote ?? "No captures received yet.")
                    .font(.caption).foregroundStyle(.secondary)
            } else {
                ForEach(rows.suffix(8)) { row in CaptureRowView(row: row) }
                if let note = inbox.lastNote {
                    Text(note).font(.caption).foregroundStyle(.secondary)
                }
            }
        }
    }
}

struct CaptureRowView: View {
    var row: PairingAccessibility.CaptureRow

    var body: some View {
        let inMemory = row.durable == false
        HStack(spacing: 8) {
            Image(systemName: inMemory ? "tray" : "checkmark.seal")
                .foregroundStyle(inMemory ? Color.secondary : Color.green)
            Text(row.captureId).font(.system(.callout, design: .monospaced))
            Text("\(row.state) · \(row.durabilityText)").font(.caption).foregroundStyle(.secondary)
            if !row.note.isEmpty {
                Text(row.note).font(.caption2).foregroundStyle(.tertiary).lineLimit(1)
            }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(row.accessibilityLabel)
        .accessibilityValue(row.accessibilityValue)
        .accessibilityIdentifier("nearby.capture.\(row.id)")
    }
}
