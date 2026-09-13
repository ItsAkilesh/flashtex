//! Verbatim text as LaTeX sets it: `verbatim`/`verbatim*` (latex.ltx
//! `\@verbatim`), `\verb`/`\verb*` (`\verb`, `\@verb`, `\@sverb`).
//!
//! The compiler gives each verbatim block its source lines and each `\verb`
//! its span; the layout re-reads the source bytes there (spaces, tabs, the
//! delimiters, the size declaration in force), so it never depends on the
//! compiler's display text.

use std::cell::RefCell;
use std::rc::Rc;

use flashtex_compiler::DocumentId;

use crate::adapter::{CharSrc, Item, Segment, TextStyle, Word};

/// What a compiler `Block::Verbatim` span opens with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockKind {
    /// `\begin{verbatim}` / `\begin{verbatim*}`.
    Verbatim { starred: bool },
    /// `\begin{lstlisting}`.
    Listing,
    /// `\lstinputlisting`.
    InputListing,
}

/// The kind of the verbatim block whose span starts at `start`.
pub fn block_kind(source: &str, start: usize) -> BlockKind {
    let rest = source.get(start..).unwrap_or("");
    if rest.starts_with("\\begin{verbatim*}") {
        BlockKind::Verbatim { starred: true }
    } else if rest.starts_with("\\begin{lstlisting}") {
        BlockKind::Listing
    } else if rest.starts_with("\\lstinputlisting") {
        BlockKind::InputListing
    } else {
        BlockKind::Verbatim { starred: false }
    }
}

/// One literal line of typewriter text as adapter items: typewriter
/// segments set literally, and a no-break space for every space and tab.
/// latex.ltx `\@vobeyspaces` makes a space `\@xobeysp`, which is
/// `\nobreakspace` (`\leavevmode\nobreak\ `), and `\@vobeytabs` makes a tab
/// `\@xobeytab`, the same one space. With `visible_spaces` (`verbatim*`,
/// `\verb*`: `\@setupverbvisiblespace`) both are `\asciispace`, the font's
/// slot 32, U+2423 here. A carriage return (CRLF source) is dropped.
pub fn literal_items(text: &str, document: DocumentId, start: usize, size_cpt: u16, visible_spaces: bool) -> Vec<Item> {
    let style = TextStyle {
        mono: true,
        literal: true,
        size_cpt,
        ..TextStyle::default()
    };
    let mut items = Vec::new();
    let mut run = String::new();
    let mut chars: Vec<CharSrc> = Vec::new();
    let flush = |items: &mut Vec<Item>, run: &mut String, chars: &mut Vec<CharSrc>| {
        if run.is_empty() {
            return;
        }
        items.push(Item::Word(Word {
            segments: vec![Segment {
                text: std::mem::take(run),
                chars: std::mem::take(chars),
                style,
            }],
        }));
    };
    for (offset, ch) in text.char_indices() {
        let src = CharSrc {
            document,
            start: start + offset,
            end: start + offset + ch.len_utf8(),
        };
        match ch {
            ' ' | '\t' if !visible_spaces => {
                flush(&mut items, &mut run, &mut chars);
                items.push(Item::Space {
                    style,
                    factor: 1000,
                    no_break: true,
                });
            }
            ' ' | '\t' => {
                run.push('\u{2423}');
                chars.push(src);
            }
            '\r' => {}
            _ => {
                run.push(ch);
                chars.push(src);
            }
        }
    }
    flush(&mut items, &mut run, &mut chars);
    items
}

/// `\verb`/`\verb*` at `start..end` as adapter items (see
/// [`literal_items`]); `None` when the span is not a `\verb` (a
/// `\lstinline`, which listings lays out). The text runs from after the
/// delimiter to the closing delimiter, or to the span's end when the
/// compiler reported it unterminated.
pub fn verb_items(source: &str, document: DocumentId, start: usize, end: usize, size_cpt: u16) -> Option<Vec<Item>> {
    let rest = source.get(start..end)?;
    let after = rest.strip_prefix("\\verb")?;
    let (starred, after) = match after.strip_prefix('*') {
        Some(a) => (true, a),
        None => (false, after),
    };
    let delim = after.chars().next()?;
    let body_start = end - after.len() + delim.len_utf8();
    let body = &after[delim.len_utf8()..];
    let body = body.strip_suffix(delim).unwrap_or(body);
    Some(literal_items(body, document, body_start, size_cpt, starred))
}

