//! Brace-aware `key=value` lists as used by fontspec (`[Scale=0.9, WordSpace={1,0,0}]`)
//! and unicode-math (`[math-style=ISO]`), plus the small TeX argument scanner
//! the command parsers share.

/// One `key` or `key=value` entry. Values have one level of surrounding
/// braces removed and are trimmed; keys are trimmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyVal {
    pub key: String,
    pub value: Option<String>,
}

/// Splits `s` at top-level commas (not inside `{}`), then each item at its
/// first top-level `=`. Empty items are dropped.
pub fn parse(s: &str) -> Vec<KeyVal> {
    split_top_level(s, ',')
        .into_iter()
        .filter_map(|item| {
            let item = item.trim();
            if item.is_empty() {
                return None;
            }
            match find_top_level(item, '=') {
                Some(i) => Some(KeyVal {
                    key: item[..i].trim().to_string(),
                    value: Some(strip_braces(item[i + 1..].trim()).to_string()),
                }),
                None => Some(KeyVal {
                    key: strip_braces(item).to_string(),
                    value: None,
                }),
            }
        })
        .collect()
}

/// Splits at `sep` outside braces.
pub fn split_top_level(s: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let mut prev_backslash = false;
    for (i, c) in s.char_indices() {
        match c {
            '{' if !prev_backslash => depth += 1,
            '}' if !prev_backslash => depth -= 1,
            c if c == sep && depth == 0 => {
                out.push(&s[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
        prev_backslash = c == '\\' && !prev_backslash;
    }
    out.push(&s[start..]);
    out
}

fn find_top_level(s: &str, target: char) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            c if c == target && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Removes one pair of braces enclosing the whole string, if they match.
pub fn strip_braces(s: &str) -> &str {
    let t = s.trim();
    if t.starts_with('{') && t.ends_with('}') && t.len() >= 2 {
        // Only strip when the first brace closes at the very end.
        let mut depth = 0i32;
        for (i, c) in t.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 && i != t.len() - 1 {
                        return t;
                    }
                }
                _ => {}
            }
        }
        return t[1..t.len() - 1].trim();
    }
    t
}

/// Removes `%` comments (an unescaped `%` to end of line), keeping byte
/// offsets stable by replacing the comment text with spaces.
pub fn blank_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut in_comment = false;
    let mut prev_backslash = false;
    for c in src.chars() {
        if c == '\n' {
            in_comment = false;
            prev_backslash = false;
            out.push(c);
            continue;
        }
        if in_comment {
            for _ in 0..c.len_utf8() {
                out.push(' ');
            }
            continue;
        }
        if c == '%' && !prev_backslash {
            in_comment = true;
            out.push(' ');
            continue;
        }
        prev_backslash = c == '\\' && !prev_backslash;
        out.push(c);
    }
    out
}

/// A cursor over TeX source that reads optional `[...]` and mandatory `{...}`
/// arguments with balanced braces.
pub struct ArgScanner<'a> {
    pub src: &'a str,
    pub pos: usize,
}

impl<'a> ArgScanner<'a> {
    pub fn new(src: &'a str, pos: usize) -> ArgScanner<'a> {
        ArgScanner { src, pos }
    }

    pub fn skip_space(&mut self) {
        while let Some(c) = self.src[self.pos..].chars().next() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    /// `[...]` if present (brackets inside braces are ignored).
    pub fn optional(&mut self) -> Option<&'a str> {
        let save = self.pos;
        self.skip_space();
        if !self.src[self.pos..].starts_with('[') {
            self.pos = save;
            return None;
        }
        let start = self.pos + 1;
        let mut depth = 0i32;
        for (i, c) in self.src[start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                ']' if depth == 0 => {
                    self.pos = start + i + 1;
                    return Some(&self.src[start..start + i]);
                }
                _ => {}
            }
        }
        self.pos = save;
        None
    }

    /// `{...}` (balanced), or a single control sequence / character.
    pub fn mandatory(&mut self) -> Option<&'a str> {
        self.skip_space();
        let rest = &self.src[self.pos..];
        let c = rest.chars().next()?;
        if c == '{' {
            let start = self.pos + 1;
            let mut depth = 1i32;
            let mut prev_backslash = false;
            for (i, c) in self.src[start..].char_indices() {
                match c {
                    '{' if !prev_backslash => depth += 1,
                    '}' if !prev_backslash => {
                        depth -= 1;
                        if depth == 0 {
                            self.pos = start + i + 1;
                            return Some(&self.src[start..start + i]);
                        }
                    }
                    _ => {}
                }
                prev_backslash = c == '\\' && !prev_backslash;
            }
            None
        } else if c == '\\' {
            let start = self.pos;
            let mut end = start + 1;
            for (i, c) in rest[1..].char_indices() {
                if c.is_ascii_alphabetic() {
                    end = start + 1 + i + 1;
                } else {
                    if i == 0 {
                        end = start + 1 + c.len_utf8();
                    }
                    break;
                }
            }
            self.pos = end;
            Some(&self.src[start..end])
        } else {
            self.pos += c.len_utf8();
            Some(&rest[..c.len_utf8()])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_values() {
        let kv = parse(
            " Scale = MatchLowercase, WordSpace={1,0,0}, Ligatures={TeX,Rare} ,Color=FF0000,",
        );
        assert_eq!(kv.len(), 4);
        assert_eq!(kv[1].key, "WordSpace");
        assert_eq!(kv[1].value.as_deref(), Some("1,0,0"));
        assert_eq!(kv[2].value.as_deref(), Some("TeX,Rare"));
    }

    #[test]
    fn scanner_reads_arguments() {
        let s = r"\setmainfont[Scale=0.9]{TeX Gyre {Termes}}[Numbers=OldStyle] rest";
        let mut a = ArgScanner::new(s, "\\setmainfont".len());
        assert_eq!(a.optional(), Some("Scale=0.9"));
        assert_eq!(a.mandatory(), Some("TeX Gyre {Termes}"));
        assert_eq!(a.optional(), Some("Numbers=OldStyle"));
        assert_eq!(a.optional(), None);
    }

    #[test]
    fn comments_blanked_in_place() {
        let s = "a % b\n50\\% c";
        let b = blank_comments(s);
        assert_eq!(b.len(), s.len());
        assert!(b.starts_with("a    "));
        assert!(b.ends_with("50\\% c"));
    }
}
