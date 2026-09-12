import SwiftUI

/// The "Grok (xAI)" section of Preferences (⌘,): the API key (stored in the
/// Keychain by GrokCredential.swift, never in UserDefaults), the provider
/// toggle, the model id and "Test connection" (GrokProbe: HTTP class only).
/// The key field is cleared after saving; the status line only ever says
/// present/absent and where from.
struct GrokPreferencesSection: View {
    @State private var draftKey = ""
    @State private var status = GrokCredential.status()
    @State private var note: String?
    @State private var probing = false
    @State private var providerEnabled = GrokPreferences.shared.providerEnabled
    @State private var model = GrokPreferences.shared.model ?? GrokCredential.defaultModel

    var body: some View {
        Section("Grok (xAI)") {
            Text(status.text)
                .font(.caption).foregroundStyle(.secondary)
                .accessibilityLabel("xAI API key status")
            HStack {
                SecureField("xAI API key", text: $draftKey, prompt: Text("xai-…"))
                    .textFieldStyle(.roundedBorder)
                    .accessibilityLabel("xAI API key")
                    .accessibilityHint("Stored in your login Keychain under tech.jay3332.flashtex.xai; never written to preferences or logs.")
                    .onSubmit(save)
                Button("Save to Keychain", action: save)
                    .disabled(draftKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                Button("Remove", action: remove)
                    .disabled(status.resolution?.source != .keychain)
                    .accessibilityHint("Deletes the Keychain item; an environment variable, if any, stays in effect.")
            }
            Toggle("Use Grok (xAI) for editor assistance", isOn: $providerEnabled)
                .onChange(of: providerEnabled) { _, on in GrokPreferences.shared.providerEnabled = on }
                .accessibilityHint("Explanations and reviewed edits in the capture review sheet are sent to xAI through the assistant helper. Off: the local provider command or none. FLASHTEX_ASSISTANT_PROVIDER in the environment overrides this.")
            HStack {
                TextField("Model", text: $model)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityLabel("Grok model id")
                    .onSubmit(saveModel)
                Button("Use", action: saveModel)
                    .disabled(!GrokCredential.isValidModel(model))
            }
            HStack {
                Button(probing ? "Testing…" : "Test connection", action: probe)
                    .disabled(probing || status.resolution == nil)
                    .accessibilityHint("One GET request to api.x.ai/v1/models with the key; the result is reported as an HTTP class only.")
                if let note {
                    Text(note).font(.caption).foregroundStyle(.secondary).textSelection(.enabled)
                        .accessibilityLabel("Grok status note")
                }
            }
            Text("Capture-to-LaTeX uses the key through flashtex-bridge (--enable-grok, XAI_API_KEY); editor assistance through flashtex-assistant-context --provider-session (FLASHTEX_GROK_API_KEY). Re-attach the bridge (Edit > Attach Capture Bridge) after changing the key.")
                .font(.caption2).foregroundStyle(.tertiary)
        }
        .onAppear { refresh() }
    }

    private func refresh() {
        status = GrokCredential.status()
        providerEnabled = GrokPreferences.shared.providerEnabled
        model = GrokPreferences.shared.model ?? GrokCredential.defaultModel
    }

    private func save() {
        switch GrokCredential.store(draftKey) {
        case .success:
            draftKey = ""
            note = "key saved to the Keychain (\(GrokCredential.keychainService))"
        case .failure(let e):
            note = e.text
        }
        status = GrokCredential.status()
    }

    private func remove() {
        switch GrokCredential.remove() {
        case .success: note = "Keychain item removed"
        case .failure(let e): note = e.text
        }
        status = GrokCredential.status()
    }

    private func saveModel() {
        guard GrokCredential.isValidModel(model) else { note = "model id must be ASCII letters, digits, - . _ (≤128)"; return }
        GrokPreferences.shared.model = model
        model = GrokPreferences.shared.model ?? GrokCredential.defaultModel
        note = "model: \(GrokCredential.model())"
    }

    private func probe() {
        guard let credential = GrokCredential.resolve() else { note = "no key to test"; return }
        probing = true
        note = "contacting \(GrokProbe.baseURL().host ?? "xAI")…"
        GrokProbe.probe(credential: credential) { outcome in
            probing = false
            note = outcome.text
        }
    }
}
