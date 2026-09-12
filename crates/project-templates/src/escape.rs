//! LaTeX-safe escaping for values substituted into template bodies.
//!
//! A project name or author string is caller-supplied free text, but it is
//! spliced into a `.tex` file inside a macro argument such as
//! `\title{...}`. Unescaped, a name containing `}` could close that argument
//! early and let the rest of the string run as arbitrary LaTeX; a name
//! containing `\` could invoke a control sequence. [`escape`] neutralizes the
//! characters LaTeX treats specially so the substituted value always reads
//! back as literal text, including when it is non-ASCII.

/// Escapes `s` so it is safe to splice into a LaTeX macro argument.
///
/// Every character is passed through unchanged except the ten LaTeX special
/// characters, which are replaced by their standard escaped form. Non-ASCII
/// text (accented letters, CJK, emoji, ...) passes through untouched: it is
/// not LaTeX-special and modern engines (`pdflatex` with `inputenc=utf8`,
/// `xelatex`, `lualatex`) render it directly.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\textbackslash{}"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '$' => out.push_str("\\$"),
            '&' => out.push_str("\\&"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '%' => out.push_str("\\%"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_plain_text_through() {
        assert_eq!(escape("Optics Lab Report"), "Optics Lab Report");
    }

    #[test]
    fn escapes_every_special_character() {
        assert_eq!(
            escape(r"\{}$&#_%~^"),
            r"\textbackslash{}\{\}\$\&\#\_\%\textasciitilde{}\textasciicircum{}"
        );
    }

    #[test]
    fn cannot_be_used_to_close_a_macro_argument_early() {
        // A naive `\title{ + name + }` splice would let this input close the
        // title early and inject a command; escaping must prevent that.
        let hostile = "}\\input{/etc/passwd}{";
        let escaped = escape(hostile);

        // Scan the escaped content alone, as it would sit inside the outer
        // `\title{ ... }`, tracking brace depth *relative to that content's
        // own start* (0). A backslash always consumes (escapes) exactly the
        // next character, so an escaped `{`/`}` never changes depth. If
        // relative depth ever goes negative, a bare `}` reached past the
        // outer opening brace and closed it early; it must instead end
        // exactly back at 0 (balanced) and never dip below it.
        let mut depth = 0i32;
        let mut min_depth = 0i32;
        let chars: Vec<char> = escaped.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '\\' => i += 1, // skip the escaped character, if any
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            min_depth = min_depth.min(depth);
            i += 1;
        }
        assert_eq!(depth, 0, "escaped content must be brace-balanced");
        assert!(
            min_depth >= 0,
            "escaped content must never close the enclosing \\title{{...}} early"
        );
    }

    #[test]
    fn passes_unicode_through_unescaped() {
        let name = "Café Ünïcödé — 論文 — Résumé";
        assert_eq!(escape(name), name);
    }
}
