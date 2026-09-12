//! Exact identity of the revision statistics were computed from.

use std::fmt;

/// The exact revision a [`crate::Statistics`] value was computed from.
///
/// FlashTeX does not define what a "revision" is — the caller does, by
/// pairing a `source` identifier (a document id, a project-index key, a file
/// path — this crate assigns no meaning to the string) with a `revision`
/// number the caller is expected to increase every time that source's
/// rendered content changes.
///
/// Two `RevisionId`s are equal only if both fields match exactly. There is
/// no fuzzy, prefix, or "close enough" comparison anywhere in this crate.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RevisionId {
    /// Caller-defined identifier for the source document.
    pub source: String,
    /// Caller-defined revision number for that source.
    pub revision: u64,
}

impl RevisionId {
    /// Build a revision identity from a source id and revision number.
    pub fn new(source: impl Into<String>, revision: u64) -> Self {
        RevisionId {
            source: source.into(),
            revision,
        }
    }
}

impl fmt::Display for RevisionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.source, self.revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_requires_both_fields() {
        let a = RevisionId::new("draft.tex", 1);
        let b = RevisionId::new("draft.tex", 2);
        let c = RevisionId::new("other.tex", 1);
        assert_eq!(a, RevisionId::new("draft.tex", 1));
        assert_ne!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn display_shows_source_and_revision() {
        assert_eq!(RevisionId::new("draft.tex", 3).to_string(), "draft.tex@3");
    }
}
