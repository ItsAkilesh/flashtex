//! End-to-end tests: building a full [`LinkAnnotation`] the way a document
//! exporter would, and checking real rectangle/source-identity/destination
//! values rather than only that the calls returned `Ok`.

use flashtex_link_annotations::{
    InternalTarget, LabelId, LabelSet, LinkAnnotation, LinkDestination, PageIndex, PageTarget,
    Point, Rect, RevisionId, SourceIdentity, SourceIdentityError, SourcePos, SourceSpan, Staleness,
    TargetError, UriError, validate_uri,
};

#[test]
fn external_link_carries_real_rect_source_identity_and_uri() {
    let source = "0".repeat(120) + "https://example.com/manual" + &"1".repeat(40);
    let rect = Rect::new(Point::new(100.0, 500.0), 200.0, 14.0).unwrap();
    let span = SourceSpan::new(SourcePos::new(120, 5, 1), SourcePos::new(147, 5, 28)).unwrap();
    let revision = RevisionId::parse("commit-abc123").unwrap();
    let identity = SourceIdentity::bind(revision.clone(), &source, span).unwrap();
    let uri = validate_uri("https://example.com/manual").unwrap();

    let link = LinkAnnotation::external(rect, identity, uri);

    assert_eq!(link.rect.origin, Point::new(100.0, 500.0));
    assert_eq!(link.rect.max_x(), 300.0);
    assert_eq!(link.rect.max_y(), 514.0);
    assert_eq!(link.source.span().start().line, 5);
    assert_eq!(link.source.span().len(), 27);
    assert_eq!(link.source.revision().as_str(), "commit-abc123");
    match &link.destination {
        LinkDestination::External(uri) => assert_eq!(uri.as_str(), "https://example.com/manual"),
        LinkDestination::Internal(_) => panic!("expected external destination"),
    }

    // Fresh against the exact revision and source it was bound to.
    assert_eq!(link.source.check_fresh(&revision, &source), Ok(()));
}

#[test]
fn internal_link_resolves_against_document_labels_and_carries_a_page_target() {
    let mut labels = LabelSet::new();
    let results_page = PageTarget::new(
        PageIndex::new(2),
        Rect::new(Point::new(50.0, 600.0), 400.0, 20.0).unwrap(),
    );
    labels.insert(LabelId::parse("fig:results").unwrap(), results_page);
    labels.insert(
        LabelId::parse("sec:intro").unwrap(),
        PageTarget::new(
            PageIndex::new(0),
            Rect::new(Point::ORIGIN, 10.0, 10.0).unwrap(),
        ),
    );
    assert_eq!(labels.len(), 2);

    let target = InternalTarget::resolve(LabelId::parse("fig:results").unwrap(), &labels).unwrap();
    assert_eq!(target.page_target(), results_page);

    let rect = Rect::new(Point::new(50.0, 60.0), 30.0, 10.0).unwrap();
    let source = "a".repeat(30);
    let span = SourceSpan::new(SourcePos::new(0, 1, 1), SourcePos::new(20, 1, 21)).unwrap();
    let identity =
        SourceIdentity::bind(RevisionId::parse("rev-1").unwrap(), &source, span).unwrap();
    let link = LinkAnnotation::internal(rect, identity, target);

    match &link.destination {
        LinkDestination::Internal(target) => {
            assert_eq!(target.label().as_str(), "fig:results");
            assert_eq!(target.page_target().page.value(), 2);
        }
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

#[test]
fn source_identity_is_detectably_stale_against_a_different_revision_of_the_same_bytes() {
    let source = "\\href{https://example.com}{click here}";
    let span = SourceSpan::new(
        SourcePos::new(0, 1, 1),
        SourcePos::new(source.len() as u32, 1, 1),
    )
    .unwrap();
    let rev_a = RevisionId::parse("rev-a").unwrap();
    let rev_b = RevisionId::parse("rev-b").unwrap();
    let identity = SourceIdentity::bind(rev_a.clone(), source, span).unwrap();

    // Identical bytes, but a different revision: must be caught, not
    // silently treated as still valid.
    let err = identity.check_fresh(&rev_b, source).unwrap_err();
    assert_eq!(
        err,
        Staleness::RevisionChanged {
            expected: rev_a,
            found: rev_b,
        }
    );
}

#[test]
fn source_identity_is_detectably_stale_when_the_text_under_the_span_changes() {
    let original = "\\href{https://example.com}{click here}";
    let span = SourceSpan::new(
        SourcePos::new(0, 1, 1),
        SourcePos::new(original.len() as u32, 1, 1),
    )
    .unwrap();
    let revision = RevisionId::parse("rev-1").unwrap();
    let identity = SourceIdentity::bind(revision.clone(), original, span).unwrap();

    let edited = "\\href{https://evil.example}{click here}";
    let err = identity.check_fresh(&revision, edited).unwrap_err();
    assert_eq!(err, Staleness::ContentChanged);
}

#[test]
fn multibyte_source_ranges_never_split_a_character() {
    // Every character in this source is multi-byte in UTF-8 (3 bytes each
    // for the CJK characters). An offset landing inside a character must be
    // rejected rather than silently truncating it.
    let source = "日本語のハイパーリンク";
    let odd_offset_inside_a_character = SourceSpan::new(
        SourcePos::new(0, 1, 1),
        SourcePos::new(4, 1, 1), // byte 4 is inside the second 3-byte character
    )
    .unwrap();
    let revision = RevisionId::parse("rev-1").unwrap();
    let err = SourceIdentity::bind(revision, source, odd_offset_inside_a_character).unwrap_err();
    assert!(matches!(err, SourceIdentityError::NotCharBoundary { .. }));

    // The character-aligned equivalent (first two characters, 6 bytes) must
    // succeed and report the exact byte length, not a character count.
    let aligned = SourceSpan::new(SourcePos::new(0, 1, 1), SourcePos::new(6, 1, 3)).unwrap();
    let revision = RevisionId::parse("rev-1").unwrap();
    let identity = SourceIdentity::bind(revision, source, aligned).unwrap();
    assert_eq!(identity.span().len(), 6);
}
