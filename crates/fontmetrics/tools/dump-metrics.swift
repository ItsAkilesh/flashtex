// dump-metrics.swift — generate crates/fontmetrics/src/tables.rs from CoreText.
//
// Usage (from the repository root, on macOS):
//
//   swiftc -O crates/fontmetrics/tools/dump-metrics.swift -o /tmp/dump-metrics
//   /tmp/dump-metrics > crates/fontmetrics/src/tables.rs
//
// or, without a separate compile step:
//
//   swift crates/fontmetrics/tools/dump-metrics.swift > crates/fontmetrics/src/tables.rs
//
// What it measures
// ----------------
// For each of the seven faces below it creates a CTFont at 1000 points, so one
// CoreText advance unit equals one 1000-unit-per-em glyph-space unit (the unit
// used by PDF base-14 AFM metrics and by PDF /Widths arrays). For every Unicode
// scalar in the WinAnsi (Windows-1252) repertoire it looks up the glyph with
// CTFontGetGlyphsForCharacters and reads its horizontal advance with
// CTFontGetAdvancesForGlyphs. Advances are per glyph: no kerning, no ligatures,
// no shaping. The result is rounded to the nearest integer and emitted as a
// sorted `(codepoint, advance)` table.
//
// The macOS system fonts are 2048 units/em TrueType, so the raw values are
// fractions like 722.16796875 (= 1479/2048 * 1000); rounding to 722 recovers the
// integer Adobe metric the base-14 fonts are defined with. The maximum rounding
// error is therefore 0.5 units per glyph (0.006 pt at 12 pt).
//
// Vertical metrics are CTFontGetAscent / GetDescent / GetCapHeight / GetXHeight
// at the same size, rounded. `descender` is emitted NEGATIVE, following the PDF
// FontDescriptor /Descent convention. `space` is the advance of U+0020 and
// `notdef` is the advance of U+003F ('?'), which is the fallback width the Rust
// crate charges for characters outside the table (crates/pdf writes such
// characters as '?').
//
// Codepoints whose glyph lookup fails are omitted from the table and reported
// on stderr; the Rust side then treats them as unknown.

import CoreText
import Foundation

struct FaceSpec {
    let rustName: String      // identifier used for the Rust statics
    let postScriptName: String
}

let faces: [FaceSpec] = [
    FaceSpec(rustName: "TIMES_ROMAN",       postScriptName: "Times-Roman"),
    FaceSpec(rustName: "TIMES_BOLD",        postScriptName: "Times-Bold"),
    FaceSpec(rustName: "TIMES_ITALIC",      postScriptName: "Times-Italic"),
    FaceSpec(rustName: "TIMES_BOLD_ITALIC", postScriptName: "Times-BoldItalic"),
    FaceSpec(rustName: "HELVETICA",         postScriptName: "Helvetica"),
    FaceSpec(rustName: "HELVETICA_BOLD",    postScriptName: "Helvetica-Bold"),
    FaceSpec(rustName: "COURIER",           postScriptName: "Courier"),
]

/// The WinAnsi (Windows-1252) repertoire as Unicode scalars, sorted.
func winAnsiScalars() -> [UInt32] {
    var set: [UInt32] = []
    set.append(contentsOf: (0x20...0x7E).map { UInt32($0) })
    set.append(contentsOf: (0xA0...0xFF).map { UInt32($0) })
    // Bytes 0x80–0x9F of Windows-1252 that map to non-Latin-1 scalars.
    let specials: [UInt32] = [
        0x20AC, // 0x80 €
        0x201A, // 0x82 ‚
        0x0192, // 0x83 ƒ
        0x201E, // 0x84 „
        0x2026, // 0x85 …
        0x2020, // 0x86 †
        0x2021, // 0x87 ‡
        0x02C6, // 0x88 ˆ
        0x2030, // 0x89 ‰
        0x0160, // 0x8A Š
        0x2039, // 0x8B ‹
        0x0152, // 0x8C Œ
        0x017D, // 0x8E Ž
        0x2018, // 0x91 ‘
        0x2019, // 0x92 ’
        0x201C, // 0x93 “
        0x201D, // 0x94 ”
        0x2022, // 0x95 •
        0x2013, // 0x96 –
        0x2014, // 0x97 —
        0x02DC, // 0x98 ˜
        0x2122, // 0x99 ™
        0x0161, // 0x9A š
        0x203A, // 0x9B ›
        0x0153, // 0x9C œ
        0x017E, // 0x9E ž
        0x0178, // 0x9F Ÿ
    ]
    set.append(contentsOf: specials)
    return set.sorted()
}

func roundToInt(_ v: CGFloat) -> Int { Int((Double(v)).rounded()) }

func osVersionString() -> String {
    let v = ProcessInfo.processInfo.operatingSystemVersion
    var build = "unknown"
    if let dict = NSDictionary(contentsOfFile: "/System/Library/CoreServices/SystemVersion.plist"),
       let b = dict["ProductBuildVersion"] as? String {
        build = b
    }
    return "macOS \(v.majorVersion).\(v.minorVersion).\(v.patchVersion) (build \(build))"
}

