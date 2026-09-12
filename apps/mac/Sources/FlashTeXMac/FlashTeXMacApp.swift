import SwiftUI

@main
struct FlashTeXMacApp: App {
    @StateObject private var model = ShellModel()

    var body: some Scene {
        WindowGroup("FlashTeX") {
            ContentView()
                .environmentObject(model)
                .frame(minWidth: 900, minHeight: 560)
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("Open Compile Result Fixture…") { model.openFixturePanel() }
                    .keyboardShortcut("o")
                Button("Reload Fixture") { model.reloadFixture() }
                    .keyboardShortcut("r")
            }
        }
    }
}
