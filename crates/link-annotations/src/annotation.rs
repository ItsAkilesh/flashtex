//! The hyperlink annotation itself: a validated destination paired with the
//! rectangle it occupies and the exact source identity (revision, content
//! hash, and byte range) it came from.

use crate::geometry::Rect;
use crate::source::SourceIdentity;
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
/// rectangle, the exact source identity it was written at ([`SourceIdentity`]
/// — revision id, content hash, and byte range), and a destination that has
/// already been validated (external, allowlisted URI) or resolved
/// (internal, confirmed to a known label, carrying its exportable page
/// target).
///
/// This type models and validates a link. It never opens, resolves, or
/// fetches anything at export time — resolution of an internal target
/// already happened when the [`LinkDestination::Internal`] value was built,
/// and an external URI's scheme was checked against the allowlist in
/// [`crate::uri::validate_uri`] before it could become a
/// [`LinkDestination::External`]. `source` was likewise only constructible
/// via [`SourceIdentity::bind`], so it always describes a real,
/// character-aligned slice of the source it was bound against.
#[derive(Clone, Debug, PartialEq)]
pub struct LinkAnnotation {
    pub rect: Rect,
    pub source: SourceIdentity,
    pub destination: LinkDestination,
}

impl LinkAnnotation {
    pub fn new(rect: Rect, source: SourceIdentity, destination: LinkDestination) -> LinkAnnotation {
        LinkAnnotation {
            rect,
            source,
            destination,
        }
    }

    pub fn external(rect: Rect, source: SourceIdentity, uri: ValidatedUri) -> LinkAnnotation {
        LinkAnnotation::new(rect, source, LinkDestination::External(uri))
    }

    pub fn internal(rect: Rect, source: SourceIdentity, target: InternalTarget) -> LinkAnnotation {
        LinkAnnotation::new(rect, source, LinkDestination::Internal(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;
    use crate::page::{PageIndex, PageTarget};
    use crate::source::{RevisionId, SourceIdentity};
    use crate::span::{SourcePos, SourceSpan};
    use crate::target::{LabelId, LabelSet};
    use crate::uri::validate_uri;

    const SOURCE: &str = "0123456789https://example.com/docs-------padding-------";

    fn identity() -> SourceIdentity {
        let span = SourceSpan::new(SourcePos::new(10, 2, 1), SourcePos::new(40, 2, 31)).unwrap();
        SourceIdentity::bind(RevisionId::parse("rev-1").unwrap(), SOURCE, span).unwrap()
    }

    fn rect() -> Rect {
        Rect::new(Point::new(72.0, 700.0), 120.0, 12.0).unwrap()
    }

    #[test]
    fn builds_external_annotation_with_real_values() {
        let uri = validate_uri("https://example.com/docs").unwrap();
        let annotation = LinkAnnotation::external(rect(), identity(), uri);

        assert_eq!(annotation.rect.origin().x(), 72.0);
        assert_eq!(annotation.rect.width(), 120.0);
        assert_eq!(annotation.source.span().start().offset, 10);
        assert_eq!(annotation.source.span().len(), 30);
        assert_eq!(annotation.source.revision().as_str(), "rev-1");
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
        let page_target = PageTarget::new(PageIndex::new(4), rect());
        known.insert(id.clone(), page_target);
        let target = InternalTarget::resolve(id.clone(), &known).unwrap();
        let annotation = LinkAnnotation::internal(rect(), identity(), target);

        match annotation.destination {
            LinkDestination::Internal(target) => {
                assert_eq!(target.label(), &id);
                assert_eq!(target.page_target().page.value(), 4);
            }
            LinkDestination::External(_) => panic!("expected an internal destination"),
        }
    }

    #[test]
    fn annotation_source_is_detectably_stale_against_a_different_revision() {
        let uri = validate_uri("https://example.com/docs").unwrap();
        let annotation = LinkAnnotation::external(rect(), identity(), uri);

        let other_revision = RevisionId::parse("rev-2").unwrap();
        assert!(
            annotation
                .source
                .check_fresh(&other_revision, SOURCE)
                .is_err()
        );
    }
}
