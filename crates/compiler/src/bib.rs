//! Manual bibliographies: `thebibliography`, `\bibitem`, `\cite`, `\nocite`.
//!
//! A `\bibitem`'s citation label depends only on how many earlier `\bibitem`s
//! precede it (or its own optional-label override) — never on page numbers or
//! line breaks. That makes it fundamentally simpler than `\label`/`\ref`,
//! which need the page-aware two-pass resolution in `layout.rs`: a
//! [`prescan`] of the literal, pre-macro-expansion token stream is enough to
//! resolve every `\cite` in one pass, even one that appears (as citations
//! normally do) before the `thebibliography` it points into. This module is
//! deliberately self-contained and does not touch that separate label/ref
//! resolution pass.
//!
//! BibTeX/biblatex `.bib` files are out of scope; `\bibliography` and
//! `\bibliographystyle` in `parser.rs` report that honestly instead of
//! silently doing nothing.

use std::collections::HashMap;

use crate::diagnostics::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::parser::{Inline, TextStyle};
use crate::Span;

/// One `\bibitem`'s resolved citation label — the bracket content `\cite`
/// substitutes: a sequential number for a plain `\bibitem{key}`, or the
/// verbatim optional argument of `\bibitem[label]{key}`.
#[derive(Debug, Clone)]
struct BibItem {
    label: String,
}

/// Every `\bibitem` found by [`prescan`], in document order, plus a
/// key → item lookup for `\cite`.
#[derive(Debug, Default)]
pub struct Bibliography {
    items: Vec<BibItem>,
    keys: HashMap<String, usize>,
}

impl Bibliography {
    fn push(&mut self, key: String, label: String, span: Span, diags: &mut Vec<Diagnostic>) {
        let index = self.items.len();
        self.items.push(BibItem { label });
        if key.is_empty() {
            return;
        }
        if self.keys.insert(key.clone(), index).is_some() {
            diags.push(Diagnostic::warning(
                format!("duplicate \\bibitem{{{key}}}; the second definition wins"),
                Some(span),
                Some("used the later entry's label for \\cite".into()),
            ));
        }
    }

    /// The label of the `index`th (0-based) `\bibitem` in document order,
    /// consulted by the real parse — see `parser::P::bib_cursor` — which
    /// walks the same literal `\bibitem`s this pre-scan already numbered.
    pub fn label_at(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|item| item.label.as_str())
    }

    fn resolve(&self, key: &str) -> Option<&str> {
        self.keys
            .get(key)
            .and_then(|&index| self.items.get(index))
            .map(|item| item.label.as_str())
    }
}

/// Scans the raw, pre-macro-expansion token stream for every `\bibitem`
/// lexically inside a `thebibliography` environment, in source order, and
/// assigns each one's citation label (a plain `\bibitem{key}` numbers
/// sequentially; `\bibitem[label]{key}` uses `label` verbatim and does not
/// consume a number, mirroring real LaTeX's `\@lbibitem`). Deliberately
/// literal: a `\bibitem` produced by a user macro is out of scope, the same
/// boundary this compiler already draws elsewhere for macro-generated
/// structure.
pub fn prescan(tokens: &[Token], diags: &mut Vec<Diagnostic>) -> Bibliography {
    let mut bibliography = Bibliography::default();
    let mut in_bibliography = false;
    let mut next_number: u32 = 1;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i].kind {
            TokenKind::Command(name) if name == "begin" || name == "end" => {
                let is_begin = name == "begin";
                match group_text(tokens, i + 1) {
                    Some((environment, after)) => {
                        if environment.trim() == "thebibliography" {
                            in_bibliography = is_begin;
                        }
                        i = after;
                    }
                    None => i += 1,
                }
            }
            TokenKind::Command(name) if name == "bibitem" && in_bibliography => {
                let span = tokens[i].span;
                let mut cursor = i + 1;
                let mut label_override = None;
                if let Some((text, after)) = optional_bracket_text(tokens, cursor) {
                    label_override = Some(text);
                    cursor = after;
                }
                if let Some((key, after)) = group_text(tokens, cursor) {
                    cursor = after;
                    let label = label_override.unwrap_or_else(|| {
                        let n = next_number;
                        next_number += 1;
                        n.to_string()
                    });
                    bibliography.push(key.trim().to_string(), label, span, diags);
                }
                i = cursor;
            }
            _ => i += 1,
        }
    }
    bibliography
}

/// The text inside the next `{...}` group starting at (after skipping
/// spaces/comments from) `i`, and the index just past its closing brace.
/// `None` if `i` is not followed by a brace group — a malformed `\bibitem`
/// or `\begin`/`\end` is left for the real parse's own diagnostics.
fn group_text(tokens: &[Token], mut i: usize) -> Option<(String, usize)> {
    while matches!(
        tokens.get(i).map(|t| &t.kind),
        Some(TokenKind::Space | TokenKind::Comment)
    ) {
        i += 1;
    }
    if !matches!(tokens.get(i).map(|t| &t.kind), Some(TokenKind::LBrace)) {
        return None;
    }
    i += 1;
    let mut depth = 1usize;
    let mut text = String::new();
    while i < tokens.len() {
        match &tokens[i].kind {
            TokenKind::LBrace => depth += 1,
            TokenKind::RBrace => {
                depth -= 1;
                if depth == 0 {
                    return Some((text, i + 1));
                }
            }
            TokenKind::Word(word) | TokenKind::Command(word) => text.push_str(word),
            TokenKind::Space | TokenKind::ParBreak => text.push(' '),
            _ => {}
        }
        i += 1;
    }
    None
}

