//! Typed errors for parsing and resolving colour expressions.
//!
//! Every failure mode is a distinct variant with the byte offset (where
//! applicable) into the *original* input string. No variant silently
//! degrades to a default colour: an unknown palette name is always an
//! error, never black.

use std::fmt;

/// Everything that can go wrong parsing or resolving a colour expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorExprError {
    /// The input was empty (after confirming it is not just too long).
    Empty,
    /// The input exceeded [`crate::MAX_INPUT_LEN`] bytes. Checked before any
    /// parsing work happens, so a huge input costs O(1), not O(n).
    TooLong { len: usize, max: usize },
    /// The expression nests deeper (through parentheses, `-` chains, or a
    /// long `!`-mix chain) than [`crate::MAX_DEPTH`] allows. This is the
    /// bound that stops unbounded recursion on a hostile input.
    TooDeep { max: usize },
    /// A byte offset into the input held a character no production expects.
    UnexpectedChar { pos: usize, found: char },
    /// The input ended mid-token (e.g. trailing `!`, unmatched `(`, a
    /// dangling `-`).
    UnexpectedEnd,
    /// Extra, unparsed input remained after a complete expression (e.g. an
    /// unmatched trailing `)`).
    TrailingInput { pos: usize },
    /// A `!`-separated weight was not an integer in `0..=100`.
    InvalidPercentage { pos: usize, text: String },
    /// A name resolved to nothing in the supplied palette. Never silently
    /// treated as black or any other default.
    UnknownColor { name: String },
}

impl fmt::Display for ColorExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorExprError::Empty => write!(f, "colour expression is empty"),
            ColorExprError::TooLong { len, max } => {
                write!(
                    f,
                    "colour expression is {len} bytes, over the {max}-byte limit"
                )
            }
            ColorExprError::TooDeep { max } => {
                write!(f, "colour expression nests deeper than the limit of {max}")
            }
            ColorExprError::UnexpectedChar { pos, found } => {
                write!(f, "unexpected character {found:?} at byte offset {pos}")
            }
            ColorExprError::UnexpectedEnd => {
                write!(f, "colour expression ended unexpectedly")
            }
            ColorExprError::TrailingInput { pos } => {
                write!(f, "unexpected trailing input at byte offset {pos}")
            }
            ColorExprError::InvalidPercentage { pos, text } => {
                write!(
                    f,
                    "invalid mix weight {text:?} at byte offset {pos}: must be an integer in 0..=100"
                )
            }
            ColorExprError::UnknownColor { name } => {
                write!(f, "unknown palette colour {name:?}")
            }
        }
    }
}

impl std::error::Error for ColorExprError {}
