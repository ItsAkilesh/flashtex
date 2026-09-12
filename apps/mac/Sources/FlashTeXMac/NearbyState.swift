import Combine
import Foundation

/// Holds the one pairing in progress and finalizes it from the listener queue.
/// Kept off the main actor because TLS sessions confirm pairings synchronously.
final class PairingCoordinator: PairingConfirmer {
    struct Pending: Equatable {
        let code: String
        let derived: Pairing.Derived
        let expiresAt: Date
    }

    let store: PairStore
    private let lock = NSLock()
    private var pending: Pending?
    /// Called (on an arbitrary queue) when a bootstrap connection became a pairing.
    var onConfirmed: ((PairRecord) -> Void)?

    init(store: PairStore) { self.store = store }

    var current: Pending? { lock.withLock { pending } }

    func begin(code: String = Pairing.generateCode(), lifetime: TimeInterval = Pairing.codeLifetime) -> Pending {
        let p = Pending(code: code, derived: Pairing.derive(code: code, salt: store.salt), expiresAt: Date().addingTimeInterval(lifetime))
        lock.withLock { pending = p }
        return p
    }

    func cancel() { lock.withLock { pending = nil } }

    /// Bootstrap PSK entry for the listener while a code is valid.
    var bootstrapEntry: NearbyListener.PSKEntry? {
        guard let p = current, Date() < p.expiresAt else { return nil }
        return .init(identity: p.derived.pairId, key: p.derived.psk, isBootstrap: true)
    }

    func confirmPairing(pairId: String, companionName: String) -> Data? {
        let record: PairRecord? = lock.withLock {
            guard let p = pending, p.derived.pairId == pairId, Date() < p.expiresAt else { return nil }
            let psk = Pairing.mintLongTermPSK()
            let r = PairRecord(pairId: pairId, psk: psk.base64EncodedString(), companionName: companionName,
                               createdAt: Date(), lastSeenAt: Date())
            guard store.upsert(r) else { return nil } // not persisted → not paired
            pending = nil
            return r
        }
        guard let record else { return nil }
        onConfirmed?(record)
        return record.pskData
    }

    func notePairSeen(pairId: String) { store.touch(pairId: pairId) }
}

/// UI-facing state for the nearby listener: advertising, port, pairing code,
/// paired companions, last received capture.
@MainActor
final class NearbyState: ObservableObject {
    @Published private(set) var isAdvertising = false
    @Published private(set) var port: UInt16?
    @Published private(set) var status = "off"
    @Published private(set) var pairingCode: String?
    @Published private(set) var codeExpiresAt: Date?
    @Published private(set) var pairs: [PairRecord] = []
    @Published private(set) var connectedPairIds: [String] = []
    @Published private(set) var lastReceivedCaptureId: String?
    @Published private(set) var log: [String] = []

    let macName: String
    let store: PairStore
    let coordinator: PairingCoordinator
    let loopbackOnly: Bool
    private let queue = DispatchQueue(label: "flashtex.nearby.listener")
    private var listener: NearbyListener?
    private weak var sink: CaptureSink?
    private weak var destinations: DestinationProvider?
    private var expiry: DispatchWorkItem?
    private var wantAdvertising = false

    init(store: PairStore = PairStore(url: PairStore.defaultURL()),
         macName: String = Host.current().localizedName ?? "Mac", loopbackOnly: Bool = false) {
        self.store = store
        self.macName = macName
        self.loopbackOnly = loopbackOnly
        self.coordinator = PairingCoordinator(store: store)
        self.pairs = store.pairs
        if let e = store.loadError { log.append(e); status = e }
        coordinator.onConfirmed = { [weak self] record in
            Task { @MainActor in self?.pairingConfirmed(record) }
        }
    }

    var fingerprint: String { Pairing.fingerprint(salt: store.salt) }
    var serviceType: String { NearbyV1.serviceType }
    var cipherSuite: String { NearbyListener.cipherSuiteName }

    var txtRecord: [String: String] {
        ["v": "1", "name": macName, "fp": fingerprint, "salt": Pairing.hex(store.salt)]
    }

    /// Where captures and destination queries go. The App attaches the ShellModel.
    func attach(sink: CaptureSink, destinations: DestinationProvider) {
        self.sink = sink
        self.destinations = destinations
        if listener != nil { restartListener() }
    }

    // MARK: advertising

    func startAdvertising() {
        wantAdvertising = true
        restartListener()
    }

