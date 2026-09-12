import AppKit
import CoreText
import FlashTeXProtocol

/// Preview/export faces. LaTeX's default is Computer Modern; we use Latin Modern
/// (GUST Font License) when it can be found, falling back to Times-Roman (the
/// Core-14 face the current compiler measures with). The face name reported by
/// the layout producer wins once runtime carries font identity; until then the
/// preview picks by `FLASHTEX_PREVIEW_FACE` (`latin-modern` | `times`).
enum PreviewFonts {
    enum Face: String { case latinModern = "latin-modern", times }

    static let latinModernSearchPaths: [String] = [
        ProcessInfo.processInfo.environment["FLASHTEX_LM_DIR"],
        Bundle.main.resourceURL?.appendingPathComponent("Fonts").path,
        // Repository copy (apps/mac/Fonts) for development and tests.
        URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().appendingPathComponent("Fonts").path,
        "/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm",
        "/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm",
        "/Library/TeX/Root/texmf-dist/fonts/opentype/public/lm",
    ].compactMap { $0 }

    /// PostScript names after registration; nil if Latin Modern was not found.
    private(set) static var latinModernRegistered: Bool = {
        for dir in latinModernSearchPaths {
            let url = URL(fileURLWithPath: dir)
            guard let files = try? FileManager.default.contentsOfDirectory(at: url, includingPropertiesForKeys: nil) else { continue }
            let otfs = files.filter { $0.pathExtension == "otf" && $0.lastPathComponent.hasPrefix("lmroman") }
            guard !otfs.isEmpty else { continue }
            CTFontManagerRegisterFontURLs(otfs as CFArray, .process, true, nil)
            return true
        }
        return false
    }()

    /// Set by the shell from the attached producer: `flashtex-render` (the new
    /// pipeline, Latin Modern metrics) → `.latinModern`; `flashtex-compiler`
    /// (Core-14 Times metrics today) → `.times`. `FLASHTEX_PREVIEW_FACE` overrides.
    static var producerFace: Face = .times

    /// `FLASHTEX_PREVIEW_FACE` read once: `ProcessInfo.environment` copies the
    /// whole environment on every access and this is consulted per drawn item.
    private static let environmentFace: Face? = Face(rawValue: ProcessInfo.processInfo.environment["FLASHTEX_PREVIEW_FACE"] ?? "")

    static var requested: Face { environmentFace ?? producerFace }

    /// Active face: Latin Modern when requested and registered, else Times.
    static var active: Face { requested == .latinModern && latinModernRegistered ? .latinModern : .times }

    /// Producer heuristic until runtime carries font identity.
    static func face(forProducer executableName: String?) -> Face {
        guard let n = executableName?.lowercased() else { return .times }
        return n.contains("render") ? .latinModern : .times
    }

    /// Legacy selection (no font hint): the active face, regular weight/style.
    static func postScriptName(size: Double, bold: Bool = false, italic: Bool = false) -> String {
        postScriptName(face: active, size: size, bold: bold, italic: italic)
    }

    /// Latin Modern optical size by nominal point size, matching LaTeX's choice
    /// (lmroman5/7/8/9/10/12/17 masters).
    static func postScriptName(face: Face, size: Double, bold: Bool, italic: Bool) -> String {
        switch face {
        case .times:
            switch (bold, italic) {
            case (false, false): return "Times-Roman"
            case (true, false): return "Times-Bold"
            case (false, true): return "Times-Italic"
            case (true, true): return "Times-BoldItalic"
            }
        case .latinModern:
            let master: Int = size < 6 ? 5 : size < 7.5 ? 7 : size < 8.5 ? 8 : size < 9.5 ? 9 : size < 11.5 ? 10 : size < 14.5 ? 12 : 17
            let style = bold && italic ? "BoldItalic" : bold ? "Bold" : italic ? "Italic" : "Regular"
            // e.g. LMRoman10-Regular, LMRoman12-Bold; 17 has only Regular.
            return master == 17 && (bold || italic) ? "LMRoman12-\(style)" : "LMRoman\(master)-\(style)"
        }
    }

    static func ctFont(size: Double) -> CTFont {
        CTFontCreateWithName(postScriptName(size: size) as CFString, size, nil)
    }

    // MARK: font-hints-v1

