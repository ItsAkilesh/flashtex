//! Macro expansion: a compiler-owned pass in front of the parser.
//!
//! `flashtex-tex-expansion` executes the expansion layer of TeX — category
//! codes, `\def`/`\let`, `\newcommand`/`\newenvironment`, conditionals,
//! registers, counters — over every document the entry `\input`s, and this
//! module turns its output back into the parser's [`Token`] vocabulary. The
//! parser therefore never sees a macro call: `\greet{world}` arrives as the
//! words of its replacement text.
//!
//! Contract kept for the parser and its consumers:
//!
//! - **Pass-through.** Every control sequence the engine does not define
//!   (all typesetting commands: `\section`, `\textbf`, `\hspace`, ...) and
//!   every grouping brace reaches the parser unchanged, with its exact source
//!   span. The parser's built-in command names are declared to the engine as
//!   host commands, so `\newcommand` refuses to redefine them and
//!   `\renewcommand` accepts them, as before.
//! - **Spans.** A token read from a source keeps its exact byte span. A token
//!   of a macro's replacement text carries the span of the outermost macro
//!   invocation in the source (the `\name` control word, as before this pass
//!   existed) *and* its definition span: the bytes inside the definition it
//!   was copied from (see [`ExpandedToken::definition`] and
//!   [`crate::parser::Parsed::expansions`]). Tokens substituted for a
//!   macro's arguments keep their own source spans.
//! - **Verbatim.** `\verb` arguments and the bodies of `verbatim`,
//!   `verbatim*` and `lstlisting` are hidden from the engine before it reads
//!   the source (their bytes are blanked in a private copy; offsets do not
//!   move), so no `%`, `\`, `$`, `{`, `}` or macro in them is interpreted.
//!   The parser keeps reading those regions from the original text.
//! - **Environments.** The engine turns `\begin{name}` into `\name` (so a
//!   `\newenvironment` definition runs); an undefined `\name` produced that
//!   way is turned back into `\begin`, `{`, `name`, `}` with the exact spans
//!   of each piece.
//! - **Diagnostics.** Engine diagnostics become compiler diagnostics at the
//!   engine's span, mapped into the owning document.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use flashtex_tex_expansion::{self as tex, CatCode, Engine, Limits, TokenKind as TexKind};

use crate::diagnostics::Diagnostic;
use crate::lexer::{tokenize_document, Token, TokenKind};
use crate::parser::{path_is_safe, SourceDocument, BUILT_INS, INCLUDE_DEPTH_LIMIT};
use crate::{DocumentId, Span};

/// One parser input token with its expansion provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct ExpandedToken {
    pub token: Token,
    /// For a token copied from a macro's replacement text: the bytes of the
    /// definition it came from. `token.span` is then the invocation span.
    pub definition: Option<Span>,
    /// True when `token.span` is an invocation span rather than the token's
    /// own source bytes.
    pub maps_to_invocation: bool,
}

/// A replacement-text run: the invocation it was expanded at, and the
/// definition bytes it was copied from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpansionSite {
    pub invocation: Span,
    pub definition: Span,
}

pub struct Expansion {
    pub tokens: Vec<ExpandedToken>,
    pub diagnostics: Vec<Diagnostic>,
    /// `\arraystretch`'s replacement text in effect at each
    /// `\begin{tabular}`/`tabular*`/`array`, keyed by that `\begin`'s span.
    pub arraystretch: HashMap<(usize, usize), String>,
}

/// Kernel definitions the parser relies on reading back.
const PRELUDE: &str = "\\def\\arraystretch{1}";

/// A document as the engine reads it: verbatim regions blanked.
struct Prepared<'a> {
    text: Cow<'a, str>,
    /// `\verb` tokens, by the byte offset of their backslash.
    verbs: HashMap<usize, Token>,
    /// Offsets of the `\/` end markers written into blanked `\verb`s.
    verb_markers: HashSet<usize>,
    /// enumitem label markers (`\Roman*` in `label=\Roman*.`), renamed to
    /// same-length undefined control words so the engine's counter
    /// primitives do not consume them; by backslash offset.
    renamed: HashMap<usize, &'static str>,
}

/// Counter formats enumitem accepts as `\<format>*` in a `label` template.
const LABEL_FORMATS: &[&str] = &["arabic", "roman", "Roman", "alph", "Alph", "fnsymbol"];

