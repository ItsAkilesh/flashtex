//! Validation for template-relative file paths.
//!
//! A template's manifest lists each file it writes by a path such as
//! `"chapters/intro.tex"`. Before anything is written to disk, every path is
//! run through [`validate`], which rejects anything that could let
//! instantiation escape the caller's target directory or write to a location
//! the caller did not ask for. Validation is intentionally strict and not
//! merely path *normalization*: `..` is never resolved and cancelled out, it
//! is refused outright, so a manifest can never depend on subtle traversal
//! arithmetic.

use std::fmt;

/// Why a template file path was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// The path was empty, or normalized to nothing (e.g. `""`, `"."`).
    Empty,
    /// The path is absolute (leads with `/`, `~`, or a Windows drive prefix
    /// such as `C:`).
    Absolute,
    /// The path contains a `..` segment. Refused outright rather than
    /// resolved, so a manifest can never rely on `..` cancelling out.
    ParentTraversal,
    /// A path segment is empty (a doubled `/`) or is exactly `.`.
    EmptySegment,
    /// The path contains a backslash, NUL, or other control character.
    ForbiddenCharacter(char),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathError::Empty => write!(f, "template file path is empty"),
            PathError::Absolute => write!(f, "template file path must be relative, not absolute"),
            PathError::ParentTraversal => {
                write!(f, "template file path contains a '..' segment")
            }
            PathError::EmptySegment => {
                write!(f, "template file path contains an empty or '.' segment")
            }
            PathError::ForbiddenCharacter(c) => {
                write!(f, "template file path contains forbidden character {c:?}")
            }
        }
    }
}

impl std::error::Error for PathError {}

/// Unicode bidirectional-formatting control characters: marks, embeddings,
/// overrides, and isolates. `char::is_control` does not cover these (they
/// are Unicode category Cf, "format", not Cc, "control"), but a file name
/// containing one — most notably U+202E RIGHT-TO-LEFT OVERRIDE — can make a
/// path render with a visually reversed or reordered extension (the classic
/// `"invoice\u{202E}fdp.exe"` trick, displayed as `"invoice...exe.pdf"`).
/// Refused unconditionally rather than only in some positions: a template
/// path has no legitimate reason to contain one.
const BIDI_CONTROL_CHARS: [char; 12] = [
    '\u{061C}', // ARABIC LETTER MARK
    '\u{200E}', // LEFT-TO-RIGHT MARK
    '\u{200F}', // RIGHT-TO-LEFT MARK
    '\u{202A}', // LEFT-TO-RIGHT EMBEDDING
    '\u{202B}', // RIGHT-TO-LEFT EMBEDDING
    '\u{202C}', // POP DIRECTIONAL FORMATTING
    '\u{202D}', // LEFT-TO-RIGHT OVERRIDE
    '\u{202E}', // RIGHT-TO-LEFT OVERRIDE
    '\u{2066}', // LEFT-TO-RIGHT ISOLATE
    '\u{2067}', // RIGHT-TO-LEFT ISOLATE
    '\u{2068}', // FIRST STRONG ISOLATE
    '\u{2069}', // POP DIRECTIONAL ISOLATE
];

