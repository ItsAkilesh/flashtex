import SwiftUI

/// Settings for capture destination and instruction defaults.
struct SettingsView: View {
    @Environment(CaptureStore.self) private var store

    var body: some View {
        @Bindable var store = store
        Form {
            Section("Capture Target") {
                TextField("Destination ID", text: $store.currentDestinationID)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                Stepper("Base Revision: \(store.currentBaseRevision)",
                        value: $store.currentBaseRevision, in: 1...1000)
            }

            Section("Statistics") {
                LabeledContent("Total Captures", value: "\(store.captures.count)")
                LabeledContent("Pencil Draws") {
                    Text("\(store.captures.filter { $0.source == .pencil }.count)")
                }
                LabeledContent("Camera/Photo") {
                    Text("\(store.captures.filter { $0.source == .camera || $0.source == .photoLibrary }.count)")
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
