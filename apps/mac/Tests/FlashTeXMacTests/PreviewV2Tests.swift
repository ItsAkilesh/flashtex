import XCTest
import CoreGraphics
import CoreText
import PDFKit
import FlashTeXProtocol
@testable import FlashTeXMac

/// Rendering, export and hit-testing of the experimental v2 display list.
/// Fonts come from the repository's bundled Latin Modern (apps/mac/Fonts),
/// resolved by content hash — never by platform name.
final class PreviewV2Tests: XCTestCase {
    static let fontsDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
        .deletingLastPathComponent().appendingPathComponent("Fonts")
    static let store = V2FontStore(directories: [fontsDir.path])

    /// The bundled lmroman10-regular.otf as a manifest entry (schema hash convention).
    static func lmRoman10() throws -> (RenderingV2.FontResource, V2FontStore.ResolvedFont) {
        guard let file = store.fonts.first(where: { $0.url.lastPathComponent == "lmroman10-regular.otf" }) else {
            throw XCTSkip("bundled lmroman10-regular.otf not found")
        }
        let cg = CGFont(CGDataProvider(url: file.url as CFURL)!)!
        let resource = RenderingV2.FontResource(fontId: "lm10", sha256: file.bytesSha256, byteLength: file.byteLength, format: "opentype-cff",
                                                faceIndex: 0, unitsPerEm: Int(cg.unitsPerEm), glyphCount: Int(cg.numberOfGlyphs),
                                                postscriptName: cg.postScriptName! as String)
        return (resource, try store.resolve(resource))
    }

    static let ticks = Double(RenderingV2.ticksPerPoint)
    static func t(_ pt: Double) -> Int64 { Int64((pt * ticks).rounded()) }

    /// Shapes `text` with CoreText's own layout (kerning + ligatures on) and
    /// turns the resulting glyph IDs/positions into a one-run display list at
    /// `origin` (top-left page space). Clusters come from the run's string
    /// indices, so a ligature is one cluster covering several source bytes.
    static func displayList(text: String, resource: RenderingV2.FontResource, font: CTFont, size: Double,
                            origin: CGPoint, page: CGSize) -> (RenderingV2.DisplayList, CTLine) {
        let attributed = NSAttributedString(string: text, attributes: [.font: font])
        let line = CTLineCreateWithAttributedString(attributed)
        let runs = CTLineGetGlyphRuns(line) as! [CTRun]
        XCTAssertEqual(runs.count, 1, "single-font text must shape to one CoreText run")
        let run = runs[0]
        let n = CTRunGetGlyphCount(run)
        var glyphs = [CGGlyph](repeating: 0, count: n)
        var positions = [CGPoint](repeating: .zero, count: n)
        var indices = [CFIndex](repeating: 0, count: n)
        CTRunGetGlyphs(run, CFRange(location: 0, length: 0), &glyphs)
        CTRunGetPositions(run, CFRange(location: 0, length: 0), &positions)
        CTRunGetStringIndices(run, CFRange(location: 0, length: 0), &indices)
        let utf16 = Array(text.utf16)
        // Cluster boundaries: UTF-16 string indices of each glyph, end-exclusive at the next glyph's index.
        func utf8Offset(utf16Index: Int) -> Int { String(utf16CodeUnits: Array(utf16[0..<utf16Index]), count: utf16Index).utf8.count }
        let ascent = CTFontGetAscent(font), descent = CTFontGetDescent(font)
        var clusters: [RenderingV2.Cluster] = []
        var v2glyphs: [RenderingV2.Glyph] = []
        for i in 0..<n {
            let start = utf8Offset(utf16Index: indices[i])
            let end = i + 1 < n ? utf8Offset(utf16Index: indices[i + 1]) : text.utf8.count
            let advance = (i + 1 < n ? positions[i + 1].x : Double(CTLineGetTypographicBounds(line, nil, nil, nil))) - positions[i].x
            let gx = origin.x + positions[i].x
            clusters.append(RenderingV2.Cluster(
                textStartByte: start, textEndByte: end,
                hitRects: [RenderingV2.Rect(x: t(gx), top: t(origin.y - ascent), width: t(advance), height: t(ascent + descent))],
                carets: [RenderingV2.Caret(textByte: start, x: t(gx), top: t(origin.y - ascent), height: t(ascent + descent))],
                sources: [RenderingV2.SourceRange(path: "main.tex", startByte: 10 + start, endByte: 10 + end)]))
            v2glyphs.append(RenderingV2.Glyph(gid: Int(glyphs[i]), originX: t(gx), baselineY: t(origin.y), advanceX: t(advance), advanceY: 0, cluster: i))
        }
        let list = RenderingV2.DisplayList(
            projectId: "golden", revision: 1, requiredFeatures: ["glyph_run", "rgba-srgb", "cluster-actualtext"],
            documents: [RenderingV2.DocumentResource(path: "main.tex", revision: 1, sha256: String(repeating: "0", count: 64), byteLength: 100)],
            fonts: [resource],
            pages: [RenderingV2.Page(number: 1, width: t(page.width), height: t(page.height),
                                     items: [.glyphRun(RenderingV2.GlyphRun(fontId: resource.fontId, fontSize: t(size), text: text, glyphs: v2glyphs, clusters: clusters, paint: .black))])],
            diagnostics: [])
        return (list, line)
    }

