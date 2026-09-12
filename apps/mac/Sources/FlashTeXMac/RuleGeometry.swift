import CoreGraphics
import FlashTeXProtocol

/// Rectangle mapping for typed `rules-v1` items, shared by the preview and the
/// CoreGraphics PDF export so both draw the same geometry. The contract's
/// `(x_pt, y_pt)` is the rectangle's TOP-LEFT corner in a y-down page space;
/// it is never reinterpreted as a text baseline.
enum RuleGeometry {
    /// Preview rectangle in view points (y down, origin at the page's top-left),
    /// scaled for display. Height never collapses below half a screen point so a
    /// hairline rule stays visible.
    static func previewRect(_ rule: RuntimeV1.PageItem.RuleItem, scale: CGFloat) -> CGRect {
        CGRect(x: rule.xPt * scale, y: rule.yPt * scale,
               width: rule.widthPt * scale, height: max(0.5, rule.heightPt * scale))
    }

    /// PDF user-space rectangle (y up, origin at the page's bottom-left) for a
    /// rule on `page`: the top edge lands at `height_pt - y_pt`.
    static func pdfRect(page: RuntimeV1.Page, rule: RuntimeV1.PageItem.RuleItem) -> CGRect {
        CGRect(x: rule.xPt, y: page.heightPt - rule.yPt - rule.heightPt,
               width: rule.widthPt, height: rule.heightPt)
    }

    /// Legacy route only (`rules-v1` not negotiated): a text item made of U+2500
    /// segments is approximated as a bar hugging its baseline (`RuleConvention`).
    /// This is an approximation of the compiler's intent, not contract geometry.
    static func legacyPDFRect(page: RuntimeV1.Page, text: RuntimeV1.PageItem.TextItem) -> CGRect? {
        guard let r = RuleConvention.rect(for: text) else { return nil }
        return CGRect(x: r.x, y: page.heightPt - r.y - r.height, width: r.width, height: r.height)
    }
}
