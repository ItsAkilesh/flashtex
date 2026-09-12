//! Bounded, allowlist-based validation for external link URIs.
//!
//! This module never performs network or filesystem access: it only checks
//! that a URI string names a scheme this crate is willing to hand to a
//! renderer as a clickable link, and that checking it costs a bounded amount
//! of work no matter how the input is crafted.
//!
//! The allowlist in [`UriScheme`] is exhaustive and explicit. There is no
//! denylist and no "anything not blocked" fallback: a scheme is accepted
//! only by being matched in [`UriScheme::from_lowercase`], so `javascript:`,
//! `data:`, `file:`, and every other scheme not listed there are rejected by
//! construction, not by a blocklist someone has to keep up to date.

use std::fmt;

/// The longest URI this crate will inspect. Anything longer is rejected
/// before any parsing happens, so the cost of validation is bounded
/// independent of what a hostile caller passes in.
pub const MAX_URI_LEN: usize = 4096;

/// The exhaustive allowlist of schemes this crate will emit as external
/// link destinations. Adding a scheme is a deliberate, explicit code change
/// here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UriScheme {
    Http,
    Https,
    Mailto,
}

impl UriScheme {
    /// Matches an already-lowercased scheme name against the allowlist.
    /// Returns `None` for anything not explicitly listed, including every
    /// dangerous or unrecognized scheme.
    fn from_lowercase(s: &str) -> Option<UriScheme> {
        match s {
            "http" => Some(UriScheme::Http),
            "https" => Some(UriScheme::Https),
            "mailto" => Some(UriScheme::Mailto),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UriScheme::Http => "http",
            UriScheme::Https => "https",
            UriScheme::Mailto => "mailto",
        }
    }
}

impl fmt::Display for UriScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A URI that has passed allowlist and shape validation. Holding one of
/// these makes no claim that the destination exists or is reachable — this
/// crate never resolves, fetches, or opens anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedUri {
    scheme: UriScheme,
    text: String,
}

impl ValidatedUri {
    pub fn scheme(&self) -> UriScheme {
        self.scheme
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UriError {
    Empty,
    TooLong { len: usize, max: usize },
    ControlCharacter { at: usize },
    MissingScheme,
    InvalidSchemeSyntax { scheme: String },
    SchemeNotAllowed { scheme: String },
}

impl fmt::Display for UriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UriError::Empty => write!(f, "uri is empty"),
            UriError::TooLong { len, max } => {
                write!(f, "uri is {len} bytes, exceeds the {max}-byte bound")
            }
            UriError::ControlCharacter { at } => {
                write!(f, "uri contains a control character at byte offset {at}")
            }
            UriError::MissingScheme => write!(f, "uri has no `scheme:` prefix"),
            UriError::InvalidSchemeSyntax { scheme } => {
                write!(f, "`{scheme}` is not a syntactically valid uri scheme")
            }
            UriError::SchemeNotAllowed { scheme } => write!(
                f,
                "scheme `{scheme}` is not in the allowlist (http, https, mailto)"
            ),
        }
    }
}

impl std::error::Error for UriError {}

