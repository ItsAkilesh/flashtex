import AppKit
import XCTest
@testable import FlashTeXMac

/// The guard for the owner's "stop the windows flashing" request: every hosted
/// test calls `HostedWindowSupport.prepare()` before building its `NSWindow`,
/// and after that the test process must be unable to activate itself. If a new
/// hosted test forgets the call, the first suite to run still installs the
/// policy — but this test fails loudly if the policy itself ever stops taking,
/// which is the failure mode that would put windows back over the owner's work.
@MainActor
final class HostedWindowSupportTests: XCTestCase {

    func testPrepareInstallsANonActivatingPolicy() {
        XCTAssertTrue(HostedWindowSupport.prepare(), "the process refused both .prohibited and .accessory")
        XCTAssertTrue(HostedWindowSupport.isNonActivating,
                      "activation policy is \(HostedWindowSupport.currentPolicy.rawValue); a regular app pulls itself forward when a window is ordered in")
    }

    func testPrepareIsIdempotent() {
        let first = HostedWindowSupport.prepare()
        let policy = HostedWindowSupport.currentPolicy
        for _ in 0..<5 { XCTAssertEqual(HostedWindowSupport.prepare(), first) }
        XCTAssertEqual(HostedWindowSupport.currentPolicy, policy)
    }

    /// Ordering a hosted window front must not leave the process active. This is
    /// the behaviour the owner actually sees; the policy is only the mechanism.
    func testOrderingAHostedWindowFrontDoesNotActivateTheApp() {
        HostedWindowSupport.prepare()
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 200, height: 120),
                              styleMask: [.titled], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer { window.orderOut(nil) }
        window.orderFrontRegardless()
        XCTAssertFalse(NSApplication.shared.isActive, "the test process activated itself by ordering a window front")
    }

    /// Every hosted window site must ask for the policy first, so a fresh
    /// process cannot flash before whichever suite happens to run first.
    func testEveryHostedWindowSiteCallsPrepareFirst() throws {
        let testsDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
        let files = FileManager.default.enumerator(at: testsDir, includingPropertiesForKeys: nil)?
            .compactMap { $0 as? URL }
            .filter { $0.pathExtension == "swift" && $0.lastPathComponent != "HostedWindowSupport.swift" } ?? []
        XCTAssertFalse(files.isEmpty, "no test sources found under \(testsDir.path)")
        var offenders: [String] = []
        var sites = 0
        for file in files {
            guard let text = try? String(contentsOf: file, encoding: .utf8) else { continue }
            let lines = text.components(separatedBy: "\n")
            for (i, line) in lines.enumerated() where line.contains("NSWindow(") {
                if file.lastPathComponent == "HostedWindowSupportTests.swift" { continue }
                sites += 1
                let previous = i > 0 ? lines[i - 1] : ""
                if !previous.contains("HostedWindowSupport.prepare()") {
                    offenders.append("\(file.lastPathComponent):\(i + 1)")
                }
            }
        }
        XCTAssertGreaterThan(sites, 0, "the scan found no hosted windows, so it is not guarding anything")
        XCTAssertEqual(offenders, [], "hosted windows built without HostedWindowSupport.prepare() on the line above")
    }
}
