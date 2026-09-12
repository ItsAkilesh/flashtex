import CryptoKit
import Foundation

/// Pairing-code derivation for the nearby transport (proposal nearby-v1).
///
/// The Mac shows a short code; both sides derive the same bootstrap PSK and
/// `pair_id` from (code, Mac salt) with HKDF-SHA256. The bootstrap PSK only
/// authenticates the first connection: the Mac then mints a random 32-byte
/// long-term PSK and hands it over inside that TLS session (`hello_ack.pair_psk`).
enum Pairing {
    static let codeLength = 6
    static let codeLifetime: TimeInterval = 120
    static let saltLength = 16
    static let pskLength = 32
    static let pskInfo = Data("flashtex-nearby-v1 psk".utf8)
    static let pairIDInfo = Data("flashtex-nearby-v1 pair-id".utf8)
    static let fingerprintInfo = Data("flashtex-nearby-v1 mac-id".utf8)

    struct Derived: Equatable {
        let pairId: String
        let psk: Data
    }

    /// Six decimal digits from the system CSPRNG (~20 bits; see the proposal's
    /// threat model for why this is only a bootstrap secret).
    static func generateCode() -> String {
        var g = SystemRandomNumberGenerator()
        return (0..<codeLength).map { _ in String(Int.random(in: 0...9, using: &g)) }.joined()
    }

    static func generateSalt() -> Data { randomBytes(saltLength) }
    static func mintLongTermPSK() -> Data { randomBytes(pskLength) }

    static func randomBytes(_ count: Int) -> Data {
        SymmetricKey(size: .init(bitCount: count * 8)).withUnsafeBytes { Data($0) }
    }

    /// Deterministic: the companion runs the same derivation from the code it
    /// typed and the `salt=` TXT value.
    static func derive(code: String, salt: Data) -> Derived {
        let ikm = SymmetricKey(data: Data(code.utf8))
        let psk = HKDF<SHA256>.deriveKey(inputKeyMaterial: ikm, salt: salt, info: pskInfo, outputByteCount: pskLength)
        let id = HKDF<SHA256>.deriveKey(inputKeyMaterial: ikm, salt: salt, info: pairIDInfo, outputByteCount: 8)
        return Derived(pairId: hex(id.withUnsafeBytes { Data($0) }), psk: psk.withUnsafeBytes { Data($0) })
    }

    /// Stable per-Mac identifier advertised as TXT `fp=` so a companion can
    /// match a discovered service to a stored pairing without connecting.
    static func fingerprint(salt: Data) -> String {
        var h = SHA256()
        h.update(data: fingerprintInfo)
        h.update(data: salt)
        return String(hex(Data(h.finalize())).prefix(16))
    }

    /// Test vector: code "123456", salt 000102…0f (see docs/nearby-v1-proposal.md).
    static let vectorPairID = "3917d7c3e5eef7ce"
    static let vectorPSKHex = "b127a48a782dbece3edbed18d027d658890624ecf21da77d19f47d5e604d1221"
    static let vectorFingerprint = "0e712816d64b7c47"
    /// helloProof(psk: vector PSK, nonce: "n-1")
    static let vectorHelloProof = "rIBrMvRrNjs2eQueTGVsBNF6K130BW6fqV93Rw//WgM="

    static let helloProofInfo = Data("flashtex-nearby-v1 hello".utf8)

    /// `hello.proof`: HMAC-SHA256(psk, "flashtex-nearby-v1 hello" || nonce), base64.
    /// Network.framework does not expose which table PSK a TLS session used, so
    /// the companion proves possession of the key for the `pair_id` it claims.
    static func helloProof(psk: Data, nonce: String) -> String {
        Data(HMAC<SHA256>.authenticationCode(for: helloProofInfo + Data(nonce.utf8), using: SymmetricKey(data: psk)))
            .base64EncodedString()
    }

    static func verifyHelloProof(_ proof: String, psk: Data, nonce: String) -> Bool {
        guard let given = Data(base64Encoded: proof) else { return false }
        return HMAC<SHA256>.isValidAuthenticationCode(given, authenticating: helloProofInfo + Data(nonce.utf8),
                                                      using: SymmetricKey(data: psk))
    }

