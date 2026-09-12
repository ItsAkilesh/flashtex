import Foundation

/// The rooted Latin Modern TeX metrics shipped in the app bundle (GH36) and
/// the producer environment that connects them to `flashtex-render`.
///
/// The render pipeline resolves TFMs from `FLASHTEX_TFM_DIRS` (colon
/// separated) before any inferred directory, and derives the `texmf` root of
/// its digest-bound required 12 pt set by stripping `/fonts/tfm/public/lm`
/// from those entries (`<root>/doc/fonts/lm/GUST-FONT-LICENSE.TXT` must sit
/// beside them). `make-app.sh` stages the five pinned TFMs and the license
/// under `Contents/Resources/texmf/…` with that exact shape; this type finds
/// that directory and prepends it to the child producer's `FLASHTEX_TFM_DIRS`
/// so the packaged app never depends on a host TeX installation for the
/// 10 pt / 12 pt text and 12 pt roman-math metrics. Explicit user entries are
/// kept, in their order, after the bundled directory; every other variable is
/// passed through untouched. Both producer routes share it: the directly
/// attached worker (`WorkerClient`) and the helper-spawned producer
/// (`PreviewControllerClient` → `flashtex-preview-controller` → compiler
/// child, which inherits the helper's environment).
enum BundledMetrics {
    /// The producer's TFM search-path variable (colon separated).
    static let environmentKey = "FLASHTEX_TFM_DIRS"

    /// Where the pinned metrics live below a `texmf` root; the suffix the
    /// producer strips to find `doc/fonts/lm/GUST-FONT-LICENSE.TXT`.
    static let tfmSubdirectory = "fonts/tfm/public/lm"

    /// The `texmf` roots probed, in order: the app bundle's
    /// `Contents/Resources/texmf`, then the repository's vendored
    /// `apps/mac/Fonts/texmf` (development builds and tests).
    static var candidateRoots: [URL] {
        var roots: [URL] = []
        if let resources = Bundle.main.resourceURL {
            roots.append(resources.appendingPathComponent("texmf"))
        }
        let repositoryFonts = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
            .appendingPathComponent("Fonts").appendingPathComponent("texmf")
        roots.append(repositoryFonts)
        return roots
    }

    /// The first candidate root that actually carries the rooted directory
    /// (`<root>/fonts/tfm/public/lm` exists and is a directory), or nil when
    /// this executable ships without the metrics (a bare `swift build`
    /// product outside the repository).
    static func tfmDirectory(roots: [URL] = candidateRoots) -> URL? {
        let fm = FileManager.default
        for root in roots {
            let dir = root.appendingPathComponent(tfmSubdirectory)
            var isDir: ObjCBool = false
            if fm.fileExists(atPath: dir.path, isDirectory: &isDir), isDir.boolValue {
                return dir.standardizedFileURL
            }
        }
        return nil
    }

    /// `existing` (`FLASHTEX_TFM_DIRS` as the user set it, possibly nil or
    /// empty) with `bundled` prepended: the bundled path first, then every
    /// non-empty explicit entry in its original order, minus repeats of the
    /// bundled path. Explicit entries stay in effect for anything the bundle
    /// does not carry (bold/italic/other design sizes).
    static func prepending(_ bundled: String, to existing: String?) -> String {
        var entries = [bundled]
        for entry in (existing ?? "").split(separator: ":", omittingEmptySubsequences: true) {
            let s = String(entry)
            if s != bundled { entries.append(s) }
        }
        return entries.joined(separator: ":")
    }

    /// The environment to launch a producer (or the helper that spawns one)
    /// with: `base` unchanged except `FLASHTEX_TFM_DIRS`, which gets the
    /// bundled directory prepended. Returns `base` untouched when no bundled
    /// directory exists, so a bare build behaves exactly as before.
    static func producerEnvironment(base: [String: String] = ProcessInfo.processInfo.environment,
                                    bundledDirectory: URL? = tfmDirectory()) -> [String: String] {
        guard let bundledDirectory else { return base }
        var env = base
        env[environmentKey] = prepending(bundledDirectory.path, to: base[environmentKey])
        return env
    }
}