fn prepare<'a>(text: &'a str, document: DocumentId) -> Prepared<'a> {
    let mut prepared = Prepared {
        text: Cow::Borrowed(text),
        verbs: HashMap::new(),
        verb_markers: HashSet::new(),
        renamed: HashMap::new(),
    };
    let has_labels = text.contains('*') && LABEL_FORMATS.iter().any(|f| text.contains(&format!("\\{f}*")));
    let has_urls = text.contains("\\url") || text.contains("\\href") || text.contains("\\nolinkurl");
    if !(has_labels
        || has_urls
        || text.contains("\\verb")
        || text.contains("verbatim")
        || text.contains("lstlisting"))
    {
        return prepared;
    }
    let tokens = tokenize_document(text, document);
    let mut bytes = text.as_bytes().to_vec();
    let mut blanked = false;
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        match &token.kind {
            TokenKind::Verb { .. } => {
                let (start, end) = (token.span.start, token.span.end);
                prepared.verbs.insert(start, token.clone());
                // `\verb` + blanks + `\/`: the engine reads one undefined
                // control word (mapped back to this token) and a control
                // symbol that restores mid-line state, so a space after the
                // argument still counts.
                if end - start >= 7 {
                    for b in &mut bytes[start + 5..end - 2] {
                        *b = b' ';
                    }
                    bytes[end - 2] = b'\\';
                    bytes[end - 1] = b'/';
                    prepared.verb_markers.insert(end - 2);
                } else {
                    for b in &mut bytes[start + 5..end] {
                        *b = b' ';
                    }
                }
                blanked = true;
                i += 1;
            }
            TokenKind::Command(name)
                if LABEL_FORMATS.contains(&name.as_str())
                    && text.as_bytes().get(token.span.end) == Some(&b'*') =>
            {
                let format = LABEL_FORMATS.iter().find(|f| **f == name.as_str()).copied().unwrap_or("arabic");
                for b in &mut bytes[token.span.start + 1..token.span.end] {
                    *b = b'Z';
                }
                prepared.renamed.insert(token.span.start, format);
                blanked = true;
                i += 1;
            }
            // hyperref reads a URL with `% # ~ _ ^ &` as other characters;
            // the parser reads those bytes raw (`url_argument`), so the engine
            // must not interpret them either.
            TokenKind::Command(name) if matches!(name.as_str(), "url" | "nolinkurl" | "href") => {
                let mut open = i + 1;
                while matches!(tokens.get(open).map(|t| &t.kind), Some(TokenKind::Space)) {
                    open += 1;
                }
                if tokens.get(open).map(|t| &t.kind) != Some(&TokenKind::LBrace) {
                    i += 1;
                    continue;
                }
                let content_start = tokens[open].span.end;
                let close = url_group_close(text, content_start);
                for b in &mut bytes[content_start..close] {
                    *b = b' ';
                }
                blanked = true;
                i = open + 1;
                while i < tokens.len() && tokens[i].span.start < close {
                    i += 1;
                }
            }
            TokenKind::Command(name) if name == "begin" => {
                let Some((env, close)) = braced_word_after(&tokens, i + 1) else {
                    i += 1;
                    continue;
                };
                if !matches!(env.as_str(), "verbatim" | "verbatim*" | "lstlisting") {
                    i += 1;
                    continue;
                }
                let mut content_start = tokens[close].span.end;
                if env == "lstlisting" {
                    content_start = after_bracket_option(text, content_start);
                }
                let end_tag = format!("\\end{{{env}}}");
                let tag_start = text[content_start..]
                    .find(end_tag.as_str())
                    .map_or(text.len(), |offset| content_start + offset);
                for b in &mut bytes[content_start..tag_start] {
                    *b = b' ';
                }
                blanked = true;
                i = close + 1;
                while i < tokens.len() && tokens[i].span.start < tag_start {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    if blanked {
        // Only whole UTF-8 sequences were replaced by ASCII spaces.
        prepared.text = Cow::Owned(String::from_utf8(bytes).expect("blanking keeps UTF-8 valid"));
    }
    prepared
}

/// The offset of the `}` closing a URL group whose content starts at `from`,
/// with the parser's `url_argument` rules (`\{`/`\}` escapes, nested braces
/// balance); the end of the text when unclosed.
fn url_group_close(text: &str, from: usize) -> usize {
    let bytes = text.as_bytes();
    let mut depth = 1usize;
    let mut pos = from;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' if matches!(bytes.get(pos + 1), Some(b'{' | b'}')) => pos += 2,
            b'{' => {
                depth += 1;
                pos += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return pos;
                }
                pos += 1;
            }
            _ => pos += 1,
        }
    }
    bytes.len()
}