/// Validates `text` as an external link destination.
///
/// Bounded: the length check runs first and every scan after it walks at
/// most [`MAX_URI_LEN`] bytes once, so cost never grows past that bound
/// regardless of input. There is no recursion and no backtracking search
/// (no regex is used), so this cannot be driven into pathological behavior.
///
/// Performs no I/O: this function only inspects the string it is given.
pub fn validate_uri(text: &str) -> Result<ValidatedUri, UriError> {
    if text.is_empty() {
        return Err(UriError::Empty);
    }
    if text.len() > MAX_URI_LEN {
        return Err(UriError::TooLong {
            len: text.len(),
            max: MAX_URI_LEN,
        });
    }
    if let Some((at, _)) = text.char_indices().find(|(_, c)| c.is_control()) {
        return Err(UriError::ControlCharacter { at });
    }

    // RFC 3986 scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
    let colon = text.find(':').ok_or(UriError::MissingScheme)?;
    let scheme_text = &text[..colon];
    let mut chars = scheme_text.chars();
    let starts_with_alpha = matches!(chars.next(), Some(c) if c.is_ascii_alphabetic());
    let rest_is_valid = chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    if scheme_text.is_empty() || !starts_with_alpha || !rest_is_valid {
        return Err(UriError::InvalidSchemeSyntax {
            scheme: scheme_text.to_string(),
        });
    }

    let lower = scheme_text.to_ascii_lowercase();
    let scheme = UriScheme::from_lowercase(&lower).ok_or(UriError::SchemeNotAllowed {
        scheme: lower.clone(),
    })?;

    Ok(ValidatedUri {
        scheme,
        text: text.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https() {
        let uri = validate_uri("https://example.com/path?q=1").unwrap();
        assert_eq!(uri.scheme(), UriScheme::Https);
        assert_eq!(uri.as_str(), "https://example.com/path?q=1");
    }

    #[test]
    fn accepts_http() {
        let uri = validate_uri("http://example.com").unwrap();
        assert_eq!(uri.scheme(), UriScheme::Http);
    }

    #[test]
    fn accepts_mailto() {
        let uri = validate_uri("mailto:person@example.com").unwrap();
        assert_eq!(uri.scheme(), UriScheme::Mailto);
    }

    #[test]
    fn scheme_match_is_case_insensitive() {
        let uri = validate_uri("HTTPS://EXAMPLE.com").unwrap();
        assert_eq!(uri.scheme(), UriScheme::Https);
    }

    #[test]
    fn rejects_javascript_scheme() {
        let err = validate_uri("javascript:alert(1)").unwrap_err();
        assert_eq!(
            err,
            UriError::SchemeNotAllowed {
                scheme: "javascript".to_string()
            }
        );
    }

    #[test]
    fn rejects_data_scheme() {
        let err = validate_uri("data:text/html,<script>alert(1)</script>").unwrap_err();
        assert_eq!(
            err,
            UriError::SchemeNotAllowed {
                scheme: "data".to_string()
            }
        );
    }

    #[test]
    fn rejects_file_scheme() {
        let err = validate_uri("file:///etc/passwd").unwrap_err();
        assert_eq!(
            err,
            UriError::SchemeNotAllowed {
                scheme: "file".to_string()
            }
        );
    }

    #[test]
    fn rejects_unknown_scheme() {
        let err = validate_uri("gopher://example.com").unwrap_err();
        assert_eq!(
            err,
            UriError::SchemeNotAllowed {
                scheme: "gopher".to_string()
            }
        );
    }

    #[test]
    fn rejects_mixed_case_javascript_scheme() {
        // Case-folding must not create a bypass of the allowlist.
        let err = validate_uri("JaVaScRiPt:alert(1)").unwrap_err();
        assert_eq!(
            err,
            UriError::SchemeNotAllowed {
                scheme: "javascript".to_string()
            }
        );
    }

    #[test]
    fn rejects_empty_string() {
        assert_eq!(validate_uri("").unwrap_err(), UriError::Empty);
    }

    #[test]
    fn rejects_missing_scheme() {
        // Protocol-relative URIs have no scheme and must not be treated as
        // implicitly safe.
        let err = validate_uri("//example.com/evil").unwrap_err();
        assert_eq!(err, UriError::MissingScheme);
    }

    #[test]
    fn rejects_no_colon_at_all() {
        let err = validate_uri("not a uri").unwrap_err();
        assert_eq!(err, UriError::MissingScheme);
    }

    #[test]
    fn rejects_scheme_with_illegal_characters() {
        let err = validate_uri("ht!tp://example.com").unwrap_err();
        assert_eq!(
            err,
            UriError::InvalidSchemeSyntax {
                scheme: "ht!tp".to_string()
            }
        );
    }

    #[test]
    fn rejects_scheme_starting_with_digit() {
        let err = validate_uri("1http://example.com").unwrap_err();
        assert!(matches!(err, UriError::InvalidSchemeSyntax { .. }));
    }

    #[test]
    fn rejects_embedded_newline() {
        let err = validate_uri("https://example.com/\nevil").unwrap_err();
        assert!(matches!(err, UriError::ControlCharacter { .. }));
    }

    #[test]
    fn rejects_embedded_nul() {
        let err = validate_uri("https://example.com/\0evil").unwrap_err();
        assert!(matches!(err, UriError::ControlCharacter { .. }));
    }

    #[test]
    fn accepts_unicode_host_and_path() {
        // Internationalized domains / paths are legitimate; only the scheme
        // is restricted.
        let uri = validate_uri("https://例え.jp/ページ?q=日本語").unwrap();
        assert_eq!(uri.scheme(), UriScheme::Https);
        assert_eq!(uri.as_str(), "https://例え.jp/ページ?q=日本語");
    }

    #[test]
    fn accepts_emoji_in_path() {
        let uri = validate_uri("https://example.com/🎉/party").unwrap();
        assert_eq!(uri.scheme(), UriScheme::Https);
    }

    #[test]
    fn rejects_unicode_scheme() {
        // A scheme must be ASCII per RFC 3986; a look-alike Unicode letter
        // before the colon is not silently normalized into an allowed one.
        let err = validate_uri("htтps://example.com").unwrap_err();
        assert!(matches!(err, UriError::InvalidSchemeSyntax { .. }));
    }

    #[test]
    fn rejects_uri_over_max_length() {
        let long = format!("https://example.com/{}", "a".repeat(MAX_URI_LEN));
        let err = validate_uri(&long).unwrap_err();
        assert!(matches!(err, UriError::TooLong { .. }));
    }

    #[test]
    fn bounded_against_absurdly_long_hostile_input() {
        // A gigabyte-scale string must fail immediately via the length
        // check, not be scanned. This proves the function cannot be made to
        // hang or perform unbounded work by input size alone.
        let hostile = "a".repeat(50_000_000);
        let err = validate_uri(&hostile).unwrap_err();
        assert_eq!(
            err,
            UriError::TooLong {
                len: hostile.len(),
                max: MAX_URI_LEN
            }
        );
    }

    #[test]
    fn max_length_boundary_is_inclusive() {
        // Exactly MAX_URI_LEN bytes of scheme-less garbage should fail on
        // MissingScheme, not TooLong, proving the boundary is `> max`, not
        // `>= max`.
        let exact = "a".repeat(MAX_URI_LEN);
        assert_eq!(exact.len(), MAX_URI_LEN);
        let err = validate_uri(&exact).unwrap_err();
        assert_eq!(err, UriError::MissingScheme);
    }
}
