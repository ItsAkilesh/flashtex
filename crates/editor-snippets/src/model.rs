use crate::error::SnippetError;

/// One piece of a parsed snippet template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Segment {
    /// Literal text, copied through verbatim on expansion.
    Text(String),
    /// A numbered placeholder. `default` is `Some(_)` only for occurrences
    /// written as `${N:...}`; bare `$N` and `${N}` carry `None`.
    Placeholder {
        index: u32,
        default: Option<Vec<Segment>>,
    },
}

/// A parsed snippet template, ready to be expanded.
///
/// Parsing (see [`Snippet::parse`]) is the only fallible step; a
/// successfully parsed `Snippet` can be expanded any number of times
/// (via [`Snippet::expand`] or [`Snippet::expand_with`]) without re-parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    pub(crate) segments: Vec<Segment>,
}

impl Snippet {
    /// Parse `source` into a [`Snippet`].
    ///
    /// Supported syntax:
    /// - `$1`, `$2`, ... - a bare numbered placeholder.
    /// - `${1}` - the same, with explicit braces.
    /// - `${1:default text}` - a placeholder whose *first* occurrence with
    ///   a default in the source supplies the initial text for every
    ///   occurrence of index `1`. The default text may itself contain
    ///   further placeholders, bounded by [`crate::MAX_NESTING_DEPTH`].
    /// - `\$`, `\}`, `\\` - escapes for a literal `$`, `}`, or `\`.
    ///
    /// # Errors
    /// Returns [`SnippetError`] for malformed syntax (unterminated
    /// placeholders, invalid escapes, missing indices) or for input that
    /// would exceed a documented bound in the [`crate::limits`] module.
    /// Parsing never panics and never loops: every recursive descent is
    /// depth-limited by [`crate::MAX_NESTING_DEPTH`] before it recurses.
    pub fn parse(source: &str) -> Result<Snippet, SnippetError> {
        crate::parser::parse(source)
    }
}
