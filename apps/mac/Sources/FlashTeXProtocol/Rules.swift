import Foundation

/// Rendering convention shared by the preview and both PDF writers: the
/// compiler emits fraction bars (and similar rules) as text items made only of
/// U+2500 BOX DRAWINGS LIGHT HORIZONTAL, each assumed 0.5 em wide, with the
/// rule hugging the baseline from above. Renderers draw such items as filled
/// rectangles rather than glyphs, so the bar never depends on font coverage.
public enum RuleConvention {
    public static let ruleScalar: Unicode.Scalar = "\u{2500}"
    public static let advanceEm = 0.5
    public static let thicknessEm = 0.0857   // 0.06 × parent size at child size 0.7 × parent

    /// Returns the rule rectangle in top-left page coordinates, or nil if the
    /// item is not a pure rule.
    public static func rect(for item: RuntimeV1.PageItem.TextItem) -> (x: Double, y: Double, width: Double, height: Double)? {
        let scalars = item.text.unicodeScalars
        guard !scalars.isEmpty, scalars.allSatisfy({ $0 == ruleScalar }) else { return nil }
        let width = Double(scalars.count) * advanceEm * item.fontSizePt
        let height = thicknessEm * item.fontSizePt
        return (item.xPt, item.baselineYPt - height, width, height)
    }
}
