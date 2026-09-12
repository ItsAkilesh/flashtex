import AppKit
import SwiftUI
import FlashTeXAccessibility

/// A bare SwiftPM executable has no bundle, so AppKit defaults to an
/// accessory-style process with no Dock icon and, when launched from a
/// non-GUI context, no visible window. Force a regular, activated app.
final class AppDelegate: NSObject, NSApplicationDelegate {
    /// Set by the App so quitting can check for unsaved edits.
    weak var model: ShellModel?

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        guard let model, model.project.anyDirty, model.documentURL != nil || !model.activeText.isEmpty else { return .terminateNow }
        let dirty = model.project.listing.filter(\.isDirty).map(\.path)
        let alert = NSAlert()
        alert.messageText = "Save changes to \(model.documentURL == nil ? "the unsaved buffer" : dirty.joined(separator: ", "))?"
        alert.informativeText = "Your edits since the last save will be lost if you don't save."
        alert.addButton(withTitle: "Save")
        alert.addButton(withTitle: "Don't Save")
        alert.addButton(withTitle: "Cancel")
        switch alert.runModal() {
        case .alertFirstButtonReturn:
            // Non-entry members first (their own rooted files, synchronous
            // compare-and-replace like saveTex), then the entry.
            var allSaved = true
            for path in dirty where path != model.project.entryPath {
                switch model.project.saveDocumentNow(path) {
                case .saved: continue
                case .conflict(let c): allSaved = false; model.captureNote = c.summary
                case .failed(let why): allSaved = false; model.captureNote = why
                }
            }
            if model.project.isDirty(model.project.entryPath) {
                if model.activePath != model.project.entryPath { model.project.switchDocument(to: model.project.entryPath) }
                if !model.saveTex() {
                    allSaved = false
                    // The rooted save helper reported an on-disk conflict: resolve it first.
                    if model.files.conflict != nil { model.resolveConflictPanel() }
                }
            }
            return allSaved && !model.project.anyDirty ? .terminateNow : .terminateCancel
        case .alertSecondButtonReturn: return .terminateNow
        default: return .terminateCancel
        }
    }

    /// Held for the app's lifetime: without it App Nap and timer coalescing
    /// quantize the main run loop of a backgrounded window to ~30 ms turns, so
    /// a compile result that arrived in 1 ms waited a whole turn to be applied
    /// (measured with tools/typing-bench: fixture keystroke->paint p50 80 ms).
    private var liveActivity: NSObjectProtocol?

    /// Coming back to the app rechecks the document on disk (external edits
    /// become an explicit conflict state, never a silent overwrite).
    func applicationDidBecomeActive(_ notification: Notification) {
        if let model { Task { @MainActor in await model.refreshDiskStatus() } } // helper route when attached
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        liveActivity = ProcessInfo.processInfo.beginActivity(
            options: [.userInitiatedAllowingIdleSystemSleep, .latencyCritical],
            reason: "Live LaTeX preview follows every keystroke")
        // Automation launches (validation suites, launch-check) must never steal
        // keyboard focus from a person typing at the machine.
        if ProcessInfo.processInfo.environment["FLASHTEX_NO_ACTIVATE"] != "1" {
            NSApp.activate(ignoringOtherApps: true)
            NSApp.windows.first?.makeKeyAndOrderFront(nil)
        } else {
            NSApp.windows.first?.orderBack(nil)
        }
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }
}

@main
struct FlashTeXMacApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var model = ShellModel()
    @StateObject private var nearby = NearbyState()
    @Environment(\.openWindow) private var openWindow

    var body: some Scene {
        WindowGroup("FlashTeX") {
            ContentView()
                .environment(model)
                .frame(minWidth: 900, minHeight: 560)
                .onAppear {
                    appDelegate.model = model; nearby.attach(sink: model, destinations: model); TypingBench.shared.install(model: model)
                    // Automation: open a secondary window at launch for evidence captures.
                    if let id = ProcessInfo.processInfo.environment["FLASHTEX_OPEN_WINDOW"], ["nearby", AccessibilityHelpView.windowID].contains(id) { openWindow(id: id) }
                }
        }
        .commands {
            NavigationCommands(model: model) // Navigation.swift
            CommandGroup(after: .help) {
                Button("FlashTeX Accessibility Help") { openWindow(id: AccessibilityHelpView.windowID) }
            }
            CommandGroup(after: .pasteboard) {
                Divider()
                Button("Pin Insertion Point") { model.pinAnchorAtCaret() }
                    .keyboardShortcut("p", modifiers: [.command, .shift])
                Button("Open Capture Proposal…") { model.openProposalPanel() }
                    .keyboardShortcut("i", modifiers: [.command, .shift])
                Button("Restore Discarded Buffer") { model.restoreDiscardedBuffer() }
                    .disabled(model.recoverableBuffer == nil)
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
                Button("Save") { model.saveTexInteractive() }
                    .keyboardShortcut("s")
                Button("Resolve On-Disk Conflict…") { model.resolveConflictPanel() }
                    .disabled(model.files.conflict == nil)
                Button("Reload From Disk…") { model.reloadFromDiskInteractive() }
                    .disabled(model.documentURL == nil)
                Button("Save As…") { model.saveTexAs() }
                    .keyboardShortcut("s", modifiers: [.command, .shift])
                Divider()
                Button("Open Compile Result Fixture…") { model.openFixturePanel() }
                    .keyboardShortcut("o", modifiers: [.command, .shift])
                Button("Reload Fixture") { model.reloadFixture() }
                    .keyboardShortcut("r")
                Button("Open Display List (v2)…") { model.openDisplayListV2Panel() } // experimental, PreviewV2View.swift
                Button("Export PDF…") { model.exportPDF() }
                    .keyboardShortcut("e", modifiers: [.command, .shift])
                    .disabled(model.result == nil)
                Button("Export PDF via Rust Writer…") { model.exportPDFViaRust() }
                    .keyboardShortcut("e", modifiers: [.command, .option])
                    .disabled(model.result == nil)
                Button("Export PDF (exact, v2)…") { model.exportPDFExact() } // ExactPDFExport.swift
                    .disabled(model.displayListV2?.frame == nil)
                Divider()
                Button("Attach Built Compiler") { model.attachDiscoveredWorker() }
                    .keyboardShortcut("k", modifiers: [.command, .shift])
                Button("Attach Render Pipeline (Latin Modern)") { model.attachDiscoveredRenderPipeline() }
                    .keyboardShortcut("r", modifiers: [.command, .shift])
                    .help("Attach flashtex-render (crates/render-pipeline) — the producer whose metrics are Latin Modern, so the preview shows Computer Modern-style text")
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
            NearbyView().environmentObject(nearby).environment(model)
        }
        .windowResizability(.contentSize)
        Window("Accessibility Help", id: AccessibilityHelpView.windowID) {
            AccessibilityHelpView() // FlashTeXAccessibility: focus order, VoiceOver notes, command table
        }
        Settings { EditorPreferencesView() } // EditorPreferences.swift (⌘,)
    }
}
