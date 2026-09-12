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
            CommandGroup(after: .pasteboard) {
                Divider()
                Button("Pin Insertion Point") { model.pinAnchorAtCaret() }
                    .keyboardShortcut("p", modifiers: [.command, .shift])
                Button("Open Capture Proposal…") { model.openProposalPanel() }
                    .keyboardShortcut("i", modifiers: [.command, .shift])
            }
            CommandGroup(replacing: .newItem) {
                Button("Open Compile Result Fixture…") { model.openFixturePanel() }
                    .keyboardShortcut("o")
                Button("Reload Fixture") { model.reloadFixture() }
                    .keyboardShortcut("r")
                Divider()
                Button("Attach Worker Executable…") { model.attachWorkerPanel() }
                    .keyboardShortcut("k")
                Button("Compile") { model.compile() }
                    .keyboardShortcut("b")
                    .disabled(!model.workerAttached)
                Button("Detach Worker") { model.detachWorker() }
                    .disabled(!model.workerAttached)
            }
        }
    }
}