/// Environments whose bodies are read verbatim: braces and comment
/// characters inside them do not count.
const VERBATIM_ENVIRONMENTS: [&str; 4] = ["verbatim", "verbatim*", "lstlisting", "comment"];

/// The size declarations of a document: `(byte, size_cpt)` at every change,
/// in order (0 is `\normalsize`).
#[derive(Debug, Default)]
pub struct SizeScopes {
    changes: Vec<(usize, u16)>,
}

impl SizeScopes {
    /// Scans `source` for `\tiny`..`\Huge`/`\normalsize` and the
    /// environments of the same names, scoped by `{...}` groups and
    /// `\begin`/`\end`, skipping comments, control symbols, `\verb`,
    /// `\lstinline` and verbatim environments.
    pub fn scan(source: &str, class_size: u32) -> SizeScopes {
        let bytes = source.as_bytes();
        let mut changes = Vec::new();
        let mut stack: Vec<u16> = Vec::new();
        let mut current = 0u16;
        let set = |changes: &mut Vec<(usize, u16)>, current: &mut u16, at: usize, size: u16| {
            if *current != size {
                *current = size;
                changes.push((at, size));
            }
        };
        let mut i = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b'%' => {
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                }
                b'{' => {
                    stack.push(current);
                    i += 1;
                }
                b'}' => {
                    let outer = stack.pop().unwrap_or(0);
                    i += 1;
                    set(&mut changes, &mut current, i, outer);
                }
                b'\\' => {
                    let name_start = i + 1;
                    let mut j = name_start;
                    while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
                        j += 1;
                    }
                    if j == name_start {
                        // A control symbol (`\%`, `\{`, `\\`): skip both bytes.
                        i = (i + 2).min(bytes.len());
                        continue;
                    }
                    let name = &source[name_start..j];
                    i = j;
                    match name {
                        "verb" => {
                            if bytes.get(i) == Some(&b'*') {
                                i += 1;
                            }
                            if let Some(delim) = source[i..].chars().next() {
                                i += delim.len_utf8();
                                while i < bytes.len() && bytes[i] != b'\n' && !source[i..].starts_with(delim) {
                                    i += 1;
                                }
                                if i < bytes.len() && bytes[i] != b'\n' {
                                    i += delim.len_utf8();
                                }
                            }
                        }
                        "lstinline" => i = skip_lstinline(source, i),
                        "begin" | "end" => {
                            let Some((env, after)) = braced_name(source, i) else { continue };
                            i = after;
                            if name == "begin" && VERBATIM_ENVIRONMENTS.contains(&env) {
                                let end_tag = format!("\\end{{{env}}}");
                                i = source[i..].find(&end_tag).map_or(bytes.len(), |k| i + k + end_tag.len());
                            } else if name == "begin" {
                                stack.push(current);
                                if let Some(size) = size_of(env, class_size) {
                                    set(&mut changes, &mut current, i, size);
                                }
                            } else {
                                let outer = stack.pop().unwrap_or(0);
                                set(&mut changes, &mut current, i, outer);
                            }
                        }
                        other => {
                            if let Some(size) = size_of(other, class_size) {
                                set(&mut changes, &mut current, i, size);
                            }
                        }
                    }
                }
                _ => i += 1,
            }
        }
        SizeScopes { changes }
    }

    /// The size declaration in force at byte `at`.
    pub fn at(&self, at: usize) -> u16 {
        match self.changes.partition_point(|(pos, _)| *pos <= at) {
            0 => 0,
            n => self.changes[n - 1].1,
        }
    }
}

thread_local! {
    /// The last document scanned: (text pointer, length, class size).
    static SCOPES: RefCell<Option<(usize, usize, u32, Rc<SizeScopes>)>> = const { RefCell::new(None) };
}

/// The size declaration in force at byte `at` of `source` (hundredths of
/// a point; 0 for `\normalsize`), scanned once per document.
pub fn size_at(source: &str, at: usize, class_size: u32) -> u16 {
    let key = (source.as_ptr() as usize, source.len(), class_size);
    SCOPES.with(|cell| {
        let mut slot = cell.borrow_mut();
        let scopes = match &*slot {
            Some((p, l, c, s)) if (*p, *l, *c) == key => s.clone(),
            _ => {
                let s = Rc::new(SizeScopes::scan(source, class_size));
                *slot = Some((key.0, key.1, key.2, s.clone()));
                s
            }
        };
        scopes.at(at)
    })
}

