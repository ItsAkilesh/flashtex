//! hyperref commands and the destination/bookmark hooks of sectioning,
//! captions, equations, list items and theorems. The records themselves and
//! their provenance (hyperref.sty/hpdftex.def/nameref.sty line numbers) live
//! in `crate::hyperref`.
//!
//! Link text is never re-typeset here: `\hyperlink{x}{text}`,
//! `\hypertarget`, `\hyperref[..]{text}` and `\texorpdfstring{tex}{pdf}`
//! re-queue their visible argument as an ordinary braced group in place of
//! the command, so math, nested commands and style scoping inside it behave
//! exactly as they do anywhere else, and every item keeps its source span.

use super::{token_text, Inline, InputToken, ReferenceForm, P};
use crate::diagnostics::Diagnostic;
use crate::hyperref::{Anchor, Bookmark, Link, LinkKind, LinkTarget};
use crate::lexer::{Token, TokenKind};
use crate::Span;

impl P<'_> {
    /// `Inline::Label` carrying hyperref's `\@currentHref` and nameref's
    /// `\@currentlabelname`.
    pub(super) fn label_inline(&self, key: String, value: String, span: Span) -> Inline {
        Inline::Label {
            key,
            value,
            span,
            anchor: self.current_anchor.clone(),
            title: self.current_label_name.clone(),
        }
    }

    fn hyperref_active(&self) -> bool {
        self.hyperref.loaded && !self.hyperref.options.draft
    }

    pub(super) fn record_link(&mut self, span: Span, kind: LinkKind, target: LinkTarget) {
        // Records never change layout and `Parsed::hyperref` is rebuilt by
        // every parse, so none of them makes layout document-global.
        if self.hyperref_active() && self.no_hyper_depth == 0 {
            self.hyperref.links.push(Link { span, kind, target });
        }
    }

    pub(super) fn record_anchor(&mut self, name: String, span: Span) {
        if self.hyperref_active() {
            self.hyperref.anchors.push(Anchor { name, span });
        }
    }

    /// `\refstepcounter`'s `\@currentHref`: a destination the next `\label`
    /// points at.
    pub(super) fn set_current_anchor(&mut self, name: String, span: Span) {
        self.record_anchor(name.clone(), span);
        self.current_anchor = name;
    }

    /// `\Hy@MakeCurrentHrefAuto{section*}` (hyperref.sty 6896-6899): starred
    /// sections, `\phantomsection`, the contents and bibliography headings.
    pub(super) fn anonymous_anchor(&mut self, span: Span) {
        self.link_counter += 1;
        self.set_current_anchor(format!("section*.{}", self.link_counter), span);
    }

    /// `\Hy@writebookmark` (hpdftex.def 1604-1640): depth filter, then the
    /// level check that clamps a jump of more than one level.
    fn add_bookmark(&mut self, level: i32, title: String, destination: String, span: Span) {
        let options = &self.hyperref.options;
        if !self.hyperref_active() || !options.bookmarks || level > options.bookmarks_depth {
            return;
        }
        let level = match self.bookmark_level {
            Some(current) if level > current + 1 => current + 1,
            _ => level,
        };
        self.bookmark_level = Some(level);
        self.hyperref.bookmarks.push(Bookmark {
            level,
            title,
            destination,
            span,
        });
    }

    /// Sectioning with hyperref and nameref: destination `section.1` (or
    /// `section*.<n>` when unnumbered), the nameref title, and a bookmark for
    /// numbered headings (`\section*` gets none).
    pub(super) fn heading_hyperref(
        &mut self,
        name: &str,
        level: u8,
        number: &str,
        title_tokens: &[InputToken],
        content: &[Inline],
        span: Span,
    ) {
        if number.is_empty() {
            self.anonymous_anchor(span);
        } else {
            self.set_current_anchor(format!("{name}.{number}"), span);
        }
        self.current_label_name = inline_text(content);
        if !number.is_empty() {
            let title = pdf_string(title_tokens);
            let title = if self.hyperref.options.bookmarks_numbered {
                format!("{number} {title}")
            } else {
                title
            };
            let destination = self.current_anchor.clone();
            self.add_bookmark(i32::from(level), title, destination, span);
        }
    }

    /// `\caption` in a float: destination `figure.<n>`/`table.<n>`, nameref
    /// title the caption text.
    pub(super) fn caption_hyperref(
        &mut self,
        counter: &str,
        number: u32,
        caption: &[Inline],
        span: Span,
    ) {
        self.set_current_anchor(format!("{counter}.{number}"), span);
        self.current_label_name = inline_text(caption);
    }

    /// enumerate `\item`: `\refstepcounter{enum<level>}` makes the item's
    /// reference text `\p@enum<level>\theenum<level>` (article.cls: `1`,
    /// `1a`, `1(a)i`, `1(a)iA`; an enumitem `label=` template's text) and
    /// hyperref's destination `Item.<n>`, counted over the whole document.
    pub(super) fn enumerate_item_hyperref(&mut self, templated_marker: Option<String>, span: Span) {
        // `\c@enum<i>` of every open enumerate (#147's `OpenList::counter`,
        // which honours enumitem `start=`/`resume`).
        let counts: Vec<u32> = self
            .list_stack
            .iter()
            .filter(|list| list.kind == "enumerate")
            .map(|list| u32::try_from(list.counter).unwrap_or(0))
            .collect();
        let value = templated_marker.unwrap_or_else(|| enumerate_reference(&counts));
        self.current_counter = Some(value);
        self.item_anchor_counter += 1;
        self.set_current_anchor(format!("Item.{}", self.item_anchor_counter), span);
    }

    /// `\ref`, `\pageref`, `\eqref`, hyperref `\autoref`/`\autopageref`,
    /// nameref `\nameref`, and the starred (unlinked) forms.
    pub(super) fn reference(&mut self, name: &str, span: Span, para: &mut Vec<Inline>) {
        let space_before = self.space_precedes(self.i - 1);
        let starred = name != "eqref" && self.take_optional_star();
        let (tokens, argument_span) = self.required_group(name, span);
        let key = token_text(&tokens).trim().to_string();
        self.document_global_state = true;
        let form = if name == "autoref" || name == "autopageref" {
            ReferenceForm::Auto {
                names: self.defined_names(),
            }
        } else if name == "nameref" {
            ReferenceForm::Name
        } else {
            ReferenceForm::Number
        };
        let span = span.merge(argument_span);
        let linked = !starred && self.no_hyper_depth == 0;
        if linked {
            self.record_link(span, LinkKind::Link, LinkTarget::Label(key.clone()));
        }
        para.push(Inline::Reference {
            key,
            page: name == "pageref" || name == "autopageref",
            equation: name == "eqref",
            span,
            space_before,
            form,
            linked,
        });
    }

    /// Argument-free macros named `\...name` the document defined, for
    /// `\autoref`'s name lookup at this point.
    ///
    /// Definitions now run in the expansion pass (`crate::expansion`), so the
    /// parser has no macro table: the sources are scanned for argument-free
    /// `\renewcommand`/`\newcommand`/`\providecommand{\...name}{..}` and
    /// `\def\...name{..}`, the last definition of each name winning
    /// (integration 2026-09-13b; the original read `self.macros`).
    fn defined_names(&self) -> Vec<(String, String)> {
        let mut names = self.user_definitions();
        names.retain(|(name, _)| name.ends_with("name"));
        names
    }

    /// Every argument-free `\renewcommand`/`\newcommand`/`\providecommand`
    /// and `\def` in the sources as `(name, body text)`, the last definition
    /// of each name winning, sorted by name. Also read for amsthm's
    /// `\proofname` and `\qedsymbol` (#149).
    pub(super) fn user_definitions(&self) -> Vec<(String, String)> {
        let mut names: Vec<(String, String)> = Vec::new();
        for document in self.documents {
            let tokens = crate::lexer::tokenize(document.text);
            let kind = |j: usize| tokens.get(j).map(|token| &token.kind);
            let skip_blanks = |mut j: usize| {
                while matches!(kind(j), Some(TokenKind::Space | TokenKind::Comment)) {
                    j += 1;
                }
                j
            };
            let mut k = 0;
            while k < tokens.len() {
                let Some(TokenKind::Command(command)) = kind(k) else {
                    k += 1;
                    continue;
                };
                if !matches!(
                    command.as_str(),
                    "renewcommand" | "newcommand" | "providecommand" | "def"
                ) {
                    k += 1;
                    continue;
                }
                let mut j = skip_blanks(k + 1);
                let braced = matches!(kind(j), Some(TokenKind::LBrace));
                if braced {
                    j = skip_blanks(j + 1);
                }
                let Some(TokenKind::Command(name)) = kind(j) else {
                    k += 1;
                    continue;
                };
                j = skip_blanks(j + 1);
                if braced {
                    if !matches!(kind(j), Some(TokenKind::RBrace)) {
                        k += 1;
                        continue;
                    }
                    j = skip_blanks(j + 1);
                }
                // `[n]` arguments (a Word) are not argument-free: skip.
                if !matches!(kind(j), Some(TokenKind::LBrace)) {
                    k += 1;
                    continue;
                }
                let body_start = j + 1;
                let mut depth = 1usize;
                j += 1;
                while j < tokens.len() && depth > 0 {
                    match tokens[j].kind {
                        TokenKind::LBrace => depth += 1,
                        TokenKind::RBrace => depth -= 1,
                        _ => {}
                    }
                    j += 1;
                }
                if depth != 0 {
                    break;
                }
                let body = body_text(&tokens[body_start..j - 1]);
                names.retain(|(existing, _)| existing != name);
                names.push((name.clone(), body));
                k = j;
            }
        }
        names.sort();
        names
    }

    /// `\sectionautorefname` etc. used directly in text, not redefined.
    pub(super) fn autoref_name_command(&mut self, name: &str, span: Span, para: &mut Vec<Inline>) {
        let space_before = self.space_precedes(self.i - 1);
        let names = self.defined_names();
        let lookup = |n: &str| names.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
        let text = crate::hyperref::default_name(name, &lookup).unwrap_or_default();
        para.push(Inline::Text {
            text,
            span,
            style: self.style,
            space_before,
        });
    }

    /// `\usepackage[options]{hyperref}` and `\hypersetup{..}`.
    pub(super) fn apply_hyperref_options(&mut self, list: &str, span: Span) {
        let xcolor = self.packages.iter().any(|package| package == "xcolor");
        let rejected = self.hyperref.options.apply(list, xcolor);
        if !rejected.is_empty() {
            self.diags.push(Diagnostic::warning(
                format!(
                    "hyperref options {} are not implemented",
                    rejected.join(", ")
                ),
                Some(span),
                Some("ignored those options and kept hyperref's defaults for them".into()),
            ));
        }
    }

    pub(super) fn hypersetup(&mut self, span: Span) {
        let (_, argument_span) = self.required_group("hypersetup", span);
        let list = self.raw_inside(argument_span);
        self.apply_hyperref_options(&list, span.merge(argument_span));
    }

    /// The exact source text inside a `{..}`/`[..]` argument span (keyval
    /// lists need their inner braces and spaces, which token text drops).
    pub(super) fn raw_inside(&self, span: Span) -> String {
        let Some(document) = self.documents.get(span.document.0) else {
            return String::new();
        };
        let raw = document.text.get(span.start..span.end).unwrap_or("");
        let raw = raw.strip_prefix(['{', '[']).unwrap_or(raw);
        raw.strip_suffix(['}', ']']).unwrap_or(raw).to_string()
    }

    /// Rewinds to the braced group `group_start..group_end` so it is parsed
    /// as ordinary text; arguments already read after the group (up to the
    /// cursor) become comments. The stream keeps its length: it is shared
    /// with the expansion cache (`Rc`), and `token_mut` logs each edit so the
    /// cached stream is restored afterwards (integration 2026-09-13b: the
    /// original splice predates the expansion pass).
    fn requeue_group(&mut self, command_start: usize, group_start: usize, group_end: usize) {
        debug_assert!(command_start <= group_start && group_end <= self.i);
        for index in group_end..self.i {
            if let Some(input) = self.token_mut(index) {
                input.token.kind = TokenKind::Comment;
            }
        }
        self.i = group_start;
    }

    /// `\hyperlink{name}{text}` (a link to a named destination) and
    /// `\hypertarget{name}{text}` (the destination itself).
    pub(super) fn hyperlink(&mut self, name: &str, span: Span) {
        let command_start = self.i - 1;
        let (target_tokens, target_span) = self.required_group(name, span);
        let target = token_text(&target_tokens).trim().to_string();
        self.skip_spaces();
        let group_start = self.i;
        let (_, text_span) = self.required_group(name, span.merge(target_span));
        if name == "hyperlink" {
            self.record_link(text_span, LinkKind::Link, LinkTarget::Destination(target));
        } else {
            self.record_anchor(target, span);
        }
        let group_end = self.i;
        self.requeue_group(command_start, group_start, group_end);
    }

    /// `\hyperdef{category}{name}{text}`: destination `category.name`.
    pub(super) fn hyperdef(&mut self, span: Span) {
        let command_start = self.i - 1;
        let (category, category_span) = self.required_group("hyperdef", span);
        let (name, name_span) = self.required_group("hyperdef", span.merge(category_span));
        let category = token_text(&category).trim().to_string();
        let name = token_text(&name).trim().to_string();
        let destination = if category.is_empty() {
            name
        } else {
            format!("{category}.{name}")
        };
        self.record_anchor(destination, span.merge(name_span));
        self.skip_spaces();
        let group_start = self.i;
        let _ = self.required_group("hyperdef", span.merge(name_span));
        let group_end = self.i;
        self.requeue_group(command_start, group_start, group_end);
    }

    /// `\hyperref[label]{text}` (hyperref.sty 4808-4810) and the four-argument
    /// `\hyperref{url}{category}{name}{text}` (`\@@hyperref`, 4832-4835).
    pub(super) fn hyperref_command(&mut self, span: Span) {
        let command_start = self.i - 1;
        if let Some((label, _)) = self.optional_bracket_argument() {
            self.skip_spaces();
            let group_start = self.i;
            let (_, text_span) = self.required_group("hyperref", span);
            let group_end = self.i;
            self.record_link(
                text_span,
                LinkKind::Link,
                LinkTarget::Label(label.trim().to_string()),
            );
            self.requeue_group(command_start, group_start, group_end);
            return;
        }
        let (url, url_span) = self.required_group("hyperref", span);
        let (category, category_span) = self.required_group("hyperref", span.merge(url_span));
        let (name, name_span) = self.required_group("hyperref", span.merge(category_span));
        self.skip_spaces();
        let group_start = self.i;
        let (_, text_span) = self.required_group("hyperref", span.merge(name_span));
        let group_end = self.i;
        let url = token_text(&url).trim().to_string();
        let category = token_text(&category).trim().to_string();
        let name = token_text(&name).trim().to_string();
        let destination = if category.is_empty() {
            name
        } else {
            format!("{category}.{name}")
        };
        if url.is_empty() {
            self.record_link(
                text_span,
                LinkKind::Link,
                LinkTarget::Destination(destination),
            );
        } else {
            self.record_link(
                text_span,
                LinkKind::Url,
                LinkTarget::Uri(format!("{url}#{destination}")),
            );
        }
        self.requeue_group(command_start, group_start, group_end);
    }

    /// `\texorpdfstring{tex}{pdf}` (hyperref.sty 783-789): typesets `tex`;
    /// `pdf_string` takes `pdf` for bookmarks.
    pub(super) fn texorpdfstring(&mut self, span: Span) {
        let command_start = self.i - 1;
        self.skip_spaces();
        let group_start = self.i;
        let (_, tex_span) = self.required_group("texorpdfstring", span);
        let group_end = self.i;
        let _ = self.required_group("texorpdfstring", span.merge(tex_span));
        self.requeue_group(command_start, group_start, group_end);
    }

    /// `\pdfbookmark[level]{text}{name}` (hpdftex.def 1744-1747): destination
    /// `name.level`, an outline entry at `level` (default 0).
    pub(super) fn pdfbookmark(&mut self, span: Span) {
        let level = match self.optional_bracket_argument() {
            Some((raw, raw_span)) => match raw.trim().parse::<i32>() {
                Ok(level) => level,
                Err(_) => {
                    self.diags.push(Diagnostic::warning(
                        format!("\\pdfbookmark level '{}' is not a number", raw.trim()),
                        Some(raw_span),
                        Some("used level 0".into()),
                    ));
                    0
                }
            },
            None => 0,
        };
        let (text, text_span) = self.required_group("pdfbookmark", span);
        let (name, name_span) = self.required_group("pdfbookmark", span.merge(text_span));
        let destination = format!("{}.{level}", token_text(&name).trim());
        let span = span.merge(name_span);
        self.record_anchor(destination.clone(), span);
        self.add_bookmark(level, pdf_string(&text), destination, span);
    }
}

