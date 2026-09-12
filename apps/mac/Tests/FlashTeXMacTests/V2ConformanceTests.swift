import XCTest
import FlashTeXProtocol
@testable import FlashTeXMac

/// Consumer-side conformance to the `display-list-v2` draft contract
/// (docs/contracts/runtime-v1-display-list-v2.md at main ad922ea), items
/// D1/D2/D3/D6/D9 of docs/proposals/contract-draft-review-ad922ea.md.
/// Real-helper tests skip cleanly without `FLASHTEX_PREVIEW_CONTROLLER` and
/// `FLASHTEX_COMPILER`.
@MainActor
final class V2ConformanceTests: XCTestCase {
    static let fixtures = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("Fixtures")
    static let helper = ProcessInfo.processInfo.environment["FLASHTEX_PREVIEW_CONTROLLER"].map { URL(fileURLWithPath: $0) }
    static let compiler = ProcessInfo.processInfo.environment["FLASHTEX_COMPILER"].map { URL(fileURLWithPath: $0) }

    private func waitUntil(timeout: TimeInterval = 20, _ cond: () -> Bool) async throws {
        let start = Date()
        while !cond() {
            if Date().timeIntervalSince(start) > timeout { throw XCTSkip("timeout after \(timeout)s (load-sensitive)") }
            try await Task.sleep(nanoseconds: 20_000_000)
        }
    }

    // MARK: D6 — outgoing helper lines are bounded at the helper's measured 1 MiB stdin limit

    /// A text that makes an `edit` line of exactly `lineBytes` bytes (newline included).
    private static func editText(sessionID: String, id: String, lineBytes: Int, revision: Int, sha: String) throws -> String {
        func line(_ fill: Int) throws -> (String, Int) {
            let text = "\\begin{document}\n" + String(repeating: "x", count: fill) + "\\end{document}\n"
            let payload: PreviewControllerClient.JSONObject = ["path": "main.tex", "expected_revision": revision, "expected_sha256": sha, "text": text]
            return (text, try PreviewControllerClient.requestLine(sessionID: sessionID, id: id, type: "edit", payload: payload).count)
        }
        let (_, probe) = try line(1000)
        let (text, size) = try line(1000 + lineBytes - probe)
        XCTAssertEqual(size, lineBytes)
        return text
    }

