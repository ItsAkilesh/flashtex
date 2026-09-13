import AppKit

/// Hosted tests build real `NSWindow`s so that AppKit and SwiftUI geometry
/// (text container widths, caret rects, scroll offsets) is measured the way the
/// shipping app measures it. Those windows are only ever ordered front, never
/// made key — but an `xctest` process still starts life as a regular app, and a
/// regular app that orders a window front is pulled in front of whoever is
/// actually using the Mac. Running the suite flashed windows over the owner's
/// work.
///
/// A non-activating activation policy fixes it at the source: the process drops
/// out of the Dock and out of the window-server's activation list, so ordering a
/// window front no longer activates anything. AppKit still builds, lays out and
/// draws the window, which is all these tests read.
///
/// `.prohibited` is preferred because it also keeps the windows off the screen;
/// `.accessory` is the fallback for any OS that refuses the stronger policy
/// (both are non-activating, which is the property the tests depend on).
///
/// Call ``prepare()`` before creating a hosted window. It is idempotent and
/// costs one atomic read after the first call.
enum HostedWindowSupport {
    private static let applyOnce: Bool = {
        let app = NSApplication.shared
        if app.setActivationPolicy(.prohibited) { return true }
        return app.setActivationPolicy(.accessory)
    }()

    /// Puts the test process into a non-activating activation policy, once.
    @discardableResult
    static func prepare() -> Bool { applyOnce }

    /// The policy actually in force, for the guard test below.
    static var currentPolicy: NSApplication.ActivationPolicy {
        NSApplication.shared.activationPolicy()
    }

    /// True when the process can never be brought to the front by ordering a
    /// window in — the property the owner's "stop the flashing" request needs.
    static var isNonActivating: Bool {
        currentPolicy == .prohibited || currentPolicy == .accessory
    }
}
