// verify-against-coretext.swift — cross-check the committed Rust tables against
// a live CoreText measurement.
//
// Usage (from the repository root, on macOS):
//
//   swift crates/fontmetrics/tools/verify-against-coretext.swift [--tolerance PT] [--size 12] [face text ...]
//
// With no face/text arguments it runs a built-in set of sentences across all
// seven faces. For each case it:
//
//   1. measures the text with CoreText, glyph by glyph via
//      CTFontGetGlyphsForCharacters + CTFontGetAdvancesForGlyphs at `size` pt
//      (the same per-glyph, kerning-free path the tables were generated with),
//   2. runs `cargo run --quiet --example measure -- <face> <size> -` in
//      crates/fontmetrics, feeding the text on stdin, to get the Rust width
//      from the committed tables (stdin, not argv: Foundation's Process
//      rewrites arguments to decomposed NFD Unicode on macOS),
//   3. prints both, their difference, and PASS/FAIL.
//
// Pass/fail criterion: the tables are rounded to whole 1/1000-em units, while
// CoreText returns fractional advances from the 2048-unit/em system fonts, so
// each glyph can legitimately differ by up to 0.5 units (0.006 pt at 12 pt).
// By default a case passes when |delta| <= 0.5 units × glyph count × size/1000
// — the rounding bound; any larger difference means a table entry is wrong.
// `--tolerance PT` replaces that with a fixed absolute bound instead. The
// output also flags each case against a fixed 0.01 pt so the reader can see
// which strings meet that stricter, glyph-count-independent target.
// Exit status is 1 if any case fails.

import CoreText
import Foundation

var fixedTolerance: Double? = nil
var size = 12.0
var cases: [(String, String)] = []

var args = Array(CommandLine.arguments.dropFirst())
while !args.isEmpty {
    let a = args.removeFirst()
    switch a {
    case "--tolerance":
        fixedTolerance = Double(args.removeFirst())!
    case "--size":
        size = Double(args.removeFirst())!
    default:
        guard !args.isEmpty else {
            FileHandle.standardError.write("expected <face> <text> pairs\n".data(using: .utf8)!)
            exit(2)
        }
        cases.append((a, args.removeFirst()))
    }
}

if cases.isEmpty {
    let sentence = "The quick brown fox jumps over the lazy dog. FlashTeX, 2026!"
    let accented = "Café naïve façade — “quoted” … 100% €5 ™"
    for face in ["Times-Roman", "Times-Bold", "Times-Italic", "Times-BoldItalic",
                 "Helvetica", "Helvetica-Bold", "Courier"] {
        cases.append((face, sentence))
        cases.append((face, accented))
    }
    cases.append(("Times-Roman", "Hello"))
}

let scriptURL = URL(fileURLWithPath: CommandLine.arguments[0]).standardizedFileURL
let crateDir = scriptURL.deletingLastPathComponent().deletingLastPathComponent().path

func coreTextWidth(face: String, text: String, size: Double) -> Double {
    let font = CTFontCreateWithName(face as CFString, CGFloat(size), nil)
    var total = 0.0
    for scalar in text.unicodeScalars {
        var units = Array(String(Character(scalar)).utf16)
        var glyphs = [CGGlyph](repeating: 0, count: units.count)
        let ok = CTFontGetGlyphsForCharacters(font, &units, &glyphs, units.count)
        if !ok || glyphs[0] == 0 {
            // Same fallback the Rust side applies: the width of '?'.
            var q: [UniChar] = [0x3F]
            var qg = [CGGlyph](repeating: 0, count: 1)
            _ = CTFontGetGlyphsForCharacters(font, &q, &qg, 1)
            glyphs = qg
        }
        var adv = [CGSize](repeating: .zero, count: 1)
        _ = CTFontGetAdvancesForGlyphs(font, .horizontal, glyphs, &adv, 1)
        total += Double(adv[0].width)
    }
    return total
}

func rustWidth(face: String, text: String, size: Double) -> (Double, Int)? {
    let p = Process()
    p.executableURL = URL(fileURLWithPath: "/usr/bin/env")
    p.arguments = ["cargo", "run", "--quiet", "--example", "measure", "--", face, String(size), "-"]
    p.currentDirectoryURL = URL(fileURLWithPath: crateDir)
    let pipe = Pipe()
    let input = Pipe()
    p.standardOutput = pipe
    p.standardInput = input
    do { try p.run() } catch {
        FileHandle.standardError.write("failed to run cargo: \(error)\n".data(using: .utf8)!)
        return nil
    }
    input.fileHandleForWriting.write(text.data(using: .utf8)!)
    input.fileHandleForWriting.closeFile()
    p.waitUntilExit()
    let data = pipe.fileHandleForReading.readDataToEndOfFile()
    guard p.terminationStatus == 0,
          let line = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) else {
        return nil
    }
    let parts = line.split(separator: " ")
    guard parts.count == 2, let w = Double(parts[0]), let n = Int(parts[1]) else { return nil }
    return (w, n)
}

print("crate: \(crateDir)")
if let t = fixedTolerance {
    print("size: \(size) pt, tolerance: fixed \(t) pt")
} else {
    print("size: \(size) pt, tolerance: rounding bound (0.5 units × glyphs × size/1000)")
}
var failures = 0
var maxDelta = 0.0
var within001 = 0
for (face, text) in cases {
    let ct = coreTextWidth(face: face, text: text, size: size)
    guard let (rs, unknown) = rustWidth(face: face, text: text, size: size) else {
        print("FAIL \(face): could not obtain Rust width for \"\(text)\"")
        failures += 1
        continue
    }
    let glyphs = text.unicodeScalars.count
    let bound = fixedTolerance ?? (0.5 * Double(glyphs) * size / 1000.0)
    let delta = rs - ct
    maxDelta = max(maxDelta, abs(delta))
    let verdict = abs(delta) <= bound ? "PASS" : "FAIL"
    if verdict == "FAIL" { failures += 1 }
    let strict = abs(delta) <= 0.01
    if strict { within001 += 1 }
    print(String(format: "%@ %-16@ n=%2d coretext=%9.4f rust=%9.4f delta=%+.4f bound=%.4f %@ unknown=%d  \"%@\"",
                 verdict, face, glyphs, ct, rs, delta, bound, strict ? "<=0.01" : " >0.01", unknown, text))
}
print(String(format: "max |delta| = %.4f pt over %d cases; %d failure(s); %d of %d within 0.01 pt",
             maxDelta, cases.count, failures, within001, cases.count))
exit(failures == 0 ? 0 : 1)