/// The `{name}` right after `at` (blanks allowed first) and the byte after
/// its `}`.
fn braced_name(source: &str, at: usize) -> Option<(&str, usize)> {
    let rest = source.get(at..)?;
    let trimmed = rest.trim_start_matches([' ', '\t']);
    let open = at + (rest.len() - trimmed.len());
    let inner = trimmed.strip_prefix('{')?;
    let close = inner.find('}')?;
    Some((inner[..close].trim(), open + 1 + close + 1))
}

/// Past `\lstinline[<keys>]<c>...<c>` or `\lstinline{...}`, from the byte
/// after the command name.
fn skip_lstinline(source: &str, mut i: usize) -> usize {
    let bytes = source.as_bytes();
    while matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }
    if bytes.get(i) == Some(&b'[') {
        if let Some(k) = source[i..].find(']') {
            i += k + 1;
        }
    }
    while matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }
    match source[i..].chars().next() {
        Some('{') => {
            let mut depth = 0usize;
            while i < bytes.len() && bytes[i] != b'\n' {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            return i + 1;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            i
        }
        Some(delim) if delim != '\n' => {
            i += delim.len_utf8();
            match source[i..].find([delim, '\n']) {
                Some(k) if source[i + k..].starts_with(delim) => i + k + delim.len_utf8(),
                Some(k) => i + k,
                None => bytes.len(),
            }
        }
        _ => i,
    }
}

/// The size a declaration or environment name selects under the class
/// size (size10/11/12.clo), in hundredths of a point; `Some(0)` for
/// `normalsize`.
fn size_of(name: &str, class_size: u32) -> Option<u16> {
    use flashtex_compiler::parser::FontSizeLevel as L;
    let level = match name {
        "normalsize" => return Some(0),
        "tiny" => L::Tiny,
        "scriptsize" => L::ScriptSize,
        "footnotesize" => L::FootnoteSize,
        "small" => L::Small,
        "large" => L::Large1,
        "Large" => L::Large2,
        "LARGE" => L::Large3,
        "huge" => L::Huge1,
        "Huge" => L::Huge2,
        _ => return None,
    };
    Some(crate::adapter::declared_size(Some(level), class_size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_follow_groups_environments_and_skip_verbatim_material() {
        let src = "a {\\small b} c \\begin{footnotesize}d \\verb|{| e\\end{footnotesize} f\n\\begin{verbatim}\n{\\Huge\n\\end{verbatim}\ng % {\\tiny\nh";
        let scopes = SizeScopes::scan(src, 10);
        let at = |needle: &str| scopes.at(src.find(needle).unwrap());
        assert_eq!(at("a "), 0);
        assert_eq!(at("b}"), 900);
        assert_eq!(at("c "), 0);
        assert_eq!(at("d "), 800);
        assert_eq!(at("e\\end"), 800);
        assert_eq!(at("f\n"), 0);
        assert_eq!(at("g "), 0);
        assert_eq!(at("h"), 0);
    }

    #[test]
    fn verb_spaces_and_tabs_are_no_break_spaces_or_visible_spaces() {
        let src = "x \\verb|a b\tc| \\verb*+d e+";
        let first = src.find("\\verb|").unwrap();
        let items = verb_items(src, DocumentId(0), first, first + "\\verb|a b\tc|".len(), 0).unwrap();
        let shape: Vec<String> = items
            .iter()
            .map(|i| match i {
                Item::Word(w) => w.text(),
                Item::Space { no_break, .. } => format!("<{no_break}>"),
                _ => "?".into(),
            })
            .collect();
        assert_eq!(shape, vec!["a", "<true>", "b", "<true>", "c"]);
        if let Item::Word(w) = &items[0] {
            assert_eq!(w.segments[0].chars[0].start, first + 6);
            assert!(w.segments[0].style.mono && w.segments[0].style.literal);
        }
        let second = src.find("\\verb*").unwrap();
        let starred = verb_items(src, DocumentId(0), second, src.len(), 0).unwrap();
        assert_eq!(starred.len(), 1);
        let Item::Word(w) = &starred[0] else { panic!() };
        assert_eq!(w.text(), "d\u{2423}e");
        assert!(verb_items("\\lstinline|x|", DocumentId(0), 0, 13, 0).is_none());
    }
}
