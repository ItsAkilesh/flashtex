//! Every bound this crate enforces, in one place.
//!
//! These exist so that parsing and expansion always terminate and never
//! allocate without limit: a snippet with absurd nesting, a huge
//! placeholder index, too many placeholders, or an overlong result is
//! rejected with a typed [`crate::SnippetError`] instead of looping or
//! blowing up memory.

/// Maximum size, in bytes, of a snippet source template accepted by
/// [`crate::Snippet::parse`].
pub const MAX_INPUT_BYTES: usize = 64 * 1024;

/// Maximum nesting depth of `${N:...}` placeholder defaults containing
/// further `${M:...}` defaults. `${1:${2:${3:text}}}` has depth 3.
pub const MAX_NESTING_DEPTH: usize = 16;

/// Largest placeholder index accepted. `$1` through this value are valid;
/// anything larger is rejected as [`crate::SnippetError::PlaceholderIndexTooLarge`].
pub const MAX_PLACEHOLDER_INDEX: u32 = 9_999;

/// Maximum number of *distinct* placeholder indices in one snippet.
pub const MAX_PLACEHOLDERS: usize = 256;

/// Maximum number of placeholder *occurrences* in one snippet, counting
/// every repeat of the same index and every placeholder nested inside a
/// default (whether or not that default is ultimately used).
pub const MAX_OCCURRENCES: usize = 2_048;

/// Maximum length, in bytes, of the fully expanded output text.
pub const MAX_OUTPUT_BYTES: usize = 1_000_000;