/// article.cls `\p@enum<level>\theenum<level>`.
fn enumerate_reference(counts: &[u32]) -> String {
    match counts {
        [] => String::new(),
        [a] => a.to_string(),
        [a, b] => format!("{a}{}", alph(*b)),
        [a, b, c] => format!("{a}({}){}", alph(*b), roman(*c)),
        [a, b, c, d, ..] => format!("{a}({}){}{}", alph(*b), roman(*c), alph(*d).to_uppercase()),
    }
}

fn alph(n: u32) -> String {
    match n {
        1..=26 => char::from(b'a' + (n - 1) as u8).to_string(),
        _ => n.to_string(),
    }
}

fn roman(mut n: u32) -> String {
    const TABLE: &[(u32, &str)] = &[
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut out = String::new();
    for &(value, digits) in TABLE {
        while n >= value {
            out.push_str(digits);
            n -= value;
        }
    }
    out
}

/// A macro body as plain text (`\renewcommand{\sectionautorefname}{Sec.}`).
fn body_text(body: &[Token]) -> String {
    let mut out = String::new();
    for token in body {
        match &token.kind {
            TokenKind::Word(word) => out.push_str(word),
            TokenKind::Space | TokenKind::ParBreak => out.push(' '),
            _ => {}
        }
    }
    out.trim().to_string()
}

/// The typeset text of inline content, words joined by their source spaces
/// (nameref titles).
fn inline_text(content: &[Inline]) -> String {
    let mut out = String::new();
    for inline in content {
        if let Inline::Text {
            text, space_before, ..
        } = inline
        {
            if *space_before && !out.is_empty() {
                out.push(' ');
            }
            out.push_str(text);
        }
    }
    out
}

/// The braced group starting at or after `start` (spaces skipped):
/// `(inner tokens, index after the closing brace)`. No group: empty.
fn group_at(tokens: &[InputToken], start: usize) -> (&[InputToken], usize) {
    let mut i = start;
    while i < tokens.len() && matches!(tokens[i].token.kind, TokenKind::Space | TokenKind::Comment)
    {
        i += 1;
    }
    if i >= tokens.len() || tokens[i].token.kind != TokenKind::LBrace {
        return (&[], i);
    }
    let open = i;
    let mut depth = 0usize;
    while i < tokens.len() {
        match tokens[i].token.kind {
            TokenKind::LBrace => depth += 1,
            TokenKind::RBrace => {
                depth -= 1;
                if depth == 0 {
                    return (&tokens[open + 1..i], i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    (&tokens[open + 1..], tokens.len())
}

/// `\pdfstringdef` for the constructs a title actually uses: words and
/// spaces kept, `~` a space, `\texorpdfstring`'s second argument, commands,
/// braces and math markers dropped.
pub(super) fn pdf_string(tokens: &[InputToken]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i].token.kind {
            TokenKind::Command(name) if name == "texorpdfstring" => {
                let (_, after_tex) = group_at(tokens, i + 1);
                let (pdf, after_pdf) = group_at(tokens, after_tex);
                out.push_str(&pdf_string(pdf));
                i = after_pdf;
                continue;
            }
            TokenKind::Word(word) => out.push_str(&word.replace('~', " ")),
            TokenKind::Space | TokenKind::ParBreak | TokenKind::LineBreak => out.push(' '),
            _ => {}
        }
        i += 1;
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Keeps only `\texorpdfstring`'s TeX argument (as a braced group) for the
/// flat inline pass that heading and caption content goes through.
pub(super) fn tex_alternatives(tokens: Vec<InputToken>) -> Vec<InputToken> {
    if !tokens
        .iter()
        .any(|t| matches!(&t.token.kind, TokenKind::Command(name) if name == "texorpdfstring"))
    {
        return tokens;
    }
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if matches!(&tokens[i].token.kind, TokenKind::Command(name) if name == "texorpdfstring") {
            let (tex, after_tex) = group_at(&tokens, i + 1);
            let (_, after_pdf) = group_at(&tokens, after_tex);
            let tex_open = after_tex.saturating_sub(tex.len() + 2);
            if after_tex > i + 1 && tex_open < after_tex {
                out.extend_from_slice(&tokens[tex_open..after_tex]);
            }
            i = after_pdf.max(i + 1);
            continue;
        }
        out.push(tokens[i].clone());
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerate_references_follow_article_cls() {
        assert_eq!(enumerate_reference(&[2]), "2");
        assert_eq!(enumerate_reference(&[1, 1]), "1a");
        assert_eq!(enumerate_reference(&[1, 2, 4]), "1(b)iv");
        assert_eq!(enumerate_reference(&[3, 1, 2, 3]), "3(a)iiC");
    }
}
