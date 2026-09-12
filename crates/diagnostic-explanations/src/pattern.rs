//! Template matching for compiler message text.
//!
//! A template is the exact message string with `*` where the compiler
//! interpolates a value. Matching consumes the whole message: the first
//! literal segment must be a prefix, the last must be a suffix, and middle
//! segments are located left to right (leftmost occurrence). Captures are the
//! text between segments and may be empty. No regular expressions.

/// Match `message` against `template`; return the captures on success.
pub fn match_template(template: &str, message: &str) -> Option<Vec<String>> {
    let segments: Vec<&str> = template.split('*').collect();
    if segments.len() == 1 {
        return (template == message).then(Vec::new);
    }

    let first = segments[0];
    let last = segments[segments.len() - 1];
    let rest = message.strip_prefix(first)?;
    // The suffix must not overlap the prefix.
    if rest.len() < last.len() {
        return None;
    }
    let body = rest.strip_suffix(last)?;

    let mut captures = Vec::with_capacity(segments.len() - 1);
    let mut cursor = body;
    for middle in &segments[1..segments.len() - 1] {
        if middle.is_empty() {
            // `**` would be ambiguous; treat as one empty capture then continue.
            captures.push(String::new());
            continue;
        }
        let at = cursor.find(middle)?;
        captures.push(cursor[..at].to_string());
        cursor = &cursor[at + middle.len()..];
    }
    captures.push(cursor.to_string());
    Some(captures)
}

#[cfg(test)]
mod tests {
    use super::match_template;

    #[test]
    fn exact_and_single_capture() {
        assert_eq!(match_template("a b", "a b"), Some(vec![]));
        assert_eq!(match_template("a b", "a bc"), None);
        assert_eq!(
            match_template("\\* is not supported", "\\foo is not supported"),
            Some(vec!["foo".into()])
        );
        assert_eq!(
            match_template("\\* is not supported", "\\foo is not supported!"),
            None
        );
    }

    #[test]
    fn two_captures_with_braces() {
        assert_eq!(
            match_template(
                "\\end{*} does not match \\begin{*}",
                "\\end{itemize} does not match \\begin{enumerate}"
            ),
            Some(vec!["itemize".into(), "enumerate".into()])
        );
    }

    #[test]
    fn empty_capture_allowed() {
        assert_eq!(
            match_template(
                "\\end{*} with no matching \\begin",
                "\\end{} with no matching \\begin"
            ),
            Some(vec![String::new()])
        );
    }

    #[test]
    fn prefix_suffix_cannot_overlap() {
        assert_eq!(match_template("ab*ab", "ab"), None);
        assert_eq!(match_template("ab*ab", "abab"), Some(vec![String::new()]));
    }
}
