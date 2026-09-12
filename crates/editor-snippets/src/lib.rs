//! Bounded, UTF-8-safe editor snippet expansion.
//!
//! This crate parses a snippet template containing numbered placeholders
//! (`$1`, `$2`, `${1:default text}`) into an expanded plain-text buffer plus
//! exact byte-offset spans for every placeholder occurrence. Occurrences
//! that share the same index are *linked*: [`Snippet::expand_with`] lets a
//! caller supply new text for one index and every occurrence of that index
//! resolves to the same new text.
//!
//! This crate never mutates a source buffer. It only computes the expanded
//! text and the byte offsets a caller should apply; applying those offsets
//! to a real document (and re-running expansion after an edit) is the
//! caller's responsibility.
//!
//! Every offset this crate returns is a valid `char` boundary into the
//! returned text, even for input containing multi-byte UTF-8 (accented
//! Latin, CJK, emoji, etc.) - see the `parser` module for how that is
//! guaranteed structurally rather than by ad-hoc checks. The same
//! guarantee extends to a [`SnippetPlan`]'s [`Anchor`]: its caret and
//! selection offsets are validated against the exact document text a plan
//! is computed against, so every offset it carries is a valid `char`
//! boundary too.
//!
//! All input and every internal recursion is bounded (see the [`limits`]
//! module): a snippet with absurd nesting, a huge placeholder index, or a
//! self-referential structure returns a typed [`SnippetError`] instead of
//! looping or allocating without limit.
//!
//! ```
//! use flashtex_editor_snippets::Snippet;
//! use std::collections::HashMap;
//!
//! let snippet = Snippet::parse("Hello, ${1:World}! $1 says hi to $1.").unwrap();
//! let expansion = snippet.expand().unwrap();
//! assert_eq!(expansion.text, "Hello, World! World says hi to World.");
//!
//! // Editing one occurrence of $1 updates every linked occurrence.
//! let mut overrides = HashMap::new();
//! overrides.insert(1, "Rust".to_string());
//! let edited = snippet.expand_with(&overrides).unwrap();
//! assert_eq!(edited.text, "Hello, Rust! Rust says hi to Rust.");
//! ```
//!
//! A [`SnippetPlan`] binds an expansion to the exact document state and
//! caret/selection it was computed against, so a caller can detect a
//! stale plan (the document moved on) before applying it. This crate
//! never mutates a document either way — a plan is inert data, and
//! applying it to a real buffer is always the caller's job:
//!
//! ```
//! use flashtex_editor_snippets::{Anchor, DocumentId, Snippet, SnippetPlan};
//! use std::collections::HashMap;
//!
//! let doc = "func ()";
//! let snippet = Snippet::parse("${1:name}").unwrap();
//! let plan = SnippetPlan::compute(&snippet, 1, doc, Anchor::Caret(5), &HashMap::new()).unwrap();
//! assert_eq!(plan.expansion().text, "name");
//!
//! // The document changed underneath the plan without its revision
//! // counter noticing - still detected as stale.
//! let current = DocumentId::new(1, "func (edited)");
//! assert!(!plan.staleness(&current).is_fresh());
//! ```

#![forbid(unsafe_code)]

mod anchor;
mod error;
mod expand;
mod identity;
mod limits;
mod model;
mod parser;
mod plan;
mod tabstops;

pub use anchor::Anchor;
pub use error::SnippetError;
pub use expand::{Expansion, PlaceholderSpan};
pub use identity::{ContentHash, DocumentId, Staleness};
pub use limits::{
    MAX_INPUT_BYTES, MAX_NESTING_DEPTH, MAX_OCCURRENCES, MAX_OUTPUT_BYTES, MAX_PLACEHOLDER_INDEX,
    MAX_PLACEHOLDERS,
};
pub use model::Snippet;
pub use plan::{PlanError, SnippetPlan};
pub use tabstops::TabStops;
