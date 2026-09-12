import Foundation
import Security

/// The Mac's xAI (Grok) credential adapter. The Rust helpers never read the
/// Keychain: `flashtex-assistant-context --provider-session` expects
/// `FLASHTEX_GROK_API_KEY` and `flashtex-bridge --enable-grok` expects
/// `XAI_API_KEY` in the child's environment, supplied by this adapter.
///
/// Source order: the Keychain generic-password item (service
/// `tech.jay3332.flashtex.xai`, account `xai`, or `XAI_API_KEY`), then the environment
/// (`XAI_API_KEY`, then `FLASHTEX_GROK_API_KEY`), else none.
/// `FLASHTEX_KEYCHAIN_OFF=1` skips the Keychain (environment-only mode, used by
/// tests and headless runs). The key is never logged, never placed in argv or
/// JSON, and never shown: every text this type produces says only
/// "present (source)" or "absent".
///
/// To supply a key without the Preferences window:
/// `security add-generic-password -U -s tech.jay3332.flashtex.xai -a xai -w '<key>'`
/// (`-U` updates an existing item); remove it with
/// `security delete-generic-password -s tech.jay3332.flashtex.xai -a xai`.
enum GrokCredential {
    static let keychainService = "tech.jay3332.flashtex.xai"
    /// The account written by Preferences and read first; `legacyKeychainAccount`
    /// is also read so an item stored under the environment-variable name works.
    static let keychainAccount = "xai"
    static let legacyKeychainAccount = "XAI_API_KEY"
    static var keychainAccounts: [String] { [keychainAccount, legacyKeychainAccount] }
    /// Environment names consulted after the Keychain, in this order.
    static let environmentNames = ["XAI_API_KEY", "FLASHTEX_GROK_API_KEY"]
    static let keychainOffVariable = "FLASHTEX_KEYCHAIN_OFF"
    /// The helper's own variable (`--provider-session` reads only this one).
    static let helperVariable = "FLASHTEX_GROK_API_KEY"
    /// The bridge's variable (`--enable-grok` reads only this one).
    static let bridgeVariable = "XAI_API_KEY"
    static let modelVariable = "FLASHTEX_GROK_MODEL"
    /// Default explanation/review model. Live evidence 2026-09-12: `grok-4.6`
    /// (reasoning) takes 55–70 s per explanation (grok-live-20260912T191209Z)
    /// but its edits pass the helper's source-bound gate; the fast
    /// non-reasoning model answers in ~4–5 s yet its proposed edits failed
    /// that gate 2/2 ("removed source differs": wrong byte offsets/removed
    /// text, grok-live-20260912T210600Z), so it is not the explanation
    /// default until the helper's prompt makes the fast model comply. Both are
    /// selectable in Preferences (`selectableModels`) and by `FLASHTEX_GROK_MODEL`.
    static let defaultModel = reasoningModel
    static let reasoningModel = "grok-4.6"
    /// The fast non-reasoning model: the capture default (`defaultCaptureModel`).
    static let fastModel = "grok-4.20-0309-non-reasoning"
    /// Ids the Preferences picker offers (any valid id can still be typed).
    static let selectableModels = [reasoningModel, fastModel]
    /// Provider-call bound for the review sheet: reasoning models were
    /// measured at 53–70 s and the helper's HTTP client gives up at 90 s, so
    /// the flight must outlive it; non-reasoning models answer in seconds.
    static func providerTimeout(for model: String) -> TimeInterval { isReasoningModel(model) ? 100 : 30 }
    static func isReasoningModel(_ model: String) -> Bool { !model.lowercased().contains("non-reasoning") }
    /// The helper's own limit (`GrokClient::new`).
    static let maxKeyBytes = 8192

    enum Source: String, Equatable {
        case keychain = "Keychain"
        case environment = "environment"
    }

    /// A resolved key. `description` (and every status text derived from it)
    /// reports presence and source only; the key itself is only ever handed
    /// to a child process environment by `inject(into:as:)`.
    struct Resolution: Equatable, CustomStringConvertible, CustomDebugStringConvertible {
        private let key: String
        let source: Source
        /// Which environment variable supplied it (nil for the Keychain).
        let variable: String?

        fileprivate init(key: String, source: Source, variable: String?) {
            self.key = key; self.source = source; self.variable = variable
        }

        var description: String { "key present (\(source.rawValue)\(variable.map { ": \($0)" } ?? ""))" }
        var debugDescription: String { description }
        /// Introspection for tests: the key's byte count, never its bytes.
        var keyByteCount: Int { key.utf8.count }

        /// Adds the key to `environment` under `name` (one of the two ways the key leaves this type).
        func inject(into environment: inout [String: String], as name: String) {
            environment[name] = key
        }

