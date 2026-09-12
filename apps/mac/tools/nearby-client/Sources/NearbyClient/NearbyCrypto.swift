import CryptoKit
import Foundation
import Network
import Security

/// Pairing-code derivation, hello proof and TLS-PSK parameters — the exact
/// counterpart of apps/mac/Sources/FlashTeXMac/Pairing.swift and
/// NearbyListener.tlsOptions/parameters (proposal §2–§4).
public enum NearbyCrypto {
    public static let pskInfo = Data("flashtex-nearby-v1 psk".utf8)
    public static let pairIDInfo = Data("flashtex-nearby-v1 pair-id".utf8)
    public static let fingerprintInfo = Data("flashtex-nearby-v1 mac-id".utf8)
    public static let helloProofInfo = Data("flashtex-nearby-v1 hello".utf8)
    public static let saltLength = 16
    public static let pskLength = 32
    /// TLS_PSK_WITH_AES_128_GCM_SHA256
    public static let cipherSuiteRawValue: UInt16 = 0x00A8

    public struct Derived: Equatable {
        public let pairId: String
        public let psk: Data
    }

    /// `psk_boot = HKDF-SHA256(code, salt, "flashtex-nearby-v1 psk", 32)`,
    /// `pair_id = hex(HKDF-SHA256(code, salt, "flashtex-nearby-v1 pair-id", 8))`.
    public static func derive(code: String, salt: Data) -> Derived {
        let ikm = SymmetricKey(data: Data(code.utf8))
        let psk = HKDF<SHA256>.deriveKey(inputKeyMaterial: ikm, salt: salt, info: pskInfo, outputByteCount: pskLength)
        let id = HKDF<SHA256>.deriveKey(inputKeyMaterial: ikm, salt: salt, info: pairIDInfo, outputByteCount: 8)
        return Derived(pairId: hex(id.withUnsafeBytes { Data($0) }), psk: psk.withUnsafeBytes { Data($0) })
    }

    /// TXT `fp` = first 16 hex chars of SHA-256("flashtex-nearby-v1 mac-id" ‖ salt).
    public static func fingerprint(salt: Data) -> String {
        var h = SHA256()
        h.update(data: fingerprintInfo)
        h.update(data: salt)
        return String(hex(Data(h.finalize())).prefix(16))
    }

    /// `hello.proof` = base64(HMAC-SHA256(psk, "flashtex-nearby-v1 hello" ‖ nonce)).
    public static func helloProof(psk: Data, nonce: String) -> String {
        Data(HMAC<SHA256>.authenticationCode(for: helloProofInfo + Data(nonce.utf8), using: SymmetricKey(data: psk)))
            .base64EncodedString()
    }

    public static func verifyHelloProof(_ proof: String, psk: Data, nonce: String) -> Bool {
        guard let given = Data(base64Encoded: proof) else { return false }
        return HMAC<SHA256>.isValidAuthenticationCode(given, authenticating: helloProofInfo + Data(nonce.utf8),
                                                      using: SymmetricKey(data: psk))
    }

    // MARK: TLS parameters (mirror of NearbyListener.clientParameters)

    /// TLS 1.2 only, suite 0x00A8 only, one PSK whose identity is `pair_id`,
    /// resumption and tickets off, TCP keepalive on / Nagle off, no AWDL.
    public static func parameters(pairId: String, psk: Data) -> NWParameters {
        let tls = NWProtocolTLS.Options()
        let sec = tls.securityProtocolOptions
        sec_protocol_options_set_min_tls_protocol_version(sec, .TLSv12)
        sec_protocol_options_set_max_tls_protocol_version(sec, .TLSv12)
        sec_protocol_options_append_tls_ciphersuite(sec, tls_ciphersuite_t(rawValue: cipherSuiteRawValue)!)
        sec_protocol_options_set_tls_resumption_enabled(sec, false)
        sec_protocol_options_set_tls_tickets_enabled(sec, false)
        sec_protocol_options_add_pre_shared_key(sec, dispatchData(psk), dispatchData(Data(pairId.utf8)))
        let tcp = NWProtocolTCP.Options()
        tcp.enableKeepalive = true
        tcp.noDelay = true
        let params = NWParameters(tls: tls, tcp: tcp)
        params.allowLocalEndpointReuse = true
        params.includePeerToPeer = false
        return params
    }

    static func dispatchData(_ data: Data) -> __DispatchData {
        data.withUnsafeBytes { DispatchData(bytes: $0) } as __DispatchData
    }

    /// Negotiated TLS version/suite of a ready connection; a client can assert
    /// `.TLSv12` / 0x00A8 like the listener does.
    public static func negotiated(_ connection: NWConnection) -> (tlsv12: Bool, suite: UInt16)? {
        guard let meta = connection.metadata(definition: NWProtocolTLS.definition) as? NWProtocolTLS.Metadata else { return nil }
        let sec = meta.securityProtocolMetadata
        return (sec_protocol_metadata_get_negotiated_tls_protocol_version(sec) == .TLSv12,
                sec_protocol_metadata_get_negotiated_tls_ciphersuite(sec).rawValue)
    }

    // MARK: hex

    public static func hex(_ data: Data) -> String { data.map { String(format: "%02x", $0) }.joined() }

    public static func data(hex: String) -> Data? {
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
