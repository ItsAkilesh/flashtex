// Fixed-DPI CoreGraphics rasterizer shared by every side of the comparison.
// Apple frameworks only (no third-party packages); links nothing from apps/mac.
//
//   rasterize pdf <file.pdf> <out-prefix> [--dpi 144]
//       CGPDFDocument -> one bitmap per page, drawn with the MediaBox mapped to
//       exactly (width_pt * dpi / 72) x (height_pt * dpi / 72) device pixels,
//       sRGB, white background, antialiased text and shapes, no font smoothing
//       tricks beyond CoreGraphics defaults. Writes <prefix>-p<N>.png plus
//       <prefix>-p<N>.rgba (raw 8-bit RGBA, row-major, no padding) and
//       <prefix>-p<N>.rgba.json ({"width","height","dpi","source"}).
//
//   rasterize preview <compile_result.json> <out-prefix> [--dpi 144]
//       "Preview-equivalent" raster: re-implements the draw that the Mac app's
//       PDFExport.render / preview Canvas perform on a runtime-v1 compile_result
//       (Times-Roman via CoreText at x_pt / baseline_y_pt, font_size_pt; items
//       consisting only of U+2500 are rules: width = count * 0.5em, thickness =
//       0.0857em, bottom on the baseline). This is NOT the SwiftUI preview
//       itself and is labelled "preview-equivalent" everywhere.
//
//   rasterize word-boxes <file.pdf>
//       PDFKit word boxes as JSON on stdout, same shape as the native-validation
//       oracle_extract (x_pt, bottom_pt, top_pt, right_pt from the page's top-left).
//
//   rasterize crop-scale <in.png> <out-prefix> --page <x> <y> <w> <h> --width <px> --height <px> [--mask-caption]
//       Crop a rectangle out of a PNG (e.g. a screen capture) and resample it to the
//       given pixel size, writing the same png/rgba/rgba.json triple. Used for the
//       native preview capture. --mask-caption paints the preview's "page N"
//       caption area (bottom-right 60x16 pt of the page) white.
//
//   rasterize annotate <in.png> <out.png> <footer text>
//       Append a white footer band and draw the text (CoreText, Helvetica 11 px, up to
//       two lines split at " | ") so the provenance travels inside the PNG pixels; the
//       same text is also stored in the PNG tEXt/iTXt "Description" metadata.
//
//   rasterize window-id <owner name>
//       JSON list of on-screen windows owned by that app (CGWindowList), no Accessibility.
//
//   rasterize find-page <capture.png>
//       Detect the white page rectangle in a window capture: pixels that are not the
//       pane background grey form runs per row; the widest consistent run in the right
//       part of the window whose block is US-letter shaped is the page. JSON {found,x,y,w,h}.
import CoreGraphics
import CoreText
import Foundation
import ImageIO
import PDFKit
import UniformTypeIdentifiers

func fail(_ msg: String) -> Never {
    FileHandle.standardError.write((msg + "\n").data(using: .utf8)!)
    exit(2)
}

func makeContext(widthPx: Int, heightPx: Int) -> CGContext {
    let cs = CGColorSpace(name: CGColorSpace.sRGB)!
    guard let ctx = CGContext(data: nil, width: widthPx, height: heightPx, bitsPerComponent: 8,
                              bytesPerRow: widthPx * 4, space: cs,
                              bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)
    else { fail("cannot create bitmap context") }
    ctx.setFillColor(CGColor(srgbRed: 1, green: 1, blue: 1, alpha: 1))
    ctx.fill(CGRect(x: 0, y: 0, width: widthPx, height: heightPx))
    ctx.setAllowsAntialiasing(true)
    ctx.setShouldAntialias(true)
    ctx.setAllowsFontSmoothing(false)
    ctx.setShouldSmoothFonts(false)
    ctx.setAllowsFontSubpixelPositioning(true)
    ctx.setShouldSubpixelPositionFonts(true)
    ctx.setAllowsFontSubpixelQuantization(false)
    ctx.setShouldSubpixelQuantizeFonts(false)
    ctx.interpolationQuality = .high
    return ctx
}