    static func pixels(_ image: CGImage) -> [UInt8] {
        let ctx = CGContext(data: nil, width: image.width, height: image.height, bitsPerComponent: 8, bytesPerRow: image.width * 4,
                            space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
        ctx.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
        return Array(UnsafeBufferPointer(start: ctx.data!.assumingMemoryBound(to: UInt8.self), count: image.width * image.height * 4))
    }

    static func differingPixels(_ a: CGImage, _ b: CGImage) -> Int {
        XCTAssertEqual(a.width, b.width); XCTAssertEqual(a.height, b.height)
        let pa = pixels(a), pb = pixels(b)
        var n = 0
        for i in stride(from: 0, to: min(pa.count, pb.count), by: 4) where pa[i..<i+4] != pb[i..<i+4] { n += 1 }
        return n
    }

    static func inkPixels(_ a: CGImage) -> Int {
        let p = pixels(a); var n = 0
        for i in stride(from: 0, to: p.count, by: 4) where p[i] < 250 || p[i+1] < 250 || p[i+2] < 250 { n += 1 }
        return n
    }

    // MARK: golden render: shared routine == CoreText's own CTLineDraw

    func testGlyphRunMatchesCTLineDrawPixelForPixel() throws {
        let (resource, resolved) = try Self.lmRoman10()
        let size = 12.0, scale = 4.0
        let font = resolved.ctFont(size: size)
        let page = CGSize(width: 120, height: 40)
        let origin = CGPoint(x: 8, y: 26)
        let (list, line) = Self.displayList(text: "AV office fi", resource: resource, font: font, size: size, origin: origin, page: page)
        try RenderingV2.validate(list)
        let frame = try V2Frame.prepare(RenderingV2.Envelope(id: "golden", payload: list), store: Self.store)

        // Reference: CoreText draws the line itself at the same baseline origin.
        let reference = GlyphRunRenderer.bitmapContext(widthPt: page.width, heightPt: page.height, scale: scale)!
        reference.setFillColor(CGColor(gray: 0, alpha: 1))
        reference.textMatrix = .identity
        reference.textPosition = CGPoint(x: origin.x, y: page.height - origin.y)
        CTLineDraw(line, reference)
        let referenceImage = reference.makeImage()!

        let ours = GlyphRunRenderer.rasterize(page: list.pages[0], frame: frame, scale: scale)!
        XCTAssertGreaterThan(Self.inkPixels(ours), 200, "the run must actually paint glyphs")
        XCTAssertEqual(Self.differingPixels(ours, referenceImage), 0, "shared draw routine must match CTLineDraw exactly")
        // The ligature shaped by CoreText is one glyph/cluster over two bytes ("fi").
        guard case .glyphRun(let run) = list.pages[0].items[0] else { return XCTFail() }
        let fi = run.clusters.last!
        XCTAssertEqual(fi.textEndByte - fi.textStartByte, 2)
        XCTAssertEqual(run.glyphs.filter { $0.cluster == run.clusters.count - 1 }.count, 1)
    }

    // MARK: export == preview for the same list

    func testPDFExportRasterEqualsPreviewRaster() throws {
        let (resource, resolved) = try Self.lmRoman10()
        let size = 10.0, scale = 3.0
        let page = CGSize(width: 100, height: 36)
        var (list, _) = Self.displayList(text: "Office", resource: resource, font: resolved.ctFont(size: size), size: size,
                                         origin: CGPoint(x: 6, y: 20), page: page)
        // Add a typed rule under the word so both primitives are covered.
        list.requiredFeatures.insert("rule", at: 1)
        list.pages[0].items.append(.rule(RenderingV2.Rule(x: Self.t(6), top: Self.t(23), width: Self.t(40), height: Self.t(0.5), paint: .black,
                                                          sources: [RenderingV2.SourceRange(path: "main.tex", startByte: 0, endByte: 5)])))
        try RenderingV2.validate(list)
        let frame = try V2Frame.prepare(RenderingV2.Envelope(id: "x", payload: list), store: Self.store)
        let preview = GlyphRunRenderer.rasterize(page: list.pages[0], frame: frame, scale: scale)!

        let pdf = GlyphRunRenderer.pdfData(frame: frame)
        let doc = try XCTUnwrap(PDFDocument(data: pdf))
        XCTAssertEqual(doc.pageCount, 1)
        let cgPage = try XCTUnwrap(doc.page(at: 0)?.pageRef)
        XCTAssertEqual(cgPage.getBoxRect(.mediaBox).size, page)
        // Rasterize the PDF page with the same bitmap setup and compare.
        let ctx = GlyphRunRenderer.bitmapContext(widthPt: page.width, heightPt: page.height, scale: scale)!
        ctx.drawPDFPage(cgPage)
        let exported = ctx.makeImage()!
        XCTAssertGreaterThan(Self.inkPixels(exported), 100)
        // The PDF path re-encodes glyph outlines through the embedded font program;
        // CoreGraphics rasterizes both from the same outlines and positions.
        XCTAssertEqual(Self.differingPixels(preview, exported), 0, "export must equal preview pixel for pixel")
    }

    // MARK: hit-testing and carets from clusters

    func testHitTestReturnsLigatureClusterWithTwoSourceBytes() throws {
        let (resource, resolved) = try Self.lmRoman10()
        let (list, _) = Self.displayList(text: "fi", resource: resource, font: resolved.ctFont(size: 12), size: 12,
                                         origin: CGPoint(x: 10, y: 20), page: CGSize(width: 60, height: 30))
        guard case .glyphRun(let run) = list.pages[0].items[0] else { return XCTFail() }
        XCTAssertEqual(run.glyphs.count, 1, "Latin Modern shapes fi to one ligature glyph")
        XCTAssertEqual(run.clusters.count, 1)
        let rect = run.clusters[0].hitRects[0]
        // Inside the glyph's hit rect → the whole cluster, both source bytes.
        let hit = try XCTUnwrap(V2Geometry.hit(page: list.pages[0], tickX: rect.x + rect.width / 2, tickY: rect.top + rect.height / 2))
        XCTAssertEqual(hit.clusterIndex, 0)
        XCTAssertEqual(hit.text, "fi")
        XCTAssertEqual(hit.sources, [RenderingV2.SourceRange(path: "main.tex", startByte: 10, endByte: 12)])
        // Half-open edges: the right/bottom edge is outside.
        XCTAssertNil(V2Geometry.hit(page: list.pages[0], tickX: rect.x + rect.width, tickY: rect.top))
        XCTAssertNil(V2Geometry.hit(page: list.pages[0], tickX: rect.x, tickY: rect.top + rect.height))
        XCTAssertNotNil(V2Geometry.hit(page: list.pages[0], tickX: rect.x, tickY: rect.top))
        // Point-space query floors to ticks.
        XCTAssertNotNil(V2Geometry.hit(page: list.pages[0], atPointX: RenderingV2.points(rect.x) + 0.001, y: RenderingV2.points(rect.top) + 0.001))
    }

    func testHitTestPrefersLaterPaintedItemAndFindsRules() {
        let rect = RenderingV2.Rect(x: 0, top: 0, width: 100, height: 100)
        let run = RenderingV2.GlyphRun(fontId: "f", fontSize: 1, text: "a", glyphs: [.init(gid: 1, originX: 0, baselineY: 50, advanceX: 10, advanceY: 0, cluster: 0)],
                                       clusters: [.init(textStartByte: 0, textEndByte: 1, hitRects: [rect], carets: [], sources: [.init(path: "main.tex", startByte: 0, endByte: 1)])], paint: .black)
        let rule = RenderingV2.Rule(x: 50, top: 50, width: 10, height: 10, paint: .black, sources: nil, syntheticReason: "fraction bar")
        let page = RenderingV2.Page(number: 1, width: 200, height: 200, items: [.glyphRun(run), .rule(rule)])
        let onRule = V2Geometry.hit(page: page, tickX: 55, tickY: 55)
        XCTAssertEqual(onRule?.itemIndex, 1)
        XCTAssertNil(onRule?.clusterIndex)
        XCTAssertEqual(onRule?.syntheticReason, "fraction bar")
        XCTAssertEqual(V2Geometry.hit(page: page, tickX: 5, tickY: 5)?.itemIndex, 0)
        XCTAssertNil(V2Geometry.hit(page: page, tickX: 150, tickY: 150))
    }

    func testCaretMapsToExactClusterCaretOrWholeClusterFallback() throws {
        let (resource, resolved) = try Self.lmRoman10()
        let (list, _) = Self.displayList(text: "AV fi", resource: resource, font: resolved.ctFont(size: 12), size: 12,
                                         origin: CGPoint(x: 10, y: 20), page: CGSize(width: 80, height: 30))
        guard case .glyphRun(let run) = list.pages[0].items[0] else { return XCTFail() }
        // Source bytes 10..: 'A'=10, 'V'=11, ' '=12, 'f'=13, 'i'=14.
        let onV = V2Geometry.clusters(containing: 11, path: "main.tex", in: list.pages[0])
        XCTAssertEqual(onV.count, 1)
        XCTAssertEqual(onV[0].clusterIndex, 1)
        XCTAssertEqual(onV[0].caret, run.clusters[1].carets[0], "1:1 byte cluster → exact caret")
        // Inside the ligature: the cluster has a caret only at its start, so the
        // caret at byte 'i' (14) has no exact geometry → whole-cluster fallback.
        let onI = V2Geometry.clusters(containing: 14, path: "main.tex", in: list.pages[0])
        XCTAssertEqual(onI.count, 1)
        XCTAssertEqual(onI[0].clusterIndex, run.clusters.count - 1)
        XCTAssertNil(onI[0].caret)
        XCTAssertEqual(onI[0].hitRects, run.clusters.last!.hitRects)
        XCTAssertEqual(V2Geometry.clusters(containing: 13, path: "main.tex", in: list.pages[0])[0].caret?.textByte, run.clusters.last!.textStartByte)
        // Other documents and out-of-range bytes match nothing.
        XCTAssertTrue(V2Geometry.clusters(containing: 11, path: "other.tex", in: list.pages[0]).isEmpty)
        XCTAssertTrue(V2Geometry.clusters(containing: 99, path: "main.tex", in: list.pages[0]).isEmpty)
    }

    // MARK: fail closed on resources

    func testUnavailableFontHashFailsTheWholeFrame() throws {
        let (resource, _) = try Self.lmRoman10()
        var missing = resource
        missing.fontId = "ghost"
        missing.sha256 = String(repeating: "ab", count: 32)
        let ok = RenderingV2.GlyphRun(fontId: "lm10", fontSize: Self.t(12), text: "a", glyphs: [.init(gid: 28, originX: 0, baselineY: Self.t(20), advanceX: 0, advanceY: 0, cluster: 0)],
                                      clusters: [.init(textStartByte: 0, textEndByte: 1, hitRects: [.init(x: 0, top: 0, width: 1, height: 1)], carets: [], sources: [.init(path: "main.tex", startByte: 0, endByte: 1)])], paint: .black)
        var bad = ok; bad.fontId = "ghost"
        let list = RenderingV2.DisplayList(projectId: "p", revision: 1, requiredFeatures: ["glyph_run", "rgba-srgb", "cluster-actualtext"],
                                           documents: [.init(path: "main.tex", revision: 1, sha256: String(repeating: "0", count: 64), byteLength: 1)],
                                           fonts: [resource, missing],
                                           pages: [.init(number: 1, width: Self.t(50), height: Self.t(50), items: [.glyphRun(ok), .glyphRun(bad)])], diagnostics: [])
        try RenderingV2.validate(list)
        XCTAssertThrowsError(try V2Frame.prepare(RenderingV2.Envelope(id: "x", payload: list), store: Self.store)) { error in
            let e = error as? RenderingV2.ValidationError
            XCTAssertEqual(e?.code, "font_resource_unavailable")
            XCTAssertTrue(e?.message.contains("font resource abab") == true, e?.message ?? "")
        }
    }

    func testManifestMismatchWithBundledBytesIsRefused() throws {
        let (resource, _) = try Self.lmRoman10()
        var wrongCount = resource; wrongCount.glyphCount = resource.glyphCount + 1
        XCTAssertThrowsError(try Self.store.resolve(wrongCount)) { XCTAssertEqual(($0 as? RenderingV2.ValidationError)?.code, "font_resource_mismatch") }
        var wrongLength = resource; wrongLength.byteLength += 1
        XCTAssertThrowsError(try Self.store.resolve(wrongLength)) { XCTAssertEqual(($0 as? RenderingV2.ValidationError)?.code, "font_resource_mismatch") }
        var wrongName = resource; wrongName.postscriptName = "Times-Roman"
        XCTAssertThrowsError(try Self.store.resolve(wrongName)) { XCTAssertEqual(($0 as? RenderingV2.ValidationError)?.code, "font_resource_mismatch") }
        var metricsOnly = resource; metricsOnly.format = "core14-afm"; metricsOnly.byteLength = 0
        XCTAssertThrowsError(try Self.store.resolve(metricsOnly)) { XCTAssertEqual(($0 as? RenderingV2.ValidationError)?.code, "font_resource_unavailable") }
    }

    func testPipelineHashConventionResolvesToTheSameBytes() throws {
        // flashtex-render names fonts by SHA-256(bytes ‖ face_index BE u32); the
        // store accepts that alongside the schema's SHA-256(bytes).
        let (resource, byBytes) = try Self.lmRoman10()
        var pipeline = resource; pipeline.sha256 = byBytes.file.face0Sha256
        let byFace0 = try Self.store.resolve(pipeline)
        XCTAssertEqual(byFace0.file.url, byBytes.file.url)
        XCTAssertEqual(byFace0.hashConvention, "bytes+face0")
        XCTAssertEqual(byBytes.hashConvention, "bytes")
    }
}

/// Shell integration: opening a real display list and navigating from clusters.
@MainActor
final class PreviewV2ShellTests: XCTestCase {
    static let fixtures = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("Fixtures")

