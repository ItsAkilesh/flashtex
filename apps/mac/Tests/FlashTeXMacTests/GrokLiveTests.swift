import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Grok/xAI wiring on the Mac side (GrokCredential.swift, GrokProvider.swift,
/// the ProposalPreview provider stage and the bridge launch). Nothing here
/// contacts xAI: the credential adapter is exercised against an in-memory
/// store (plus one real Keychain round trip under a throwaway service name),
/// the provider session against `Fixtures/fake_grok_session.py`, the
/// connection probe against `Fixtures/fake_xai_http.py` on loopback, and the
/// bridge launch against `Fixtures/fake_bridge.py`. The real helper, when
/// built with `--features grok`, is only started and cancelled (no admit).
@MainActor
final class GrokLiveTests: XCTestCase {
    static let fixtures = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("Fixtures")
    static let fakeSession = fixtures.appendingPathComponent("fake_grok_session.py")
    static let fakeHelper = fixtures.appendingPathComponent("fake_assistant_context.py")
    static let fakeXAI = fixtures.appendingPathComponent("fake_xai_http.py")
    static let envOnly = ["FLASHTEX_KEYCHAIN_OFF": "1"]

    private func temporaryDefaults() -> UserDefaults {
        let name = "flashtex.grok.tests.\(UUID().uuidString)"
        let d = UserDefaults(suiteName: name)!
        d.removePersistentDomain(forName: name)
        return d
    }

    // MARK: credential adapter

    func testResolutionOrderIsKeychainThenEnvironmentThenNone() throws {
        let keychain = MemoryKeychain([GrokCredential.keychainService + "\u{0}" + GrokCredential.keychainAccount: " kc-key \n"])
        let env = ["XAI_API_KEY": "env-key", "FLASHTEX_GROK_API_KEY": "helper-key", "PATH": "/usr/bin"]
        let fromKeychain = try XCTUnwrap(GrokCredential.resolve(environment: env, keychain: keychain))
        XCTAssertEqual(fromKeychain.source, .keychain)
        XCTAssertNil(fromKeychain.variable)
        XCTAssertEqual(fromKeychain.keyByteCount, 6, "trimmed")
        var out: [String: String] = [:]
        fromKeychain.inject(into: &out, as: "K")
        XCTAssertEqual(out, ["K": "kc-key"])

        let fromEnv = try XCTUnwrap(GrokCredential.resolve(environment: env, keychain: MemoryKeychain()))
        XCTAssertEqual(fromEnv.source, .environment)
        XCTAssertEqual(fromEnv.variable, "XAI_API_KEY")
        let fromHelperVar = try XCTUnwrap(GrokCredential.resolve(environment: ["FLASHTEX_GROK_API_KEY": "helper-key"], keychain: MemoryKeychain()))
        XCTAssertEqual(fromHelperVar.variable, "FLASHTEX_GROK_API_KEY")

        // FLASHTEX_KEYCHAIN_OFF=1 skips the store even when it holds a key.
        let skipped = GrokCredential.status(environment: ["FLASHTEX_KEYCHAIN_OFF": "1"], keychain: keychain)
        XCTAssertNil(skipped.resolution)
        XCTAssertTrue(skipped.keychainSkipped)
        XCTAssertTrue(skipped.text.hasPrefix("xAI API key: absent"), skipped.text)
        let envWhileSkipped = try XCTUnwrap(GrokCredential.resolve(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "e"], keychain: keychain))
        XCTAssertEqual(envWhileSkipped.source, .environment)

        XCTAssertNil(GrokCredential.resolve(environment: ["PATH": "/usr/bin"], keychain: MemoryKeychain()))
        // Unusable values are not credentials.
        XCTAssertNil(GrokCredential.resolve(environment: ["XAI_API_KEY": "  \n"], keychain: MemoryKeychain()))
        XCTAssertNil(GrokCredential.resolve(environment: ["XAI_API_KEY": "bad\u{01}key"], keychain: MemoryKeychain()))
        XCTAssertEqual(GrokCredential.validate(String(repeating: "x", count: 8193)), .failure(.tooLong))
        // A Keychain failure is reported, not thrown, and the environment still wins.
        let broken = MemoryKeychain(); broken.failWith = .status(errSecInteractionNotAllowed)
        let status = GrokCredential.status(environment: ["XAI_API_KEY": "e"], keychain: broken)
        XCTAssertEqual(status.resolution?.source, .environment)
        XCTAssertNotNil(status.keychainProblem)
        let none = GrokCredential.status(environment: [:], keychain: broken)
        XCTAssertTrue(none.text.contains("(") && none.text.contains("-25308"), none.text)
    }

    func testNoTextEverContainsTheKey() throws {
        let secret = "xai-SUPER-SECRET-VALUE-0123456789"
        let keychain = MemoryKeychain()
        assertOK(GrokCredential.store(secret, keychain: keychain))
        let status = GrokCredential.status(environment: [:], keychain: keychain)
        let r = try XCTUnwrap(status.resolution)
        for text in [r.description, r.debugDescription, "\(r)", String(reflecting: r), status.text,
                     ProposalPreview.GrokProviderConfiguration(model: "grok-4.6", credential: r, helper: nil).identityText] {
            XCTAssertFalse(text.contains(secret), text)
            XCTAssertFalse(text.contains("SUPER"), text)
        }
        XCTAssertEqual(r.description, "key present (Keychain)")
        XCTAssertEqual(status.text, "xAI API key: key present (Keychain)")
        let envR = try XCTUnwrap(GrokCredential.resolve(environment: ["XAI_API_KEY": secret], keychain: MemoryKeychain()))
        XCTAssertEqual(envR.description, "key present (environment: XAI_API_KEY)")
        // Removal.
        assertOK(GrokCredential.remove(keychain: keychain))
        XCTAssertTrue(keychain.isEmpty)
        XCTAssertNil(GrokCredential.resolve(environment: [:], keychain: keychain))
        if case .failure(.invalid(.empty)) = GrokCredential.store("", keychain: keychain) {} else { XCTFail("empty key stored") }
    }

    /// The real `SecItem` store, under a throwaway service so the user's own
    /// item (`tech.jay3332.flashtex.xai`) is never read or written by tests.
    func testSecItemKeychainRoundTripUnderAThrowawayService() throws {
        let service = "tech.jay3332.flashtex.xai.test-\(UUID().uuidString.lowercased())"
        let store = SecItemKeychain()
        defer { _ = store.delete(service: service, account: GrokCredential.keychainAccount) }
        XCTAssertEqual(try store.read(service: service, account: GrokCredential.keychainAccount).get(), nil)
        switch store.write("first-key", service: service, account: GrokCredential.keychainAccount) {
        case .failure(let e): throw XCTSkip("Keychain unavailable to this test process: \(e.text)")
        case .success: break
        }
        XCTAssertEqual(try store.read(service: service, account: GrokCredential.keychainAccount).get(), "first-key")
        // Update in place (SecItemUpdate path), then delete; delete is idempotent.
        assertOK(store.write("second-key", service: service, account: GrokCredential.keychainAccount))
        XCTAssertEqual(try store.read(service: service, account: GrokCredential.keychainAccount).get(), "second-key")
        assertOK(store.delete(service: service, account: GrokCredential.keychainAccount))
        XCTAssertEqual(try store.read(service: service, account: GrokCredential.keychainAccount).get(), nil)
        assertOK(store.delete(service: service, account: GrokCredential.keychainAccount))
    }

    func testModelAndPreferences() {
        let defaults = temporaryDefaults()
        let prefs = GrokPreferences(defaults: defaults)
        XCTAssertEqual(prefs.providerMode, .auto, "default: Grok whenever a key is present")
        XCTAssertFalse(prefs.providerEnabled)
        XCTAssertNil(prefs.model)
        // grok-4.6 stays the explanation default: the fast model's edits failed the helper's
        // source-bound gate live (docs/evidence/grok-live-20260912T210600Z); captures use the fast one.
        XCTAssertEqual(GrokCredential.defaultModel, "grok-4.6")
        XCTAssertEqual(GrokCredential.model(environment: [:], preferences: prefs), "grok-4.6")
        XCTAssertEqual(GrokCredential.captureModel(environment: [:], preferences: prefs), "grok-4.20-0309-non-reasoning")
        XCTAssertEqual(GrokCredential.selectableModels, ["grok-4.6", "grok-4.20-0309-non-reasoning"], "both selectable")
        XCTAssertEqual(GrokCredential.providerTimeout(for: "grok-4.20-0309-non-reasoning"), 30)
        XCTAssertEqual(GrokCredential.providerTimeout(for: "grok-4.6"), 100)
        XCTAssertTrue(GrokCredential.isReasoningModel("grok-4.6"))
        XCTAssertFalse(GrokCredential.isReasoningModel("grok-4.20-0309-non-reasoning"))
        prefs.model = "grok-4.6-mini"
        XCTAssertEqual(GrokCredential.model(environment: [:], preferences: prefs), "grok-4.6-mini")
        XCTAssertEqual(GrokCredential.model(environment: ["FLASHTEX_GROK_MODEL": "grok-x"], preferences: prefs), "grok-x", "environment wins")
        XCTAssertEqual(GrokCredential.model(environment: ["FLASHTEX_GROK_MODEL": "bad model!"], preferences: prefs), "grok-4.6-mini", "invalid env value ignored")
        prefs.model = "grok-4.20-0309-non-reasoning"
        XCTAssertEqual(prefs.model, "grok-4.20-0309-non-reasoning", "the fast model is a persisted choice")
        prefs.model = GrokCredential.defaultModel
        XCTAssertNil(prefs.model, "the default is not persisted")
        prefs.model = "not valid"
        XCTAssertNil(prefs.model)
        prefs.providerEnabled = true
        XCTAssertTrue(GrokPreferences(defaults: defaults).providerEnabled)
        XCTAssertEqual(GrokPreferences(defaults: defaults).providerMode, .on)
        prefs.providerMode = .off
        XCTAssertFalse(GrokPreferences(defaults: defaults).providerEnabled)
        // Migration from the pre-mode boolean: absent → auto, explicit true → on,
        // explicit false → off (an opt-out survives a key appearing); a stored mode wins.
        XCTAssertEqual(GrokPreferences(defaults: temporaryDefaults()).providerMode, .auto)
        let legacyTrue = temporaryDefaults()
        legacyTrue.set(true, forKey: GrokPreferences.providerKey)
        XCTAssertEqual(GrokPreferences(defaults: legacyTrue).providerMode, .on)
        let legacyFalse = temporaryDefaults()
        legacyFalse.set(false, forKey: GrokPreferences.providerKey)
        XCTAssertEqual(GrokPreferences(defaults: legacyFalse).providerMode, .off)
        legacyFalse.set(GrokPreferences.ProviderMode.auto.rawValue, forKey: GrokPreferences.providerModeKey)
        XCTAssertEqual(GrokPreferences(defaults: legacyFalse).providerMode, .auto, "an explicit new mode takes precedence over the legacy boolean")
        let legacyFalseKeyed = ProposalPreview.ExplanationConfiguration.fromEnvironment(
            ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "k"], bundleExecutableDirectory: nil,
            preferences: GrokPreferences(defaults: { let d = temporaryDefaults(); d.set(false, forKey: GrokPreferences.providerKey); return d }()),
            keychain: MemoryKeychain())
        XCTAssertNil(legacyFalseKeyed.grok, "a legacy opt-out with a key present never selects Grok")
        XCTAssertEqual(legacyFalseKeyed.grokStatusText, "Grok: off")
        var changes = 0
        let token = NotificationCenter.default.addObserver(forName: GrokPreferences.didChange, object: nil, queue: nil) { _ in changes += 1 }
        prefs.providerMode = .auto
        prefs.model = "grok-4.6"
        NotificationCenter.default.removeObserver(token)
        XCTAssertEqual(changes, 2, "the status bar re-reads on every preference change")
        XCTAssertTrue(GrokCredential.isValidModel("grok-4.6"))
        XCTAssertFalse(GrokCredential.isValidModel(""))
        XCTAssertFalse(GrokCredential.isValidModel(String(repeating: "a", count: 129)))
    }

    // MARK: provider selection and child environments

    func testProviderSelectionFromEnvironmentAndPreference() {
        let defaults = temporaryDefaults()
        let prefs = GrokPreferences(defaults: defaults)
        let keychain = MemoryKeychain()
        // Nothing selected and no key (mode auto): disabled, as before; the pill says off.
        let off = ProposalPreview.ExplanationConfiguration.fromEnvironment(Self.envOnly, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain)
        XCTAssertNil(off.provider); XCTAssertNil(off.grok)
        XCTAssertEqual(off.providerIdentityText, "provider disabled (set FLASHTEX_ASSISTANT_PROVIDER to a local command)")
        XCTAssertEqual(off.grokStatusText, "Grok: off")
        XCTAssertEqual(off.providerTimeout, 60, "local provider bound unchanged")
        XCTAssertEqual(GrokStatusPill.current(environment: Self.envOnly, preferences: prefs, keychain: keychain), GrokStatusPill(on: false, text: "Grok: off", help: off.grokStatusHelp))
        // Mode auto with a key (environment or Keychain) and no selector: Grok is the default provider, 100 s bound.
        var keyed = Self.envOnly; keyed["XAI_API_KEY"] = "k"
        let autoEnv = ProposalPreview.ExplanationConfiguration.fromEnvironment(keyed, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain)
        XCTAssertNil(autoEnv.provider)
        XCTAssertEqual(autoEnv.grok?.model, "grok-4.6")
        XCTAssertEqual(autoEnv.grok?.credential?.source, .environment)
        XCTAssertEqual(autoEnv.providerTimeout, 100)
        XCTAssertEqual(autoEnv.grokStatusText, "Grok: on (grok-4.6)")
        XCTAssertTrue(GrokStatusPill.current(environment: keyed, preferences: prefs, keychain: keychain).on)
        let stored = MemoryKeychain([GrokCredential.keychainService + "\u{0}" + GrokCredential.keychainAccount: "kc-key"])
        let autoKeychain = ProposalPreview.ExplanationConfiguration.fromEnvironment([:], bundleExecutableDirectory: nil, preferences: prefs, keychain: stored)
        XCTAssertEqual(autoKeychain.grok?.credential?.source, .keychain)
        XCTAssertTrue(ProposalPreview(executable: nil, explanation: autoKeychain).grokLive)
        // The fast model gets the short bound; "Never" turns Grok off even with a key.
        prefs.model = "grok-4.20-0309-non-reasoning"
        let fast = ProposalPreview.ExplanationConfiguration.fromEnvironment(keyed, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain)
        XCTAssertEqual(fast.grok?.model, "grok-4.20-0309-non-reasoning"); XCTAssertEqual(fast.providerTimeout, 30)
        XCTAssertEqual(fast.grokStatusText, "Grok: on (grok-4.20-0309-non-reasoning)")
        prefs.model = nil
        prefs.providerMode = .off
        let never = ProposalPreview.ExplanationConfiguration.fromEnvironment(keyed, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain)
        XCTAssertNil(never.grok); XCTAssertEqual(never.grokStatusText, "Grok: off")
        prefs.providerMode = .auto
        // FLASHTEX_ASSISTANT_PROVIDER=grok without a key: selected, no credential, honest text.
        var env = Self.envOnly; env["FLASHTEX_ASSISTANT_PROVIDER"] = "grok"
        let grokNoKey = ProposalPreview.ExplanationConfiguration.fromEnvironment(env, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain)
        XCTAssertNil(grokNoKey.provider)
        XCTAssertEqual(grokNoKey.grok?.model, "grok-4.6")
        XCTAssertNil(grokNoKey.grok?.credential)
        XCTAssertTrue(grokNoKey.providerIdentityText.hasPrefix("provider: Grok (xAI) grok-4.6, no API key"), grokNoKey.providerIdentityText)
        XCTAssertEqual(grokNoKey.grokStatusText, "Grok: off")
        XCTAssertTrue(grokNoKey.grokStatusHelp.contains("no API key"), grokNoKey.grokStatusHelp)
        // With a key and a model.
        env["XAI_API_KEY"] = "k"; env["FLASHTEX_GROK_MODEL"] = "grok-4.6-mini"
        let grokKey = ProposalPreview.ExplanationConfiguration.fromEnvironment(env, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain)
        XCTAssertEqual(grokKey.grok?.credential?.source, .environment)
        XCTAssertEqual(grokKey.providerIdentityText, "provider: Grok (xAI) grok-4.6-mini, key present (environment: XAI_API_KEY)")
        XCTAssertEqual(grokKey.providerTimeout, 100, "an unknown id is treated as a reasoning model")
        // Case-insensitive selector; a dedicated grok-built helper path is honoured.
        env["FLASHTEX_ASSISTANT_PROVIDER"] = "Grok"; env["FLASHTEX_ASSISTANT_CONTEXT_GROK"] = Self.fakeSession.path
        XCTAssertEqual(ProposalPreview.ExplanationConfiguration.fromEnvironment(env, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain).grok?.helper, Self.fakeSession)
        // "Always" selects Grok when the variable is unset, even without a key...
        prefs.providerMode = .on
        let byPref = ProposalPreview.ExplanationConfiguration.fromEnvironment(Self.envOnly, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain)
        XCTAssertNotNil(byPref.grok); XCTAssertNil(byPref.provider); XCTAssertNil(byPref.grok?.credential)
        // ...but an explicit local command in the variable wins over the preference.
        var local = Self.envOnly; local["FLASHTEX_ASSISTANT_PROVIDER"] = WorkerClientTests.python.path
        let byPath = ProposalPreview.ExplanationConfiguration.fromEnvironment(local, bundleExecutableDirectory: nil, preferences: prefs, keychain: keychain)
        XCTAssertEqual(byPath.provider, WorkerClientTests.python); XCTAssertNil(byPath.grok)
        XCTAssertFalse(ProposalPreview(executable: nil, explanation: byPath).grokLive)
        XCTAssertTrue(ProposalPreview(executable: nil, explanation: grokKey).grokLive)
        XCTAssertTrue(ProposalPreview(executable: nil, explanation: grokKey).explanationProviderEnabled)
    }

    func testKeyLandsOnlyOnTheGrokChildEnvironment() async throws {
        let app = ["PATH": "/usr/bin", "HOME": "/Users/x", "XAI_API_KEY": "app-key", "FLASHTEX_GROK_API_KEY": "app-key-2",
                   "OPENAI_API_KEY": "other", "HTTPS_PROXY": "http://p", "FLASHTEX_COMPILER": "/c", "FLASHTEX_GROK_MODEL": "grok-x"]
        // The one-shot helper and the grok session start from the same offline set: no key.
        for role in [ProposalPreview.ExplanationConfiguration.ChildRole.helper, .grok] {
            let env = ProposalPreview.ExplanationConfiguration.childEnvironment(for: role, from: app)
            XCTAssertEqual(env, ["PATH": "/usr/bin", "HOME": "/Users/x"], "\(role)")
        }
        // Only the session's own launch adds the helper's variable, from the credential.
        let credential = try XCTUnwrap(GrokCredential.resolve(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "the-key"], keychain: MemoryKeychain()))
        var delivered: Result<OneShotProcess.Output, OneShotProcess.Failure>?
        let session = try GrokProviderSession(helper: Self.fakeSession, model: "grok-4.6", sessionId: "unit-1", credential: credential,
                                              environment: ProposalPreview.ExplanationConfiguration.childEnvironment(for: .grok, from: app)) { delivered = $0 }
        XCTAssertEqual(session.launch.arguments, ["--provider-session", "unit-1", "grok-4.6"])
        XCTAssertEqual(session.launch.environmentKeys, ["FLASHTEX_GROK_API_KEY", "HOME", "PATH"])
        XCTAssertFalse(session.launch.arguments.joined().contains("the-key"))
        try await Task.sleep(nanoseconds: 300_000_000)
        XCTAssertTrue(session.isRunning, "the fake accepted the argv/env gates")
        session.cancel()
        try await waitUntil("cancelled") { delivered != nil }
        XCTAssertEqual(delivered, .failure(.cancelled))
        try await waitUntil("exited") { !session.isRunning }

        // The bridge: secrets stripped; XAI_API_KEY + model only with a credential.
        let without = GrokCredential.bridgeEnvironment(credential: nil, model: "grok-4.6", from: app)
        XCTAssertEqual(without, ["PATH": "/usr/bin", "HOME": "/Users/x", "FLASHTEX_COMPILER": "/c"])
        let with = GrokCredential.bridgeEnvironment(credential: credential, model: "grok-4.6-mini", from: app)
        XCTAssertEqual(with, ["PATH": "/usr/bin", "HOME": "/Users/x", "FLASHTEX_COMPILER": "/c", "XAI_API_KEY": "the-key", "FLASHTEX_GROK_MODEL": "grok-4.6-mini"])
        let launchNoKey = ShellModel.bridgeGrokLaunch(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "PATH": "/usr/bin"], keychain: MemoryKeychain(), preferences: GrokPreferences(defaults: temporaryDefaults()))
        XCTAssertFalse(launchNoKey.enableGrok); XCTAssertNil(launchNoKey.environment["XAI_API_KEY"]); XCTAssertNil(launchNoKey.credential)
        let launchKey = ShellModel.bridgeGrokLaunch(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "k"], keychain: MemoryKeychain(), preferences: GrokPreferences(defaults: temporaryDefaults()))
        XCTAssertTrue(launchKey.enableGrok); XCTAssertEqual(launchKey.environment["XAI_API_KEY"], "k"); XCTAssertEqual(launchKey.environment["FLASHTEX_GROK_MODEL"], GrokCredential.defaultCaptureModel)
    }

    // MARK: the provider stage through the session double

    private func input(_ text: String, anchorByte: Int, revision: Int = 1) -> ProposalPreview.Input {
        let ctx = String(decoding: Array(text.utf8.dropFirst(anchorByte).prefix(Insertion.contextLength)), as: UTF8.self)
        return .init(documents: [.init(path: "main.tex", text: text)], entryPath: "main.tex",
                     anchor: InsertionAnchor(id: "a1", path: "main.tex", byteOffset: anchorByte, revision: revision, contextAfter: ctx),
                     editorRevision: revision, projectId: "demo")
    }

    private func grokPreview(key: Bool = true, providerTimeout: TimeInterval = 5) -> ProposalPreview {
        var c = ProposalPreview.ExplanationConfiguration(helper: WorkerClientTests.python, helperArguments: [Self.fakeHelper.path])
        c.helperTimeout = 5
        c.providerTimeout = providerTimeout
        c.grok = .init(model: "grok-4.6-test",
                       credential: key ? GrokCredential.resolve(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "fixture-key"], keychain: MemoryKeychain()) : nil,
                       helper: Self.fakeSession)
        return ProposalPreview(executable: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path], explanation: c)
    }

    private func waitUntil(_ what: String, timeout: TimeInterval = 8, _ cond: @escaping @MainActor () -> Bool) async throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if cond() { return }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
        XCTFail("timed out waiting for \(what)")
    }

    private func waitForReady(_ p: ProposalPreview) async throws {
        try await waitUntil("preview ready") { if case .ready = p.state { return true }; return false }
    }

    private func waitSettled(_ p: ProposalPreview) async throws {
        try await waitUntil("explanation settled", timeout: 12) { !p.explanationInFlight }
    }

    /// prepare → Grok session (admit, poll, validated proposal) → helper review →
    /// explicit approve → amended draft. The session child is the only launch
    /// carrying the key; the one-shot helper launches stay credential-free.
    func testGrokSessionReplyIsReviewedAndApprovedLikeAnyProvider() async throws {
        let preview = grokPreview()
        XCTAssertTrue(preview.grokLive)
        XCTAssertTrue(preview.explanationStatusText.hasSuffix("provider: Grok (xAI) grok-4.6-test, key present (environment: XAI_API_KEY)"), preview.explanationStatusText)
        preview.update(input: input("A\n%diag:1 tail %groksnapshot\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitUntil("awaiting provider") { if case .awaitingProvider = preview.explanationState { return true }; return false }
        XCTAssertTrue(preview.explanationStatusText.hasPrefix("Asking Grok (grok-4.6-test)…"), preview.explanationStatusText)
        XCTAssertNotNil(preview.providerStartedAt, "the sheet's elapsed counter runs while the live stage does")
        XCTAssertNotNil(preview.runningChildProcessIdentifier)
        try await waitSettled(preview)
        guard case .ready(let e) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(e.text.hasPrefix("Fake Grok (grok-4.6-test) explanation of 2 diagnostic(s)"), e.text)
        XCTAssertTrue(e.text.hasSuffix(" env: FLASHTEX_GROK_API_KEY key:present"), "no other FLASHTEX_* variable reached the session: \(e.text)")
        XCTAssertEqual(e.edits.count, 1)
        XCTAssertFalse(e.applied)
        let reviewId = try XCTUnwrap(e.reviewId)
        XCTAssertEqual(preview.childLaunches.map(\.stage), [.probe, .prepare, .provider, .review])
        XCTAssertEqual(preview.childLaunches.map(\.role), [.helper, .helper, .grok, .helper])
        for l in preview.childLaunches where l.role == .helper {
            XCTAssertFalse(l.environmentKeys.contains("FLASHTEX_GROK_API_KEY"), "\(l.stage)")
            XCTAssertFalse(l.environmentKeys.contains { $0.hasPrefix("FLASHTEX_") })
        }
        let grokLaunch = try XCTUnwrap(preview.childLaunches.first { $0.role == .grok })
        XCTAssertEqual(grokLaunch.executable, Self.fakeSession)
        XCTAssertEqual(grokLaunch.arguments.prefix(1), ["--provider-session"])
        XCTAssertEqual(grokLaunch.arguments.last, "grok-4.6-test")
        XCTAssertTrue(grokLaunch.arguments[1].hasPrefix("mac-explain-1-"), grokLaunch.arguments[1])
        XCTAssertTrue(grokLaunch.environmentKeys.contains("FLASHTEX_GROK_API_KEY"))
        XCTAssertFalse(grokLaunch.environmentKeys.contains("XAI_API_KEY"))
        XCTAssertNil(preview.runningChildProcessIdentifier, "the session child exited after the reply")

        preview.approveReviewedEdit()
        try await waitSettled(preview)
        guard case .approved(let a) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(a.reviewId, reviewId)
        XCTAssertEqual(a.amendedLatex, "% grok-reviewed: %diag:0 new")
        XCTAssertFalse(a.applied)
        XCTAssertEqual(preview.shadow?.shadowText, "A\n%diag:0 new\n%diag:1 tail %groksnapshot\n", "the document is untouched")
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    func testGrokSelectedWithoutKeySendsNothingAndSaysSo() async throws {
        let preview = grokPreview(key: false)
        XCTAssertFalse(preview.grokLive)
        XCTAssertTrue(preview.explanationProviderEnabled)
        preview.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitSettled(preview)
        guard case .failed(let why) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(why.hasPrefix("Grok (xAI) is selected but no API key is present"), why)
        XCTAssertTrue(why.hasSuffix("sent nowhere"), why)
        XCTAssertEqual(preview.childLaunches.map(\.stage), [.probe, .prepare], "no provider launch of any kind")
        XCTAssertNil(preview.lastGrokLaunch)
        XCTAssertTrue(preview.canExplain, "the review stays usable")
        preview.close()
        try await waitUntil("worker terminated") { !preview.workerIsRunning }
    }

    func testGrokSessionFailureTimeoutAndCancellationSurfaceHonestly() async throws {
        // Helper-reported failure: the helper collapses the HTTP class, and the text says so.
        let preview = grokPreview(providerTimeout: 2)
        preview.update(input: input("A\n%diag:1 tail %grokfail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(preview)
        preview.explain()
        try await waitSettled(preview)
        guard case .failed(let why) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertTrue(why.hasPrefix("provider refused (provider): Grok request failed (helper state \"failed\""), why)
        XCTAssertTrue(why.contains("401/403") && why.contains("429"), why)
        guard case .exited(_, .provider, .provider, 1)? = preview.recoveryLog.events.last else { return XCTFail("\(String(describing: preview.recoveryLog.events.last))") }
        XCTAssertNil(preview.runningChildProcessIdentifier)

        // A helper that never becomes ready: the provider timeout ends the session.
        preview.update(input: input("A\n%diag:1 tail %grokhang\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 2 && !preview.isInFlight }
        preview.explain()
        try await waitUntil("timed out", timeout: 15) { if case .failed = preview.explanationState { return true }; return false }
        guard case .failed(let slow) = preview.explanationState else { return XCTFail("\(preview.explanationState)") }
        XCTAssertEqual(slow, "provider gave no reply within 2 s (provider)")
        XCTAssertNil(preview.runningChildProcessIdentifier)

        // Cancellation while the session is polling: the child is terminated, the late reply dropped.
        preview.update(input: input("A\n%diag:1 tail %grokslow\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitUntil("recompiled") { preview.shadowCompileCount == 3 && !preview.isInFlight }
        preview.explain()
        try await waitUntil("session running") { if case .awaitingProvider = preview.explanationState { return preview.runningChildProcessIdentifier != nil }; return false }
        let pid = try XCTUnwrap(preview.runningChildProcessIdentifier)
        XCTAssertNotNil(preview.providerStartedAt)
        let stale = preview.staleExplanationReplies
        preview.cancelExplanationByReviewer()
        XCTAssertEqual(preview.explanationState, .cancelled("cancelled by reviewer"))
        XCTAssertNil(preview.providerStartedAt, "the elapsed counter stops with the session")
        try await waitUntil("late reply discarded") { preview.staleExplanationReplies == stale + 1 }
        try await waitUntil("child gone") { kill(pid, 0) != 0 }
        XCTAssertEqual(preview.explanationState, .cancelled("cancelled by reviewer"))

        // A session helper without the grok feature / without a key exits 2 at launch.
        var c = preview.explanationConfiguration
        c.grok?.model = "no-grok-feature" // the double then prints usage and exits 2, like a helper built without the feature
        let wrong = ProposalPreview(executable: WorkerClientTests.python, arguments: [WorkerClientTests.fakeWorker.path], explanation: c)
        wrong.update(input: input("A\n%diag:1 tail\n", anchorByte: 2), latex: "%diag:0 new")
        try await waitForReady(wrong)
        wrong.explain()
        try await waitSettled(wrong)
        guard case .failed(let usage) = wrong.explanationState else { return XCTFail("\(wrong.explanationState)") }
        XCTAssertTrue(usage.hasPrefix("provider refused (provider): helper"), usage)
        preview.close(); wrong.close()
        try await waitUntil("workers terminated") { !preview.workerIsRunning && !wrong.workerIsRunning }
    }

    /// The real helper, if one is built here: with the `grok` feature it stays
    /// up in `--provider-session` mode until cancelled (no admit is sent, so no
    /// network); without it, the launch fails with the feature message.
    func testRealHelperProviderSessionStartsOrRefusesWithoutNetwork() async throws {
        guard let helper = ProposalPreview.ExplanationConfiguration.locateHelper() else {
            throw XCTSkip("build crates/assistant-context or set FLASHTEX_ASSISTANT_CONTEXT")
        }
        let credential = try XCTUnwrap(GrokCredential.resolve(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "dummy-not-a-key"], keychain: MemoryKeychain()))
        var delivered: Result<OneShotProcess.Output, OneShotProcess.Failure>?
        let session = try GrokProviderSession(helper: helper, model: "grok-4.6", sessionId: "mac-test-\(UUID().uuidString.prefix(8))", credential: credential,
                                              environment: ProposalPreview.ExplanationConfiguration.childEnvironment(for: .grok, from: ProcessInfo.processInfo.environment)) { delivered = $0 }
        try await Task.sleep(nanoseconds: 700_000_000)
        if session.isRunning {
            session.cancel()
            try await waitUntil("cancelled") { delivered != nil }
            XCTAssertEqual(delivered, .failure(.cancelled))
        } else {
            try await waitUntil("exit delivered") { delivered != nil }
            guard case .failure(.exited(let code, let output, _)) = delivered else { return XCTFail("\(String(describing: delivered))") }
            XCTAssertEqual(code, 1)
            let text = String(decoding: output, as: UTF8.self)
            XCTAssertTrue(text.contains("built without the grok feature"), text)
        }
        try await waitUntil("exited") { !session.isRunning }
    }

    // MARK: connection probe against a loopback stub (401 / 429 / timeout classes)

    private func startStub() throws -> (process: Process, port: Int) {
        let p = Process()
        p.executableURL = WorkerClientTests.python
        p.arguments = [Self.fakeXAI.path]
        let stdin = Pipe(), stdout = Pipe()
        p.standardInput = stdin; p.standardOutput = stdout
        try p.run()
        let line = stdout.fileHandleForReading.availableData
        let text = String(decoding: line, as: UTF8.self)
        guard text.hasPrefix("PORT "), let port = Int(text.dropFirst(5).trimmingCharacters(in: .whitespacesAndNewlines)) else {
            p.terminate(); throw XCTSkip("stub did not report a port: \(text)")
        }
        return (p, port)
    }

    func testProbeReportsHTTPClassOnly() async throws {
        let stub = try startStub()
        defer { stub.process.terminate() }
        let base = try XCTUnwrap(GrokProbe.baseURL(environment: ["FLASHTEX_GROK_BASE_URL": "http://127.0.0.1:\(stub.port)"]))
        XCTAssertEqual(base.absoluteString, "http://127.0.0.1:\(stub.port)")
        // Only loopback http or https overrides are honoured.
        XCTAssertEqual(GrokProbe.baseURL(environment: ["FLASHTEX_GROK_BASE_URL": "http://evil.example"]), GrokProbe.defaultBaseURL)
        XCTAssertEqual(GrokProbe.baseURL(environment: ["FLASHTEX_GROK_BASE_URL": "ftp://127.0.0.1"]), GrokProbe.defaultBaseURL)
        XCTAssertEqual(GrokProbe.baseURL(environment: [:]), URL(string: "https://api.x.ai"))
        func probe(_ key: String, timeout: TimeInterval = 10) async throws -> GrokProbe.Outcome {
            let credential = try XCTUnwrap(GrokCredential.resolve(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": key], keychain: MemoryKeychain()))
            return await withCheckedContinuation { k in
                GrokProbe.probe(credential: credential, baseURL: base, timeout: timeout) { k.resume(returning: $0) }
            }
        }
        let ok = try await probe("good-key")
        XCTAssertEqual(ok, .accepted(200)); XCTAssertTrue(ok.keyAccepted)
        XCTAssertEqual(ok.text, "connection OK (HTTP 200): the key was accepted")
        let unauthorized = try await probe("wrong-key")
        XCTAssertEqual(unauthorized, .unauthorized(401)); XCTAssertFalse(unauthorized.keyAccepted)
        XCTAssertFalse(unauthorized.text.contains("wrong-key"))
        let rate = try await probe("rate-key"); XCTAssertEqual(rate, .rateLimited)
        let server = try await probe("server-key"); XCTAssertEqual(server, .serverError(503))
        let slow = try await probe("slow-key", timeout: 1); XCTAssertEqual(slow, .timeout)
        XCTAssertEqual(GrokProbe.classify(status: 302), .unexpected(302))
        XCTAssertEqual(GrokProbe.classify(status: 403), .unauthorized(403))
        // Nothing listening: a transport failure, not a crash.
        stub.process.terminate()
        try await waitUntil("stub gone") { !stub.process.isRunning }
        let credential = try XCTUnwrap(GrokCredential.resolve(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "good-key"], keychain: MemoryKeychain()))
        let down: GrokProbe.Outcome = await withCheckedContinuation { k in
            GrokProbe.probe(credential: credential, baseURL: base, timeout: 3) { k.resume(returning: $0) }
        }
        guard case .transport = down else { return XCTFail("\(down)") }
    }

    // MARK: the bridge, launched through the shell with the key in its environment

    func testBridgeConversionUsesEnableGrokAndTheKeyOnlyWhenPresent() async throws {
        let model = ShellModel()
        model.autoCompile = false
        let store = try BridgeClientTests.tempStore()
        // Without a key: as before (no --enable-grok) and provider_disabled becomes plain guidance.
        let noKey = ShellModel.bridgeGrokLaunch(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "PATH": ProcessInfo.processInfo.environment["PATH"] ?? "/usr/bin"],
                                                keychain: MemoryKeychain(), preferences: GrokPreferences(defaults: temporaryDefaults()))
        var ok = await model.attachBridgeAndWait(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeBridge.path], storeDirectory: store,
                                                 enableGrok: noKey.enableGrok, environment: noKey.environment, ledger: ShellModelBridgeTests.fakeLedger, discoverLedger: false)
        XCTAssertTrue(ok, model.captureNote ?? model.bridgeStatus)
        XCTAssertEqual(model.bridge?.grokEnabled, false)
        model.caretUTF16 = 0
        model.pinAnchorAtCaret()
        try await waitUntil("pin \(model.bridgeStatus) / \(model.captureNote ?? "")") { model.bridgeDestination != nil }
        let image = try BridgeClientTests.fixtureCapture().image
        let received = await model.submitCapture(image: image, captureId: "grok-gate-1", instructions: "%grokgate"); XCTAssertNotNil(received, model.captureNote ?? "")
        let refused = await model.convertCapture(captureId: "grok-gate-1"); XCTAssertNil(refused)
        XCTAssertTrue(model.captureNote?.contains("provider_disabled") == true, model.captureNote ?? "")
        XCTAssertTrue(model.captureNote?.contains("Preferences (⌘,)") == true, model.captureNote ?? "")
        model.detachBridge()

        // With a key: --enable-grok, XAI_API_KEY and the model reach the bridge; the proposal is reviewed as usual.
        let withKey = ShellModel.bridgeGrokLaunch(environment: ["FLASHTEX_KEYCHAIN_OFF": "1", "XAI_API_KEY": "fixture-bridge-key", "FLASHTEX_GROK_MODEL": "grok-4.6-vision",
                                                                "PATH": ProcessInfo.processInfo.environment["PATH"] ?? "/usr/bin"],
                                                  keychain: MemoryKeychain(), preferences: GrokPreferences(defaults: temporaryDefaults()))
        ok = await model.attachBridgeAndWait(executable: BridgeClientTests.python, arguments: [BridgeClientTests.fakeBridge.path], storeDirectory: store,
                                             enableGrok: withKey.enableGrok, environment: withKey.environment, ledger: ShellModelBridgeTests.fakeLedger, discoverLedger: false)
        XCTAssertTrue(ok, model.captureNote ?? model.bridgeStatus)
        XCTAssertEqual(model.bridge?.grokEnabled, true)
        model.pinAnchorAtCaret() // a new bridge process holds no destination; pin again as the user would
        try await waitUntil("pin \(model.bridgeStatus) / \(model.captureNote ?? "")") { model.bridgeDestination != nil }
        let received2 = await model.submitCapture(image: image, captureId: "grok-gate-2", instructions: "%grokgate"); XCTAssertNotNil(received2, model.captureNote ?? "")
        let converted = await model.convertCapture(captureId: "grok-gate-2"); let proposal = try XCTUnwrap(converted, model.captureNote ?? "")
        XCTAssertEqual(proposal.latex, "\\fakecapture{grok-gate-2}")
        XCTAssertTrue(proposal.ambiguities.contains("model:grok-4.6-vision key:present"), "\(proposal.ambiguities)")
        XCTAssertEqual(model.proposals.count, 1, "queued for the review sheet; nothing inserted")
        XCTAssertEqual(model.activeText, "Hello FlashTeX.\n")
        // Bridge provider errors map to guidance without the key.
        let note = ShellModel.conversionFailureNote(BridgeClient.Failure.bridge(.init(code: "provider_auth_error", message: "Grok returned HTTP 401")), grokEnabled: true)
        XCTAssertTrue(note.contains("401/403") && note.contains("Test Connection"), note)
        XCTAssertTrue(ShellModel.conversionFailureNote(BridgeClient.Failure.bridge(.init(code: "provider_rate_limited", message: "")), grokEnabled: true).contains("429"))
        XCTAssertTrue(ShellModel.conversionFailureNote(BridgeClient.Failure.bridge(.init(code: "provider_timeout", message: "")), grokEnabled: true).contains("90 s"))
        XCTAssertTrue(ShellModel.conversionFailureNote(BridgeClient.Failure.timeout(3), grokEnabled: true).hasPrefix("Conversion unavailable (Grok enabled): no reply"))
        model.detachBridge()
    }
}

private func assertOK<E: Error>(_ result: Result<Void, E>, file: StaticString = #filePath, line: UInt = #line) {
    if case .failure(let e) = result { XCTFail("\(e)", file: file, line: line) }
}