/// `{ word }` starting at `index` (spaces skipped before the brace): the
/// word and the index of the closing brace.
fn braced_word_after(tokens: &[Token], mut index: usize) -> Option<(String, usize)> {
    while matches!(tokens.get(index)?.kind, TokenKind::Space) {
        index += 1;
    }
    if tokens.get(index)?.kind != TokenKind::LBrace {
        return None;
    }
    let TokenKind::Word(word) = &tokens.get(index + 1)?.kind else {
        return None;
    };
    if tokens.get(index + 2)?.kind != TokenKind::RBrace {
        return None;
    }
    Some((word.clone(), index + 2))
}

/// Mirrors the parser's `optional_bracket_argument` for `lstlisting`: the
/// body starts after a `[...]` that follows the environment name (whitespace
/// without a blank line may precede it).
fn after_bracket_option(text: &str, from: usize) -> usize {
    let bytes = text.as_bytes();
    let mut j = from;
    let mut newlines = 0;
    while j < bytes.len() && (bytes[j] as char).is_ascii_whitespace() {
        if bytes[j] == b'\n' {
            newlines += 1;
        }
        j += 1;
    }
    if newlines >= 2 || bytes.get(j) != Some(&b'[') {
        return from;
    }
    match text[j..].find(']') {
        Some(offset) => j + offset + 1,
        None => from,
    }
}

struct Converter<'d> {
    documents: &'d [SourceDocument<'d>],
    document_by_path: HashMap<&'d str, usize>,
    /// Engine source id -> document index (`None`: the prelude).
    source_documents: HashMap<u32, Option<usize>>,
    entry: usize,
    out: Vec<ExpandedToken>,
    diagnostics: Vec<Diagnostic>,
    arraystretch: HashMap<(usize, usize), String>,
    /// Characters of the word being assembled, with its provenance.
    word: Option<PendingWord>,
    last_span: Span,
}

struct PendingWord {
    text: String,
    span: Span,
    definition: Option<Span>,
    maps: bool,
}

/// Where one engine token belongs in the parser's input.
#[derive(Clone, Copy)]
struct Placement {
    span: Span,
    definition: Option<Span>,
    maps: bool,
    /// The token's own source bytes (definition bytes for replacement text),
    /// when it has any.
    real: Option<Span>,
}

impl<'d> Converter<'d> {
    fn span(&self, span: tex::Span) -> Option<Span> {
        if span.is_synthetic() {
            return None;
        }
        let document = (*self.source_documents.get(&span.source_id)?)?;
        let text = self.documents[document].text;
        let (start, end) = (span.start as usize, span.end as usize);
        (end <= text.len() && start <= end).then(|| Span::in_document(DocumentId(document), start, end))
    }

    fn place(&self, token: &tex::Token, origin: Option<tex::Span>) -> Placement {
        let own = self.span(token.span);
        match origin {
            Some(invocation) => {
                let direct = !token.span.is_synthetic()
                    && token.span.source_id == invocation.source_id
                    && token.span.start >= invocation.end;
                match (direct, own, self.span(invocation)) {
                    (true, Some(own), _) => Placement { span: own, definition: None, maps: false, real: Some(own) },
                    (_, own, Some(at)) => Placement { span: at, definition: own, maps: true, real: own },
                    (_, own, None) => Placement {
                        span: own.unwrap_or(self.last_span),
                        definition: None,
                        maps: own.is_none(),
                        real: own,
                    },
                }
            }
            None => match own {
                Some(own) => Placement { span: own, definition: None, maps: false, real: Some(own) },
                None => Placement { span: self.last_span, definition: None, maps: true, real: None },
            },
        }
    }

