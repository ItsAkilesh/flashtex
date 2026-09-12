use std::fmt;

/// Everything that can go wrong parsing or expanding a snippet.
///
/// Every variant that names an `offset` carries a byte offset into the
/// original source text; it is always a valid `char` boundary, safe to use
/// as a slice index or to display alongside the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnippetError {
    /// The source was larger than [`crate::MAX_INPUT_BYTES`].
    InputTooLarge {
        /// The source length, in bytes, that was rejected.
        len: usize,
    },
    /// A `\` escape at the end of input with nothing left to escape.
    UnterminatedEscape {
        /// Offset of the trailing `\`.
        offset: usize,
    },
    /// A `\` followed by a character that is not one of the escapable
    /// characters (`$`, `}`, `\`).
    InvalidEscape {
        /// Offset of the `\`.
        offset: usize,
        /// The character that followed it.
        found: char,
    },
    /// `${` was opened but never closed with a matching `}`.
    UnterminatedPlaceholder {
        /// Offset of the `$` that opened the placeholder.
        offset: usize,
    },
    /// `${` (or `${N:`) was not followed by at least one decimal digit
    /// where a placeholder index was expected.
    InvalidPlaceholderIndex {
        /// Offset where a digit was expected.
        offset: usize,
    },
    /// The placeholder index parsed but is above
    /// [`crate::MAX_PLACEHOLDER_INDEX`] (or otherwise does not fit).
    PlaceholderIndexTooLarge {
        /// Offset of the `$` that introduced the placeholder.
        offset: usize,
    },
    /// A placeholder default was nested more than
    /// [`crate::MAX_NESTING_DEPTH`] levels deep, e.g.
    /// `${1:${2:${3:...}}}` repeated past the bound.
    NestingTooDeep {
        /// Offset of the `$` that would have exceeded the bound.
        offset: usize,
    },
    /// The snippet defines more distinct placeholder indices than
    /// [`crate::MAX_PLACEHOLDERS`].
    TooManyPlaceholders {
        /// The number of distinct indices seen before the bound tripped.
        count: usize,
    },
    /// The snippet has more placeholder occurrences than
    /// [`crate::MAX_OCCURRENCES`].
    TooManyOccurrences {
        /// The number of occurrences seen before the bound tripped.
        count: usize,
    },
    /// A placeholder's default text refers back to itself, directly
    /// (`${1:$1}`) or transitively through another placeholder
    /// (`${1:$2}` together with `${2:$1}`).
    SelfReferential {
        /// The index at which the cycle was detected.
        index: u32,
    },
    /// The expanded text would exceed [`crate::MAX_OUTPUT_BYTES`].
    OutputTooLarge {
        /// The output length, in bytes, that was rejected.
        len: usize,
    },
}

impl fmt::Display for SnippetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputTooLarge { len } => {
                write!(f, "snippet source is {len} bytes, exceeding the limit")
            }
            Self::UnterminatedEscape { offset } => {
                write!(f, "dangling '\\' at byte offset {offset}")
            }
            Self::InvalidEscape { offset, found } => {
                write!(
                    f,
                    "'\\{found}' is not a valid escape at byte offset {offset}"
                )
            }
            Self::UnterminatedPlaceholder { offset } => {
                write!(f, "unterminated '${{' starting at byte offset {offset}")
            }
            Self::InvalidPlaceholderIndex { offset } => {
                write!(
                    f,
                    "expected a decimal placeholder index at byte offset {offset}"
                )
            }
            Self::PlaceholderIndexTooLarge { offset } => {
                write!(f, "placeholder index at byte offset {offset} is too large")
            }
            Self::NestingTooDeep { offset } => {
                write!(
                    f,
                    "placeholder default nesting is too deep at byte offset {offset}"
                )
            }
            Self::TooManyPlaceholders { count } => {
                write!(
                    f,
                    "snippet defines {count} distinct placeholders, exceeding the limit"
                )
            }
            Self::TooManyOccurrences { count } => {
                write!(
                    f,
                    "snippet has {count} placeholder occurrences, exceeding the limit"
                )
            }
            Self::SelfReferential { index } => {
                write!(f, "placeholder ${index} refers back to itself")
            }
            Self::OutputTooLarge { len } => {
                write!(
                    f,
                    "expanded snippet would be {len} bytes, exceeding the limit"
                )
            }
        }
    }
}

impl std::error::Error for SnippetError {}
