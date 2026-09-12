import Foundation
import Network
import UIKit

/// Discovers the FlashTeX Mac host via Bonjour and sends capture_submit payloads
/// over a TCP JSON Lines connection.
///
/// Service type: _flashtex._tcp  (Mac side must advertise this with NWListener)
/// Protocol: runtime-v1 JSON Lines — one complete JSON object per line.
/// Handshake: companion sends `{"protocol_version":1,"type":"hello","id":"<uuid>","payload":{"role":"companion"}}`
/// Mac acknowledges with `capture_received` per accepted capture.
@Observable
final class BonjourTransport {
    static let shared = BonjourTransport()

    enum State: Equatable {
        case idle
        case browsing
        case connecting(String)   // host name
        case connected(String)    // host name
        case failed(String)       // error description
    }

    private(set) var state: State = .idle
    private(set) var receivedAcks: Set<String> = []

    private var browser: NWBrowser?
    private var connection: NWConnection?
    private var receiveBuffer = Data()
    private let queue = DispatchQueue(label: "flashtex.bonjour", qos: .userInitiated)

    // Service type advertised by the Mac shell
    static let serviceType = "_flashtex._tcp."

    private init() {}

    // MARK: - Public API

    func startBrowsing() {
        guard state == .idle || isFailed else { return }
        state = .browsing
        let params = NWParameters.tcp
        let browser = NWBrowser(for: .bonjour(type: Self.serviceType, domain: nil), using: params)
        self.browser = browser

        browser.browseResultsChangedHandler = { [weak self] results, _ in
            guard let self else { return }
            if let result = results.first {
                self.connect(to: result.endpoint)
            }
        }
        browser.stateUpdateHandler = { [weak self] newState in
            if case .failed(let error) = newState {
                self?.state = .failed("Browser: \(error.localizedDescription)")
            }
        }
        browser.start(queue: queue)
    }

    func disconnect() {
        browser?.cancel()
        browser = nil
        connection?.cancel()
        connection = nil
        state = .idle
        receiveBuffer = Data()
    }

    /// Send a capture_submit envelope over the network when connected.
    /// Returns true if the payload was dispatched over the network.
    @discardableResult
    func send(_ jsonLine: String) -> Bool {
        let line = jsonLine.hasSuffix("\n") ? jsonLine : jsonLine + "\n"
        guard let data = line.data(using: .utf8) else { return false }

        if case .connected = state, let conn = connection {
            conn.send(content: data, completion: .contentProcessed { [weak self] error in
                if let error {
                    self?.state = .failed("Send: \(error.localizedDescription)")
                }
            })
            return true
        }
        return false
    }

    // MARK: - Private

    private var isFailed: Bool {
        if case .failed = state { return true }
        return false
    }

    private func connect(to endpoint: NWEndpoint) {
        browser?.cancel()
        browser = nil

        let hostName: String
        if case .service(let name, _, _, _) = endpoint {
            hostName = name
        } else {
            hostName = endpoint.debugDescription
        }
        state = .connecting(hostName)

        let params = NWParameters.tcp
        params.allowLocalEndpointReuse = true
        let conn = NWConnection(to: endpoint, using: params)
        connection = conn

        conn.stateUpdateHandler = { [weak self] newState in
            guard let self else { return }
            switch newState {
            case .ready:
                self.state = .connected(hostName)
                self.sendHello()
                self.startReceiving()
            case .failed(let error):
                self.state = .failed("Connection: \(error.localizedDescription)")
                self.connection = nil
            case .cancelled:
                if case .connected = self.state { self.state = .idle }
            default:
                break
            }
        }
        conn.start(queue: queue)
    }

    private func sendHello() {
        let helloID = UUID().uuidString
        let hello = """
        {"protocol_version":1,"type":"hello","id":"\(helloID)","payload":{"role":"companion"}}
        """
        _ = send(hello)
    }

    private func startReceiving() {
        connection?.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isComplete, error in
            guard let self else { return }
            if let data { self.receiveBuffer.append(data) }
            self.processBuffer()
            if !isComplete && error == nil {
                self.startReceiving()
            }
        }
    }

    private func processBuffer() {
        while let range = receiveBuffer.range(of: Data([0x0A])) { // newline
            let lineData = receiveBuffer[receiveBuffer.startIndex..<range.lowerBound]
            receiveBuffer.removeSubrange(receiveBuffer.startIndex...range.lowerBound)
            handleLine(lineData)
        }
    }

    private func handleLine(_ data: Data) {
        guard let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let type_ = json["type"] as? String,
              let payload = json["payload"] as? [String: Any] else { return }

        if type_ == "capture_received",
           let captureId = payload["capture_id"] as? String {
            DispatchQueue.main.async {
                self.receivedAcks.insert(captureId)
            }
        }
    }
}
