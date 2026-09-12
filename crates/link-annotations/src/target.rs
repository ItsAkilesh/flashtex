//! Internal targets: references to labels defined elsewhere in the same
//! document (for example, a cross-reference to a figure or section).
//!
//! This module never invents, guesses at, or defers resolution of a label.
//! It only confirms that a reference names a label the caller has told it
//! exists in the document being exported, and hands back the exportable
//! [`PageTarget`] (page + rectangle) that label was registered with. An
//! unresolvable reference is always a typed error — this module has no path
//! that produces a dangling internal destination.

use crate::page::PageTarget;
use std::collections::HashMap;
use std::fmt;

/// The longest label id this crate will accept.
pub const MAX_LABEL_LEN: usize = 256;

/// A validated label identifier: non-empty, within the length bound, and
/// free of control characters. Carries no claim that the label is actually
/// defined anywhere — see [`InternalTarget::resolve`] for that.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LabelId(String);

impl LabelId {
    /// Validates a label reference. Bounded: the length check runs before
    /// any scan, so cost never exceeds [`MAX_LABEL_LEN`] bytes of work.
    pub fn parse(text: &str) -> Result<LabelId, LabelError> {
        if text.is_empty() {
            return Err(LabelError::Empty);
        }
        if text.len() > MAX_LABEL_LEN {
            return Err(LabelError::TooLong {
                len: text.len(),
                max: MAX_LABEL_LEN,
            });
        }
        if let Some((at, _)) = text.char_indices().find(|(_, c)| c.is_control()) {
            return Err(LabelError::ControlCharacter { at });
        }
        Ok(LabelId(text.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LabelError {
    Empty,
    TooLong { len: usize, max: usize },
    ControlCharacter { at: usize },
}

impl fmt::Display for LabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabelError::Empty => write!(f, "label is empty"),
            LabelError::TooLong { len, max } => {
                write!(f, "label is {len} bytes, exceeds the {max}-byte bound")
            }
            LabelError::ControlCharacter { at } => {
                write!(f, "label contains a control character at byte offset {at}")
            }
        }
    }
}

impl std::error::Error for LabelError {}

/// The set of labels known to be defined in the document being exported,
/// each mapped to the exportable [`PageTarget`] (page + rectangle) a PDF
/// exporter would jump to. This crate does not discover labels itself: the
/// caller (the document model) supplies them.
#[derive(Clone, Debug, Default)]
pub struct LabelSet(HashMap<LabelId, PageTarget>);

impl LabelSet {
    pub fn new() -> LabelSet {
        LabelSet(HashMap::new())
    }

    /// Records `id` as defined at `target`. Returns the previous target if
    /// `id` was already present (last registration wins, same as
    /// `HashMap::insert`).
    pub fn insert(&mut self, id: LabelId, target: PageTarget) -> Option<PageTarget> {
        self.0.insert(id, target)
    }

    pub fn contains(&self, id: &LabelId) -> bool {
        self.0.contains_key(id)
    }

