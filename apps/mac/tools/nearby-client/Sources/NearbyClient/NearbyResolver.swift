import dnssd
import Foundation
import Network

/// Resolves a Bonjour service endpoint to `host:port` with `DNSServiceResolve`.
///
/// Why: an `NWConnection` opened directly on an `NWBrowser` result
/// (`.service(name:type:domain:)`) never leaves `.preparing` when the TLS-PSK
/// handshake is refused — Network.framework treats the failed candidate as
/// "wait for a better path" and neither `.waiting` nor `.failed` is delivered
/// (measured on macOS 26: >60 s, one server-side attempt). Through a
/// `.hostPort` endpoint the same refusal is reported at once as
/// `.waiting(-9820: bad MAC)`. So connections resolve first, then dial host:port.
public enum NearbyResolver {
    public struct Resolved: Equatable {
        public let host: String
        public let port: UInt16
        public var endpoint: NWEndpoint { .hostPort(host: NWEndpoint.Host(host), port: NWEndpoint.Port(rawValue: port)!) }
    }

    public static func resolve(_ endpoint: NWEndpoint, timeout: TimeInterval = 5) async throws -> Resolved {
        guard case .service(let name, let type, let domain, _) = endpoint else {
            throw NearbyError.invalidInput("not a Bonjour service endpoint: \(endpoint)")
        }
        return try await resolve(name: name, type: type, domain: domain, timeout: timeout)
    }

    public static func resolve(name: String, type: String, domain: String, timeout: TimeInterval = 5) async throws -> Resolved {
        let box = Box()
        return try await withCheckedThrowingContinuation { cont in
            box.finish = { cont.resume(with: $0) }
            var ref: DNSServiceRef?
            let ctx = Unmanaged.passRetained(box).toOpaque()
            let err = DNSServiceResolve(&ref, 0, UInt32(kDNSServiceInterfaceIndexAny), name, type, domain, { _, _, _, errorCode, _, hosttarget, port, _, _, context in
                guard let context else { return }
                let box = Unmanaged<Box>.fromOpaque(context).takeUnretainedValue()
                guard errorCode == kDNSServiceErr_NoError, let hosttarget else {
                    box.done(.failure(NearbyError.browseFailed("DNSServiceResolve error \(errorCode)"))); return
                }
                var host = String(cString: hosttarget)
                if host.hasSuffix(".") { host.removeLast() }
                box.done(.success(Resolved(host: host, port: UInt16(bigEndian: port))))
            }, ctx)
            guard err == kDNSServiceErr_NoError, let ref else {
                Unmanaged<Box>.fromOpaque(ctx).release()
                cont.resume(throwing: NearbyError.browseFailed("DNSServiceResolve failed to start: \(err)"))
                return
            }
            box.ref = ref
            box.release = { Unmanaged<Box>.fromOpaque(ctx).release() }
            DNSServiceSetDispatchQueue(ref, box.queue)
            box.queue.asyncAfter(deadline: .now() + timeout) {
                box.done(.failure(NearbyError.timeout("resolving \(name).\(type)\(domain) took more than \(timeout)s")))
            }
        }
    }

    private final class Box: @unchecked Sendable {
        let queue = DispatchQueue(label: "flashtex.nearby.resolve")
        var ref: DNSServiceRef?
        var finish: ((Result<Resolved, Error>) -> Void)?
        var release: (() -> Void)?
        private var completed = false
        /// First result wins; later callbacks (other interfaces) are ignored.
        func done(_ r: Result<Resolved, Error>) {
            queue.async {
                guard !self.completed else { return }
                self.completed = true
                if let ref = self.ref { DNSServiceRefDeallocate(ref); self.ref = nil }
                self.finish?(r)
                self.finish = nil
                self.release?()
                self.release = nil
            }
        }
    }
}
