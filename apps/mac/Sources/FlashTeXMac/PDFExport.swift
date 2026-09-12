import AppKit
import CoreGraphics
import CoreText
import FlashTeXProtocol

/// Renders a runtime-v1 `compile_result` to PDF with CoreGraphics.
///
/// This draws exactly the positioned items the compiler reported — one PDF page
/// per `pages` entry at `width_pt` × `height_pt`, each text item in a serif font
/// at `font_size_pt` with its baseline at `baseline_y_pt` from the top of the
/// page. It is not a TeX-engine PDF and carries no fonts, images, or links
/// beyond what the contract's text items describe. Unknown item kinds are skipped.
enum PDFExport {
    /// Font used for text items. PDF pages embed a subset of this font.
    static let fontName = "Times-Roman"

    static func render(_ result: RuntimeV1.CompileResult, dark: Bool = false) -> Data {
        let data = NSMutableData()
        guard let consumer = CGDataConsumer(data: data),
              let ctx = CGContext(consumer: consumer, mediaBox: nil, nil)
        else { return Data() }

        let background = dark ? CGColor(gray: 0.16, alpha: 1) : CGColor(gray: 1, alpha: 1)
        let foreground = dark ? CGColor(gray: 1, alpha: 1) : CGColor(gray: 0, alpha: 1)

        for page in result.pages {
            var mediaBox = CGRect(x: 0, y: 0, width: page.widthPt, height: page.heightPt)
            ctx.beginPDFPage([kCGPDFContextMediaBox as String: NSData(bytes: &mediaBox, length: MemoryLayout<CGRect>.size)] as CFDictionary)
            ctx.setFillColor(background)
            ctx.fill(mediaBox)
            draw(page, in: ctx, foreground: foreground)
            ctx.endPDFPage()
        }
        ctx.closePDF()
        return data as Data
    }

    /// Draws every text item with CoreText so the baseline lands exactly at
    /// `baseline_y_pt`. PDF space has a bottom-left origin, so y is flipped.
    private static func draw(_ page: RuntimeV1.Page, in ctx: CGContext, foreground: CGColor) {
        ctx.textMatrix = .identity
        for item in page.items {
            guard case .text(let t) = item else { continue }
            let font = CTFontCreateWithName(fontName as CFString, t.fontSizePt, nil)
            let attributed = NSAttributedString(string: t.text, attributes: [
                .font: font,
                .foregroundColor: NSColor(cgColor: foreground) ?? .black,
            ])
            let line = CTLineCreateWithAttributedString(attributed)
            ctx.textPosition = CGPoint(x: t.xPt, y: page.heightPt - t.baselineYPt)
            CTLineDraw(line, ctx)
        }
    }
}

extension ShellModel {
    /// `File > Export PDF…`: writes the current preview via `PDFExport.render`.
    /// Reports the saved path (or failure) in `captureNote`.
    func exportPDF() {
        guard let result else {
            captureNote = "Nothing to export: no compile result loaded."
            return
        }
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.pdf]
        panel.nameFieldStringValue = "\(result.projectId)-r\(result.revision).pdf"
        panel.message = "Export the preview's reported layout as PDF (not a TeX-engine PDF)"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            // Dark preview is a viewing mode only; the exported document is always white.
            try PDFExport.render(result, dark: false).write(to: url, options: .atomic)
            captureNote = "Exported \(result.pages.count) page\(result.pages.count == 1 ? "" : "s") to \(url.path)"
        } catch {
            captureNote = "PDF export failed: \(error.localizedDescription)"
        }
    }
}
