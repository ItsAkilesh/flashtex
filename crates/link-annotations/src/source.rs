//! Exact source identity: binds a value to the precise source revision,
//! content hash, and byte range it was computed from.
//!
//! This exists so a link computed against one version of the source can be
//! detected as stale when checked against a different revision or against
//! source text whose bytes at that same range have changed — rather than
//! being silently trusted forever. Nothing here reads a file, diffs a
//! revision, or performs any I/O: [`SourceIdentity::bind`] and
//! [`SourceIdentity::check_fresh`] only ever look at the `&str` they are
//! given.
//!
//! The content hash is a fast, non-cryptographic hash (FNV-1a) used purely
//! to notice accidental drift. It is not a security boundary — the
//! allowlist in [`crate::uri`] is — so it makes no attempt to resist a
//! deliberate collision.

use crate::span::SourceSpan;
use std::fmt;

/// The longest revision id this crate will accept.
pub const MAX_REVISION_LEN: usize = 256;

/// An opaque identifier for a source revision (a commit hash, a document
/// version counter, anything the caller uses to distinguish "this exact
/// version of the source" from another). Validated the same way a
/// [`crate::target::LabelId`] is: non-empty, bounded, no control characters.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionId(String);

impl RevisionId {
    /// Bounded: the length check runs before any scan, so cost never
    /// exceeds [`MAX_REVISION_LEN`] bytes of work.
    pub fn parse(text: &str) -> Result<RevisionId, RevisionError> {
        if text.is_empty() {
            return Err(RevisionError::Empty);
        }
        if text.len() > MAX_REVISION_LEN {
            return Err(RevisionError::TooLong {
                len: text.len(),
                max: MAX_REVISION_LEN,
            });
        }
        if let Some((at, _)) = text.char_indices().find(|(_, c)| c.is_control()) {
            return Err(RevisionError::ControlCharacter { at });
        }
        Ok(RevisionId(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RevisionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RevisionError {
    Empty,
    TooLong { len: usize, max: usize },
    ControlCharacter { at: usize },
}

impl fmt::Display for RevisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RevisionError::Empty => write!(f, "revision id is empty"),
            RevisionError::TooLong { len, max } => {
                write!(
                    f,
                    "revision id is {len} bytes, exceeds the {max}-byte bound"
                )
            }
            RevisionError::ControlCharacter { at } => {
                write!(
                    f,
                    "revision id contains a control character at byte offset {at}"
                )
            }
        }
    }
}

impl std::error::Error for RevisionError {}

/// A non-cryptographic content hash (64-bit FNV-1a) over exactly the bytes
/// of a source range, used only to detect drift between two readings of
/// "the same" span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentHash(u64);

impl ContentHash {
    fn of(bytes: &[u8]) -> ContentHash {
        const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
        let mut hash = FNV_OFFSET_BASIS;
        for &byte in bytes {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        ContentHash(hash)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceIdentityError {
    /// The span's end offset falls past the end of the given source text.
    SpanOutOfBounds { end: u32, source_len: usize },
    /// An offset does not fall on a UTF-8 character boundary in the given
    /// source text — binding here would silently split a multi-byte
    /// character, so it is refused instead.
    NotCharBoundary { offset: u32 },
}

impl fmt::Display for SourceIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceIdentityError::SpanOutOfBounds { end, source_len } => write!(
                f,
                "span end offset {end} is past the end of the source ({source_len} bytes)"
            ),
            SourceIdentityError::NotCharBoundary { offset } => write!(
                f,
                "byte offset {offset} does not fall on a utf-8 character boundary"
            ),
        }
    }
}

impl std::error::Error for SourceIdentityError {}

/// Why a previously bound [`SourceIdentity`] is no longer trustworthy
/// against a given revision and source text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Staleness {
    /// The current revision id differs from the one this identity was
    /// bound against.
    RevisionChanged {
        expected: RevisionId,
        found: RevisionId,
    },
    /// The current source text is too short, or the span no longer falls
    /// on a character boundary in it.
    RangeInvalid,
    /// Same revision and a valid range, but the bytes at that exact range
    /// hashed differently — the source changed under this span.
    ContentChanged,
}

impl fmt::Display for Staleness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Staleness::RevisionChanged { expected, found } => {
                write!(
                    f,
                    "bound to revision `{expected}`, but checked against `{found}`"
                )
            }
            Staleness::RangeInvalid => {
                write!(
                    f,
                    "source range no longer falls within the given source text"
                )
            }
            Staleness::ContentChanged => {
                write!(f, "source bytes at the bound range no longer match")
            }
        }
    }
}

impl std::error::Error for Staleness {}