func writeOutputs(ctx: CGContext, prefix: String, page: Int, dpi: Double, source: String, extra: [String: Any] = [:]) {
    guard let image = ctx.makeImage() else { fail("makeImage failed") }
    let pngURL = URL(fileURLWithPath: "\(prefix)-p\(page).png")
    guard let dest = CGImageDestinationCreateWithURL(pngURL as CFURL, UTType.png.identifier as CFString, 1, nil)
    else { fail("cannot create \(pngURL.path)") }
    CGImageDestinationAddImage(dest, image, nil)
    guard CGImageDestinationFinalize(dest) else { fail("cannot write \(pngURL.path)") }
    // Raw RGBA: alpha is always 255 on a white opaque background, so the
    // premultiplied buffer equals straight RGBA.
    let w = ctx.width, h = ctx.height
    guard let data = ctx.data else { fail("no bitmap data") }
    var raw = Data(count: w * h * 4)
    raw.withUnsafeMutableBytes { dst in
        for row in 0..<h {
            memcpy(dst.baseAddress! + row * w * 4, data + row * ctx.bytesPerRow, w * 4)
        }
    }
    try! raw.write(to: URL(fileURLWithPath: "\(prefix)-p\(page).rgba"))
    var meta: [String: Any] = ["width": w, "height": h, "dpi": dpi, "color_space": "sRGB IEC61966-2.1",
                               "format": "RGBA8 row-major top-down", "source": source, "page": page]
    for (k, v) in extra { meta[k] = v }
    let json = try! JSONSerialization.data(withJSONObject: meta, options: [.sortedKeys])
    try! json.write(to: URL(fileURLWithPath: "\(prefix)-p\(page).rgba.json"))
}

func parseDPI(_ args: inout [String]) -> Double {
    var dpi = 144.0
    if let i = args.firstIndex(of: "--dpi"), i + 1 < args.count {
        guard let d = Double(args[i + 1]) else { fail("bad --dpi") }
        dpi = d
        args.removeSubrange(i...(i + 1))
    }
    return dpi
}

// MARK: pdf

func rasterizePDF(_ path: String, prefix: String, dpi: Double) {
    guard let doc = CGPDFDocument(URL(fileURLWithPath: path) as CFURL) else { fail("cannot open PDF \(path)") }
    let scale = dpi / 72.0
    var pages: [[String: Any]] = []
    for n in 1...max(doc.numberOfPages, 1) {
        guard let page = doc.page(at: n) else { continue }
        let box = page.getBoxRect(.mediaBox)
        let w = Int((box.width * scale).rounded()), h = Int((box.height * scale).rounded())
        let ctx = makeContext(widthPx: w, heightPx: h)
        ctx.saveGState()
        ctx.scaleBy(x: scale, y: scale)
        ctx.translateBy(x: -box.minX, y: -box.minY)
        // Honour /Rotate the same way for every producer (all fixtures are 0).
        ctx.concatenate(page.getDrawingTransform(.mediaBox, rect: box, rotate: 0, preserveAspectRatio: true))
        ctx.drawPDFPage(page)
        ctx.restoreGState()
        writeOutputs(ctx: ctx, prefix: prefix, page: n, dpi: dpi, source: path,
                     extra: ["mediabox_pt": [box.minX, box.minY, box.width, box.height], "rotate": page.rotationAngle])
        pages.append(["page": n, "width_pt": box.width, "height_pt": box.height, "width_px": w, "height_px": h])
    }
    let summary: [String: Any] = ["pages": pages, "dpi": dpi, "source": path, "page_count": doc.numberOfPages]
    print(String(data: try! JSONSerialization.data(withJSONObject: summary, options: [.sortedKeys]), encoding: .utf8)!)
}

// MARK: preview-equivalent

let ruleScalar: Unicode.Scalar = "\u{2500}"
let ruleAdvanceEm = 0.5
let ruleThicknessEm = 0.0857