    func stopAdvertising() {
        wantAdvertising = false
        cancelPairing()
        listener?.stop()
        listener = nil
        isAdvertising = false
        port = nil
        connectedPairIds = []
        status = "off"
        note("stopped")
    }

    /// Rebuilds the listener from the pair store plus any pending bootstrap key,
    /// keeping already-authenticated connections alive.
    private func restartListener() {
        guard wantAdvertising else { return }
        var psks = pairs.compactMap { r -> NearbyListener.PSKEntry? in
            guard let k = r.pskData, k.count == Pairing.pskLength else { return nil }
            return .init(identity: r.pairId, key: k, isBootstrap: false)
        }
        if let b = coordinator.bootstrapEntry { psks.append(b) }
        let config = NearbyListener.Configuration(
            psks: psks, macName: macName, port: port,
            advertisement: .init(name: macName, txt: txtRecord), loopbackOnly: loopbackOnly)
        let next = NearbyListener(configuration: config, sink: sink, destinations: destinations,
                                  pairing: coordinator, queue: queue) { [weak self] event in
            Task { @MainActor in self?.handle(event) }
        }
        let previous = listener
        listener = next
        status = "starting…"
        let start: () -> Void = { [weak self, weak next] in
            Task { @MainActor in
                guard let self, let next, self.listener === next else { return }
                do { try next.start() } catch {
                    self.status = "listener failed: \(error.localizedDescription)"
                    self.note(self.status)
                    self.isAdvertising = false
                }
            }
        }
        if let old = previous {
            next.adoptConnections(from: old)
            old.stop(keepConnections: true, completion: start)
        } else {
            start()
        }
    }

    private func handle(_ event: NearbyListener.Event) {
        switch event {
        case .ready(let p):
            port = p
            isAdvertising = true
            status = "advertising \(serviceType) on port \(p)"
            note("ready on port \(p)")
        case .failed(let why):
            status = "listener error: \(why)"
            note(status)
            if port != nil, listener != nil {
                // The requested port may be gone; try once more with an ephemeral one.
                port = nil
                listener = nil
                restartListener()
            } else {
                isAdvertising = false
            }
        case .stopped:
            break
        case .connectionOpened:
            note("authenticated connection")
        case .hello(let id, let name, let bootstrap):
            connectedPairIds.append(id)
            note(bootstrap ? "paired \(name) (\(id))" : "hello from \(name) (\(id))")
        case .capture(let id):
            lastReceivedCaptureId = id
            note("capture \(id)")
        case .connectionClosed(let id, let reason):
            if let id, let i = connectedPairIds.firstIndex(of: id) { connectedPairIds.remove(at: i) }
            note("closed \(id ?? "unauthenticated"): \(reason)")
        }
    }

    // MARK: pairing

    /// Shows a fresh code for `Pairing.codeLifetime`; the derived bootstrap PSK
    /// is accepted only until then.
    func beginPairing() {
        if !wantAdvertising { wantAdvertising = true }
        let p = coordinator.begin()
        pairingCode = p.code
        codeExpiresAt = p.expiresAt
        expiry?.cancel()
        let item = DispatchWorkItem { [weak self] in self?.pairingExpired() }
        expiry = item
        DispatchQueue.main.asyncAfter(deadline: .now() + Pairing.codeLifetime, execute: item)
        note("pairing code issued for \(p.derived.pairId)")
        restartListener()
    }

    func cancelPairing() {
        expiry?.cancel()
        expiry = nil
        let had = pairingCode != nil
        coordinator.cancel()
        pairingCode = nil
        codeExpiresAt = nil
        if had, listener != nil { restartListener() }
    }

    private func pairingExpired() {
        guard pairingCode != nil else { return }
        note("pairing code expired")
        cancelPairing()
    }

    private func pairingConfirmed(_ record: PairRecord) {
        expiry?.cancel()
        expiry = nil
        pairingCode = nil
        codeExpiresAt = nil
        pairs = store.pairs
        note("stored pairing \(record.pairId) for \(record.companionName)")
        // Drop the bootstrap key and add the long-term one; the confirming
        // connection is adopted, not dropped.
        restartListener()
    }

    func forget(pairId: String) {
        store.remove(pairId: pairId)
        pairs = store.pairs
        note("forgot \(pairId)")
        if listener != nil { restartListener() }
    }

    func refreshPairs() { pairs = store.pairs }

    private func note(_ line: String) {
        log.append(line)
        if log.count > 100 { log.removeFirst(log.count - 100) }
    }
}
