import SwiftUI

/// The "Capture conversion" section of Preferences (⌘,): which provider turns
/// an iPad/Nearby capture (drawing or photo) into LaTeX, its model, and its
/// API key (stored in the Keychain by ConversionCredential.swift, never in
/// UserDefaults). This is the editor's only model-backed feature; it is off
/// whenever the provider is "None" or no key is present.
/// The key field is cleared after saving; the status line only ever says
/// present/absent and where from.
struct ConversionPreferencesSection: View {
    @State private var provider = ConversionPreferences.shared.provider
    @State private var draftKey = ""
    @State private var status = ConversionCredential.status(for: ConversionPreferences.shared.provider)
    @State private var note: String?
    @State private var model = ""
    /// The picker's selection: a known id, or "custom" to type one.
    @State private var pickedModel = ""
    private static let customModel = "custom"
    static func pick(_ model: String, for provider: ConversionProvider) -> String {
        provider.selectableModels.contains(model) ? model : customModel
    }

    var body: some View {
        Section("Capture conversion") {
            Picker("Provider", selection: $provider) {
                ForEach(ConversionProvider.allCases) { Text($0.label).tag($0) }
            }
            .onChange(of: provider) { _, p in
                ConversionPreferences.shared.provider = p
                refresh()
            }
            .accessibilityLabel("Capture conversion provider")
            .accessibilityHint("Which service converts a captured drawing or photo to LaTeX through flashtex-bridge. None disables conversion; captures are still received and journaled. FLASHTEX_CONVERSION_PROVIDER in the environment overrides this.")
            Text(status.text)
                .font(.caption).foregroundStyle(.secondary)
                .accessibilityLabel("Conversion API key status")
            if provider.needsCredential {
                HStack {
                    SecureField("\(provider.displayName) API key", text: $draftKey, prompt: Text("API key"))
                        .textFieldStyle(.roundedBorder)
                        .accessibilityLabel("\(provider.displayName) API key")
                        .accessibilityHint("Stored in your login Keychain under \(provider.keychainService); never written to preferences or logs.")
                        .onSubmit(save)
                    Button("Save to Keychain", action: save)
                        .disabled(draftKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    Button("Remove", action: remove)
                        .disabled(status.resolution?.source != .keychain)
                        .accessibilityHint("Deletes the Keychain item; an environment variable, if any, stays in effect.")
                }
                Picker("Model", selection: $pickedModel) {
                    ForEach(provider.selectableModels, id: \.self) { id in
                        Text(id == provider.defaultModel ? "\(id) (default)" : id).tag(id)
                    }
                    Text("Other…").tag(Self.customModel)
                }
                .onChange(of: pickedModel) { _, picked in
                    guard picked != Self.customModel, picked != model else { return }
                    model = picked
                    saveModel()
                }
                .accessibilityLabel("Conversion model")
                if pickedModel == Self.customModel {
                    HStack {
                        TextField("Model id", text: $model)
                            .textFieldStyle(.roundedBorder)
                            .accessibilityLabel("Conversion model id")
                            .onSubmit(saveModel)
                        Button("Use", action: saveModel)
                            .disabled(!ConversionCredential.isValidModel(model))
                    }
                }
            }
            if let note {
                Text(note).font(.caption).foregroundStyle(.secondary).textSelection(.enabled)
                    .accessibilityLabel("Conversion status note")
            }
            Text("Conversion runs in flashtex-bridge with the key only in that process's environment; the app itself never contacts a provider. Re-attach the bridge (Edit > Attach Capture Bridge) after changing the provider or key. Every converted proposal is reviewed before anything is inserted.")
                .font(.caption2).foregroundStyle(.tertiary)
        }
        .onAppear { refresh() }
    }

    private func refresh() {
        provider = ConversionPreferences.shared.provider
        status = ConversionCredential.status(for: provider)
        model = ConversionCredential.model(for: provider, environment: [:])
        pickedModel = Self.pick(model, for: provider)
    }

    private func save() {
        switch ConversionCredential.store(draftKey, for: provider) {
        case .success:
            draftKey = ""
            note = "key saved to the Keychain (\(provider.keychainService))"
        case .failure(let e):
            note = e.text
        }
        status = ConversionCredential.status(for: provider)
        NotificationCenter.default.post(name: ConversionPreferences.didChange, object: nil)
    }

    private func remove() {
        switch ConversionCredential.remove(for: provider) {
        case .success: note = "Keychain item removed"
        case .failure(let e): note = e.text
        }
        status = ConversionCredential.status(for: provider)
        NotificationCenter.default.post(name: ConversionPreferences.didChange, object: nil)
    }

    private func saveModel() {
        guard ConversionCredential.isValidModel(model) else { note = "model id must be ASCII letters, digits, - . _ (≤128)"; return }
        ConversionPreferences.shared.setModel(model, for: provider)
        model = ConversionCredential.model(for: provider, environment: [:])
        pickedModel = Self.pick(model, for: provider)
        note = "model: \(model)"
    }
}
