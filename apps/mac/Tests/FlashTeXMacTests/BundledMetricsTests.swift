import CryptoKit
import Foundation
import XCTest
@testable import FlashTeXMac

/// GH36: the rooted Latin Modern metrics vendored under `apps/mac/Fonts/texmf`
/// (staged by `make-app.sh` into `Contents/Resources/texmf`) and the producer
/// environment that connects them (`BundledMetrics`).
final class BundledMetricsTests: XCTestCase {
    /// `apps/mac` of this checkout.
    private static let macDir = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
    private static let vendoredRoot = macDir.appendingPathComponent("Fonts/texmf")

    /// The Commander's pinned manifest entries for the rooted tree
    /// (`crates/rendering-core/docs/handoffs/native-assets/manifest.json`).
    private static let pinned: [(path: String, sha256: String, bytes: Int)] = [
        ("fonts/tfm/public/lm/ec-lmr10.tfm", "cd13479f463b9a575d053dd7bf0884daa46bfdeffe4b7f537c193861652ac9e5", 12056),
        ("fonts/tfm/public/lm/ec-lmr12.tfm", "299021120f0a29ef61278a2363903bd8defbb8faaade458eb79067342aecb56f", 12092),
        ("fonts/tfm/public/lm/rm-lmr12.tfm", "9d4e3d8e39a41b93d91f79c1c47d2297efb7b1af220b94860693c08361f227aa", 11888),
        ("fonts/tfm/public/lm/rm-lmr6.tfm", "eb0bfdf8db3ae1409639fac9c88f84923872500d882d9ff8dc37aff445c723fe", 11836),
        ("fonts/tfm/public/lm/rm-lmr8.tfm", "80bcbfd844d2310ac1d3bead45aee25e91b1a4a0a60ff1771959b9a1e90ec1a2", 11864),
        ("doc/fonts/lm/GUST-FONT-LICENSE.TXT", "49ea6cb9257bbee0a3979c48a774cd221550ac1c20c95549efe45fc99cc18050", 1377),
    ]

    private static func sha256Hex(_ data: Data) -> String {
        SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
    }

    // MARK: vendored tree

    func testVendoredTreeMatchesPinnedManifest() throws {
        for entry in Self.pinned {
            let url = Self.vendoredRoot.appendingPathComponent(entry.path)
            let data = try Data(contentsOf: url)
            XCTAssertEqual(data.count, entry.bytes, entry.path)
            XCTAssertEqual(Self.sha256Hex(data), entry.sha256, entry.path)
        }
    }

    // MARK: environment

