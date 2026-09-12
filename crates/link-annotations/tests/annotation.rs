//! End-to-end tests: building a full [`LinkAnnotation`] the way a document
//! exporter would, and checking real rectangle/source-span/destination
//! values rather than only that the calls returned `Ok`.

use flashtex_link_annotations::{
    InternalTarget, LabelId, LabelSet, LinkAnnotation, LinkDestination, Point, Rect, SourcePos,
    SourceSpan, TargetError, UriError, validate_uri,
};

#[test]
fn external_link_carries_real_rect_span_and_uri() {
    let rect = Rect::new(Point::new(100.0, 500.0), 200.0, 14.0).unwrap();
    let span = SourceSpan::new(SourcePos::new(120, 5, 1), SourcePos::new(160, 5, 41)).unwrap();
    let uri = validate_uri("https://example.com/manual").unwrap();

    let link = LinkAnnotation::external(rect, span, uri);

    assert_eq!(link.rect.origin, Point::new(100.0, 500.0));
    assert_eq!(link.rect.max_x(), 300.0);
    assert_eq!(link.rect.max_y(), 514.0);
    assert_eq!(link.span.start.line, 5);
    assert_eq!(link.span.len(), 40);
    match &link.destination {
        LinkDestination::External(uri) => assert_eq!(uri.as_str(), "https://example.com/manual"),
        LinkDestination::Internal(_) => panic!("expected external destination"),
    }
}

#[test]
fn internal_link_resolves_against_document_labels_and_carries_real_values() {
    let mut labels = LabelSet::new();
    labels.insert(LabelId::parse("fig:results").unwrap());
    labels.insert(LabelId::parse("sec:intro").unwrap());
    assert_eq!(labels.len(), 2);

    let target = InternalTarget::resolve(LabelId::parse("fig:results").unwrap(), &labels).unwrap();

    let rect = Rect::new(Point::new(50.0, 60.0), 30.0, 10.0).unwrap();
    let span = SourceSpan::new(SourcePos::new(0, 1, 1), SourcePos::new(20, 1, 21)).unwrap();
    let link = LinkAnnotation::internal(rect, span, target);

    match &link.destination {
        LinkDestination::Internal(target) => assert_eq!(target.label().as_str(), "fig:results"),
        LinkDestination::External(_) => panic!("expected internal destination"),
    }
}

#[test]
fn dangling_internal_reference_is_a_hard_error_not_a_silent_destination() {
    let labels = LabelSet::new();
    let err = InternalTarget::resolve(LabelId::parse("fig:does-not-exist").unwrap(), &labels)
        .unwrap_err();
    assert_eq!(
        err,
        TargetError::Unresolved(LabelId::parse("fig:does-not-exist").unwrap())
    );
}

#[test]
fn dangerous_schemes_never_reach_a_link_annotation() {
    for hostile in [
        "javascript:alert(document.cookie)",
        "data:text/html,<b>x</b>",
        "file:///etc/passwd",
    ] {
        let err = validate_uri(hostile).unwrap_err();
        assert!(matches!(err, UriError::SchemeNotAllowed { .. }));
    }
}

#[test]
fn rect_and_span_reject_malformed_input_before_an_annotation_can_be_built() {
    assert!(Rect::new(Point::new(0.0, 0.0), -5.0, 10.0).is_err());
    assert!(SourceSpan::new(SourcePos::new(100, 1, 1), SourcePos::new(0, 1, 1)).is_err());
}