    static func hex(_ data: Data) -> String { data.map { String(format: "%02x", $0) }.joined() }

    static func data(hex: String) -> Data? {
        guard hex.count % 2 == 0 else { return nil }
        var out = Data(capacity: hex.count / 2)
        var i = hex.startIndex
        while i < hex.endIndex {
            let j = hex.index(i, offsetBy: 2)
            guard let b = UInt8(hex[i..<j], radix: 16) else { return nil }
            out.append(b)
            i = j
        }
        return out
    }
}

/// One paired companion. `psk` is the long-term key (base64, 32 bytes).
struct PairRecord: Codable, Equatable, Identifiable {
    var pairId: String
    var psk: String
    var companionName: String
    var createdAt: Date
    var lastSeenAt: Date?
    var id: String { pairId }
    enum CodingKeys: String, CodingKey {
        case pairId = "pair_id", psk, companionName = "companion_name"
        case createdAt = "created_at", lastSeenAt = "last_seen_at"
    }
    var pskData: Data? { Data(base64Encoded: psk) }
}

/// Plain-file store for pairings: `~/Library/Application Support/FlashTeX/pairs.json`,
/// mode 0600, written atomically. Not the Keychain — see the proposal's
/// "Not provided" list; moving the keys there is a follow-up.
final class PairStore {
    struct File: Codable {
        var version: Int
        var salt: String
        var pairs: [PairRecord]
    }

    let url: URL
    private let lock = NSLock()
    private var file: File
    private(set) var loadError: String?

    static func defaultURL() -> URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Library/Application Support")
        return base.appendingPathComponent("FlashTeX/pairs.json")
    }

    init(url: URL) {
        self.url = url
        let dec = JSONDecoder()
        dec.dateDecodingStrategy = .iso8601
        if let data = try? Data(contentsOf: url) {
            if let f = try? dec.decode(File.self, from: data), f.version == 1, Pairing.data(hex: f.salt) != nil {
                file = f
            } else {
                // Keep a corrupt file rather than silently overwriting it.
                file = File(version: 1, salt: Pairing.hex(Pairing.generateSalt()), pairs: [])
                loadError = "\(url.path) is not a version-1 pair store; starting empty without overwriting it"
            }
        } else {
            file = File(version: 1, salt: Pairing.hex(Pairing.generateSalt()), pairs: [])
            do { try persist() } catch { loadError = "cannot create \(url.path): \(error.localizedDescription)" }
        }
    }

    var salt: Data { lock.withLock { Pairing.data(hex: file.salt) ?? Data() } }
    var pairs: [PairRecord] { lock.withLock { file.pairs } }
    func pair(id: String) -> PairRecord? { lock.withLock { file.pairs.first { $0.pairId == id } } }

    @discardableResult
    func upsert(_ record: PairRecord) -> Bool {
        lock.withLock {
            guard loadError == nil else { return false }
            file.pairs.removeAll { $0.pairId == record.pairId }
            file.pairs.append(record)
            return (try? persist()) != nil
        }
    }

    @discardableResult
    func remove(pairId: String) -> Bool {
        lock.withLock {
            file.pairs.removeAll { $0.pairId == pairId }
            return (try? persist()) != nil
        }
    }

    func touch(pairId: String) {
        lock.withLock {
            guard let i = file.pairs.firstIndex(where: { $0.pairId == pairId }) else { return }
            file.pairs[i].lastSeenAt = Date()
            _ = try? persist()
        }
    }

    /// Caller holds `lock` (or is the initializer).
    private func persist() throws {
        let fm = FileManager.default
        let dir = url.deletingLastPathComponent()
        try fm.createDirectory(at: dir, withIntermediateDirectories: true,
                               attributes: [.posixPermissions: 0o700])
        let enc = JSONEncoder()
        enc.outputFormatting = [.prettyPrinted, .sortedKeys]
        enc.dateEncodingStrategy = .iso8601
        let data = try enc.encode(file)
        try data.write(to: url, options: [.atomic])
        try fm.setAttributes([.posixPermissions: 0o600], ofItemAtPath: url.path)
    }
}