    private func model(withFixtureText: Bool = true) throws -> ShellModel {
        let model = ShellModel()
        if withFixtureText {
            let tex = try String(contentsOf: Self.fixtures.appendingPathComponent("display-list-v2-text.tex"), encoding: .utf8)
            model.replaceProject(entryText: tex)
        }
        return model
    }

    func testOpeningTheRealDisplayListNavigatesLigatureClustersToSourceBytes() throws {
        let model = try model()
        XCTAssertFalse(model.previewV2, "v1 stays the default")
        model.loadDisplayListV2(url: Self.fixtures.appendingPathComponent("display-list-v2-text.json"))
        guard case .loaded(let frame, _) = model.displayListV2 else { return XCTFail("expected a prepared frame: \(String(describing: model.displayListV2))") }
        XCTAssertTrue(model.previewV2)
        let page = frame.list.pages[0]
        guard case .glyphRun(let office) = page.items[4] else { return XCTFail() }
        // Click the ffi ligature (one glyph, cluster 1, three source bytes).
        let ffi = office.clusters[1].hitRects[0]
        let hit = try XCTUnwrap(V2Geometry.hit(page: page, tickX: ffi.x + ffi.width / 2, tickY: ffi.top + ffi.height / 2))
        XCTAssertEqual(hit.text, "ffi")
        model.navigateV2(hit)
        let sel = try XCTUnwrap(model.selection)
        XCTAssertEqual(sel.path, "main.tex")
        XCTAssertEqual((model.activeText as NSString).substring(with: sel.nsRange), "ffi")
        XCTAssertTrue(model.navigationNote?.hasPrefix("Selected main.tex bytes 75..<78") == true, model.navigationNote ?? "")
        // café: the é cluster maps to the three source bytes of \'e.
        guard case .glyphRun(let cafe) = page.items[14] else { return XCTFail() }
        let e = cafe.clusters[3].hitRects[0]
        model.navigateV2(try XCTUnwrap(V2Geometry.hit(page: page, tickX: e.x, tickY: e.top)))
        XCTAssertEqual((model.activeText as NSString).substring(with: try XCTUnwrap(model.selection).nsRange), "\\'e")
        // Caret sync back into the preview: the editor caret inside \'e lights the é cluster (whole-cluster fallback).
        let matches = V2Geometry.clusters(containing: 142, path: "main.tex", in: page)
        XCTAssertEqual(matches.map(\.clusterIndex), [3])
        XCTAssertNil(matches[0].caret)
    }