/// Validates `raw` as a template-relative file path and returns its
/// slash-separated segments on success.
///
/// A valid path:
/// - is non-empty;
/// - does not start with `/` or `~`, and does not contain a Windows drive
///   prefix such as `C:`;
/// - contains no `..` segment, anywhere;
/// - contains no empty (`//`) or `.` segment;
/// - contains no backslash, NUL, other control character, or Unicode
///   bidirectional-formatting control character (see
///   [`BIDI_CONTROL_CHARS`]).
///
/// Ordinary non-ASCII text — including non-NFC (decomposed) Unicode, mixed
/// scripts, and Unicode noncharacters/replacement characters — is accepted
/// unchanged: this function performs no normalization of its own, so a
/// segment's exact code points are preserved rather than silently
/// canonicalized to some other, merely-equivalent form.
pub fn validate(raw: &str) -> Result<Vec<&str>, PathError> {
    if raw.is_empty() {
        return Err(PathError::Empty);
    }
    if let Some(c) = raw.chars().find(|c| {
        matches!(c, '\\' | ':' | '\0') || c.is_control() || BIDI_CONTROL_CHARS.contains(c)
    }) {
        return Err(PathError::ForbiddenCharacter(c));
    }
    if raw.starts_with('/') || raw.starts_with('~') {
        return Err(PathError::Absolute);
    }
    let mut segments = Vec::new();
    for seg in raw.split('/') {
        match seg {
            "" | "." => return Err(PathError::EmptySegment),
            ".." => return Err(PathError::ParentTraversal),
            s => segments.push(s),
        }
    }
    if segments.is_empty() {
        return Err(PathError::Empty);
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_relative_paths() {
        assert_eq!(validate("report.tex").unwrap(), vec!["report.tex"]);
        assert_eq!(
            validate("chapters/intro.tex").unwrap(),
            vec!["chapters", "intro.tex"]
        );
    }

    #[test]
    fn rejects_parent_traversal() {
        assert_eq!(validate("../etc/passwd"), Err(PathError::ParentTraversal));
        assert_eq!(
            validate("chapters/../../etc/passwd"),
            Err(PathError::ParentTraversal)
        );
        // Even a traversal that would mathematically cancel out is refused
        // outright: `..` is never resolved, only rejected.
        assert_eq!(validate("a/../a.tex"), Err(PathError::ParentTraversal));
    }

    #[test]
    fn rejects_absolute_paths() {
        assert_eq!(validate("/etc/passwd"), Err(PathError::Absolute));
        assert_eq!(validate("~/x.tex"), Err(PathError::Absolute));
        assert_eq!(
            validate("C:/Users/x.tex"),
            Err(PathError::ForbiddenCharacter(':'))
        );
    }

    #[test]
    fn rejects_forbidden_characters_and_segments() {
        assert_eq!(
            validate("a\\b.tex"),
            Err(PathError::ForbiddenCharacter('\\'))
        );
        assert_eq!(
            validate("a\0b.tex"),
            Err(PathError::ForbiddenCharacter('\0'))
        );
        assert_eq!(validate(""), Err(PathError::Empty));
        assert_eq!(validate("a//b.tex"), Err(PathError::EmptySegment));
        assert_eq!(validate("./a.tex"), Err(PathError::EmptySegment));
    }

    /// A right-to-left override (the classic extension-spoofing character,
    /// e.g. `"invoice\u{202E}fdp.exe"` rendering as if it ended `.exe.pdf`)
    /// must be refused outright, not silently written to disk.
    #[test]
    fn rejects_right_to_left_override_and_other_bidi_control_characters() {
        assert_eq!(
            validate("invoice\u{202E}fdp.exe"),
            Err(PathError::ForbiddenCharacter('\u{202E}'))
        );
        for c in BIDI_CONTROL_CHARS {
            assert_eq!(
                validate(&format!("a{c}b.tex")),
                Err(PathError::ForbiddenCharacter(c)),
                "expected {c:?} (U+{:04X}) to be refused",
                c as u32
            );
        }
    }

    /// Non-NFC (decomposed) Unicode is ordinary text to this function: it is
    /// accepted, and — critically — the exact code points are preserved
    /// rather than silently normalized to NFC or any other canonical form.
    /// Silent normalization would be exactly the kind of implicit
    /// approximation this crate must never perform on a caller-supplied
    /// identity.
    #[test]
    fn accepts_non_nfc_unicode_without_normalizing_it() {
        // "é" as `e` (U+0065) + COMBINING ACUTE ACCENT (U+0301), i.e. NFD,
        // not the single precomposed U+00E9 codepoint (NFC).
        let nfd = "e\u{0301}tude.tex";
        assert!(
            !nfd.contains('\u{00E9}'),
            "sanity: this literal must not contain the precomposed form"
        );
        let segments = validate(nfd).unwrap();
        assert_eq!(
            segments,
            vec![nfd],
            "exact decomposed form must survive unchanged"
        );
    }

    /// Rust's `&str` guarantees well-formed UTF-8, so a lone UTF-16
    /// surrogate half (U+D800..=U+DFFF) can never actually appear in one —
    /// there is no `char` value for it. The nearest real-world proxies are
    /// exercised instead: the standalone replacement character (what a lossy
    /// UTF-8 conversion of ill-formed UTF-16 containing an unpaired
    /// surrogate turns it into) and a Unicode noncharacter. Neither is a
    /// control or bidi character, so both are ordinary, accepted text; the
    /// guarantee under test is simply that validation does not panic on
    /// them.
    #[test]
    fn accepts_replacement_character_and_noncharacters_without_panicking() {
        assert_eq!(
            validate("bad\u{FFFD}encoding.tex").unwrap(),
            vec!["bad\u{FFFD}encoding.tex"]
        );
        assert_eq!(validate("\u{FFFF}.tex").unwrap(), vec!["\u{FFFF}.tex"]);
        assert_eq!(validate("\u{FDD0}.tex").unwrap(), vec!["\u{FDD0}.tex"]);
    }
}
