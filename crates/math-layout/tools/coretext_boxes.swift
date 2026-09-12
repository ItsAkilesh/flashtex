// CoreText placement of a runtime-v1 compile_result: the preview side of the
// preview/PDF box-parity check (tools/box_parity.py).
//
// Reads a compile_result (rules-v1 + font-hints-v1 items as emitted by
// flashtex-math-corpus), draws page 1 the way the Mac app's export/preview
// draw does — one CoreText line per text item at (x_pt, page_height −
// baseline_y_pt), one filled rectangle per rule item at (x_pt, page_height −
// (y_pt + height_pt), width_pt, height_pt) — into a CoreGraphics PDF, and
// records the box each draw call actually used, in the same top-left
// coordinates as the compile_result:
//   text: {"kind":"text","text","x_pt","baseline_y_pt","font","advance_pt",
//          "ink":{"left_pt","right_pt","top_pt","bottom_pt"}}
//   rule: {"kind":"rule","x_pt","y_pt","width_pt","height_pt"}
// Fonts: a font hint "Latin Modern Roman" resolves to the installed face of
// that name when present, else Times-Roman/Times-Italic (the same substitution
// crates/pdf makes without an LM installation); "Latin Modern Math *" hints
// resolve to Times-Roman. The chosen PostScript name is recorded per item,
// so the parity check knows when the two sides drew different faces.
//
// Usage: swiftc -O -o coretext_boxes coretext_boxes.swift
//        ./coretext_boxes compile_result.json out.pdf > boxes.json
import CoreGraphics
import CoreText
import Foundation

let args = CommandLine.arguments
guard args.count == 3,
      let data = FileManager.default.contents(atPath: args[1]),
      let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
      let payload = root["payload"] as? [String: Any],
      let pages = payload["pages"] as? [[String: Any]], let page = pages.first
else {
    FileHandle.standardError.write("usage: coretext_boxes compile_result.json out.pdf > boxes.json\n".data(using: .utf8)!)
    exit(2)
}
let wpt = page["width_pt"] as? Double ?? 612
let hpt = page["height_pt"] as? Double ?? 792
let items = page["items"] as? [[String: Any]] ?? []

func resolveFont(hint: [String: Any]?, size: Double) -> CTFont {
    let family = hint?["family"] as? String ?? "Times-Roman"
    let style = hint?["style"] as? String ?? "normal"
    let weight = hint?["weight"] as? String ?? "normal"
    let italic = style == "italic" || style == "oblique"
    let bold = weight == "bold"
    if family.hasPrefix("Latin Modern Roman") || family.hasPrefix("LMRoman") {
        let ps = "LMRoman10-" + (bold && italic ? "BoldItalic" : bold ? "Bold" : italic ? "Italic" : "Regular")
        let f = CTFontCreateWithName(ps as CFString, size, nil)
        if CTFontCopyPostScriptName(f) as String == ps { return f }
    }
    let ps = bold && italic ? "Times-BoldItalic" : bold ? "Times-Bold" : italic ? "Times-Italic" : "Times-Roman"
    return CTFontCreateWithName(ps as CFString, size, nil)
}

var mediaBox = CGRect(x: 0, y: 0, width: wpt, height: hpt)
guard let consumer = CGDataConsumer(url: URL(fileURLWithPath: args[2]) as CFURL),
      let ctx = CGContext(consumer: consumer, mediaBox: &mediaBox, nil)
else { exit(2) }
ctx.beginPDFPage(nil)
ctx.setFillColor(CGColor(srgbRed: 0, green: 0, blue: 0, alpha: 1))

var out: [[String: Any]] = []
for item in items {
    guard let kind = item["kind"] as? String else { continue }
    if kind == "rule" {
        guard let x = item["x_pt"] as? Double, let y = item["y_pt"] as? Double,
              let w = item["width_pt"] as? Double, let h = item["height_pt"] as? Double else { continue }
        let rect = CGRect(x: x, y: hpt - (y + h), width: w, height: h)
        ctx.fill(rect)
        out.append(["kind": "rule", "x_pt": rect.minX, "y_pt": hpt - rect.maxY,
                    "width_pt": rect.width, "height_pt": rect.height])
        continue
    }
    guard kind == "text", let text = item["text"] as? String,
          let x = item["x_pt"] as? Double, let baseline = item["baseline_y_pt"] as? Double,
          let size = item["font_size_pt"] as? Double else { continue }
    let font = resolveFont(hint: item["font"] as? [String: Any], size: size)
    let attr = NSAttributedString(string: text, attributes: [
        kCTFontAttributeName as NSAttributedString.Key: font,
        kCTForegroundColorAttributeName as NSAttributedString.Key: CGColor(srgbRed: 0, green: 0, blue: 0, alpha: 1),
    ])
    let line = CTLineCreateWithAttributedString(attr)
    let origin = CGPoint(x: x, y: hpt - baseline)
    ctx.textPosition = origin
    // Image bounds are reported at the context's current text position, so
    // sample them before the draw advances it: absolute page coordinates, y up.
    let ink = CTLineGetImageBounds(line, ctx)
    CTLineDraw(line, ctx)
    let advance = CTLineGetTypographicBounds(line, nil, nil, nil)
    out.append(["kind": "text", "text": text, "x_pt": origin.x, "baseline_y_pt": hpt - origin.y,
                "font": CTFontCopyPostScriptName(font) as String, "font_size_pt": size,
                "advance_pt": advance,
                "ink": ["left_pt": ink.minX, "right_pt": ink.maxX,
                        "top_pt": hpt - ink.maxY, "bottom_pt": hpt - ink.minY]])
}
ctx.endPDFPage()
ctx.closePDF()

let result: [String: Any] = ["page": ["width_pt": wpt, "height_pt": hpt], "items": out,
                             "renderer": "CoreText CTLineDraw per item into a CoreGraphics PDF context"]
let json = try! JSONSerialization.data(withJSONObject: result, options: [.sortedKeys])
FileHandle.standardOutput.write(json)
FileHandle.standardOutput.write("\n".data(using: .utf8)!)