/// The text inside a `[...]` immediately at (after skipping spaces/comments
/// from) `i`, mirroring `parser::P::optional_bracket_argument`'s word-based
/// bracket matching (brackets are ordinary lexer word characters, never
/// their own token kind) against a plain token slice instead of the live
/// parse cursor.
fn optional_bracket_text(tokens: &[Token], mut i: usize) -> Option<(String, usize)> {
    while matches!(
        tokens.get(i).map(|t| &t.kind),
        Some(TokenKind::Space | TokenKind::Comment)
    ) {
        i += 1;
    }
    let TokenKind::Word(first) = &tokens.get(i)?.kind else {
        return None;
    };
    if !first.starts_with('[') {
        return None;
    }
    let mut raw = first.clone();
    let mut found = raw.contains(']');
    i += 1;
    while !found && i < tokens.len() {
        match &tokens[i].kind {
            TokenKind::Word(word) => {
                raw.push_str(word);
                found = word.contains(']');
            }
            TokenKind::Space | TokenKind::ParBreak => raw.push(' '),
            TokenKind::Command(word) => {
                raw.push('\\');
                raw.push_str(word);
            }
            _ => {}
        }
        i += 1;
    }
    if !found {
        return None;
    }
    let content = raw
        .strip_prefix('[')
        .unwrap_or(&raw)
        .split_once(']')
        .map_or(raw.as_str(), |(inside, _)| inside)
        .to_string();
    Some((content, i))
}

/// Builds the `Inline`s for one `\cite`/`\cite[note]{key1,key2,...}`:
/// `[<label>, <label>, ..., note]`. An undefined key renders as a bold `?`
/// (only the `?` is bold, matching real LaTeX's `\@citex`/`\bfseries ?`
/// recovery — the brackets and separators stay normal weight) and pushes one
/// warning, matching `\@citex`'s own "Citation ... undefined" warning.
pub fn cite_inlines(
    keys: &[String],
    note: Option<String>,
    bibliography: &Bibliography,
    span: Span,
    diags: &mut Vec<Diagnostic>,
) -> Vec<Inline> {
    let mut out = vec![text_run("[", span, TextStyle::default(), true)];
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            out.push(text_run(", ", span, TextStyle::default(), false));
        }
        match bibliography.resolve(key) {
            Some(label) => out.push(text_run(label, span, TextStyle::default(), false)),
            None => {
                out.push(text_run("?", span, TextStyle::BOLD, false));
                diags.push(Diagnostic::warning(
                    format!("citation '{key}' is undefined"),
                    Some(span),
                    Some("rendered '?' in place of the undefined citation".into()),
                ));
            }
        }
    }
    if let Some(note) = note {
        // `~` is TeX's tie: an ordinary interword space that just does not
        // break a line. This layout never breaks inside a `\cite` note, so a
        // plain space renders it faithfully.
        let note = note.replace('~', " ");
        out.push(text_run(
            &format!(", {note}"),
            span,
            TextStyle::default(),
            false,
        ));
    }
    out.push(text_run("]", span, TextStyle::default(), false));
    out
}

fn text_run(text: &str, span: Span, style: TextStyle, space_before: bool) -> Inline {
    Inline::Text {
        text: text.to_string(),
        span,
        style,
        space_before,
    }
}

/// The `[<widest-label>]` bracket text a `\bibitem`'s own marker or
/// `thebibliography`'s `\labelwidth` measures, given the environment's
/// widest-label argument (`\begin{thebibliography}{99}`'s `"99"`).
pub fn label_bracket(text: &str) -> String {
    format!("[{text}]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize_document;
    use crate::DocumentId;

    fn scan(source: &str) -> (Bibliography, Vec<Diagnostic>) {
        let tokens = tokenize_document(source, DocumentId::default());
        let mut diags = Vec::new();
        let bibliography = prescan(&tokens, &mut diags);
        (bibliography, diags)
    }

    #[test]
    fn numbers_plain_bibitems_in_order() {
        let (bib, diags) = scan(
            r"\begin{thebibliography}{9}\bibitem{a}First.\bibitem{b}Second.\end{thebibliography}",
        );
        assert!(diags.is_empty());
        assert_eq!(bib.resolve("a"), Some("1"));
        assert_eq!(bib.resolve("b"), Some("2"));
        assert_eq!(bib.label_at(0), Some("1"));
        assert_eq!(bib.label_at(1), Some("2"));
    }

    #[test]
    fn optional_label_overrides_the_number_without_consuming_one() {
        let (bib, _) = scan(
            r"\begin{thebibliography}{9}\bibitem[Knuth 1984]{tex}A.\bibitem{b}B.\end{thebibliography}",
        );
        assert_eq!(bib.resolve("tex"), Some("Knuth 1984"));
        // The un-labelled entry after it is still numbered 1: the labelled
        // entry did not consume a number, matching `\@lbibitem`.
        assert_eq!(bib.resolve("b"), Some("1"));
    }

    #[test]
    fn duplicate_key_warns_and_the_second_wins() {
        let (bib, diags) = scan(
            r"\begin{thebibliography}{9}\bibitem{a}First.\bibitem{a}Second.\end{thebibliography}",
        );
        assert_eq!(bib.resolve("a"), Some("2"));
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("duplicate \\bibitem{a}"));
    }

    #[test]
    fn bibitem_outside_thebibliography_is_not_registered() {
        let (bib, _) = scan(r"\bibitem{a}Stray.");
        assert_eq!(bib.resolve("a"), None);
    }
}