        /// Sets `Authorization: Bearer <key>` on `request` (the other way; the probe only).
        func applyBearer(to request: inout URLRequest) {
            request.setValue("Bearer " + key, forHTTPHeaderField: "Authorization")
        }
    }

    /// Rejects what the helper would reject (`GrokClient::new`), so a bad key
    /// fails here with a clear message rather than as a child exit.
    static func validate(_ raw: String) -> Result<String, ValidationError> {
        let key = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        if key.isEmpty { return .failure(.empty) }
        if key.utf8.count > maxKeyBytes { return .failure(.tooLong) }
        if key.unicodeScalars.contains(where: { $0.value < 0x20 || $0.value == 0x7f }) { return .failure(.controlCharacters) }
        return .success(key)
    }

    enum ValidationError: Error, Equatable {
        case empty, tooLong, controlCharacters
        var text: String {
            switch self {
            case .empty: return "the key is empty"
            case .tooLong: return "the key exceeds \(GrokCredential.maxKeyBytes) bytes"
            case .controlCharacters: return "the key contains control characters"
            }
        }
    }

    /// Model id for both helpers: `FLASHTEX_GROK_MODEL`, else the preference,
    /// else `defaultModel`. Invalid values (the helper's own rule: ASCII
    /// alphanumerics plus `-._`, at most 128 bytes) fall back to the default.
    static func model(environment: [String: String] = ProcessInfo.processInfo.environment,
                      preferences: GrokPreferences = .shared) -> String {
        for candidate in [environment[modelVariable], preferences.model] {
            if let candidate, isValidModel(candidate) { return candidate }
        }
        return defaultModel
    }

    /// Model for capture-to-LaTeX (the bridge's vision request):
    /// `FLASHTEX_GROK_CAPTURE_MODEL`, else the same as `model(...)`. Live runs
    /// showed the reasoning model exceeding the bridge's fixed 90 s timeout on
    /// an image while a non-reasoning model answered in ~3 s.
    static let captureModelVariable = "FLASHTEX_GROK_CAPTURE_MODEL"
    /// Live evidence 2026-09-12 (docs/evidence/grok-live-20260912T191209Z):
    /// this id converted a 900×260 handwriting PNG in 2.8 s and the result
    /// compiled with zero diagnostics; `grok-4.6` hit the bridge's 90 s timeout.
    static let defaultCaptureModel = fastModel
    static func captureModel(environment: [String: String] = ProcessInfo.processInfo.environment,
                             preferences: GrokPreferences = .shared) -> String {
        for candidate in [environment[captureModelVariable], environment[modelVariable], preferences.model] {
            if let candidate, isValidModel(candidate) { return candidate }
        }
        return defaultCaptureModel
    }

    static func isValidModel(_ model: String) -> Bool {
        !model.isEmpty && model.utf8.count <= 128
            && model.utf8.allSatisfy { ($0 >= 0x30 && $0 <= 0x39) || ($0 >= 0x41 && $0 <= 0x5a) || ($0 >= 0x61 && $0 <= 0x7a) || $0 == 0x2d || $0 == 0x2e || $0 == 0x5f }
    }

    // MARK: resolution

    /// Keychain (unless `FLASHTEX_KEYCHAIN_OFF=1`), then the environment names
    /// in order. A Keychain error other than "no item" is reported through
    /// `keychainProblem` on the returned status, never thrown at callers.
    static func resolve(environment: [String: String] = ProcessInfo.processInfo.environment,
                        keychain: any GrokKeychainStore = SecItemKeychain.shared) -> Resolution? {
        status(environment: environment, keychain: keychain).resolution
    }

    /// What the Preferences window and the review sheet show.
    struct Status: Equatable {
        var resolution: Resolution?
        var keychainSkipped: Bool
        /// A Keychain read failure (OSStatus text), when one occurred.
        var keychainProblem: String?

        var text: String {
            if let resolution { return "xAI API key: \(resolution.description)" }
            var where_ = keychainSkipped ? "Keychain skipped (\(GrokCredential.keychainOffVariable)=1)" : "no Keychain item \(GrokCredential.keychainService)"
            if let keychainProblem { where_ += " (\(keychainProblem))" }
            return "xAI API key: absent — \(where_); \(GrokCredential.environmentNames.joined(separator: " / ")) unset"
        }
    }

