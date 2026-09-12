import SwiftUI

@main
struct FlashTeXPadApp: App {
    @StateObject private var model = PadModel()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(model)
                .onAppear {
                    // Deterministic launch states for tests and screenshots
                    // (`-flashtexpad-open sample|fixture`); no network is touched.
                    let args = ProcessInfo.processInfo.arguments
                    if let i = args.firstIndex(of: "-flashtexpad-open"), i + 1 < args.count {
                        switch args[i + 1] {
                        case "sample": model.openBundledSample()
                        case "fixture": model.openReviewFixture()
                        default: break
                        }
                    }
                }
        }
    }
}