    func testStaleBufferIsRefusedAndSyntheticContentHasNoSource() throws {
        let model = try model()
        model.loadDisplayListV2(url: Self.fixtures.appendingPathComponent("display-list-v2-text.json"))
        guard case .loaded(let frame, _) = model.displayListV2 else { return XCTFail() }
        let page = frame.list.pages[0]
        guard case .glyphRun(let run) = page.items[2] else { return XCTFail() }
        let r = run.clusters[0].hitRects[0]
        let hit = try XCTUnwrap(V2Geometry.hit(page: page, tickX: r.x, tickY: r.top))
        model.updateActiveText("edited " + model.activeText)
        model.selection = nil
        model.navigateV2(hit)
        XCTAssertNil(model.selection, "a display list for other bytes never selects")
        XCTAssertTrue(model.navigationNote?.contains("differs from the current buffer") == true, model.navigationNote ?? "")
        model.navigateV2(V2Geometry.Hit(itemIndex: 0, clusterIndex: nil, text: nil, sources: [], syntheticReason: "fraction bar", rect: r))
        XCTAssertEqual(model.navigationNote, "Generated content (fraction bar) has no source range.")
        XCTAssertNil(model.selection)
    }

    func testRefusedDisplayListShowsNoFrame() throws {
        let model = try model()
        model.loadDisplayListV2(url: Self.fixtures.appendingPathComponent("display-list-v2-math.json"))
        guard case .failed(let error, let url) = model.displayListV2 else { return XCTFail("expected refusal") }
        XCTAssertEqual(error.code, "font_resource_unavailable")
        XCTAssertEqual(url.lastPathComponent, "display-list-v2-math.json")
        XCTAssertNil(model.displayListV2?.frame)
        XCTAssertTrue(model.captureNote?.hasPrefix("Display list refused: font_resource_unavailable") == true, model.captureNote ?? "")
        // A runtime-v1 fixture is not a display list either.
        model.loadDisplayListV2(url: Self.fixtures.deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent("protocol/fixtures/compile-result.json"))
        guard case .failed(let e2, _) = model.displayListV2 else { return XCTFail() }
        XCTAssertEqual(e2.code, "unsupported_protocol_version")
    }
}