    static func status(environment: [String: String] = ProcessInfo.processInfo.environment,
                       keychain: any GrokKeychainStore = SecItemKeychain.shared) -> Status {
        let skipped = environment[keychainOffVariable] == "1"
        var problem: String?
        if !skipped {
            accounts: for account in keychainAccounts {
                switch keychain.read(service: keychainService, account: account) {
                case .success(let stored?):
                    if case .success(let key) = validate(stored) {
                        return Status(resolution: Resolution(key: key, source: .keychain, variable: nil), keychainSkipped: false, keychainProblem: nil)
                    }
                    problem = "stored item (account \(account)) is not a usable key"
                case .success(nil): continue accounts
                case .failure(let why): problem = why.text; break accounts
                }
            }
        }
        for name in environmentNames {
            if let raw = environment[name], case .success(let key) = validate(raw) {
                return Status(resolution: Resolution(key: key, source: .environment, variable: name), keychainSkipped: skipped, keychainProblem: problem)
            }
        }
        return Status(resolution: nil, keychainSkipped: skipped, keychainProblem: problem)
    }

    // MARK: storing (Preferences)

    /// Stores (or replaces) the Keychain item. The Preferences window is the
    /// only caller; the key comes straight from its secure field.
    static func store(_ raw: String, keychain: any GrokKeychainStore = SecItemKeychain.shared) -> Result<Void, StoreError> {
        switch validate(raw) {
        case .failure(let e): return .failure(.invalid(e))
        case .success(let key):
            return keychain.write(key, service: keychainService, account: keychainAccount).mapError { .keychain($0) }
        }
    }

    /// Deletes both accounts (`xai` and the legacy `XAI_API_KEY`).
    static func remove(keychain: any GrokKeychainStore = SecItemKeychain.shared) -> Result<Void, StoreError> {
        for account in keychainAccounts {
            if case .failure(let e) = keychain.delete(service: keychainService, account: account) { return .failure(.keychain(e)) }
        }
        return .success(())
    }

    enum StoreError: Error, Equatable {
        case invalid(ValidationError)
        case keychain(KeychainError)
        var text: String {
            switch self {
            case .invalid(let e): return "not stored: \(e.text)"
            case .keychain(let e): return "Keychain: \(e.text)"
            }
        }
    }

    // MARK: child environments

    /// Environment for `flashtex-bridge`: the app's environment with every
    /// credential-, token- or proxy-like variable removed, plus `XAI_API_KEY`
    /// (and `FLASHTEX_GROK_MODEL`) only when a key resolved. Without a key the
    /// bridge runs exactly as before this adapter existed, minus stray secrets.
    static func bridgeEnvironment(credential: Resolution?, model: String,
                                  from environment: [String: String] = ProcessInfo.processInfo.environment) -> [String: String] {
        var env = environment.filter { !ProposalPreview.ExplanationConfiguration.isSensitiveVariable($0.key) }
        env.removeValue(forKey: modelVariable)
        if let credential {
            credential.inject(into: &env, as: bridgeVariable)
            env[modelVariable] = model
        }
        return env
    }
}

// MARK: - Keychain

enum KeychainError: Error, Equatable {
    case status(OSStatus)
    var text: String {
        if case .status(let s) = self {
            let message = SecCopyErrorMessageString(s, nil).map { $0 as String } ?? "OSStatus \(s)"
            return "\(message) (\(s))"
        }
        return "unknown"
    }
}

/// Generic-password storage; `SecItemKeychain` is the real one, tests may use
/// `MemoryKeychain`. `read` returns `.success(nil)` when no item exists.
protocol GrokKeychainStore {
    func read(service: String, account: String) -> Result<String?, KeychainError>
    func write(_ secret: String, service: String, account: String) -> Result<Void, KeychainError>
    func delete(service: String, account: String) -> Result<Void, KeychainError>
}

/// `SecItem*` against the login keychain (the legacy file-based keychain; the
/// data-protection keychain needs an entitled, signed app). No third-party code.
/// After an ad-hoc re-sign of a rebuilt app, macOS may ask once whether the new
/// build may read the item; that prompt is the system's, not this app's.
struct SecItemKeychain: GrokKeychainStore {
    static let shared = SecItemKeychain()

    private func query(service: String, account: String) -> [String: Any] {
        [kSecClass as String: kSecClassGenericPassword,
         kSecAttrService as String: service,
         kSecAttrAccount as String: account]
    }

    func read(service: String, account: String) -> Result<String?, KeychainError> {
        var q = query(service: service, account: account)
        q[kSecReturnData as String] = true
        q[kSecMatchLimit as String] = kSecMatchLimitOne
        var item: CFTypeRef?
        let status = SecItemCopyMatching(q as CFDictionary, &item)
        switch status {
        case errSecSuccess:
            guard let data = item as? Data else { return .success(nil) }
            return .success(String(decoding: data, as: UTF8.self))
        case errSecItemNotFound: return .success(nil)
        default: return .failure(.status(status))
        }
    }