func renderPreviewEquivalent(_ jsonPath: String, prefix: String, dpi: Double) {
    guard let data = FileManager.default.contents(atPath: jsonPath),
          let env = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
          let payload = env["payload"] as? [String: Any],
          let pages = payload["pages"] as? [[String: Any]]
    else { fail("cannot parse compile_result \(jsonPath)") }
    let scale = dpi / 72.0
    let font12 = CTFontCreateWithName("Times-Roman" as CFString, 12, nil)
    let fontName = CTFontCopyPostScriptName(font12) as String
    var summary: [[String: Any]] = []
    for (idx, page) in pages.enumerated() {
        let wpt = page["width_pt"] as? Double ?? 612, hpt = page["height_pt"] as? Double ?? 792
        let w = Int((wpt * scale).rounded()), h = Int((hpt * scale).rounded())
        let ctx = makeContext(widthPx: w, heightPx: h)
        ctx.saveGState()
        ctx.scaleBy(x: scale, y: scale)
        ctx.setFillColor(CGColor(srgbRed: 0, green: 0, blue: 0, alpha: 1))
        ctx.textMatrix = .identity
        var textItems = 0, ruleItems = 0, skipped = 0
        for item in page["items"] as? [[String: Any]] ?? [] {
            guard (item["kind"] as? String ?? "text") == "text",
                  let text = item["text"] as? String,
                  let x = item["x_pt"] as? Double, let baseline = item["baseline_y_pt"] as? Double,
                  let size = item["font_size_pt"] as? Double
            else { skipped += 1; continue }
            let scalars = text.unicodeScalars
            if !scalars.isEmpty && scalars.allSatisfy({ $0 == ruleScalar }) {
                let rw = Double(scalars.count) * ruleAdvanceEm * size, rh = ruleThicknessEm * size
                ctx.fill(CGRect(x: x, y: hpt - baseline, width: rw, height: rh))
                ruleItems += 1
                continue
            }
            let font = CTFontCreateWithName("Times-Roman" as CFString, size, nil)
            let attr = NSAttributedString(string: text, attributes: [
                kCTFontAttributeName as NSAttributedString.Key: font,
                kCTForegroundColorAttributeName as NSAttributedString.Key: CGColor(srgbRed: 0, green: 0, blue: 0, alpha: 1),
            ])
            let line = CTLineCreateWithAttributedString(attr)
            ctx.textPosition = CGPoint(x: x, y: hpt - baseline)
            CTLineDraw(line, ctx)
            textItems += 1
        }
        ctx.restoreGState()
        writeOutputs(ctx: ctx, prefix: prefix, page: idx + 1, dpi: dpi, source: jsonPath,
                     extra: ["renderer": "preview-equivalent CoreText/CoreGraphics", "font": fontName,
                             "text_items": textItems, "rule_items": ruleItems, "skipped_items": skipped])
        summary.append(["page": idx + 1, "width_px": w, "height_px": h, "text_items": textItems,
                        "rule_items": ruleItems, "skipped_items": skipped])
    }
    let out: [String: Any] = ["pages": summary, "dpi": dpi, "font": fontName, "source": jsonPath, "page_count": pages.count]
    print(String(data: try! JSONSerialization.data(withJSONObject: out, options: [.sortedKeys]), encoding: .utf8)!)
}

// MARK: word boxes (PDFKit; same convention as tools/native-validation/oracle_extract.swift)

func wordBoxes(_ path: String) {
    guard let doc = PDFDocument(url: URL(fileURLWithPath: path)) else { fail("cannot open \(path)") }
    var pages: [[String: Any]] = []
    for i in 0..<doc.pageCount {
        guard let page = doc.page(at: i) else { continue }
        let box = page.bounds(for: .mediaBox)
        var words: [[String: Any]] = []
        if let s = page.string {
            let ns = s as NSString
            let ws = CharacterSet.whitespacesAndNewlines
            var idx = 0
            let n = ns.length
            while idx < n {
                while idx < n, let u = Unicode.Scalar(ns.character(at: idx)), ws.contains(u) { idx += 1 }
                if idx >= n { break }
                let start = idx
                while idx < n {
                    if let u = Unicode.Scalar(ns.character(at: idx)), ws.contains(u) { break }
                    idx += 1
                }
                let range = NSRange(location: start, length: idx - start)
                guard let sel = page.selection(for: range) else { continue }
                let b = sel.bounds(for: page)
                if b.isNull || b.isEmpty { continue }
                words.append(["text": ns.substring(with: range),
                              "x_pt": Double(b.minX - box.minX), "bottom_pt": Double(box.maxY - b.minY),
                              "top_pt": Double(box.maxY - b.maxY), "right_pt": Double(b.maxX - box.minX)])
            }
        }
        pages.append(["number": i + 1, "width_pt": Double(box.width), "height_pt": Double(box.height), "words": words])
    }
    print(String(data: try! JSONSerialization.data(withJSONObject: ["pages": pages], options: [.sortedKeys]), encoding: .utf8)!)
}

// MARK: crop-scale (native preview capture post-processing)

