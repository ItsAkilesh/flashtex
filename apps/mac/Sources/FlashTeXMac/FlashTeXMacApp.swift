import AppKit
import SwiftUI

/// A bare SwiftPM executable has no bundle, so AppKit defaults to an
/// accessory-style process with no Dock icon and, when launched from a
/// non-GUI context, no visible window. Force a regular, activated app.
final class AppDelegate: NSObject, NSApplicationDelegate {
    /// Set by the App so quitting can check for unsaved edits.
    weak var model: ShellModel?

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        guard let model, model.isDirty, model.documentURL != nil || !model.activeText.isEmpty else { return .terminateNow }
        let alert = NSAlert()
        alert.messageText = "Save changes to \(model.documentURL?.lastPathComponent ?? "the unsaved buffer")?"
        alert.informativeText = "Your edits since the last save will be lost if you don't save."
        alert.addButton(withTitle: "Save")
        alert.addButton(withTitle: "Don't Save")
        alert.addButton(withTitle: "Cancel")
        switch alert.runModal() {
        case .alertFirstButtonReturn: return model.saveTex() ? .terminateNow : .terminateCancel
        case .alertSecondButtonReturn: return .terminateNow
        default: return .terminateCancel
        }
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        NSApp.activate(ignoringOtherApps: true)
        NSApp.windows.first?.makeKeyAndOrderFront(nil)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
}

@main
struct FlashTeXMacApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var model = ShellModel()
    @StateObject private var nearby = NearbyState()
    @Environment(\.openWindow) private var openWindow

    var body: some Scene {
        WindowGroup("FlashTeX") {
            ContentView()
                .environmentObject(model)
                .frame(minWidth: 900, minHeight: 560)
                .onAppear { appDelegate.model = model; nearby.attach(sink: model, destinations: model) }
        }
        .commands {
            NavigationCommands(model: model) // Navigation.swift
            CommandGroup(after: .pasteboard) {
                Divider()
                Button("Pin Insertion Point") { model.pinAnchorAtCaret() }
                    .keyboardShortcut("p", modifiers: [.command, .shift])
                Button("Open Capture Proposal…") { model.openProposalPanel() }
                    .keyboardShortcut("i", modifiers: [.command, .shift])
                Divider()
                Button("Attach Capture Bridge") { model.attachDiscoveredBridge() }
                Button("Detach Capture Bridge") { model.detachBridge() }
                    .disabled(!model.bridgeAttached)
                Button("Submit Sample Capture…") { model.submitSampleCapturePanel() }
                    .keyboardShortcut("u", modifiers: [.command, .shift])
                    .disabled(!model.bridgeAttached)
                Button("Convert Capture") { model.convertLatestCapture() }
                    .keyboardShortcut("g", modifiers: [.command, .shift])
                    .disabled(model.latestConvertibleCapture == nil)
                Button("Retry Bridge Receipt") { model.retryBridgeReceipt() }
                    .disabled(model.bridge?.pendingTransaction == nil)
                Button("Retry Bridge Reconciliation") { Task { await model.retryBridgeReconciliation() } }
                    .disabled(!model.bridgeAttached)
                Divider()
                Button("Nearby Companion…") { openWindow(id: "nearby") }
                    .keyboardShortcut("n", modifiers: [.command, .shift])
            }
            CommandGroup(replacing: .newItem) {
                Button("Open LaTeX File…") { model.openTexPanel() }
                    .keyboardShortcut("o")
                Button("Save") { model.saveTex() }
                    .keyboardShortcut("s")
                Button("Save As…") { model.saveTexAs() }
                    .keyboardShortcut("s", modifiers: [.command, .shift])
                Divider()
                Button("Open Compile Result Fixture…") { model.openFixturePanel() }
                    .keyboardShortcut("o", modifiers: [.command, .shift])
                Button("Reload Fixture") { model.reloadFixture() }
                    .keyboardShortcut("r")
                Button("Export PDF…") { model.exportPDF() }
                    .keyboardShortcut("e", modifiers: [.command, .shift])
                    .disabled(model.result == nil)
                Button("Export PDF via Rust Writer…") { model.exportPDFViaRust() }
                    .keyboardShortcut("e", modifiers: [.command, .option])
                    .disabled(model.result == nil)
                Divider()
                Button("Attach Built Compiler") { model.attachDiscoveredWorker() }
                    .keyboardShortcut("k", modifiers: [.command, .shift])
                Button("Attach Worker Executable…") { model.attachWorkerPanel() }
                    .keyboardShortcut("k")
                Button("Compile") { model.compile() }
                    .keyboardShortcut("b")
                    .disabled(!model.workerAttached)
                Button("Detach Worker") { model.detachWorker() }
                    .disabled(!model.workerAttached)
            }
        }
        Window("Nearby Companion", id: "nearby") {
            NearbyView().environmentObject(nearby).environmentObject(model)
        }
        .windowResizability(.contentSize)
    }
}