    func testTFMDirectoryPrefersFirstRootThatExists() throws {
        let missing = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-no-texmf-\(UUID().uuidString)")
        let found = BundledMetrics.tfmDirectory(roots: [missing, Self.vendoredRoot])
        XCTAssertEqual(found?.path, Self.vendoredRoot.appendingPathComponent(BundledMetrics.tfmSubdirectory).standardizedFileURL.path)
        XCTAssertNil(BundledMetrics.tfmDirectory(roots: [missing]))
        // A file (not a directory) at the rooted path does not count.
        let file = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-texmf-file-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: file.appendingPathComponent("fonts/tfm/public"), withIntermediateDirectories: true)
        try Data("x".utf8).write(to: file.appendingPathComponent(BundledMetrics.tfmSubdirectory))
        defer { try? FileManager.default.removeItem(at: file) }
        XCTAssertNil(BundledMetrics.tfmDirectory(roots: [file]))
    }

    func testDefaultDiscoveryFindsTheRepositoryCopyFromASwiftBuildProduct() {
        // Tests run from a bare build product: the bundle has no Resources/texmf,
        // so the vendored repository tree is what the producer would get.
        XCTAssertEqual(BundledMetrics.tfmDirectory()?.path,
                       Self.vendoredRoot.appendingPathComponent(BundledMetrics.tfmSubdirectory).standardizedFileURL.path)
    }

    func testPrependingKeepsExplicitEntriesInOrderAfterTheBundledDirectory() {
        XCTAssertEqual(BundledMetrics.prepending("/b", to: nil), "/b")
        XCTAssertEqual(BundledMetrics.prepending("/b", to: ""), "/b")
        XCTAssertEqual(BundledMetrics.prepending("/b", to: "/u1:/u2"), "/b:/u1:/u2")
        XCTAssertEqual(BundledMetrics.prepending("/b", to: "/u1::/b:/u2:"), "/b:/u1:/u2", "empty entries dropped, bundled repeat dropped")
    }

    func testProducerEnvironmentPrependsAndPassesEverythingElseThrough() {
        let base = ["PATH": "/usr/bin", "FLASHTEX_TFM_DIRS": "/user/tfm", "FLASHTEX_RENDER": "/x"]
        let env = BundledMetrics.producerEnvironment(base: base, bundledDirectory: URL(fileURLWithPath: "/App/Contents/Resources/texmf/fonts/tfm/public/lm"))
        XCTAssertEqual(env["FLASHTEX_TFM_DIRS"], "/App/Contents/Resources/texmf/fonts/tfm/public/lm:/user/tfm")
        XCTAssertEqual(env["PATH"], "/usr/bin")
        XCTAssertEqual(env["FLASHTEX_RENDER"], "/x")
        XCTAssertEqual(env.count, 3)
        let unset = BundledMetrics.producerEnvironment(base: ["PATH": "/usr/bin"], bundledDirectory: URL(fileURLWithPath: "/b"))
        XCTAssertEqual(unset["FLASHTEX_TFM_DIRS"], "/b")
    }

    func testProducerEnvironmentIsUntouchedWithoutABundledDirectory() {
        let base = ["PATH": "/usr/bin", "FLASHTEX_TFM_DIRS": "/user/tfm"]
        XCTAssertEqual(BundledMetrics.producerEnvironment(base: base, bundledDirectory: nil), base)
    }

    // MARK: real producer (env route, host TeX excluded)

    /// Missing-metric diagnostic codes the producer emits (render-pipeline
    /// `typeset.rs`): non-required TFM absent, required 12 pt set absent,
    /// Latin Modern face substituted.
    private static let missingMetricCodes: Set<String> = ["tfm_missing", "required_metrics_unavailable", "font_unavailable"]

    private struct ProducerRun {
        var results: Int
        var missing: [(id: String, code: String, message: String)]
    }

    /// Runs `FLASHTEX_RENDER` on the 10 pt multi-document request (the
    /// verified capture's input) plus 12 pt text and 12 pt math, with an
    /// `env -i`-style environment (no PATH to texbin, empty HOME, no
    /// FLASHTEX_*/TEXMF*) and host TeX trees denied by sandbox-exec.
    private func runProducer(_ executable: String, tfmDirs: String?, home: URL) throws -> ProducerRun {
        let requests = [
            #"{"protocol_version":1,"id":"preview-1","type":"compile","payload":{"project_id":"p","revision":1,"entry_path":"main.tex","documents":[{"path":"chapter.tex","text":"Chapter text with \\(a+b\\).\n"},{"path":"main.tex","text":"\\documentclass{article}\n\\begin{document}\nOffice AV fi.\\input{chapter}\n\\end{document}\n"},{"path":"refs.bib","text":"@article{sample, title={Example}, author={A. Author}, year={2026}}\n"}]}}"#,
            #"{"protocol_version":1,"id":"text-12pt","type":"compile","payload":{"project_id":"p","revision":2,"entry_path":"main.tex","documents":[{"path":"main.tex","text":"\\documentclass[12pt]{article}\n\\begin{document}\nOffice AV fi. Twelve point text.\n\\end{document}\n"}],"layout_capabilities":["display-list-v2"]}}"#,
            #"{"protocol_version":1,"id":"math-12pt","type":"compile","payload":{"project_id":"p","revision":3,"entry_path":"main.tex","documents":[{"path":"main.tex","text":"\\documentclass[12pt]{article}\n\\begin{document}\nBody $x^2 + y_1$ text. \\[ \\sum_{i=1}^{n} a_i \\]\n\\end{document}\n"}],"layout_capabilities":["display-list-v2"]}}"#,
        ]
        let profile = home.appendingPathComponent("no-host-tex.sb")
        try """
        (version 1)
        (allow default)
        (deny file-read* (subpath "/usr/local/texlive"))
        (deny file-read* (subpath "/Library/TeX"))
        (deny file-read* (subpath "/usr/share/texmf"))
        (deny file-read* (subpath "/usr/share/texlive"))

        """.write(to: profile, atomically: true, encoding: .utf8)
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/sandbox-exec")
        process.arguments = ["-f", profile.path, executable]
        var env = ["PATH": "/usr/bin:/bin", "HOME": home.path,
                   // The OTFs: a bare producer outside a bundle has no ../Resources/Fonts.
                   "FLASHTEX_FONT_DIRS": Self.macDir.appendingPathComponent("Fonts").path]
        if let tfmDirs { env["FLASHTEX_TFM_DIRS"] = tfmDirs }
        process.environment = env
        process.currentDirectoryURL = home
        let stdin = Pipe(), stdout = Pipe()
        process.standardInput = stdin
        process.standardOutput = stdout
        process.standardError = FileHandle.nullDevice
        try process.run()
        try stdin.fileHandleForWriting.write(contentsOf: Data((requests.joined(separator: "\n") + "\n").utf8))
        try stdin.fileHandleForWriting.close()
        let output = stdout.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        var run = ProducerRun(results: 0, missing: [])
        for line in output.split(separator: UInt8(ascii: "\n")) {
            guard let obj = try JSONSerialization.jsonObject(with: Data(line)) as? [String: Any],
                  obj["type"] as? String == "compile_result" else { continue }
            run.results += 1
            let id = obj["id"] as? String ?? "?"
            let diags = (obj["payload"] as? [String: Any])?["diagnostics"] as? [[String: Any]] ?? []
            for d in diags {
                let code = d["code"] as? String ?? ""
                if Self.missingMetricCodes.contains(code) {
                    run.missing.append((id, code, d["message"] as? String ?? ""))
                }
            }
        }
        return run
    }

    func testRealProducerHasNoMissingMetricsThroughTheEnvRouteWithHostTeXExcluded() throws {
        guard let render = ProcessInfo.processInfo.environment["FLASHTEX_RENDER"],
              FileManager.default.isExecutableFile(atPath: render) else {
            throw XCTSkip("FLASHTEX_RENDER not set to a built flashtex-render")
        }
        guard FileManager.default.isExecutableFile(atPath: "/usr/bin/sandbox-exec") else { throw XCTSkip("sandbox-exec unavailable") }
        let home = FileManager.default.temporaryDirectory.appendingPathComponent("flashtex-texmf-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: home, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: home) }

        // Exactly what WorkerClient / PreviewControllerClient hand the child.
        let env = BundledMetrics.producerEnvironment(base: [:], bundledDirectory: BundledMetrics.tfmDirectory(roots: [Self.vendoredRoot]))
        let tfmDirs = try XCTUnwrap(env[BundledMetrics.environmentKey])
        let routed = try runProducer(render, tfmDirs: tfmDirs, home: home)
        XCTAssertEqual(routed.results, 3)
        XCTAssertTrue(routed.missing.isEmpty, "env route must leave no missing-metric diagnostics: \(routed.missing)")

        // User entries preserved after the bundled directory: still clean.
        let withUser = try runProducer(render, tfmDirs: tfmDirs + ":" + home.appendingPathComponent("user").path, home: home)
        XCTAssertEqual(withUser.results, 3)
        XCTAssertTrue(withUser.missing.isEmpty, "\(withUser.missing)")

        // One metric removed from a copy of the tree: an explicit failure, never silence.
        let copy = home.appendingPathComponent("texmf")
        try FileManager.default.copyItem(at: Self.vendoredRoot, to: copy)
        try FileManager.default.removeItem(at: copy.appendingPathComponent("fonts/tfm/public/lm/ec-lmr10.tfm"))
        let removed = try runProducer(render, tfmDirs: copy.appendingPathComponent(BundledMetrics.tfmSubdirectory).path, home: home)
        XCTAssertEqual(removed.results, 3)
        XCTAssertTrue(removed.missing.contains { $0.id == "preview-1" && $0.message.contains("ec-lmr10.tfm") },
                      "10 pt request must name the missing ec-lmr10.tfm: \(removed.missing)")
    }
}