func cropScale(_ input: String, prefix: String, args: [String]) {
    func val(_ flag: String, _ count: Int) -> [Double] {
        guard let i = args.firstIndex(of: flag), i + count < args.count else { fail("missing \(flag)") }
        return (1...count).map { k -> Double in
            guard let d = Double(args[i + k]) else { fail("bad \(flag)") }
            return d
        }
    }
    let r = val("--page", 4), tw = Int(val("--width", 1)[0]), th = Int(val("--height", 1)[0])
    guard let src = CGImageSourceCreateWithURL(URL(fileURLWithPath: input) as CFURL, nil),
          let img = CGImageSourceCreateImageAtIndex(src, 0, nil) else { fail("cannot read \(input)") }
    guard let cropped = img.cropping(to: CGRect(x: r[0], y: r[1], width: r[2], height: r[3])) else { fail("crop failed") }
    let ctx = makeContext(widthPx: tw, heightPx: th)
    ctx.interpolationQuality = .high
    ctx.draw(cropped, in: CGRect(x: 0, y: 0, width: tw, height: th))
    let mask = args.contains("--mask-caption")
    if mask {
        // PreviewView overlays "page N" (caption2, padding 4) at the page's bottom-trailing corner.
        let s = Double(tw) / 612.0
        ctx.setFillColor(CGColor(srgbRed: 1, green: 1, blue: 1, alpha: 1))
        ctx.fill(CGRect(x: Double(tw) - 60 * s, y: 0, width: 60 * s, height: 16 * s))
    }
    writeOutputs(ctx: ctx, prefix: prefix, page: 1, dpi: 0, source: input,
                 extra: ["crop_px": r, "resampled_from_px": [cropped.width, cropped.height],
                         "scale_x": Double(tw) / Double(cropped.width), "scale_y": Double(th) / Double(cropped.height),
                         "caption_masked": mask, "renderer": "native preview capture (screencapture), resampled"])
    print("{\"cropped_px\":[\(cropped.width),\(cropped.height)],\"out_px\":[\(tw),\(th)]}")
}

// MARK: annotate (provenance footer burned into overlay/heatmap PNGs)

func annotate(_ input: String, _ output: String, _ text: String) {
    guard let src = CGImageSourceCreateWithURL(URL(fileURLWithPath: input) as CFURL, nil),
          let img = CGImageSourceCreateImageAtIndex(src, 0, nil) else { fail("cannot read \(input)") }
    let w = img.width, h = img.height
    let fontSize = max(9.0, min(11.0, Double(w) / 110.0))
    let font = CTFontCreateWithName("Helvetica" as CFString, fontSize, nil)
    // Split into lines that fit the width.
    var lines: [String] = []
    var current = ""
    for part in text.components(separatedBy: " | ") {
        let candidate = current.isEmpty ? part : current + " | " + part
        let attr = NSAttributedString(string: candidate, attributes: [kCTFontAttributeName as NSAttributedString.Key: font])
        if CTLineGetTypographicBounds(CTLineCreateWithAttributedString(attr), nil, nil, nil) > Double(w) - 8, !current.isEmpty {
            lines.append(current); current = part
        } else { current = candidate }
    }
    if !current.isEmpty { lines.append(current) }
    let lineH = Int(fontSize * 1.35) + 1
    let band = lineH * lines.count + 6
    let ctx = makeContext(widthPx: w, heightPx: h + band)
    ctx.draw(img, in: CGRect(x: 0, y: band, width: w, height: h))
    ctx.setFillColor(CGColor(srgbRed: 0.85, green: 0.85, blue: 0.85, alpha: 1))
    ctx.fill(CGRect(x: 0, y: band - 1, width: w, height: 1))
    ctx.textMatrix = .identity
    for (i, line) in lines.enumerated() {
        let attr = NSAttributedString(string: line, attributes: [
            kCTFontAttributeName as NSAttributedString.Key: font,
            kCTForegroundColorAttributeName as NSAttributedString.Key: CGColor(srgbRed: 0.2, green: 0.2, blue: 0.2, alpha: 1)])
        ctx.textPosition = CGPoint(x: 4, y: Double(band - 3 - lineH * (i + 1)) + fontSize * 0.3)
        CTLineDraw(CTLineCreateWithAttributedString(attr), ctx)
    }
    guard let out = ctx.makeImage() else { fail("makeImage") }
    guard let dest = CGImageDestinationCreateWithURL(URL(fileURLWithPath: output) as CFURL, UTType.png.identifier as CFString, 1, nil)
    else { fail("cannot create \(output)") }
    let png: [String: Any] = [kCGImagePropertyPNGDescription as String: text, kCGImagePropertyPNGTitle as String: "FlashTeX visual corpus evidence"]
    CGImageDestinationAddImage(dest, out, [kCGImagePropertyPNGDictionary as String: png] as CFDictionary)
    guard CGImageDestinationFinalize(dest) else { fail("cannot write \(output)") }
    print("{\"lines\":\(lines.count),\"band_px\":\(band)}")
}