    func write(_ secret: String, service: String, account: String) -> Result<Void, KeychainError> {
        let data = Data(secret.utf8)
        let q = query(service: service, account: account)
        let update = SecItemUpdate(q as CFDictionary, [kSecValueData as String: data] as CFDictionary)
        if update == errSecSuccess { return .success(()) }
        guard update == errSecItemNotFound else { return .failure(.status(update)) }
        var add = q
        add[kSecValueData as String] = data
        add[kSecAttrLabel as String] = "FlashTeX xAI API key"
        let status = SecItemAdd(add as CFDictionary, nil)
        return status == errSecSuccess ? .success(()) : .failure(.status(status))
    }

    func delete(service: String, account: String) -> Result<Void, KeychainError> {
        let status = SecItemDelete(query(service: service, account: account) as CFDictionary)
        return status == errSecSuccess || status == errSecItemNotFound ? .success(()) : .failure(.status(status))
    }
}

/// In-memory store for tests (no Keychain access, no prompts).
final class MemoryKeychain: GrokKeychainStore {
    private var items: [String: String] = [:]
    private let lock = NSLock()
    var failWith: KeychainError?
    init(_ initial: [String: String] = [:]) { items = initial }
    private func k(_ s: String, _ a: String) -> String { s + "\u{0}" + a }
    func read(service: String, account: String) -> Result<String?, KeychainError> {
        if let failWith { return .failure(failWith) }
        return .success(lock.withLock { items[k(service, account)] })
    }
    func write(_ secret: String, service: String, account: String) -> Result<Void, KeychainError> {
        if let failWith { return .failure(failWith) }
        lock.withLock { items[k(service, account)] = secret }
        return .success(())
    }
    func delete(service: String, account: String) -> Result<Void, KeychainError> {
        if let failWith { return .failure(failWith) }
        lock.withLock { items.removeValue(forKey: k(service, account)) }
        return .success(())
    }
    var isEmpty: Bool { lock.withLock { items.isEmpty } }
}

// MARK: - Preferences (provider selection, model)

/// The two non-secret Grok settings, in `UserDefaults` under their own keys
/// (separate from `EditorPreferences` so the editor schema is untouched).
/// `FLASHTEX_ASSISTANT_PROVIDER=grok` / `FLASHTEX_GROK_MODEL` in the
/// environment take precedence over these at configuration time.
final class GrokPreferences {
    static let shared = GrokPreferences(defaults: .standard)
    /// Pre-mode boolean (read for migration while no mode is stored: an
    /// explicit `true` is `.on`, an explicit `false` stays `.off` — a user who
    /// opted out is never switched to live xAI by a key appearing).
    static let providerKey = "FlashTeX.Grok.v1.providerEnabled"
    static let providerModeKey = "FlashTeX.Grok.v1.providerMode"
    static let modelKey = "FlashTeX.Grok.v1.model"
    /// Posted (main queue) after any setter so the status bar re-reads.
    static let didChange = Notification.Name("FlashTeX.Grok.preferencesDidChange")
    private let defaults: UserDefaults

    init(defaults: UserDefaults) { self.defaults = defaults }

    /// "Use Grok (xAI) for editor assistance": `auto` (default) selects Grok
    /// whenever a key resolves and falls back to the local provider (or none)
    /// otherwise; `on` selects Grok even without a key (the sheet then says so
    /// and sends nothing); `off` never selects it.
    enum ProviderMode: String, CaseIterable, Equatable {
        case auto, on, off
        var label: String {
            switch self {
            case .auto: return "Automatic (when a key is present)"
            case .on: return "Always"
            case .off: return "Never"
            }
        }
    }

    var providerMode: ProviderMode {
        get {
            if let raw = defaults.string(forKey: Self.providerModeKey), let mode = ProviderMode(rawValue: raw) { return mode }
            if defaults.object(forKey: Self.providerKey) != nil { return defaults.bool(forKey: Self.providerKey) ? .on : .off }
            return .auto
        }
        set {
            defaults.set(newValue.rawValue, forKey: Self.providerModeKey)
            defaults.removeObject(forKey: Self.providerKey)
            notify()
        }
    }

    /// Compatibility view of `providerMode`: true only for `.on`.
    var providerEnabled: Bool {
        get { providerMode == .on }
        set { providerMode = newValue ? .on : .off }
    }

    private func notify() {
        NotificationCenter.default.post(name: Self.didChange, object: self)
    }

    /// nil means "use the default model".
    var model: String? {
        get { defaults.string(forKey: Self.modelKey).flatMap { GrokCredential.isValidModel($0) ? $0 : nil } }
        set {
            if let newValue, GrokCredential.isValidModel(newValue), newValue != GrokCredential.defaultModel {
                defaults.set(newValue, forKey: Self.modelKey)
            } else {
                defaults.removeObject(forKey: Self.modelKey)
            }
            notify()
        }
    }
}
