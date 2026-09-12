import SwiftUI

/// Settings for capture destination and instruction defaults.
///
/// Rev 3: destination ID and base revision are now persisted to UserDefaults via
/// `CaptureStore`.  Changes take effect immediately on the next staged capture.
struct SettingsView: View {
    @Environment(CaptureStore.self) private var store

    var body: some View {
        @Bindable var store = store
        Form {
            Section {
                TextField("Destination ID", text: $store.currentDestinationID)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .accessibilityLabel("Destination ID")
                    .accessibilityHint("The anchor or document ID that captures are sent to; persisted between launches")

                Stepper("Base Revision: \(store.currentBaseRevision)",
                        value: $store.currentBaseRevision, in: 1...9999)
                    .accessibilityLabel("Base revision \(store.currentBaseRevision)")
                    .accessibilityHint("The document revision at the destination; persisted between launches")
            } header: {
                Text("Capture Target")
            } footer: {
                Text("Changes are saved automatically and restored on next launch.")
                    .font(.caption)
            }

            Section("Statistics") {
                LabeledContent("Total Captures", value: "\(store.captures.count)")
                LabeledContent("Pencil Draws") {
                    Text("\(store.captures.filter { $0.source == .pencil }.count)")
                }
                LabeledContent("Camera / Photo") {
                    Text("\(store.captures.filter { $0.source == .camera || $0.source == .photoLibrary }.count)")
                }
                LabeledContent("Sent over Network") {
                    Text("\(store.captures.filter { $0.networkSent }.count)")
                }
            }

            Section("Protocol") {
                LabeledContent("Version", value: "1")
                LabeledContent("Message Type", value: "capture_submit")
                LabeledContent("Image Format", value: "image/png")
            }
        }
        .navigationTitle("Settings")
    }
}
