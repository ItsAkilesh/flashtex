import SwiftUI

/// Shows the generated capture_submit JSON payload for inspection.
struct PayloadPreviewView: View {
    let json: String
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                Text(prettyJSON)
                    .font(.system(.caption, design: .monospaced))
                    .textSelection(.enabled)
                    .padding()
            }
            .navigationTitle("Capture Payload")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") { dismiss() }
                }
                ToolbarItem(placement: .topBarLeading) {
                    ShareLink(item: json) {
                        Label("Share", systemImage: "square.and.arrow.up")
                    }
                }
            }
        }
    }

    private var prettyJSON: String {
        guard let data = json.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data),
              let pretty = try? JSONSerialization.data(withJSONObject: obj, options: [.prettyPrinted, .sortedKeys]),
              let str = String(data: pretty, encoding: .utf8) else {
            return json
        }
        // Truncate base64 for display
        let lines = str.components(separatedBy: "\n")
        let truncated = lines.map { line in
            if line.contains("data_base64") && line.count > 120 {
                let prefix = String(line.prefix(80))
                return prefix + "...[truncated]"
            }
            return line
        }
        return truncated.joined(separator: "\n")
    }
}