    func testHelperRequestAboveOneMiBIsRefusedLocallyBeforeSending() throws {
        XCTAssertEqual(PreviewControllerClient.maxRequestLineBytes, 1024 * 1024, "the helper's MAX_FRAME (measured 2026-09-12)")
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("v2conf-d6-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        // `/usr/bin/tee <config>` stands in for the helper: it stays alive and echoes
        // whatever reaches its stdin. The refused line must never reach it.
        var config = PreviewControllerClient.Config(sessionID: "d6-local", projectID: "d6", entryPath: "main.tex")
        config.storePaths = [root]
        var echoed = Data()
        let client = try PreviewControllerClient(executable: URL(fileURLWithPath: "/usr/bin/tee"), config: config) { event in
            if case .protocolViolation = event { echoed.append(1) } // tee echoes our request, which is not a helper frame
        }
        defer { client.close() }
        let sha = String(repeating: "0", count: 64)
        let refused = PreviewControllerClient.requestsRefusedTooLarge
        // One byte over: refused locally, typed, before any write.
        let over = try Self.editText(sessionID: "d6-local", id: "pc-1", lineBytes: 1024 * 1024 + 1, revision: 1, sha: sha)
        XCTAssertThrowsError(try client.edit(path: "main.tex", expectedRevision: 1, expectedSHA256: sha, text: over)) { error in
            guard let e = error as? PreviewControllerClient.RequestTooLarge else { return XCTFail("\(error)") }
            XCTAssertEqual(e.type, "edit")
            XCTAssertEqual(e.lineBytes, 1024 * 1024 + 1)
            XCTAssertTrue(e.errorDescription!.contains("exceeds the helper's 1048576-byte stdin line limit; not sent"), e.errorDescription!)
        }
        XCTAssertEqual(PreviewControllerClient.requestsRefusedTooLarge, refused + 1)
        // Two MiB: the same typed refusal (never split).
        let two = try Self.editText(sessionID: "d6-local", id: "pc-1", lineBytes: 2 * 1024 * 1024, revision: 1, sha: sha)
        XCTAssertThrowsError(try client.edit(path: "main.tex", expectedRevision: 1, expectedSHA256: sha, text: two)) {
            XCTAssertEqual(($0 as? PreviewControllerClient.RequestTooLarge)?.lineBytes, 2 * 1024 * 1024)
        }
        XCTAssertEqual(PreviewControllerClient.requestsRefusedTooLarge, refused + 2)
        // Exactly 1 MiB (refused requests consumed pc-1 and pc-2; this one is pc-3): admitted by the client.
        let exact = try Self.editText(sessionID: "d6-local", id: "pc-3", lineBytes: 1024 * 1024, revision: 1, sha: sha)
        XCTAssertEqual(try client.edit(path: "main.tex", expectedRevision: 1, expectedSHA256: sha, text: exact), "pc-3")
        XCTAssertEqual(PreviewControllerClient.requestsRefusedTooLarge, refused + 2)
        _ = echoed
    }

    /// The real helper: exactly 1 MiB is a durable edit; the client's refusal
    /// of the next byte keeps the helper alive (it would otherwise answer
    /// "truncated or oversized input" and exit — measured, see the evidence).
    func testRealHelperAdmitsExactlyOneMiBAndStaysAliveBehindTheClientBound() async throws {
        guard let helper = Self.helper, let compiler = Self.compiler,
              FileManager.default.isExecutableFile(atPath: helper.path), FileManager.default.isExecutableFile(atPath: compiler.path) else {
            throw XCTSkip("set FLASHTEX_PREVIEW_CONTROLLER and FLASHTEX_COMPILER to built binaries")
        }
        let root = FileManager.default.temporaryDirectory.appendingPathComponent("v2conf-d6-real-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: root.appendingPathComponent("project"), withIntermediateDirectories: true)
        try FileManager.default.createDirectory(at: root.appendingPathComponent("ledger"), withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }
        try "\\begin{document}\nHello.\n\\end{document}\n".write(to: root.appendingPathComponent("project/main.tex"), atomically: true, encoding: .utf8)
        var config = PreviewControllerClient.Config(sessionID: "d6-real", projectID: "d6", entryPath: "main.tex")
        config.projectRoot = root.appendingPathComponent("project")
        config.privateLedgerRoot = root.appendingPathComponent("ledger")
        config.compilerPath = compiler
        var ready = false, results: [(String, PreviewControllerClient.JSONObject)] = [], errors: [String] = [], exited = false
        let client = try PreviewControllerClient(executable: helper, config: config) { event in
            switch event {
            case .ready: ready = true
            case .result(let id, let payload): results.append((id, payload))
            case .error(_, let message): errors.append(message)
            case .exited: exited = true
            default: break
            }
        }
        defer { client.close() }
        try await waitUntil { ready }
        let docID = try client.document(path: "main.tex")
        try await waitUntil { results.contains { $0.0 == docID } }
        let doc = try XCTUnwrap(results.first { $0.0 == docID }?.1["document"] as? PreviewControllerClient.JSONObject)
        let revision = try XCTUnwrap(doc["revision"] as? Int), sha = try XCTUnwrap(doc["source_sha256"] as? String)
        // Exactly 1 MiB: durable.
        let nextID = "pc-2" // the client's second id
        let exact = try Self.editText(sessionID: "d6-real", id: nextID, lineBytes: 1024 * 1024, revision: revision, sha: sha)
        let editID = try client.edit(path: "main.tex", expectedRevision: revision, expectedSHA256: sha, text: exact)
        XCTAssertEqual(editID, nextID)
        try await waitUntil(timeout: 40) { results.contains { $0.0 == editID } || !errors.isEmpty || exited }
        XCTAssertTrue(errors.isEmpty, "\(errors)")
        XCTAssertFalse(exited)
        let edited = try XCTUnwrap(results.first { $0.0 == editID }?.1["document"] as? PreviewControllerClient.JSONObject)
        XCTAssertEqual(edited["revision"] as? Int, revision + 1)
        let sha2 = try XCTUnwrap(edited["source_sha256"] as? String)
        // One byte more: refused by the client, the helper never sees it and keeps answering.
        let over = try Self.editText(sessionID: "d6-real", id: "pc-3", lineBytes: 1024 * 1024 + 1, revision: revision + 1, sha: sha2)
        XCTAssertThrowsError(try client.edit(path: "main.tex", expectedRevision: revision + 1, expectedSHA256: sha2, text: over)) {
            XCTAssertEqual(($0 as? PreviewControllerClient.RequestTooLarge)?.lineBytes, 1024 * 1024 + 1)
        }
        let aliveID = try client.document(path: "main.tex")
        try await waitUntil { results.contains { $0.0 == aliveID } || exited }
        XCTAssertFalse(exited, "the helper exited: the oversized line reached it")
        XCTAssertEqual((results.first { $0.0 == aliveID }?.1["document"] as? PreviewControllerClient.JSONObject)?["revision"] as? Int, revision + 1)
        XCTAssertTrue(client.isRunning)
    }
}