    /// A requested family the preview could not honor and the face used instead.
    struct Substitution: Hashable {
        var family: String
        var weight: RuntimeV1.PageItem.FontHint.Weight
        var style: RuntimeV1.PageItem.FontHint.Style
        var usedFace: String
        var requested: String {
            family + (weight == .bold ? " bold" : "") + (style == .italic ? " italic" : "")
        }
        var description: String { "font substituted: \(requested) → \(usedFace)" }
    }

    struct Resolved: Equatable {
        var postScriptName: String
        /// Non-nil when the requested family was not available and another face
        /// was used; the requested metrics were then NOT preserved.
        var substitution: Substitution?
    }

    /// Families that map to the registered Latin Modern Roman masters.
    static func isLatinModernFamily(_ family: String) -> Bool {
        let f = family.lowercased().trimmingCharacters(in: .whitespaces)
        return f.hasPrefix("latin modern") || f.hasPrefix("lmroman") || f == "lm roman" || f == "computer modern" || f.hasPrefix("computer modern")
    }

    /// Families that map to the Core-14 Times faces.
    static func isTimesFamily(_ family: String) -> Bool { core14Family(family) == .times }

    /// The three Core-14 text families macOS ships (`Times-Roman`, `Helvetica`,
    /// `Courier` with their Bold/Italic-or-Oblique faces). `flashtex-compiler`
    /// names its hint families after the Core-14 *face* it measured with
    /// (`Times-Bold`, `Times-Italic`, …); the face suffix is accepted as an alias
    /// of the family and the hint's own weight/style pick the face.
    enum Core14Family: String { case times, helvetica, courier }

    static func core14Family(_ family: String) -> Core14Family? {
        var f = family.lowercased().trimmingCharacters(in: .whitespaces)
        for suffix in ["-bolditalic", "-boldoblique", "-bold", "-italic", "-oblique", "-roman"] where f.hasSuffix(suffix) {
            f = String(f.dropLast(suffix.count)); break
        }
        switch f {
        case "times", "times new roman", "times roman", "timesnewroman": return .times
        case "helvetica", "helvetica neue", "arial": return .helvetica
        case "courier", "courier new": return .courier
        default: return nil
        }
    }

    /// Core-14 PostScript face for a family at the requested weight/style.
    static func core14PostScriptName(_ family: Core14Family, bold: Bool, italic: Bool) -> String {
        switch family {
        case .times: return postScriptName(face: .times, size: 10, bold: bold, italic: italic)
        case .helvetica, .courier:
            let base = family == .helvetica ? "Helvetica" : "Courier"
            switch (bold, italic) {
            case (false, false): return base
            case (true, false): return base + "-Bold"
            case (false, true): return base + "-Oblique"
            case (true, true): return base + "-BoldOblique"
            }
        }
    }

    /// Resolves an explicit `font-hints-v1` hint to a PostScript face. Latin
    /// Modern families use the registered LM masters (weight/style honored);
    /// Core-14 families (Times, Helvetica, Courier) use their Core-14 faces;
    /// anything else — or Latin Modern when it is not registered on this
    /// machine — falls back to Times with the requested weight/style and is
    /// reported as a substitution. A nil hint is legacy selection
    /// (`postScriptName(size:)`), never a substitution.
    static func resolve(hint: RuntimeV1.PageItem.FontHint?, size: Double) -> Resolved {
        guard let hint else { return Resolved(postScriptName: postScriptName(size: size), substitution: nil) }
        let bold = hint.weight == .bold, italic = hint.style == .italic
        if isLatinModernFamily(hint.family), latinModernRegistered {
            return Resolved(postScriptName: postScriptName(face: .latinModern, size: size, bold: bold, italic: italic), substitution: nil)
        }
        if let family = core14Family(hint.family) {
            return Resolved(postScriptName: core14PostScriptName(family, bold: bold, italic: italic), substitution: nil)
        }
        let times = postScriptName(face: .times, size: size, bold: bold, italic: italic)
        return Resolved(postScriptName: times,
                        substitution: Substitution(family: hint.family, weight: hint.weight, style: hint.style, usedFace: times))
    }

    /// Every distinct substitution a result would need, in first-seen order.
    static func substitutions(in result: RuntimeV1.CompileResult) -> [Substitution] {
        var seen = Set<Substitution>(), out: [Substitution] = []
        for page in result.pages {
            for case .text(let t) in page.items {
                if let sub = resolve(hint: t.font, size: t.fontSizePt).substitution, seen.insert(sub).inserted { out.append(sub) }
            }
        }
        return out
    }
}