// MARK: window-id / find-page (native preview capture)

func windowIDs(_ owner: String) {
    let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: AnyObject]] ?? []
    var out: [[String: Any]] = []
    for e in list where (e[kCGWindowOwnerName as String] as? String) == owner {
        out.append(["id": e[kCGWindowNumber as String] as? Int ?? -1,
                    "bounds": e[kCGWindowBounds as String] as? [String: Any] ?? [:],
                    "name": e[kCGWindowName as String] as? String ?? "",
                    "pid": e[kCGWindowOwnerPID as String] as? Int ?? -1,
                    "layer": e[kCGWindowLayer as String] as? Int ?? 0])
    }
    print(String(data: try! JSONSerialization.data(withJSONObject: out), encoding: .utf8)!)
}

func findPage(_ path: String) {
    guard let src = CGImageSourceCreateWithURL(URL(fileURLWithPath: path) as CFURL, nil),
          let img = CGImageSourceCreateImageAtIndex(src, 0, nil) else { fail("cannot read \(path)") }
    let w = img.width, h = img.height
    let ctx = makeContext(widthPx: w, heightPx: h)
    ctx.draw(img, in: CGRect(x: 0, y: 0, width: w, height: h))
    guard let data = ctx.data else { fail("no data") }
    let px = data.bindMemory(to: UInt8.self, capacity: ctx.bytesPerRow * h)
    // Background = the pane colour (light or dark appearance): the most common colour
    // in a thin band just inside the window's right edge, middle 60% of its height.
    var counts: [UInt32: Int] = [:]
    for y in stride(from: h / 5, to: h * 4 / 5, by: 2) {
        for x in (w - 12)..<(w - 4) {
            let i = (y * ctx.bytesPerRow)  /* memory row 0 = top of the image */ + x * 4
            counts[UInt32(px[i]) << 16 | UInt32(px[i + 1]) << 8 | UInt32(px[i + 2]), default: 0] += 1
        }
    }
    let bg = counts.max { $0.value < $1.value }!.key
    let bgr = Int(bg >> 16 & 0xff), bgg = Int(bg >> 8 & 0xff), bgb = Int(bg & 0xff)
    func isBackground(_ x: Int, _ y: Int) -> Bool {
        let i = (y * ctx.bytesPerRow)  /* memory row 0 = top of the image */ + x * 4
        return abs(Int(px[i]) - bgr) <= 3 && abs(Int(px[i + 1]) - bgg) <= 3 && abs(Int(px[i + 2]) - bgb) <= 3
    }
    // Per row: rightmost run of non-background pixels (gaps <= 12 px bridged) wider than 200 px.
    var runs: [Int: (Int, Int)] = [:]
    var key: [String: Int] = [:]
    for y in 0..<h {
        var x = w - 1
        var best: (Int, Int)? = nil
        while x >= 0 {
            if isBackground(x, y) { x -= 1; continue }
            let x1 = x
            var gap = 0
            var x0 = x
            while x0 >= 0 {
                if isBackground(x0, y) { gap += 1; if gap > 12 { break } } else { gap = 0 }
                x0 -= 1
            }
            let start = x0 + gap + 1
            if x1 - start + 1 >= 200 { best = (start, x1); break }
            x = x0
        }
        if let b = best {
            runs[y] = b
            key["\(b.0 / 4),\(b.1 / 4)", default: 0] += 1
        }
    }
    guard let mode = key.max(by: { $0.value < $1.value })?.key else {
        print("{\"found\":false,\"reason\":\"no wide runs\"}"); return
    }
    let parts = mode.split(separator: ",").map { Int($0)! * 4 }
    let mx0 = parts[0], mx1 = parts[1]
    // Rows whose run matches the mode within 6 px; largest contiguous block.
    var bestBlock = (0, -1), cur = (0, -1)
    for y in 0..<h {
        if let r = runs[y], abs(r.0 - mx0) <= 6, abs(r.1 - mx1) <= 6 {
            if cur.1 == y - 1 { cur.1 = y } else { cur = (y, y) }
            if cur.1 - cur.0 > bestBlock.1 - bestBlock.0 { bestBlock = cur }
        }
    }
    var xs0: [Int] = [], xs1: [Int] = []
    for y in bestBlock.0...max(bestBlock.0, bestBlock.1) { if let r = runs[y] { xs0.append(r.0); xs1.append(r.1) } }
    xs0.sort(); xs1.sort()
    var x0 = xs0.isEmpty ? mx0 : xs0[xs0.count / 2], x1 = xs1.isEmpty ? mx1 : xs1[xs1.count / 2]
    var y0 = bestBlock.0, y1 = bestBlock.1
    // Refine each edge inward (up to 6 px) until the edge line is mostly paper
    // white; the run detection includes the anti-aliased page border pixels.
    func isWhite(_ x: Int, _ y: Int) -> Bool {
        let i = (y * ctx.bytesPerRow) + x * 4
        return px[i] >= 250 && px[i + 1] >= 250 && px[i + 2] >= 250
    }
    func colWhiteFraction(_ x: Int) -> Double {
        var c = 0; for y in y0...y1 where isWhite(x, y) { c += 1 }; return Double(c) / Double(y1 - y0 + 1)
    }
    func rowWhiteFraction(_ y: Int) -> Double {
        var c = 0; for x in x0...x1 where isWhite(x, y) { c += 1 }; return Double(c) / Double(x1 - x0 + 1)
    }
    var steps = 0
    while steps < 6, x1 - x0 > 50, colWhiteFraction(x0) < 0.6 { x0 += 1; steps += 1 }
    steps = 0
    while steps < 6, x1 - x0 > 50, colWhiteFraction(x1) < 0.6 { x1 -= 1; steps += 1 }
    steps = 0
    while steps < 6, y1 - y0 > 50, rowWhiteFraction(y0) < 0.6 { y0 += 1; steps += 1 }
    steps = 0
    while steps < 6, y1 - y0 > 50, rowWhiteFraction(y1) < 0.6 { y1 -= 1; steps += 1 }
    let pw = x1 - x0 + 1, ph = y1 - y0 + 1
    let aspect = ph > 0 ? Double(pw) / Double(ph) : 0
    let ok = ph > 0 && abs(aspect - 612.0 / 792.0) < 0.03
    let out: [String: Any] = ["found": ok, "x": x0, "y": y0, "w": pw, "h": ph, "aspect": aspect, "raw_block": [bestBlock.0, bestBlock.1],
                              "expected_aspect": 612.0 / 792.0, "background_rgb": [bgr, bgg, bgb], "image_px": [w, h]]
    print(String(data: try! JSONSerialization.data(withJSONObject: out, options: [.sortedKeys]), encoding: .utf8)!)
}

var args = Array(CommandLine.arguments.dropFirst())
guard args.count >= 2 else { fail("usage: rasterize pdf|preview|word-boxes|crop-scale|window-id|find-page ...") }
let mode = args.removeFirst()
switch mode {
case "pdf":
    let dpi = parseDPI(&args)
    guard args.count == 2 else { fail("usage: rasterize pdf <file.pdf> <out-prefix> [--dpi N]") }
    rasterizePDF(args[0], prefix: args[1], dpi: dpi)
case "preview":
    let dpi = parseDPI(&args)
    guard args.count == 2 else { fail("usage: rasterize preview <compile_result.json> <out-prefix> [--dpi N]") }
    renderPreviewEquivalent(args[0], prefix: args[1], dpi: dpi)
case "word-boxes":
    wordBoxes(args[0])
case "crop-scale":
    cropScale(args[0], prefix: args[1], args: Array(args.dropFirst(2)))
case "window-id":
    windowIDs(args[0])
case "annotate":
    guard args.count == 3 else { fail("usage: rasterize annotate <in.png> <out.png> <text>") }
    annotate(args[0], args[1], args[2])
case "find-page":
    findPage(args[0])
default:
    fail("unknown mode \(mode)")
}
