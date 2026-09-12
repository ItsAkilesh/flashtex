//! Byte-offset text helpers shared by the context window and the suggestion
//! heuristics. Every function takes and returns UTF-8 byte offsets that land on
//! character boundaries, so callers can slice without panicking.

/// Largest character boundary `<= i` (clamped to the text length).
pub fn snap_down(text: &str, i: usize) -> usize {
    let mut i = i.min(text.len());
    while !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Smallest character boundary `>= i` (clamped to the text length).
pub fn snap_up(text: &str, i: usize) -> usize {
    let mut i = i.min(text.len());
    while !text.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Clamp and snap a span so `&text[start..end]` is valid and non-inverted.
pub fn clamp_span(text: &str, start: usize, end: usize) -> (usize, usize) {
    let s = snap_down(text, start);
    let e = snap_up(text, end.max(start));
    (s, e.max(s))
}

/// Start of the line containing `pos` (the byte after the previous '\n').
pub fn line_start(text: &str, pos: usize) -> usize {
    let pos = snap_down(text, pos);
    text[..pos].rfind('\n').map_or(0, |i| i + 1)
}

/// End of the line containing `pos`, excluding the '\n' itself.
pub fn line_end(text: &str, pos: usize) -> usize {
    let pos = snap_up(text, pos);
    text[pos..].find('\n').map_or(text.len(), |i| pos + i)
}

/// 1-based line number and 1-based column (in characters) of `pos`.
pub fn line_and_column(text: &str, pos: usize) -> (usize, usize) {
    let pos = snap_down(text, pos);
    let line = text[..pos].matches('\n').count() + 1;
    let column = text[line_start(text, pos)..pos].chars().count() + 1;
    (line, column)
}

/// Byte offset where the paragraph containing `pos` ends: the position just
/// after the last non-whitespace character before the next blank line
/// (a '\n' followed by only spaces/tabs and another '\n'), or the end of the
/// trimmed text when there is no blank line after `pos`.
pub fn paragraph_end(text: &str, pos: usize) -> usize {
    let pos = snap_up(text, pos);
    let bytes = text.as_bytes();
    let mut i = pos;
    let mut stop = text.len();
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            // Is the following line blank?
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\r') {
                j += 1;
            }
            if j >= bytes.len() || bytes[j] == b'\n' {
                stop = i;
                break;
            }
        }
        i += 1;
    }
    trim_end_whitespace(text, pos, stop)
}

/// Move `end` backwards over whitespace, but not before `floor`.
pub fn trim_end_whitespace(text: &str, floor: usize, end: usize) -> usize {
    let floor = snap_up(text, floor);
    let mut e = snap_down(text, end);
    while e > floor {
        let prev = text[..e].chars().next_back().unwrap();
        if !prev.is_whitespace() {
            break;
        }
        e -= prev.len_utf8();
    }
    e.max(floor)
}

/// Significant characters of `text` from `from`: comments (`%` to end of
/// line) are skipped, and an escaped character (`\{`, `\}`, `\$`, `\%`, `\\`)
/// is skipped entirely so brace/dollar scans do not see it. Control words
/// (`\name`) are yielded as their backslash so callers can detect commands.
pub fn significant(text: &str, from: usize) -> Vec<(usize, char)> {
    let mut out = Vec::new();
    let mut it = text[from..]
        .char_indices()
        .map(|(i, c)| (i + from, c))
        .peekable();
    while let Some((i, c)) = it.next() {
        match c {
            '%' => {
                // The newline that ends a comment stays significant.
                while let Some(&(_, d)) = it.peek() {
                    if d == '\n' {
                        break;
                    }
                    it.next();
                }
            }
            '\\' => match it.peek().copied() {
                Some((_, d)) if d.is_alphabetic() => out.push((i, '\\')),
                Some(_) => {
                    it.next();
                }
                None => {}
            },
            _ => out.push((i, c)),
        }
    }
    out
}

/// Control word starting at `pos` (which must point at its backslash); returns
/// the name without the backslash and the byte just past it.
pub fn command_at(text: &str, pos: usize) -> Option<(&str, usize)> {
    let rest = text.get(pos..)?;
    let rest = rest.strip_prefix('\\')?;
    let len: usize = rest
        .chars()
        .take_while(|c| c.is_alphabetic())
        .map(char::len_utf8)
        .sum();
    if len == 0 {
        return None;
    }
    Some((&rest[..len], pos + 1 + len))
}

/// A `\begin{name}` / `\end{name}` found by [`environments`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvMarker {
    pub is_begin: bool,
    pub name: String,
    /// Span of the whole `\begin{name}` including braces.
    pub start: usize,
    pub end: usize,
}

/// Every `\begin{...}` and `\end{...}` from `from`, in order, ignoring comments.
pub fn environments(text: &str, from: usize) -> Vec<EnvMarker> {
    let mut out = Vec::new();
    for (i, c) in significant(text, from) {
        if c != '\\' {
            continue;
        }
        let Some((name, after)) = command_at(text, i) else {
            continue;
        };
        let is_begin = match name {
            "begin" => true,
            "end" => false,
            _ => continue,
        };
        let Some((arg, end)) = braced_argument(text, after) else {
            continue;
        };
        out.push(EnvMarker {
            is_begin,
            name: arg.trim().to_string(),
            start: i,
            end,
        });
    }
    out
}

