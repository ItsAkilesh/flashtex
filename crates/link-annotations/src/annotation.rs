//! The hyperlink annotation itself: a validated destination paired with the
//! rectangle it occupies and the source span it came from.

use crate::geometry::Rect;
use crate::span::SourceSpan;
use crate::target::InternalTarget;
use crate::uri::ValidatedUri;

/// Where a link points: either out to a validated external URI, or in to a
/// label defined elsewhere in the same document. Both variants have already
/// passed validation by construction — there is no "unvalidated" variant.
#[derive(Clone, Debug, PartialEq)]
pub enum LinkDestination {
    External(ValidatedUri),
    Internal(InternalTarget),
}

/// A single hyperlink annotation ready for document export: its clickable
/// rectangle, the source span it was written at, and a destination that has
/// already been validated (external, allowlisted URI) or resolved
/// (internal, confirmed to a known label).
///
/// This type models and validates a link. It never opens, resolves, or
/// fetches anything at export time — resolution of an internal target
/// already happened when the [`LinkDestination::Internal`] value was built,
/// and an external URI's scheme was checked against the allowlist in
/// [`crate::uri::validate_uri`] before it could become a
/// [`LinkDestination::External`].
#[derive(Clone, Debug, PartialEq)]
pub struct LinkAnnotation {
    pub rect: Rect,
    pub span: SourceSpan,
    pub destination: LinkDestination,
}

impl LinkAnnotation {
    pub fn new(rect: Rect, span: SourceSpan, destination: LinkDestination) -> LinkAnnotation {
        LinkAnnotation {
            rect,
            span,
            destination,
        }
    }

    pub fn external(rect: Rect, span: SourceSpan, uri: ValidatedUri) -> LinkAnnotation {
        LinkAnnotation::new(rect, span, LinkDestination::External(uri))
    }

    pub fn internal(rect: Rect, span: SourceSpan, target: InternalTarget) -> LinkAnnotation {
        LinkAnnotation::new(rect, span, LinkDestination::Internal(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;
    use crate::span::SourcePos;
    use crate::target::{LabelId, LabelSet};
    use crate::uri::validate_uri;

    fn span() -> SourceSpan {
        SourceSpan::new(SourcePos::new(10, 2, 1), SourcePos::new(40, 2, 31)).unwrap()
    }

    fn rect() -> Rect {
        Rect::new(Point::new(72.0, 700.0), 120.0, 12.0).unwrap()
    }

    #[test]
    fn builds_external_annotation_with_real_values() {
        let uri = validate_uri("https://example.com/docs").unwrap();
        let annotation = LinkAnnotation::external(rect(), span(), uri);

        assert_eq!(annotation.rect.origin.x, 72.0);
        assert_eq!(annotation.rect.width, 120.0);
        assert_eq!(annotation.span.start.offset, 10);
        assert_eq!(annotation.span.len(), 30);
        match annotation.destination {
            LinkDestination::External(uri) => {
                assert_eq!(uri.as_str(), "https://example.com/docs");
            }
            LinkDestination::Internal(_) => panic!("expected an external destination"),
        }
    }

    #[test]
    fn builds_internal_annotation_after_resolution() {
        let id = LabelId::parse("fig:tree").unwrap();
        let mut known = LabelSet::new();
        known.insert(id.clone());
        let target = InternalTarget::resolve(id.clone(), &known).unwrap();
        let annotation = LinkAnnotation::internal(rect(), span(), target);

        match annotation.destination {
            LinkDestination::Internal(target) => assert_eq!(target.label(), &id),
            LinkDestination::External(_) => panic!("expected an internal destination"),
        }
    }
}