/// Binds a value to the exact source revision, byte range, and content it
/// was computed from.
///
/// A `SourceIdentity` makes no claim about *any other* revision or source
/// text: [`SourceIdentity::check_fresh`] is the only way to ask whether it
/// still holds against a specific (revision, source) pair, and it always
/// answers explicitly rather than assuming freshness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceIdentity {
    revision: RevisionId,
    span: SourceSpan,
    content_hash: ContentHash,
}

impl SourceIdentity {
    /// Binds `span` against `source` at `revision`. Rejects a span that runs
    /// past the end of `source`, or whose start/end does not fall on a
    /// UTF-8 character boundary — this is the only way to construct a
    /// `SourceIdentity`, so every instance is guaranteed to describe a real,
    /// character-aligned slice of `source` at the moment it was built.
    pub fn bind(
        revision: RevisionId,
        source: &str,
        span: SourceSpan,
    ) -> Result<SourceIdentity, SourceIdentityError> {
        let slice = slice_for(source, span)?;
        Ok(SourceIdentity {
            revision,
            span,
            content_hash: ContentHash::of(slice.as_bytes()),
        })
    }

    pub fn revision(&self) -> &RevisionId {
        &self.revision
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_hash
    }

    /// Checks this identity against `current_revision` and `current_source`.
    ///
    /// Returns `Ok(())` only when the revision id matches exactly *and* the
    /// bytes at the bound span still hash the same in `current_source`.
    /// Every other case is a typed [`Staleness`] reason — there is no path
    /// that treats a mismatched revision or changed content as still fresh.
    pub fn check_fresh(
        &self,
        current_revision: &RevisionId,
        current_source: &str,
    ) -> Result<(), Staleness> {
        if &self.revision != current_revision {
            return Err(Staleness::RevisionChanged {
                expected: self.revision.clone(),
                found: current_revision.clone(),
            });
        }
        let slice = slice_for(current_source, self.span).map_err(|_| Staleness::RangeInvalid)?;
        if ContentHash::of(slice.as_bytes()) != self.content_hash {
            return Err(Staleness::ContentChanged);
        }
        Ok(())
    }
}

