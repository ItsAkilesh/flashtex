//! Document identity: what a [`crate::SnippetPlan`] is computed against,
//! and how to tell a plan is stale.

use std::fmt;

/// A content hash of a document's exact UTF-8 bytes.
///
/// This is a plain FNV-1a 64-bit hash: dependency-free and fully
/// deterministic for the same bytes, on this machine or any other, in this
/// process or a later one. (`std::collections::hash_map::DefaultHasher` is
/// deliberately not used here: the standard library documents its
/// algorithm as unspecified and subject to change between Rust versions,
/// which would make a [`DocumentId`] saved from one build meaningless when
/// compared under a different one.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash(u64);

impl ContentHash {
    /// Hashes `text`'s exact bytes.
    pub fn of(text: &str) -> ContentHash {
        const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        let mut hash = OFFSET_BASIS;
        for byte in text.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(PRIME);
        }
        ContentHash(hash)
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Identifies one exact state of a document: the caller's own revision
/// counter (an id, a timestamp, an edit sequence number — this crate
/// treats it as an opaque `u64` it never interprets) plus a
/// [`ContentHash`] of the document's exact bytes at that revision.
///
/// A [`crate::SnippetPlan`] is computed against one `DocumentId`. Compare
/// it with the document's current `DocumentId` using [`DocumentId::compare`]
/// (or [`crate::SnippetPlan::staleness`]) before applying it: a caller's
/// revision counter can fail to change even though the underlying bytes
/// did, so this deliberately checks both fields rather than trusting the
/// revision alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentId {
    /// The caller's own revision id, opaque to this crate.
    pub revision: u64,
    /// A hash of the document's exact bytes at `revision`.
    pub content_hash: ContentHash,
}

impl DocumentId {
    /// Builds the identity of `text` at `revision`.
    pub fn new(revision: u64, text: &str) -> DocumentId {
        DocumentId {
            revision,
            content_hash: ContentHash::of(text),
        }
    }

    /// Compares `self` (typically the identity a plan was computed
    /// against) with `current` (the document's identity now), returning a
    /// typed [`Staleness`] rather than a bare `bool`.
    pub fn compare(&self, current: &DocumentId) -> Staleness {
        if self.revision != current.revision {
            return Staleness::RevisionMismatch {
                planned: self.revision,
                current: current.revision,
            };
        }
        if self.content_hash != current.content_hash {
            return Staleness::ContentMismatch {
                planned: self.content_hash,
                current: current.content_hash,
            };
        }
        Staleness::Fresh
    }
}

/// The result of comparing the [`DocumentId`] a plan was computed against
/// with a document's current identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staleness {
    /// Same revision, same content hash: the plan's `anchor` and
    /// `expansion` are still safe to apply as computed.
    Fresh,
    /// The caller's own revision counter differs from the one the plan was
    /// computed against.
    RevisionMismatch {
        /// The revision the plan was computed against.
        planned: u64,
        /// The document's revision now.
        current: u64,
    },
    /// The revision counter is unchanged but the content hash differs:
    /// bytes were edited without the caller's revision id changing to
    /// reflect it. A plan is never safe to apply in this state, even
    /// though comparing `revision` alone would suggest it is fresh.
    ContentMismatch {
        /// The content hash the plan was computed against.
        planned: ContentHash,
        /// The document's content hash now.
        current: ContentHash,
    },
}

impl Staleness {
    /// True only for [`Staleness::Fresh`].
    pub fn is_fresh(&self) -> bool {
        matches!(self, Staleness::Fresh)
    }
}
