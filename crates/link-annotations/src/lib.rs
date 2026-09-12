//! FlashTeX link annotation model.
//!
//! A typed, source-mapped hyperlink destination and rectangle model for
//! document export: a validated destination (an allowlisted external URI,
//! or a resolved reference to a label defined elsewhere in the document),
//! paired with the clickable rectangle on the page and the source span the
//! link annotation was written at.
//!
//! This crate models and validates links. It never navigates, opens,
//! resolves, or fetches anything, and it performs no network or filesystem
//! I/O of any kind — every check here is pure computation over the strings,
//! numbers, and label sets it is given.
//!
//! # Security
//!
//! [`uri::validate_uri`] checks a URI's scheme against an **explicit
//! allowlist** ([`uri::UriScheme`]: `http`, `https`, `mailto`), never a
//! denylist — an unlisted scheme is rejected by construction, so
//! `javascript:`, `data:`, `file:`, and every other scheme not in the
//! allowlist fail with a typed [`uri::UriError`]. Validation is bounded: a
//! length check runs before any scan, so a hostile or absurdly long input
//! fails immediately rather than being scanned, and no regex or recursion
//! is used anywhere in the crate.
//!
//! Internal targets ([`target::InternalTarget::resolve`]) fail explicitly,
//! naming the unresolved label, when a reference does not match a label the
//! caller has told this crate is defined — there is no path that produces a
//! dangling internal destination. A resolved internal target carries an
//! exportable [`page::PageTarget`] (page + rectangle) for the PDF exporter.
//!
//! Every annotation is bound to an exact [`source::SourceIdentity`] —
//! revision id, content hash, and byte range — so that re-checking it
//! against a different revision or changed source text is caught as
//! [`source::Staleness`] rather than silently trusted. Byte ranges are
//! validated to fall on UTF-8 character boundaries in the exact source text
//! they are bound against, so a range can never split a multi-byte
//! character.
//!
//! ```
//! use flashtex_link_annotations::{
//!     LinkAnnotation, LinkDestination, Point, Rect, RevisionId, SourceIdentity, SourcePos,
//!     SourceSpan, validate_uri,
//! };
//!
//! let source = "see \\href{https://example.com}{here} for more";
//! let rect = Rect::new(Point::new(72.0, 700.0), 120.0, 12.0).unwrap();
//! let span = SourceSpan::new(SourcePos::new(4, 1, 5), SourcePos::new(37, 1, 38)).unwrap();
//! let revision = RevisionId::parse("rev-1").unwrap();
//! let identity = SourceIdentity::bind(revision.clone(), source, span).unwrap();
//! let uri = validate_uri("https://example.com").unwrap();
//! let link = LinkAnnotation::external(rect, identity, uri);
//! assert!(matches!(link.destination, LinkDestination::External(_)));
//!
//! // Checked against a different revision, the same bytes are still stale.
//! let other_revision = RevisionId::parse("rev-2").unwrap();
//! assert!(link.source.check_fresh(&other_revision, source).is_err());
//!
//! // A denied scheme never becomes a link destination.
//! assert!(validate_uri("javascript:alert(1)").is_err());
//! ```

pub mod annotation;
pub mod geometry;
pub mod page;
pub mod source;
pub mod span;
pub mod target;
pub mod uri;

pub use annotation::{LinkAnnotation, LinkDestination};
pub use geometry::{Point, Rect, RectError};
pub use page::{PageIndex, PageTarget};
pub use source::{
    ContentHash, MAX_REVISION_LEN, RevisionError, RevisionId, SourceIdentity, SourceIdentityError,
    Staleness,
};
pub use span::{SourcePos, SourceSpan, SpanError};
pub use target::{InternalTarget, LabelError, LabelId, LabelSet, MAX_LABEL_LEN, TargetError};
pub use uri::{MAX_URI_LEN, UriError, UriScheme, ValidatedUri, validate_uri};