var out = ""
func emit(_ s: String) { out += s + "\n" }

let scalars = winAnsiScalars()
let osVersion = osVersionString()

emit("//! Advance-width and vertical-metric tables for the PDF base-14 text faces.")
emit("//!")
emit("//! GENERATED FILE — do not edit by hand. Regenerate with")
emit("//! `swift crates/fontmetrics/tools/dump-metrics.swift > crates/fontmetrics/src/tables.rs`.")
emit("//!")
emit("//! Source: the macOS system fonts, measured through CoreText")
emit("//! (`CTFontCreateWithName` at 1000 pt, `CTFontGetGlyphsForCharacters`,")
emit("//! `CTFontGetAdvancesForGlyphs`, per glyph, no kerning or ligatures) and")
emit("//! rounded to the nearest 1/1000 em. These are Apple's metric-compatible")
emit("//! equivalents of the base-14 fonts, not Adobe's AFM files.")
emit("//!")
emit("//! Generated on: \(osVersion)")
emit("//! Repertoire: WinAnsi (U+0020–U+007E, U+00A0–U+00FF, and the 27 Windows-1252")
emit("//! 0x80–0x9F specials), \(scalars.count) code points per face.")
emit("")
emit("use crate::FaceMetrics;")
emit("")

for face in faces {
    let font = CTFontCreateWithName(face.postScriptName as CFString, 1000, nil)
    let resolved = CTFontCopyPostScriptName(font) as String
    let familyName = CTFontCopyFamilyName(font) as String
    let path = (CTFontCopyAttribute(font, kCTFontURLAttribute) as? URL)?.path ?? "unknown"
    let upem = CTFontGetUnitsPerEm(font)

    if resolved != face.postScriptName {
        FileHandle.standardError.write(
            "warning: requested \(face.postScriptName) but CoreText resolved \(resolved)\n".data(using: .utf8)!)
    }

    var rows: [(UInt32, Int)] = []
    var missing: [UInt32] = []
    for cp in scalars {
        guard let scalar = Unicode.Scalar(cp) else { missing.append(cp); continue }
        var units = Array(String(Character(scalar)).utf16)
        var glyphs = [CGGlyph](repeating: 0, count: units.count)
        let ok = CTFontGetGlyphsForCharacters(font, &units, &glyphs, units.count)
        // All WinAnsi scalars are in the BMP, so units.count == 1 here.
        if !ok || glyphs[0] == 0 || units.count != 1 {
            missing.append(cp)
            continue
        }
        var advances = [CGSize](repeating: .zero, count: 1)
        _ = CTFontGetAdvancesForGlyphs(font, .horizontal, glyphs, &advances, 1)
        rows.append((cp, roundToInt(advances[0].width)))
    }
    for cp in missing {
        FileHandle.standardError.write(
            "warning: \(face.postScriptName): no glyph for U+\(String(format: "%04X", cp)); omitted\n".data(using: .utf8)!)
    }

    func advanceOf(_ cp: UInt32) -> Int {
        rows.first(where: { $0.0 == cp })?.1 ?? 0
    }
    let ascender = roundToInt(CTFontGetAscent(font))
    let descender = -roundToInt(CTFontGetDescent(font))
    let capHeight = roundToInt(CTFontGetCapHeight(font))
    let xHeight = roundToInt(CTFontGetXHeight(font))
    let space = advanceOf(0x20)
    let notdef = advanceOf(0x3F)

    emit("// \(face.postScriptName): PostScript name \"\(resolved)\", family \"\(familyName)\",")
    emit("// file \(path), \(upem) units/em in the source font.")
    emit("#[rustfmt::skip]")
    emit("pub static \(face.rustName): [(u32, u16); \(rows.count)] = [")
    for (cp, adv) in rows {
        let hex = String(format: "0x%04X", cp)
        let scalar = Unicode.Scalar(cp)!
        // Keep the comment ASCII-safe for readers; show the character where printable.
        let shown: String
        if cp == 0x20 { shown = "space" }
        else if cp == 0xA0 { shown = "no-break space" }
        else if cp == 0xAD { shown = "soft hyphen" }
        else { shown = String(Character(scalar)) }
        emit("    (\(hex), \(adv)), // \(shown)")
    }
    emit("];")
    emit("")
    emit("pub static \(face.rustName)_FACE: FaceMetrics = FaceMetrics {")
    emit("    postscript_name: \"\(resolved)\",")
    emit("    ascender: \(ascender),")
    emit("    descender: \(descender),")
    emit("    cap_height: \(capHeight),")
    emit("    x_height: \(xHeight),")
    emit("    space: \(space),")
    emit("    notdef: \(notdef),")
    emit("    advances: &\(face.rustName),")
    emit("};")
    emit("")
}

// Exactly one trailing newline, so the output is byte-stable under rustfmt.
while out.hasSuffix("\n\n") { out.removeLast() }
FileHandle.standardOutput.write(out.data(using: .utf8)!)
