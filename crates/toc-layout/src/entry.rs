//! Explicit section/page records for a contents-style entry (a
//! `\tableofcontents` or `\listoffigures` line).
//!
//! An [`EntryRecord`] always carries an *explicit* absolute page number: this
//! crate never infers, approximates, or renumbers a page on its own. A
//! [`RelativeEntry`] carries an explicit offset *within the body* instead,
//! for the case where the absolute page is not known until the front matter
//! (the contents/figures lists themselves) has been paginated — see
//! [`crate::converge`].

use std::fmt;

/// One-based physical page number as printed in the rendered document.
pub type PageNumber = u32;

/// Maximum nesting depth accepted for a contents entry (0 = top level, e.g.
/// `\section`; deeper levels are `\subsection`, `\subsubsection`, ...).
pub const MAX_LEVEL: u8 = 5;

/// A malformed entry input, rejected before any layout work is attempted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryError {
    /// The title is empty, or contains only whitespace.
    EmptyTitle,
    /// Page zero was given; this model has no page ordinal below 1.
    PageZero,
    /// `level` exceeds [`MAX_LEVEL`].
    LevelTooDeep { level: u8, max: u8 },
    /// A body offset of zero was given for a [`RelativeEntry`]; the first
    /// body page is offset 1, not 0.
    BodyOffsetZero,
    /// `front_matter_pages + body_offset` overflowed [`PageNumber`].
    PageOverflow,
}

impl fmt::Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryError::EmptyTitle => write!(f, "entry title is empty or all whitespace"),
            EntryError::PageZero => write!(f, "page number is zero; pages are numbered from 1"),
            EntryError::LevelTooDeep { level, max } => {
                write!(f, "entry level {level} exceeds the maximum of {max}")
            }
            EntryError::BodyOffsetZero => {
                write!(f, "body offset is zero; body pages are numbered from 1")
            }
            EntryError::PageOverflow => {
                write!(
                    f,
                    "front matter pages plus body offset overflowed the page counter"
                )
            }
        }
    }
}

impl std::error::Error for EntryError {}

/// A contents/list-of-figures entry with an explicit, already-resolved
/// absolute page number.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryRecord {
    pub title: String,
    pub page: PageNumber,
    pub level: u8,
}

impl EntryRecord {
    /// Validates and builds an entry. Rejects an empty/whitespace-only
    /// title, page zero, and a level beyond [`MAX_LEVEL`].
    pub fn new(title: impl Into<String>, page: PageNumber, level: u8) -> Result<Self, EntryError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(EntryError::EmptyTitle);
        }
        if page == 0 {
            return Err(EntryError::PageZero);
        }
        if level > MAX_LEVEL {
            return Err(EntryError::LevelTooDeep {
                level,
                max: MAX_LEVEL,
            });
        }
        Ok(Self { title, page, level })
    }
}

/// A contents/list-of-figures entry whose page is not yet absolute: it is
/// `body_offset` pages into the body, and the body starts right after
/// however many pages the front matter (this entry's own list) ends up
/// occupying. See [`crate::converge::converge_front_matter_pages`] for how
/// that page count is found.
/// `title`, `level`, and `body_offset` are deliberately private:
/// [`RelativeEntry::new`] is the only way to build one, so `body_offset`
/// is never zero for any live `RelativeEntry`. If the fields were `pub`,
/// a caller could assemble a `RelativeEntry { title, level, body_offset }`
/// struct literal directly, skip that check, and hand [`RelativeEntry::resolve`]
/// a `body_offset: 0` — `front_matter_pages.checked_add(0)` never
/// overflows, so `resolve` would succeed with `page == front_matter_pages`,
/// silently landing a body entry on the front matter's own last page
/// instead of the first page of the body, with no error anywhere.
#[derive(Debug, Clone, PartialEq)]
pub struct RelativeEntry {
    title: String,
    level: u8,
    body_offset: PageNumber,
}

impl RelativeEntry {
    /// Validates and builds a relative entry. Rejects an empty/whitespace
    /// title, a level beyond [`MAX_LEVEL`], and a body offset of zero.
    pub fn new(
        title: impl Into<String>,
        level: u8,
        body_offset: PageNumber,
    ) -> Result<Self, EntryError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(EntryError::EmptyTitle);
        }
        if level > MAX_LEVEL {
            return Err(EntryError::LevelTooDeep {
                level,
                max: MAX_LEVEL,
            });
        }
        if body_offset == 0 {
            return Err(EntryError::BodyOffsetZero);
        }
        Ok(Self {
            title,
            level,
            body_offset,
        })
    }

    /// The entry's title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The entry's nesting level (see [`EntryRecord::level`]).
    pub fn level(&self) -> u8 {
        self.level
    }

    /// Pages into the body this entry resolves to (never zero).
    pub fn body_offset(&self) -> PageNumber {
        self.body_offset
    }

    /// Resolves this entry to an absolute [`EntryRecord`] once the number
    /// of front-matter pages preceding the body is known.
    pub fn resolve(&self, front_matter_pages: PageNumber) -> Result<EntryRecord, EntryError> {
        let page = front_matter_pages
            .checked_add(self.body_offset)
            .ok_or(EntryError::PageOverflow)?;
        EntryRecord::new(self.title.clone(), page, self.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for a defect where `RelativeEntry`'s fields being
    /// `pub` let a caller build one via a
    /// `RelativeEntry { title, level, body_offset }` struct literal,
    /// skipping `new`'s `body_offset != 0` check entirely. A
    /// `body_offset: 0` built that way made `resolve` succeed with
    /// `page == front_matter_pages` — silently landing a body entry on
    /// the front matter's own last page, with no error anywhere.
    ///
    /// Now that the fields are private, `RelativeEntry::new` is the only
    /// way to build one (the struct-literal bypass is a compile error
    /// from outside this module), so this same zero body offset must be
    /// rejected at construction, never reach `resolve` at all.
    #[test]
    fn zero_body_offset_is_a_typed_error_not_a_bypassable_field() {
        let err = RelativeEntry::new("Chapter 1", 0, 0).unwrap_err();
        assert_eq!(err, EntryError::BodyOffsetZero);
    }
}
