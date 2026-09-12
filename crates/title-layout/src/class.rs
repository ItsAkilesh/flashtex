//! Which LaTeX document class a document declares.
//!
//! `flashtex-document-style` only models `article` (see its crate docs and
//! `Stylesheet::article`): its page geometry, size table, and sectioning
//! spacing are all transcribed from `article.cls`/`size1x.clo`. `report` and
//! `book` redefine `\maketitle`, the `abstract` environment (or omit it, for
//! `book`), and much of the size/geometry machinery differently, and
//! `letter` has no `\maketitle`/`abstract` at all. This crate has no
//! measurements for any of those, so it never guesses at them.

use std::fmt;

/// A `\documentclass{...}` name relevant to title/abstract layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DocumentClass {
    /// The only class this crate measures.
    Article,
    Report,
    Book,
    Letter,
}

impl DocumentClass {
    /// The class name as written after `\documentclass`.
    pub fn name(self) -> &'static str {
        match self {
            DocumentClass::Article => "article",
            DocumentClass::Report => "report",
            DocumentClass::Book => "book",
            DocumentClass::Letter => "letter",
        }
    }

    /// Parses a bare class name (no options), matching `\documentclass{name}`.
    /// Unrecognized names return `None`, not a guess.
    pub fn parse(s: &str) -> Option<DocumentClass> {
        Some(match s {
            "article" => DocumentClass::Article,
            "report" => DocumentClass::Report,
            "book" => DocumentClass::Book,
            "letter" => DocumentClass::Letter,
            _ => return None,
        })
    }

    /// Whether this crate has measurements for `class`. Only `article` does.
    pub fn is_supported(self) -> bool {
        matches!(self, DocumentClass::Article)
    }
}

impl fmt::Display for DocumentClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