/// If a `{...}` (after optional spaces/tabs) starts at `pos`, return its
/// content and the byte just past the closing brace. Nested braces balance.
pub fn braced_argument(text: &str, pos: usize) -> Option<(&str, usize)> {
    let mut i = pos;
    while i < text.len() && (text.as_bytes()[i] == b' ' || text.as_bytes()[i] == b'\t') {
        i += 1;
    }
    if text.as_bytes().get(i) != Some(&b'{') {
        return None;
    }
    let mut depth = 0usize;
    for (j, c) in significant(text, i) {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&text[i + 1..j], j + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// Where the group opened at `open` (a '{') would end if it were balanced:
/// the byte just past its matching '}', or `None` if it never closes.
pub fn matching_close_brace(text: &str, open: usize) -> Option<usize> {
    braced_argument(text, open).map(|(_, end)| end)
}

/// Position of the '{' whose group is still open at `close` (a '}' that the
/// compiler reported as unmatched), scanning backwards with a brace stack over
/// significant characters. `None` when every earlier '{' is balanced.
pub fn unmatched_open_before(text: &str, close: usize) -> Option<usize> {
    let mut stack: Vec<usize> = Vec::new();
    for (i, c) in significant(text, 0) {
        if i >= close {
            break;
        }
        match c {
            '{' => stack.push(i),
            '}' => {
                stack.pop();
            }
            _ => {}
        }
    }
    stack.pop()
}

/// The next '$' at or after `from` that is not escaped; `None` if absent.
pub fn next_dollar(text: &str, from: usize) -> Option<usize> {
    significant(text, from)
        .into_iter()
        .find(|&(_, c)| c == '$')
        .map(|(i, _)| i)
}

/// Optimal-string-alignment edit distance over characters: insertions,
/// deletions, substitutions and adjacent transpositions each cost 1, so the
/// most common typo (`sectoin`) is one step from `section`.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let w = b.len() + 1;
    let mut d = vec![0usize; (a.len() + 1) * w];
    for i in 0..=a.len() {
        d[i * w] = i;
    }
    for (j, cell) in d.iter_mut().enumerate().take(b.len() + 1) {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (d[(i - 1) * w + j] + 1)
                .min(d[i * w + j - 1] + 1)
                .min(d[(i - 1) * w + j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(d[(i - 2) * w + j - 2] + 1);
            }
            d[i * w + j] = best;
        }
    }
    d[a.len() * w + b.len()]
}

/// Up to `limit` supported names closest to `name`, nearest first, ties
/// broken alphabetically. Only names within a distance budget that scales
/// with the length of `name` are returned, so unrelated commands never appear.
pub fn closest_commands<'a>(
    name: &str,
    supported: &[&'a str],
    limit: usize,
) -> Vec<(&'a str, usize)> {
    let budget = (name.chars().count() / 3).clamp(1, 3);
    let lower = name.to_lowercase();
    let mut scored: Vec<(&str, usize)> = supported
        .iter()
        .filter(|s| !s.is_empty() && **s != name)
        .map(|s| {
            let d = edit_distance(&lower, &s.to_lowercase());
            // A supported name that merely extends or prefixes the typed one
            // (e.g. `sub` -> `subsection`) is a plausible completion.
            let d = if s.starts_with(&lower) || lower.starts_with(*s) {
                d.min(budget)
            } else {
                d
            };
            (*s, d)
        })
        .filter(|(_, d)| *d <= budget)
        .collect();
    scored.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)));
    scored.truncate(limit);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapping_lands_on_boundaries() {
        let t = "aé😀b";
        assert_eq!(snap_down(t, 2), 1);
        assert_eq!(snap_up(t, 2), 3);
        assert_eq!(snap_down(t, 5), 3);
        assert_eq!(snap_up(t, 5), 7);
        assert_eq!(snap_up(t, 99), t.len());
    }

    #[test]
    fn paragraph_end_stops_before_blank_line() {
        let t = "one\ntwo  \n\nthree";
        assert_eq!(paragraph_end(t, 0), 7);
        assert_eq!(paragraph_end(t, 10), t.len());
        assert_eq!(paragraph_end("tail   \n", 0), 4);
    }

    #[test]
    fn significant_skips_comments_and_escapes() {
        let s = significant("a\\{b%{\n{", 0);
        let chars: String = s.iter().map(|(_, c)| *c).collect();
        assert_eq!(chars, "ab\n{");
    }

    #[test]
    fn environments_and_braces() {
        let t = "x \\begin{a} {q} \\end{a}";
        let envs = environments(t, 0);
        assert_eq!(envs.len(), 2);
        assert_eq!(envs[0].name, "a");
        assert_eq!(&t[envs[0].start..envs[0].end], "\\begin{a}");
        assert_eq!(matching_close_brace(t, 12), Some(15));
        assert_eq!(unmatched_open_before("{a{b}c}}", 7), None);
        assert_eq!(unmatched_open_before("{a{b}c}", 6), Some(0));
    }

    #[test]
    fn closest_prefers_small_distance() {
        let sup = ["section", "subsection", "textbf", "emph"];
        let c = closest_commands("secton", &sup, 3);
        assert_eq!(c[0].0, "section");
        assert!(closest_commands("tikz", &sup, 3).is_empty());
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("sectoin", "section"), 1);
    }
}
