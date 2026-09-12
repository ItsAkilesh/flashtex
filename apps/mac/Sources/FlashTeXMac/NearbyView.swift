import SwiftUI

/// `Edit > Nearby Companion…` window: advertising, pairing code, paired
/// devices, and what arrived. Everything shown is the real listener state;
/// captures land in an in-memory inbox until the bridge client takes over.
struct NearbyView: View {
    @EnvironmentObject private var nearby: NearbyState
    @EnvironmentObject private var model: ShellModel

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            advertisingSection
            Divider()
            pairingSection
            Divider()
            pairedSection
            Divider()
            InboxSection(inbox: model.nearbyInbox, lastCaptureId: nearby.lastReceivedCaptureId)
            Divider()
            logSection
        }
        .padding(16)
        .frame(minWidth: 460, idealWidth: 500, minHeight: 520)
    }

    private var advertisingSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("Nearby companion").font(.headline)
                Spacer()
                Toggle("Advertise", isOn: Binding(
                    get: { nearby.isAdvertising },
                    set: { $0 ? nearby.startAdvertising() : nearby.stopAdvertising() }))
                .toggleStyle(.switch)
            }
            Text(nearby.status).font(.caption).foregroundStyle(.secondary)
            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 2) {
                GridRow { Text("Bonjour").foregroundStyle(.secondary); Text("\(nearby.serviceType)  “\(nearby.macName)”") }
                GridRow { Text("Port").foregroundStyle(.secondary); Text(nearby.port.map(String.init) ?? "—") }
                GridRow { Text("Mac id (fp)").foregroundStyle(.secondary); Text(nearby.fingerprint).font(.system(.body, design: .monospaced)) }
                GridRow { Text("TLS").foregroundStyle(.secondary); Text("1.2, \(nearby.cipherSuite), no resumption") }
            }
            .font(.callout)
            Text("Proposal nearby-v1 — not yet a published contract. Captures are kept in memory only (durable: false).")
                .font(.caption2).foregroundStyle(.secondary)
        }
    }

    private var pairingSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Pair a companion").font(.headline)
            if let code = nearby.pairingCode, let expires = nearby.codeExpiresAt {
                HStack(alignment: .firstTextBaseline, spacing: 16) {
                    Text(code.split(every: 3))
                        .font(.system(size: 34, weight: .semibold, design: .monospaced))
                        .textSelection(.enabled)
                    TimelineView(.periodic(from: .now, by: 1)) { ctx in
                        let left = max(0, Int(expires.timeIntervalSince(ctx.date).rounded(.up)))
                        Text("expires in \(left) s").font(.callout).foregroundStyle(left <= 15 ? .red : .secondary)
                    }
                    Spacer()
                    Button("Cancel") { nearby.cancelPairing() }
                }
                Text("Enter this code on the companion while “\(nearby.macName)” is selected. The code is valid for one pairing.")
                    .font(.caption).foregroundStyle(.secondary)
            } else {
                HStack {
                    Button("Show Pairing Code") { nearby.beginPairing() }
                    Text("A 6-digit code, valid for \(Int(Pairing.codeLifetime)) s; starts advertising if needed.")
                        .font(.caption).foregroundStyle(.secondary)
                }
            }
        }
    }

    private var pairedSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Paired companions").font(.headline)
            if nearby.pairs.isEmpty {
                Text("None yet.").font(.callout).foregroundStyle(.secondary)
            } else {
                ForEach(nearby.pairs) { pair in
                    HStack {
                        VStack(alignment: .leading, spacing: 1) {
                            HStack(spacing: 6) {
                                Text(pair.companionName).font(.callout)
                                if nearby.connectedPairIds.contains(pair.pairId) {
                                    Text("connected").font(.caption2).padding(.horizontal, 5).padding(.vertical, 1)
                                        .background(Color.green.opacity(0.2), in: Capsule())
                                }
                            }
                            Text("\(pair.pairId) · paired \(pair.createdAt.formatted(date: .abbreviated, time: .shortened))"
                                 + (pair.lastSeenAt.map { " · seen \($0.formatted(date: .abbreviated, time: .shortened))" } ?? ""))
                                .font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        Button("Forget") { nearby.forget(pairId: pair.pairId) }
                    }
                }
            }
            Text("Keys live in \(nearby.store.url.path) (mode 0600), not the Keychain.")
                .font(.caption2).foregroundStyle(.secondary)
        }
    }

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
        }
    }
}

private struct InboxSection: View {
    @ObservedObject var inbox: NearbyInbox
    var lastCaptureId: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Received captures").font(.headline)
            Text("Last capture id: \(lastCaptureId ?? inbox.lastCaptureId ?? "—")")
                .font(.system(.callout, design: .monospaced)).textSelection(.enabled)
            Text(inbox.lastNote ?? "\(inbox.received.count) capture(s) in memory; none forwarded to a bridge yet.")
                .font(.caption).foregroundStyle(.secondary)
        }
    }
}

private extension String {
    func split(every n: Int) -> String {
        stride(from: 0, to: count, by: n).map { i -> String in
            let s = index(startIndex, offsetBy: i)
            let e = index(s, offsetBy: n, limitedBy: endIndex) ?? endIndex
            return String(self[s..<e])
        }.joined(separator: " ")
    }
}
