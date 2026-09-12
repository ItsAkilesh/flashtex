import SwiftUI

/// Shows list of previous captures with thumbnails and payload previews.
struct CaptureHistoryView: View {
    @Environment(CaptureStore.self) private var store
    private let bonjour = BonjourTransport.shared

    var body: some View {
        List {
            if store.captures.isEmpty {
                ContentUnavailableView(
                    "No Captures Yet",
                    systemImage: "doc.text.magnifyingglass",
                    description: Text("Draw with Apple Pencil or take a photo to create a capture.")
                )
            } else {
                ForEach(store.captures) { capture in
                    NavigationLink {
                        PayloadPreviewView(json: capture.payloadJSON)
                    } label: {
                        HStack(spacing: 12) {
                            if let thumb = capture.imageThumbnail {
                                Image(uiImage: thumb)
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                                    .frame(width: 50, height: 50)
                                    .clipShape(RoundedRectangle(cornerRadius: 8))
                            } else {
                                Image(systemName: iconName(for: capture.source))
                                    .foregroundStyle(.blue)
                                    .frame(width: 50, height: 50)
                            }
                            VStack(alignment: .leading, spacing: 2) {
                                Text(capture.source.rawValue)
                                    .font(.headline)
                                Text(capture.id)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .lineLimit(1)
                                Text(capture.timestamp, style: .relative)
                                    .font(.caption2)
                                    .foregroundStyle(.tertiary)
                            }
                            Spacer()
                            VStack(alignment: .trailing, spacing: 2) {
                                let receiptConfirmed = bonjour.receivedAcks.contains(capture.id)
                                Image(systemName: deliveryIcon(for: capture, receiptConfirmed: receiptConfirmed))
                                    .foregroundStyle(deliveryColor(for: capture, receiptConfirmed: receiptConfirmed))
                                    .font(.caption)
                                Text(deliveryLabel(for: capture, receiptConfirmed: receiptConfirmed))
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                            .accessibilityElement(children: .combine)
                            .accessibilityLabel(deliveryLabel(
                                for: capture,
                                receiptConfirmed: bonjour.receivedAcks.contains(capture.id)
                            ))
                        }
                    }
                    .contextMenu {
                        if !capture.networkSent {
                            Button {
                                store.retryNetworkDelivery(captureID: capture.id)
                            } label: {
                                Label("Retry delivery", systemImage: "arrow.clockwise")
                            }
                        }
                    }
                }
            }
        }
        .navigationTitle("History")
    }

    private func iconName(for source: CaptureStore.CaptureRecord.CaptureSource) -> String {
        switch source {
        case .pencil: return "pencil.tip"
        case .camera: return "camera"
        case .photoLibrary: return "photo"
        }
    }

    private func deliveryIcon(for capture: CaptureStore.CaptureRecord, receiptConfirmed: Bool) -> String {
        if !capture.networkSent { return "terminal" }
        return receiptConfirmed ? "checkmark.circle.fill" : "arrow.up.circle"
    }

    private func deliveryColor(for capture: CaptureStore.CaptureRecord, receiptConfirmed: Bool) -> Color {
        if !capture.networkSent { return .secondary }
        return receiptConfirmed ? .green : .orange
    }

    private func deliveryLabel(for capture: CaptureStore.CaptureRecord, receiptConfirmed: Bool) -> String {
        if !capture.networkSent { return "Saved locally" }
        return receiptConfirmed ? "Mac received" : "Awaiting receipt"
    }
}
