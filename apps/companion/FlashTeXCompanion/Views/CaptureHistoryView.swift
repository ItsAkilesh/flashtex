import SwiftUI

/// Shows list of previous captures with thumbnails and payload previews.
struct CaptureHistoryView: View {
    @Environment(CaptureStore.self) private var store

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
                            Image(systemName: "checkmark.circle.fill")
                                .foregroundStyle(.green)
                                .font(.caption)
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
}
