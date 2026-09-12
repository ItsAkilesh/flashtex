//! Bounded bibliography header scanner. It never evaluates entries or invokes BibTeX.
use super::{next_char, span, trivia, Diagnostic, Symbol, SymbolKind, MAX_GROUP_DEPTH};

const MAX_KEY_BYTES: usize = 4096;
const MAX_ENTRY_BYTES: usize = 1024 * 1024;

fn header(source: &str, at: usize) -> Option<(String, usize)> {
    if source.as_bytes().get(at) != Some(&b'@') {
        return None;
    }
    let mut end = at + 1;
    while source
        .as_bytes()
        .get(end)
        .is_some_and(u8::is_ascii_alphabetic)
    {
        end += 1;
    }
    if end == at + 1 {
        return None;
    }
    let open = trivia(source, end);
    if !matches!(source.as_bytes().get(open), Some(b'{' | b'(')) {
        return None;
    }
    Some((source[at + 1..end].to_ascii_lowercase(), open))
}

/// Escapes, nested braces and quoted field values shield entry-looking text.
/// Recovery at a new line's @header is allowed only outside quoted/nested values.
fn entry_end(
    source: &str,
    start: usize,
    open: u8,
    recover: bool,
    quote_aware: bool,
) -> (usize, Option<&'static str>) {
    let base = usize::from(open == b'{');
    let mut depth = base;
    let mut quoted = false;
    let mut cursor = start;
    let mut line_start = false;
    while cursor < source.len() {
        if cursor - start > MAX_ENTRY_BYTES || depth > MAX_GROUP_DEPTH {
            return (
                source.len(),
                Some("Bibliography entry exceeds lexical bounds; remaining source excluded"),
            );
        }
        let byte = source.as_bytes()[cursor];
        if line_start && recover && !quoted && depth == base {
            let candidate = trivia(source, cursor);
            if header(source, candidate).is_some() {
                return (
                    candidate,
                    Some("Unclosed bibliography entry; resumed at the next top-level header"),
                );
            }
        }
        line_start = byte == b'\n';
        if byte == b'\\' {
            cursor = if cursor + 1 < source.len() {
                next_char(source, cursor + 1)
            } else {
                source.len()
            };
            continue;
        }
        if quote_aware && byte == b'"' && depth == base {
            quoted = !quoted;
        } else if byte == b'{' {
            depth += 1;
        } else if byte == b'}' {
            if !quoted && open == b'{' && depth == base {
                return (cursor + 1, None);
            }
            if depth > base {
                depth -= 1;
            }
        } else if byte == b')' && open == b'(' && depth == 0 && !quoted {
            return (cursor + 1, None);
        }
        cursor = next_char(source, cursor);
    }
    (
        source.len(),
        Some("Unclosed bibliography entry; remaining source excluded"),
    )
}

pub(super) fn scan(
    file: &str,
    revision: u64,
    source: &str,
) -> (Vec<Symbol>, Vec<Diagnostic>, Vec<super::BibliographyRecord>) {
    let mut symbols = Vec::new();
    let mut diagnostics = Vec::new();
    let mut records = Vec::new();
    let mut cursor = 0;
    while cursor < source.len() {
        cursor = trivia(source, cursor);
        if cursor == source.len() {
            break;
        }
        if source.as_bytes()[cursor] == b'\\' {
            cursor = if cursor + 1 < source.len() {
                next_char(source, cursor + 1)
            } else {
                source.len()
            };
            continue;
        }
        let Some((kind, open)) = header(source, cursor) else {
            cursor = next_char(source, cursor);
            continue;
        };
        let entry_start = cursor;
        let metadata = ["comment", "preamble", "string"].contains(&kind.as_str());
        let mut body_start = open + 1;
        let mut key = None;
        if !metadata {
            let start = trivia(source, body_start);
            let mut end = start;
            while end < source.len() && end - start <= MAX_KEY_BYTES {
                let ch = source[end..].chars().next().unwrap();
                if ch == ',' || ch.is_whitespace() || ch.is_control() || "{}()\\\"%=#@".contains(ch)
                {
                    break;
                }
                end = next_char(source, end);
            }
            let comma = trivia(source, end);
            if end > start
                && end - start <= MAX_KEY_BYTES
                && source.as_bytes().get(comma) == Some(&b',')
            {
                symbols.push(Symbol {
                    name: source[start..end].to_owned(),
                    kind: SymbolKind::CitationDefinition,
                    source: span(file, revision, start, end),
                });
                key = Some(span(file, revision, start, end));
                body_start = comma + 1;
            } else {
                diagnostics.push(Diagnostic { message: "Invalid or over-limit bibliography key; expected a literal key followed by comma".into(), source: span(file, revision, entry_start, open + 1) });
            }
        }
        let (after, issue) = entry_end(
            source,
            body_start,
            source.as_bytes()[open],
            !metadata,
            kind != "comment",
        );
        if let Some(message) = issue {
            diagnostics.push(Diagnostic {
                message: message.into(),
                source: span(file, revision, entry_start, open + 1),
            });
        }
        if key.is_some() || kind == "string" {
            let record = super::bibliography_values::parse(
                file,
                revision,
                source,
                super::bibliography_values::RecordBounds {
                    kind,
                    key,
                    start: entry_start,
                    body_start,
                    end: after,
                    body_end: if issue.is_none() { after - 1 } else { after },
                    closed: issue.is_none(),
                },
            );
            diagnostics.extend(record.diagnostics.clone());
            records.push(record);
        }
        cursor = after;
    }
    (symbols, diagnostics, records)
}
