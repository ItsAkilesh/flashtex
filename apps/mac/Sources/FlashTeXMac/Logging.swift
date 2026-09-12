import Foundation

/// Optional line-oriented log sink for automation: set `FLASHTEX_LOG=<path>` and
/// the shell appends worker/bridge status lines (`ISO8601 tab message`). Nothing is
/// written when the variable is unset. Used by scripts/launch-check.sh.
enum FlashTeXLog {
    private static let url: URL? = ProcessInfo.processInfo.environment["FLASHTEX_LOG"].map { URL(fileURLWithPath: $0) }
    private static let queue = DispatchQueue(label: "flashtex.log")
    private static let stamp = ISO8601DateFormatter()

    static var isEnabled: Bool { url != nil }

    static func write(_ message: String) {
        guard let url else { return }
        let line = stamp.string(from: Date()) + "\t" + message + "\n"
        queue.async {
            if let h = try? FileHandle(forWritingTo: url) {
                defer { try? h.close() }
                _ = try? h.seekToEnd()
                try? h.write(contentsOf: Data(line.utf8))
            } else {
                try? Data(line.utf8).write(to: url)
            }
        }
    }
}