    fn source_text(&self, span: Span) -> &'d str {
        self.documents[span.document.0].text.get(span.start..span.end).unwrap_or("")
    }

    fn flush_word(&mut self) {
        if let Some(word) = self.word.take() {
            self.out.push(ExpandedToken {
                token: Token { kind: TokenKind::Word(word.text), span: word.span },
                definition: word.definition,
                maps_to_invocation: word.maps,
            });
        }
    }

    fn push(&mut self, kind: TokenKind, at: Placement) {
        self.flush_word();
        self.last_span = at.span;
        // Whitespace runs collapse the way the parser's own tokenizer
        // produces them: one `Space`, or one `ParBreak` if the run holds a
        // paragraph break.
        match (&kind, self.out.last().map(|t| &t.token.kind)) {
            (TokenKind::Space, Some(TokenKind::Space | TokenKind::ParBreak)) => return,
            (TokenKind::ParBreak, Some(TokenKind::ParBreak)) => return,
            (TokenKind::ParBreak, Some(TokenKind::Space)) => {
                self.out.pop();
            }
            _ => {}
        }
        self.out.push(ExpandedToken {
            token: Token { kind, span: at.span },
            definition: at.definition,
            maps_to_invocation: at.maps,
        });
    }

    fn push_char(&mut self, c: char, at: Placement) {
        self.last_span = at.span;
        if let Some(word) = &mut self.word {
            let contiguous = word.maps == at.maps
                && if at.maps {
                    word.span == at.span
                        && match (word.definition, at.definition) {
                            (Some(a), Some(b)) => a.document == b.document && a.end == b.start,
                            (None, None) => true,
                            _ => false,
                        }
                } else {
                    word.span.document == at.span.document && word.span.end == at.span.start
                };
            if contiguous {
                word.text.push(c);
                if at.maps {
                    if let (Some(d), Some(b)) = (word.definition, at.definition) {
                        word.definition = Some(d.merge(b));
                    }
                } else {
                    word.span = word.span.merge(at.span);
                }
                return;
            }
        }
        self.flush_word();
        self.word = Some(PendingWord {
            text: c.to_string(),
            span: at.span,
            definition: at.definition,
            maps: at.maps,
        });
    }

    /// `\begin`/`\end`, `{`, `name`, `}` for an environment the engine left
    /// to the parser. Piece spans are exact when the bytes after the
    /// control word spell `{name}`; otherwise every piece gets `at`.
    fn push_environment(&mut self, command: &str, name: &str, at: Placement) {
        let pieces = at.real.and_then(|real| {
            let text = self.documents[real.document.0].text;
            let bytes = text.as_bytes();
            let mut open = real.end;
            while open < bytes.len() && (bytes[open] as char).is_ascii_whitespace() {
                open += 1;
            }
            if bytes.get(open) != Some(&b'{') {
                return None;
            }
            let close = open + 1 + text[open + 1..].find('}')?;
            (text[open + 1..close].trim() == name).then_some((open, close))
        });
        let piece = |start: usize, end: usize| -> Placement {
            match (pieces, at.real) {
                (Some(_), Some(real)) => {
                    let exact = Span::in_document(real.document, start, end);
                    if at.maps {
                        Placement { span: at.span, definition: Some(exact), maps: true, real: Some(exact) }
                    } else {
                        Placement { span: exact, definition: None, maps: false, real: Some(exact) }
                    }
                }
                _ => at,
            }
        };
        let (command_at, open_at, word_at, close_at) = match (pieces, at.real) {
            (Some((open, close)), Some(real)) => (
                piece(real.start, real.end),
                piece(open, open + 1),
                piece(open + 1, close),
                piece(close, close + 1),
            ),
            _ => (at, at, at, at),
        };
        self.push(TokenKind::Command(command.to_string()), command_at);
        self.push(TokenKind::LBrace, open_at);
        self.flush_word();
        self.out.push(ExpandedToken {
            token: Token { kind: TokenKind::Word(name.to_string()), span: word_at.span },
            definition: word_at.definition,
            maps_to_invocation: word_at.maps,
        });
        self.push(TokenKind::RBrace, close_at);
    }
}

