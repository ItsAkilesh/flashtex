// Word-box extractor for oracle_compare.py. Apple PDFKit only (no third-party
// packages). Usage: oracle_extract <file.pdf>  -> JSON on stdout:
// {"pages":[{"number":1,"width_pt":612,"height_pt":792,
//            "words":[{"text":"Hello","x_pt":72.0,"bottom_pt":86.6,"top_pt":77.9,"right_pt":98.1}, ...]}]}
// Coordinates are points with a top-left origin (bottom_pt is the distance from
// the page top to the bottom of the word's glyph box, i.e. baseline + descent).
import Foundation
import PDFKit

struct Word: Encodable {
    let text: String
    let x_pt: Double
    let bottom_pt: Double
    let top_pt: Double
    let right_pt: Double
}
struct Page: Encodable {
    let number: Int
    let width_pt: Double
    let height_pt: Double
    let words: [Word]
}
struct Doc: Encodable { let pages: [Page] }

guard CommandLine.arguments.count == 2,
      let doc = PDFDocument(url: URL(fileURLWithPath: CommandLine.arguments[1])) else {
    FileHandle.standardError.write("usage: oracle_extract <file.pdf>\n".data(using: .utf8)!)
    exit(2)
}

var pages: [Page] = []
for i in 0..<doc.pageCount {
    guard let page = doc.page(at: i) else { continue }
    let box = page.bounds(for: .mediaBox)
    var words: [Word] = []
    if let s = page.string {
        let ns = s as NSString
        let ws = CharacterSet.whitespacesAndNewlines
        var idx = 0
        let n = ns.length
        while idx < n {
            // skip whitespace
            while idx < n, let u = Unicode.Scalar(ns.character(at: idx)), ws.contains(u) { idx += 1 }
            if idx >= n { break }
            let start = idx
            while idx < n {
                if let u = Unicode.Scalar(ns.character(at: idx)), ws.contains(u) { break }
                idx += 1
            }
            let range = NSRange(location: start, length: idx - start)
            let text = ns.substring(with: range)
            guard let sel = page.selection(for: range) else { continue }
            let b = sel.bounds(for: page)
            if b.isNull || b.isEmpty { continue }
            // PDFKit page space: origin bottom-left of the media box.
            words.append(Word(
                text: text,
                x_pt: Double(b.minX - box.minX),
                bottom_pt: Double(box.maxY - b.minY),
                top_pt: Double(box.maxY - b.maxY),
                right_pt: Double(b.maxX - box.minX)))
        }
    }
    pages.append(Page(number: i + 1, width_pt: Double(box.width), height_pt: Double(box.height), words: words))
}

let enc = JSONEncoder()
enc.outputFormatting = [.sortedKeys]
let data = try! enc.encode(Doc(pages: pages))
FileHandle.standardOutput.write(data)
FileHandle.standardOutput.write("\n".data(using: .utf8)!)
