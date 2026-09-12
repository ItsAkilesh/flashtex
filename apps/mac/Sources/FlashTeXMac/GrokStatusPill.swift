import Foundation

/// What the status bar shows for Grok (xAI): computed from the same
/// `ExplanationConfiguration` the review sheet would use, so "on" means the
/// next explanation is a live call and "off" means it is not. Never a key.
struct GrokStatusPill: Equatable {
    var on: Bool
    var text: String
    var help: String

    @MainActor static func current(environment: [String: String] = ProcessInfo.processInfo.environment,
                                   preferences: GrokPreferences = .shared,
                                   keychain: any GrokKeychainStore = SecItemKeychain.shared) -> GrokStatusPill {
        let c = ProposalPreview.ExplanationConfiguration.fromEnvironment(environment, preferences: preferences, keychain: keychain)
        return GrokStatusPill(on: c.grok?.credential != nil, text: c.grokStatusText, help: c.grokStatusHelp)
    }
}