/// Run the expansion pass over the entry document (and everything it
/// includes).
pub fn expand_project(documents: &[SourceDocument<'_>], entry: usize) -> Expansion {
    let prepared: Vec<Prepared<'_>> = documents
        .iter()
        .enumerate()
        .map(|(index, document)| prepare(document.text, DocumentId(index)))
        .collect();
    let total_bytes: usize = documents.iter().map(|d| d.text.len()).sum();
    let limits = Limits {
        max_expansion_steps: 2_000_000 + 32 * total_bytes as u64,
        max_output_tokens: 2_000_000 + 8 * total_bytes as u64,
        ..Limits::default()
    };
    let entry_text: &str = prepared.get(entry).map_or("", |p| p.text.as_ref());
    let mut engine = Engine::with_limits(entry_text, limits);
    engine.set_emit_grouping(true);
    for name in BUILT_INS {
        engine.declare_host_command(name);
    }
    for name in ["verb", "input", "include"] {
        engine.declare_host_command(name);
    }
    let prelude_id = engine.push_input(PRELUDE);

    let mut conv = Converter {
        documents,
        document_by_path: documents.iter().enumerate().map(|(i, d)| (d.path, i)).collect(),
        source_documents: HashMap::from([(0, Some(entry)), (prelude_id, None)]),
        entry,
        out: Vec::new(),
        diagnostics: Vec::new(),
        arraystretch: HashMap::new(),
        word: None,
        last_span: Span::in_document(DocumentId(entry), 0, 0),
    };
    let _ = conv.entry;

    let mut lookahead: Vec<(tex::Token, Option<tex::Span>)> = Vec::new();
    loop {
        let next = if lookahead.is_empty() {
            engine.next_content_token_with_origin()
        } else {
            Some(lookahead.remove(0))
        };
        let Some((token, origin)) = next else { break };
        let at = conv.place(&token, origin);
        match &token.kind {
            TexKind::Char(c, cat) => match cat {
                CatCode::BeginGroup => conv.push(TokenKind::LBrace, at),
                CatCode::EndGroup => conv.push(TokenKind::RBrace, at),
                CatCode::MathShift => conv.push(TokenKind::MathShift, at),
                CatCode::Superscript => conv.push(TokenKind::Superscript, at),
                CatCode::Subscript => conv.push(TokenKind::Subscript, at),
                CatCode::Space => conv.push(TokenKind::Space, at),
                _ => conv.push_char(*c, at),
            },
            TexKind::ActiveChar(c) => conv.push_char(*c, at),
            TexKind::Param(n) => {
                conv.push_char('#', at);
                conv.push_char(char::from(b'0' + n.min(&9)), at);
            }
            TexKind::Eof => {}
            TexKind::ControlSequence(name) => {
                let real_text = at.real.map_or("", |real| conv.source_text(real));
                match name.as_str() {
                    "\\" => conv.push(TokenKind::LineBreak, at),
                    "[" => conv.push(TokenKind::DisplayMathOpen, at),
                    "]" => conv.push(TokenKind::DisplayMathClose, at),
                    "par" if !real_text.starts_with('\\') && at.real.is_some() => conv.push(TokenKind::ParBreak, at),
                    "verb" => {
                        let verb = at
                            .real
                            .and_then(|real| prepared[real.document.0].verbs.get(&real.start).cloned());
                        match verb {
                            Some(verb) => conv.push(verb.kind, at),
                            None => conv.push(TokenKind::Command(name.clone()), at),
                        }
                    }
                    _ if name.bytes().all(|b| b == b'Z')
                        && at
                            .real
                            .and_then(|real| prepared[real.document.0].renamed.get(&real.start))
                            .is_some() =>
                    {
                        let original = at
                            .real
                            .and_then(|real| prepared[real.document.0].renamed.get(&real.start))
                            .copied()
                            .unwrap_or("arabic");
                        conv.push(TokenKind::Command(original.to_string()), at);
                    }
                    "/" if at
                        .real
                        .is_some_and(|real| prepared[real.document.0].verb_markers.contains(&real.start)) => {}
                    "input" | "include" if origin.is_none() && real_text == format!("\\{name}") => {
                        // Read the braced path through the engine.
                        let mut taken = Vec::new();
                        let mut path = String::new();
                        let mut ok = false;
                        let mut depth = 0usize;
                        while let Some((t, o)) = engine.next_content_token_with_origin() {
                            let kind = t.kind.clone();
                            taken.push((t, o));
                            match kind {
                                TexKind::Char(_, CatCode::Space) if depth == 0 => {}
                                TexKind::Char(_, CatCode::BeginGroup) => {
                                    depth += 1;
                                    if depth > 1 {
                                        path.push('{');
                                    }
                                }
                                TexKind::Char(_, CatCode::EndGroup) if depth > 0 => {
                                    depth -= 1;
                                    if depth == 0 {
                                        ok = true;
                                        break;
                                    }
                                    path.push('}');
                                }
                                _ if depth == 0 => break,
                                TexKind::Char(c, _) | TexKind::ActiveChar(c) => path.push(c),
                                TexKind::ControlSequence(cs) => {
                                    path.push('\\');
                                    path.push_str(&cs);
                                }
                                _ => {}
                            }
                        }
                        if !ok {
                            conv.push(TokenKind::Command(name.clone()), at);
                            lookahead.extend(taken);
                            continue;
                        }
                        include(&mut conv, &mut engine, &prepared, name, path.trim(), at.span);
                    }
                    _ if name.chars().count() == 1 && !name.chars().all(char::is_alphabetic) => {
                        conv.flush_word();
                        conv.push(TokenKind::Word(name.clone()), at);
                    }
                    _ if real_text == "\\begin" && name != "begin" => {
                        if matches!(name.as_str(), "tabular" | "tabular*" | "array") {
                            if let Some(text) = engine.macro_replacement_text("arraystretch") {
                                conv.arraystretch.insert((at.span.document.0, at.span.start), text);
                            }
                        }
                        conv.push_environment("begin", name, at);
                    }
                    _ if real_text == "\\end" && name.len() > 3 && name.starts_with("end") => {
                        conv.push_environment("end", &name[3..], at);
                    }
                    _ => conv.push(TokenKind::Command(name.clone()), at),
                }
            }
        }
    }
    conv.flush_word();

    let fallback = conv.last_span;
    for diagnostic in engine.diagnostics() {
        // The parser reports unbalanced environments itself.
        if diagnostic.message.contains("without matching \\begin") {
            continue;
        }
        let span = conv.span(diagnostic.span).or(if diagnostic.span.is_synthetic() {
            Some(fallback)
        } else {
            None
        });
        let recovery = Some(recovery_for(&diagnostic.message).to_string());
        conv.diagnostics.push(match diagnostic.severity {
            tex::Severity::Error => Diagnostic::error(diagnostic.message.clone(), span, recovery),
            tex::Severity::Warning => Diagnostic::warning(diagnostic.message.clone(), span, recovery),
        });
    }

    Expansion { tokens: conv.out, diagnostics: conv.diagnostics, arraystretch: conv.arraystretch }
}

fn recovery_for(message: &str) -> &'static str {
    if message.contains("\\newcommand cannot redefine") {
        "kept the existing command definition"
    } else if message.contains("\\renewcommand cannot redefine") {
        "defined the command anyway"
    } else if message.contains("limit exceeded") {
        "stopped expanding; the rest of the input was not typeset"
    } else {
        "continued expanding after the problem"
    }
}

