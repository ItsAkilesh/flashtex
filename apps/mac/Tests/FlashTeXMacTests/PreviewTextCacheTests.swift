import XCTest
import CoreText
@testable import FlashTeXMac

/// The preview line cache must not change geometry: keys are exact sizes and
/// the fonts are built at exactly the requested size; both dictionaries stay
/// bounded.
final class PreviewTextCacheTests: XCTestCase {
    func testKeysDistinguishNearlyEqualSizesAndFontsAreExact() {
        let cache = PreviewTextCache()
        let a = PreviewTextCache.key(text: "x", postScriptName: "Times-Roman", size: 10.0)
        let b = PreviewTextCache.key(text: "x", postScriptName: "Times-Roman", size: 10.004)
        XCTAssertNotEqual(a, b, "10.000 and 10.004 pt are different entries")
        XCTAssertEqual(a.size, 10.0)
        XCTAssertEqual(b.size, 10.004)
        _ = cache.line(for: a); _ = cache.line(for: b)
        XCTAssertEqual(cache.count, 2)
        XCTAssertEqual(cache.fontCount, 2)
        XCTAssertEqual(CTFontGetSize(cache.font("Times-Roman", size: 10.004)), 10.004, accuracy: 0)
        XCTAssertEqual(CTFontGetSize(cache.font("Times-Roman", size: 9.999999)), 9.999999, accuracy: 0)
        // Metrics scale with the exact size, not a quantized one.
        let w10 = cache.line(for: a).width, w10004 = cache.line(for: b).width
        XCTAssertGreaterThan(w10004, w10)
        XCTAssertEqual(w10004 / w10, 10.004 / 10.0, accuracy: 1e-9)
    }

    func testIdenticalRequestsHitAndCacheIsNotQuantized() {
        let cache = PreviewTextCache()
        let k = PreviewTextCache.key(text: "Hello", postScriptName: "Times-Roman", size: 12.25)
        _ = cache.line(for: k); _ = cache.line(for: k)
        XCTAssertEqual(cache.hits, 1); XCTAssertEqual(cache.misses, 1)
        let k2 = PreviewTextCache.key(text: "Hello", postScriptName: "Times-Roman", size: 12.25.nextUp)
        _ = cache.line(for: k2)
        XCTAssertEqual(cache.misses, 2, "one ulp apart is a distinct entry")
    }

    func testEvictionBoundsBothLinesAndFonts() {
        let cache = PreviewTextCache()
        cache.capacity = 8
        cache.fontCapacity = 4
        for i in 0..<20 {
            _ = cache.line(for: PreviewTextCache.key(text: "t\(i)", postScriptName: "Times-Roman", size: 10))
        }
        XCTAssertLessThanOrEqual(cache.count, 8)
        for i in 0..<20 {
            _ = cache.font("Times-Roman", size: CGFloat(10 + i))
        }
        XCTAssertLessThanOrEqual(cache.fontCount, 4, "fonts are bounded independently of lines")
        XCTAssertLessThanOrEqual(cache.count, 8)
    }
}