/// Shared bounds/char-boundary check used by both `bind` and `check_fresh`,
/// so staleness detection re-validates the range exactly the way binding
/// did rather than trusting stored offsets against new text.
fn slice_for(source: &str, span: SourceSpan) -> Result<&str, SourceIdentityError> {
    let start = span.start.offset as usize;
    let end = span.end.offset as usize;
    if end > source.len() {
        return Err(SourceIdentityError::SpanOutOfBounds {
            end: span.end.offset,
            source_len: source.len(),
        });
    }
    if !source.is_char_boundary(start) {
        return Err(SourceIdentityError::NotCharBoundary {
            offset: span.start.offset,
        });
    }
    if !source.is_char_boundary(end) {
        return Err(SourceIdentityError::NotCharBoundary {
            offset: span.end.offset,
        });
    }
    Ok(&source[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::SourcePos;

    fn rev(s: &str) -> RevisionId {
        RevisionId::parse(s).unwrap()
    }

    #[test]
    fn rejects_empty_revision() {
        assert_eq!(RevisionId::parse("").unwrap_err(), RevisionError::Empty);
    }

    #[test]
    fn rejects_overlong_revision() {
        let long = "a".repeat(MAX_REVISION_LEN + 1);
        let err = RevisionId::parse(&long).unwrap_err();
        assert_eq!(
            err,
            RevisionError::TooLong {
                len: long.len(),
                max: MAX_REVISION_LEN
            }
        );
    }

    #[test]
    fn bounded_against_absurdly_long_revision() {
        let hostile = "a".repeat(50_000_000);
        let err = RevisionId::parse(&hostile).unwrap_err();
        assert_eq!(
            err,
            RevisionError::TooLong {
                len: hostile.len(),
                max: MAX_REVISION_LEN
            }
        );
    }

    #[test]
    fn rejects_control_character_in_revision() {
        let err = RevisionId::parse("abc\ndef").unwrap_err();
        assert!(matches!(err, RevisionError::ControlCharacter { .. }));
    }

    #[test]
    fn binds_to_exact_slice_and_hashes_it() {
        let source = "see \\href{https://example.com}{here} for more";
        let span = SourceSpan::new(SourcePos::new(4, 1, 5), SourcePos::new(37, 1, 38)).unwrap();
        let identity = SourceIdentity::bind(rev("r1"), source, span).unwrap();
        assert_eq!(identity.revision().as_str(), "r1");
        assert_eq!(identity.span().start.offset, 4);
        // Same bytes, hashed independently, must agree.
        let expected = ContentHash::of(&source.as_bytes()[4..37]);
        assert_eq!(identity.content_hash(), expected);
    }

    #[test]
    fn rejects_span_past_end_of_source() {
        let source = "short";
        let span = SourceSpan::new(SourcePos::new(0, 1, 1), SourcePos::new(50, 1, 51)).unwrap();
        let err = SourceIdentity::bind(rev("r1"), source, span).unwrap_err();
        assert_eq!(
            err,
            SourceIdentityError::SpanOutOfBounds {
                end: 50,
                source_len: source.len()
            }
        );
    }

    #[test]
    fn rejects_span_that_splits_a_multibyte_character() {
        // "日" is 3 bytes (0xE6 0x97 0xA5); offset 1 lands inside it.
        let source = "日本語のリンク";
        let bad_end = SourcePos::new(1, 1, 1);
        let span = SourceSpan::new(SourcePos::new(0, 1, 1), bad_end).unwrap();
        let err = SourceIdentity::bind(rev("r1"), source, span).unwrap_err();
        assert_eq!(err, SourceIdentityError::NotCharBoundary { offset: 1 });
    }

    #[test]
    fn accepts_multibyte_span_aligned_on_character_boundaries() {
        let source = "日本語のリンクです";
        // "日本語" is 3 chars * 3 bytes = 9 bytes, all boundary-aligned.
        let span = SourceSpan::new(SourcePos::new(0, 1, 1), SourcePos::new(9, 1, 4)).unwrap();
        let identity = SourceIdentity::bind(rev("r1"), source, span).unwrap();
        assert_eq!(identity.span().len(), 9);
    }

    #[test]
    fn fresh_when_revision_and_bytes_are_unchanged() {
        let source = "click \\href{https://example.com}{me}";
        let span = SourceSpan::new(SourcePos::new(6, 1, 7), SourcePos::new(33, 1, 34)).unwrap();
        let identity = SourceIdentity::bind(rev("r1"), source, span).unwrap();
        assert_eq!(identity.check_fresh(&rev("r1"), source), Ok(()));
    }

    #[test]
    fn stale_when_revision_id_differs_even_if_bytes_are_identical() {
        // The whole point of binding a revision: identical bytes at a
        // different revision must still be flagged, not silently accepted.
        let source = "click \\href{https://example.com}{me}";
        let span = SourceSpan::new(SourcePos::new(6, 1, 7), SourcePos::new(33, 1, 34)).unwrap();
        let identity = SourceIdentity::bind(rev("r1"), source, span).unwrap();
        let err = identity.check_fresh(&rev("r2"), source).unwrap_err();
        assert_eq!(
            err,
            Staleness::RevisionChanged {
                expected: rev("r1"),
                found: rev("r2"),
            }
        );
    }

    #[test]
    fn stale_when_source_bytes_at_the_span_changed_under_the_same_revision() {
        let original = "click \\href{https://example.com}{me}";
        let span = SourceSpan::new(SourcePos::new(6, 1, 7), SourcePos::new(33, 1, 34)).unwrap();
        let identity = SourceIdentity::bind(rev("r1"), original, span).unwrap();

        // Same revision id, but the bytes under the same span were edited.
        let edited = "click \\href{https://evil.example}{me}";
        let err = identity.check_fresh(&rev("r1"), edited).unwrap_err();
        assert_eq!(err, Staleness::ContentChanged);
    }

    #[test]
    fn stale_when_source_shrinks_so_the_span_no_longer_fits() {
        let original = "click \\href{https://example.com}{me}";
        let span = SourceSpan::new(SourcePos::new(6, 1, 7), SourcePos::new(33, 1, 34)).unwrap();
        let identity = SourceIdentity::bind(rev("r1"), original, span).unwrap();

        let truncated = "click";
        let err = identity.check_fresh(&rev("r1"), truncated).unwrap_err();
        assert_eq!(err, Staleness::RangeInvalid);
    }

    #[test]
    fn stale_check_is_utf8_safe_when_multibyte_source_shifted() {
        let original = "見よ \\href{https://example.com}{ここ} です";
        // Compute the byte offsets for the href text span from the actual
        // string so this test does not hardcode brittle offsets.
        let start = original.find("\\href").unwrap() as u32;
        let end = start + "\\href{https://example.com}{ここ}".len() as u32;
        let span = SourceSpan::new(SourcePos::new(start, 1, 1), SourcePos::new(end, 1, 1)).unwrap();
        let identity = SourceIdentity::bind(rev("r1"), original, span).unwrap();
        assert_eq!(identity.check_fresh(&rev("r1"), original), Ok(()));

        // Insert a multibyte character before the span in a "new revision";
        // the absolute byte range now points at shifted, mismatched bytes.
        let shifted = format!("見よ絵 {}", &original[original.find('\\').unwrap()..]);
        let err = identity.check_fresh(&rev("r2"), &shifted).unwrap_err();
        assert_eq!(
            err,
            Staleness::RevisionChanged {
                expected: rev("r1"),
                found: rev("r2"),
            }
        );
    }
}