fn include<'p>(
    conv: &mut Converter<'_>,
    engine: &mut Engine<'p>,
    prepared: &'p [Prepared<'_>],
    command: &str,
    requested: &str,
    span: Span,
) {
    let skip = |conv: &mut Converter<'_>, message: String, recovery: &str| {
        conv.diagnostics.push(Diagnostic::error(message, Some(span), Some(recovery.into())));
    };
    if requested.is_empty() {
        return skip(
            conv,
            format!("\\{command} requires a non-empty project-relative path"),
            "skipped the empty include and continued",
        );
    }
    if !path_is_safe(requested) {
        return skip(
            conv,
            format!("rejected include path '{requested}': paths must be project-relative with no parent traversal"),
            "skipped the unsafe include and continued",
        );
    }
    let appended = format!("{requested}.tex");
    let Some(index) = conv
        .document_by_path
        .get(requested)
        .copied()
        .or_else(|| conv.document_by_path.get(appended.as_str()).copied())
    else {
        return skip(
            conv,
            format!("included file not found: looked for '{requested}' and '{appended}'"),
            "skipped the missing include and continued",
        );
    };
    let open: Vec<usize> = engine
        .open_input_ids()
        .into_iter()
        .filter_map(|id| conv.source_documents.get(&id).copied().flatten())
        .collect();
    if let Some(cycle_start) = open.iter().position(|active| *active == index) {
        let mut cycle: Vec<&str> = open[cycle_start..].iter().map(|i| conv.documents[*i].path).collect();
        cycle.push(conv.documents[index].path);
        return skip(
            conv,
            format!("include cycle detected: {}", cycle.join(" -> ")),
            "skipped the cyclic include and continued",
        );
    }
    if open.len() > INCLUDE_DEPTH_LIMIT {
        return skip(
            conv,
            format!(
                "include depth exceeds the limit of {INCLUDE_DEPTH_LIMIT} while loading '{}'",
                conv.documents[index].path
            ),
            "skipped the too-deep include and continued",
        );
    }
    let id = engine.push_input(prepared[index].text.as_ref());
    conv.source_documents.insert(id, Some(index));
}
