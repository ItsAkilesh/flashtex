import SwiftUI

/// Shows list of previous captures with their payload previews.
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
                        HStack {
                            Image(systemName: iconName(for: capture.source))
                                .foregroundStyle(.blue)
                                .frame(width: 30)
                            VStack(alignment: .leading) {
                                Text(capture.source.rawValue)
                                    .font(.headline)
                                Text(capture.id)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                Text(capture.timestamp, style: .relative)
                                    .font(.caption2)
                                    .foregroundStyle(.tertiary)
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
}
