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
    /// A `model:components` literal (e.g. `rgb:1,0,0`) named a colour model
    /// this crate does not support. Only `gray`, `rgb`, and `cmyk` are
    /// recognised — exactly the three variants of
    /// [`flashtex_vector_graphics::Color`]. Never approximated into a
    /// supported model or defaulted to black.
    UnsupportedColorModel { pos: usize, name: String },
    /// A `model:components` literal supplied the wrong number of
    /// comma-separated components for its model (`gray` takes 1, `rgb`
    /// takes 3, `cmyk` takes 4).
    InvalidComponentCount {
        model: String,
        expected: usize,
        found: usize,
    },
    /// A component of a `model:components` literal parsed as a number but
    /// fell outside the required `0.0..=1.0` range.
    ComponentOutOfRange { pos: usize, text: String },
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
            ColorExprError::UnsupportedColorModel { pos, name } => {
                write!(
                    f,
                    "unsupported colour model {name:?} at byte offset {pos}: only gray, rgb, and cmyk literals are supported"
                )
            }
            ColorExprError::InvalidComponentCount {
                model,
                expected,
                found,
            } => {
                write!(
                    f,
                    "{model:?} literal takes {expected} component(s), found {found}"
                )
            }
            ColorExprError::ComponentOutOfRange { pos, text } => {
                write!(
                    f,
                    "colour component {text:?} at byte offset {pos} is out of the required 0.0..=1.0 range"
                )
            }
        }
    }
}

impl std::error::Error for ColorExprError {}