    pub fn get(&self, id: &LabelId) -> Option<&PageTarget> {
        self.0.get(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromIterator<(LabelId, PageTarget)> for LabelSet {
    fn from_iter<T: IntoIterator<Item = (LabelId, PageTarget)>>(iter: T) -> Self {
        LabelSet(iter.into_iter().collect())
    }
}

/// An internal link destination that has been confirmed to resolve to a
/// label defined in this document, carrying the exportable page target
/// (page + rectangle) a PDF exporter needs to build a `/GoTo` destination.
#[derive(Clone, Debug, PartialEq)]
pub struct InternalTarget {
    label: LabelId,
    page_target: PageTarget,
}

impl InternalTarget {
    pub fn label(&self) -> &LabelId {
        &self.label
    }

    pub fn page_target(&self) -> PageTarget {
        self.page_target
    }

    /// Resolves `label` against `known`. Fails explicitly, naming the label
    /// that could not be found, rather than ever producing a dangling
    /// target — there is no code path in this function that returns `Ok`
    /// for a label absent from `known`.
    pub fn resolve(label: LabelId, known: &LabelSet) -> Result<InternalTarget, TargetError> {
        match known.get(&label) {
            Some(target) => Ok(InternalTarget {
                label,
                page_target: *target,
            }),
            None => Err(TargetError::Unresolved(label)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetError {
    Unresolved(LabelId),
}

impl fmt::Display for TargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetError::Unresolved(label) => {
                write!(
                    f,
                    "internal target `{label}` does not resolve to any known label"
                )
            }
        }
    }
}

impl std::error::Error for TargetError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Point, Rect};
    use crate::page::PageIndex;

    fn page_target(page: u32) -> PageTarget {
        PageTarget::new(
            PageIndex::new(page),
            Rect::new(Point::new(0.0, 0.0), 10.0, 10.0).unwrap(),
        )
    }

    #[test]
    fn rejects_empty_label() {
        assert_eq!(LabelId::parse("").unwrap_err(), LabelError::Empty);
    }

    #[test]
    fn rejects_overlong_label() {
        let long = "x".repeat(MAX_LABEL_LEN + 1);
        let err = LabelId::parse(&long).unwrap_err();
        assert_eq!(
            err,
            LabelError::TooLong {
                len: long.len(),
                max: MAX_LABEL_LEN
            }
        );
    }

    #[test]
    fn max_length_boundary_is_inclusive() {
        let exact = "x".repeat(MAX_LABEL_LEN);
        assert!(LabelId::parse(&exact).is_ok());
    }

    #[test]
    fn rejects_control_character_in_label() {
        let err = LabelId::parse("fig:one\ntwo").unwrap_err();
        assert!(matches!(err, LabelError::ControlCharacter { .. }));
    }

    #[test]
    fn accepts_unicode_label() {
        let label = LabelId::parse("図:一").unwrap();
        assert_eq!(label.as_str(), "図:一");
    }

    #[test]
    fn bounded_against_absurdly_long_label() {
        let hostile = "x".repeat(50_000_000);
        let err = LabelId::parse(&hostile).unwrap_err();
        assert_eq!(
            err,
            LabelError::TooLong {
                len: hostile.len(),
                max: MAX_LABEL_LEN
            }
        );
    }

    #[test]
    fn resolves_known_label_and_carries_its_page_target() {
        let id = LabelId::parse("fig:tree").unwrap();
        let mut known = LabelSet::new();
        known.insert(id.clone(), page_target(2));
        let target = InternalTarget::resolve(id.clone(), &known).unwrap();
        assert_eq!(target.label(), &id);
        assert_eq!(target.page_target().page.value(), 2);
    }

    #[test]
    fn fails_explicitly_on_unresolvable_label() {
        let id = LabelId::parse("fig:missing").unwrap();
        let known = LabelSet::new();
        let err = InternalTarget::resolve(id.clone(), &known).unwrap_err();
        assert_eq!(err, TargetError::Unresolved(id));
    }

    #[test]
    fn does_not_resolve_against_unrelated_labels() {
        let wanted = LabelId::parse("sec:intro").unwrap();
        let mut known = LabelSet::new();
        known.insert(LabelId::parse("sec:conclusion").unwrap(), page_target(0));
        let err = InternalTarget::resolve(wanted.clone(), &known).unwrap_err();
        assert_eq!(err, TargetError::Unresolved(wanted));
    }

    #[test]
    fn last_registration_for_a_label_wins_and_returns_previous_target() {
        let id = LabelId::parse("fig:tree").unwrap();
        let mut known = LabelSet::new();
        let previous = known.insert(id.clone(), page_target(1));
        assert_eq!(previous, None);
        let previous = known.insert(id.clone(), page_target(9));
        assert_eq!(previous, Some(page_target(1)));
        let target = InternalTarget::resolve(id, &known).unwrap();
        assert_eq!(target.page_target().page.value(), 9);
    }
}
