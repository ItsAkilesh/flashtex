//! Two-column mode as *document state*: `\twocolumn` and `\onecolumn`.
//!
//! ```tex
//! \def\twocolumn{\@restonecolfalse\if@twocolumn\@restonecoltrue\fi
//!   \clearpage \@twocolumntrue \col@number \tw@
//!   \@ifnextchar [\@topnewpage\@floatplacement}
//! \def\onecolumn{\@restonecolfalse
//!   \clearpage \@twocolumnfalse \col@number \@ne ...}
//! ```
//! (latex.ltx lines 20256-20275, TeX Live 2026.)
//!
//! Before this module the pipeline had no model of the commands at all:
//! `\if@twocolumn` was read once from `\documentclass`'s option list, so a
//! document that asks for two columns with the *command* — the usual way
//! to do it when the class options are otherwise spoken for — was set in
//! one column from beginning to end (GH#743). A fix keyed on class options
//! cannot reach that case however many sites it patches, because the class
//! options are not where the answer is; what decides it is a command the
//! document may run, and run more than once.
//!
//! So the state lives here, as the class option plus every switch the
//! source performs, in order, and every `\if@twocolumn` question is asked
//! *at a position*. The shape is the one the pipeline already uses for the
//! other body commands the compiler discards ([`crate::adapter`]'s
//! `body_commands` / `ChromeEvent`): scan the source, keep the byte offsets,
//! fold them in document order.
//!
//! **What the commands change, measured.** Only `\if@twocolumn`,
//! `\col@number` and `\columnwidth`. The `twocolumn` class *option* is read
//! by `size1<n>.clo` while the class is still loading, so it also doubles
//! `\textwidth` and sets `\parindent` to `1em`, `\marginparsep` to `10pt`
//! and `\leftmargini` to `2em`; the commands run long afterwards and leave
//! all of those alone. Against pdflatex (TeX Live 2026, `article`, 10pt,
//! letter paper), `\twocolumn` in the preamble keeps the one-column text
//! block — first column at x = 133.768 bp where the option gives 72.0 —
//! and keeps `\parindent` at 15pt where the option gives 1em (9.963 bp).
//! Its two columns are at 133.768 and 310.605 bp, a step of 176.837 bp =
//! `\columnwidth` + `\columnsep` of the *one-column* `\textwidth`. That is
//! exactly [`flashtex_class_geometry::ResolvedDocument::set_twocolumn`],
//! which is why the command feeds that and not the class options.
//!
//! **What is not here yet.** The page frame is still one frame for the
//! whole document, so a switch that comes after material — measured: it
//! starts a new page, and the columns change from that page on — sets the
//! state every `\if@twocolumn` test reads, but not the column count of the
//! pages. That case is reported, not silently approximated. So is
//! `\twocolumn[<material>]` on a command that is *not* the document's
//! first material, which `\@topnewpage` cannot set either (it opens with
//! `\@nodocument`). The argument of a command that is, this module finds
//! for the pipeline to box ([`ColumnMode::top_material`]).

/// Two-column mode over one document's source.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColumnMode {
    /// `\if@twocolumn` where the document's first material is set: the
    /// class option, then every switch that precedes that material.
    start: bool,
    /// `(byte offset of the command, `\if@twocolumn` after it)` for every
    /// switch that comes *after* the document's first material, in source
    /// order. These are the ones the page frame cannot follow yet.
    later: Vec<(usize, bool)>,
    /// Byte range of every `\twocolumn`/`\onecolumn` command in the source,
    /// including the ones folded into [`ColumnMode::start`]. The pipeline
    /// supersedes the pinned compiler's "unknown command" for each of them
    /// the way it supersedes `abstract`'s.
    spans: Vec<(usize, usize)>,
    /// Index into the pipeline's `texts` of the source that was scanned:
    /// the entry document. `\twocolumn` inside an `\input` file is not
    /// seen, the same limitation `adapter::body_commands` has.
    document: usize,
    /// `(byte of the `[`, byte of the `]`)` of the optional argument of a
    /// `\twocolumn` that *is* the document's first material — the only
    /// place `\@topnewpage` can run (it opens with `\@nodocument`, and a
    /// `\twocolumn` after material would have to change the column count
    /// of the pages, which is [`ColumnMode::unmodelled`]).
    top: Option<(usize, usize)>,
}

/// Every `\twocolumn`/`\onecolumn` in `source`, outside comments and outside
/// macro-definition bodies, in order, as `(start byte, end byte,
/// `\if@twocolumn` after it)`.
fn switches(source: &str) -> Vec<(usize, usize, bool)> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            // A control symbol is one character long, `\%` included, so the
            // escaped percent never opens a comment.
            b'\\' => {
                // A definition body is not executed text: skip it whole, so
                // a switch inside an uninvoked (or only later invoked) macro
                // never moves the mode. This is the narrow version of
                // `adapter::skip_macro_definition` for this byte scanner:
                // that helper is `adapter`-private, misses `[n]`-argument
                // bodies, and does not cover `\newenvironment`'s two
                // bodies, so it is not reused here.
                if let Some(end) = definition_end(source, i) {
                    i = end;
                    continue;
                }
                let mut step = 2;
                for (name, on) in [("twocolumn", true), ("onecolumn", false)] {
                    let Some(rest) = source[i + 1..].strip_prefix(name) else {
                        continue;
                    };
                    // A control word ends at the first non-letter, so
                    // `\twocolumnfoo` is a different (unknown) command.
                    if !rest.starts_with(|c: char| c.is_ascii_alphabetic()) {
                        out.push((i, i + 1 + name.len(), on));
                    }
                    step = 1 + name.len();
                    break;
                }
                i += step;
            }
            _ => i += 1,
        }
    }
    out
}

/// Where the document's first typeset material can start: the end of
/// `\begin{document}`, or 0 in a body-only source.
///
/// A switch before that point is the preamble form, which is how a
/// document usually asks for two columns when the class options are taken.
/// A switch after it is only "before the first material" when nothing but
/// whitespace and comments separates the two — deliberately the narrowest
/// honest test, since anything wider would have to decide which commands
/// typeset something, and that is the compiler's job, not a scanner's.
fn first_material(source: &str) -> usize {
    let Some(at) = source.find("\\begin{document}") else {
        return 0;
    };
    let mut i = at + "\\begin{document}".len();
    let bytes = source.as_bytes();
    loop {
        let before = i;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes.get(i) == Some(&b'%') {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        }
        if i == before {
            return i;
        }
    }
}

/// The `[` that `\twocolumn`'s `\@ifnextchar [` sees after the command
/// ends at `end`, if any.
///
/// `\@ifnextchar` skips space tokens, so spaces, tabs and one newline may
/// stand between; a *blank* line is a `\par`, which is not a space token
/// and does not reach the `[`.
///
/// `pub(crate)` so the `twocolumn_top_material` limitation in [`crate::adapter`]
/// applies exactly this rule instead of its own whitespace test.
pub(crate) fn optional_bracket(source: &str, end: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut i = end;
    let mut newlines = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                newlines += 1;
                if newlines > 1 {
                    return None;
                }
                i += 1;
            }
            b' ' | b'\t' | b'\r' => i += 1,
            b'[' => return Some(i),
            _ => return None,
        }
    }
    None
}

/// Byte just past the macro definition a `\` at `at` opens, when the control
/// word there is one of `\newcommand`, `\renewcommand`, `\providecommand`,
/// `\def`, `\newenvironment` or `\renewenvironment`; `None` otherwise.
///
/// A body that never runs cannot switch the columns — real pdflatex keeps
/// the class's own column count when the macro is never invoked — so the
/// byte scan skips the definition whole: the defined name, any `[...]`
/// argument specs, and the brace-delimited bodies (two for the
/// environments). Comment- and escape-aware like [`switches`]' own scan,
/// and each body is one correctly brace-matched skip; a malformed
/// definition ends the skip where the parse gives up, and scanning resumes
/// there.
fn definition_end(source: &str, at: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut i = at + 1;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    match &source[at + 1..i] {
        "newcommand" | "renewcommand" | "providecommand" => {
            i = skip_ws_comments(source, i);
            if bytes.get(i) == Some(&b'*') {
                i = skip_ws_comments(source, i + 1);
            }
            // The defined name, `{...}` or a control sequence.
            i = skip_control_or_group(source, i)?;
            i = skip_arg_specs(source, i);
            Some(skip_group(source, i)?)
        }
        "def" => {
            // `\def\name<parameter text>{body}`.
            i = skip_control_or_group(source, skip_ws_comments(source, i))?;
            // The parameter text holds no braces to match.
            while i < bytes.len() && bytes[i] != b'{' {
                i += 1;
            }
            Some(skip_group(source, i)?)
        }
        "newenvironment" | "renewenvironment" => {
            i = skip_ws_comments(source, i);
            if bytes.get(i) == Some(&b'*') {
                i = skip_ws_comments(source, i + 1);
            }
            i = skip_control_or_group(source, i)?;
            i = skip_arg_specs(source, i);
            // The begin code and the end code.
            i = skip_group(source, i)?;
            Some(skip_group(source, i)?)
        }
        _ => None,
    }
}

/// `i` past whitespace and `%` comments.
fn skip_ws_comments(source: &str, mut i: usize) -> usize {
    let bytes = source.as_bytes();
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes.get(i) != Some(&b'%') {
            return i;
        }
        while i < bytes.len() && bytes[i] != b'\n' {
            i += 1;
        }
    }
}

/// Byte just past the `{...}` group opening at `i` — one correctly
/// brace-matched skip, comment- and escape-aware like [`switches`]' own
/// scan — or `None` when `i` is not a `{` or the group never closes.
fn skip_group(source: &str, i: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(i) != Some(&b'{') {
        return None;
    }
    let mut j = i + 1;
    let mut depth = 1i32;
    while j < bytes.len() {
        match bytes[j] {
            b'\\' => j += 2,
            b'%' => {
                while j < bytes.len() && bytes[j] != b'\n' {
                    j += 1;
                }
            }
            b'{' => {
                depth += 1;
                j += 1;
            }
            b'}' => {
                depth -= 1;
                j += 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => j += 1,
        }
    }
    None
}

/// Byte just past the defined name at `i`: a `{...}` group or a control
/// sequence; `None` when neither follows.
fn skip_control_or_group(source: &str, i: usize) -> Option<usize> {
    let i = skip_ws_comments(source, i);
    let bytes = source.as_bytes();
    if bytes.get(i) == Some(&b'{') {
        return skip_group(source, i);
    }
    if bytes.get(i) != Some(&b'\\') {
        return None;
    }
    let mut j = (i + 1).min(bytes.len());
    if bytes.get(j).is_some_and(|c| c.is_ascii_alphabetic()) {
        while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
            j += 1;
        }
        Some(j)
    } else if j < bytes.len() {
        // A control symbol, e.g. the `\,` in `\def\,{...}`.
        Some(j + 1)
    } else {
        None
    }
}

/// Byte just past any `[...]` argument specs at `i` (`\newcommand`'s `[n]`
/// and `[default]`).
fn skip_arg_specs(source: &str, mut i: usize) -> usize {
    let bytes = source.as_bytes();
    loop {
        i = skip_ws_comments(source, i);
        if bytes.get(i) != Some(&b'[') {
            return i;
        }
        i += 1;
        while i < bytes.len() && bytes[i] != b']' {
            i += 1;
        }
        if i < bytes.len() {
            i += 1;
        }
    }
}

/// The `]` that ends `\long\def\@topnewpage[#1]`'s delimited argument: the
/// first one outside braces and outside comments. A nested `[` does not
/// count — TeX matches a delimited parameter against the delimiter text
/// alone, at brace level zero.
fn closing_bracket(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut i = open + 1;
    let mut depth = 0i32;
    while i < bytes.len() {
        match bytes[i] {
            // A control symbol is one character long, so `\]`, `\%`, `\{`
            // and `\\` never delimit, comment or nest.
            b'\\' => i += 2,
            b'%' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
            }
            b']' if depth == 0 => return Some(i),
            _ => i += 1,
        }
    }
    None
}

impl ColumnMode {
    /// The mode of `source` under a class whose `twocolumn` option is
    /// `class_option`.
    pub fn scan(source: &str, document: usize, class_option: bool) -> ColumnMode {
        let material = first_material(source);
        let mut mode = ColumnMode {
            start: class_option,
            later: Vec::new(),
            spans: Vec::new(),
            document,
            top: None,
        };
        for (at, end, on) in switches(source) {
            mode.spans.push((at, end));
            if at <= material {
                mode.start = on;
                // `\@topnewpage` runs `\@nodocument` first, so only a
                // `\twocolumn` that is itself the document's first material
                // can carry the box; one in the preamble is an error in
                // LaTeX, and one after material is `unmodelled`.
                if on && at == material {
                    mode.top = optional_bracket(source, end).and_then(|open| closing_bracket(source, open).map(|close| (open, close)));
                }
            } else {
                mode.later.push((at, on));
            }
        }
        mode
    }

    /// The byte offsets of the `[` and `]` around `\twocolumn`'s optional
    /// argument, when the command is the document's first material.
    /// `\@topnewpage` sets what lies between them in a `\textwidth` box
    /// above both columns of the page the command starts.
    pub fn top_material(&self) -> Option<(usize, usize)> {
        self.top
    }

    /// Byte ranges of the `\twocolumn`/`\onecolumn` commands themselves.
    pub fn spans(&self) -> &[(usize, usize)] {
        &self.spans
    }

    /// `\if@twocolumn` where the document's first material is set. This is
    /// the one the page frame follows
    /// ([`flashtex_class_geometry::ResolvedDocument::set_twocolumn`]).
    pub fn start(&self) -> bool {
        self.start
    }

    /// The document the switches were read from.
    pub fn document(&self) -> usize {
        self.document
    }

    /// `\if@twocolumn` at byte offset `pos` of document `document`. A
    /// position in an `\input` file, whose switches are not scanned, reads
    /// the document's starting state.
    pub fn at(&self, document: usize, pos: usize) -> bool {
        if document != self.document {
            return self.start;
        }
        self.later
            .iter()
            .take_while(|(at, _)| *at < pos)
            .last()
            .map_or(self.start, |(_, on)| *on)
    }

    /// The switches the page frame cannot follow yet: those after the first
    /// material that actually change the column count from what the frame
    /// was built with.
    pub fn unmodelled(&self) -> Vec<(usize, bool)> {
        let mut state = self.start;
        let mut out = Vec::new();
        for &(at, on) in &self.later {
            if on != state {
                out.push((at, on));
            }
            state = on;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan_test(source: &str, class_option: bool) -> ColumnMode {
        ColumnMode::scan(source, 0, class_option)
    }

    #[test]
    fn a_preamble_command_is_the_documents_column_state() {
        let m = scan_test("\\documentclass{article}\n\\twocolumn\n\\begin{document}x\\end{document}", false);
        assert!(m.start());
        assert!(m.unmodelled().is_empty());
    }

    #[test]
    fn a_command_before_any_material_is_too() {
        // pdflatex sets these two documents identically.
        let m = scan_test("\\documentclass{article}\n\\begin{document}\n% hi\n  \\twocolumn\nx\\end{document}", false);
        assert!(m.start());
        assert!(m.unmodelled().is_empty());
    }

    #[test]
    fn a_command_after_material_is_positional_and_reported() {
        let s = "\\documentclass{article}\n\\begin{document}\nA\n\\twocolumn\nB\\end{document}";
        let m = scan_test(s, false);
        assert!(!m.start());
        let at = s.find("\\twocolumn").unwrap();
        assert!(!m.at(0, at));
        assert!(m.at(0, at + 1));
        assert_eq!(m.unmodelled(), vec![(at, true)]);
    }

    #[test]
    fn onecolumn_undoes_the_class_option() {
        let m = scan_test("\\documentclass[twocolumn]{article}\n\\onecolumn\n\\begin{document}x\\end{document}", true);
        assert!(!m.start());
    }

    #[test]
    fn a_switch_that_changes_nothing_is_not_reported() {
        let s = "\\documentclass[twocolumn]{article}\n\\begin{document}\nA\n\\twocolumn\nB\\end{document}";
        assert!(scan_test(s, true).unmodelled().is_empty());
    }

    #[test]
    fn comments_and_longer_names_are_not_switches() {
        let m = scan_test("\\documentclass{article}\n% \\twocolumn\n\\twocolumnfoo\n\\begin{document}x\\end{document}", false);
        assert!(!m.start());
        assert!(m.unmodelled().is_empty());
    }

    #[test]
    fn the_first_materials_twocolumn_carries_the_topnewpage_box() {
        let src = "\\begin{document}\n\\twocolumn[Banner]\nAaa\n\\end{document}\n";
        let m = scan_test(src, false);
        let (open, close) = m.top_material().expect("an optional argument");
        assert_eq!(&src[open..=close], "[Banner]");
    }

    #[test]
    fn the_argument_ends_at_the_first_bracket_outside_braces() {
        // TeX matches `\@topnewpage`'s delimited parameter at brace level
        // zero, so the `]` inside the group does not end it.
        let src = "\\begin{document}\n\\twocolumn[{\\Large a]b} c]\nAaa\n\\end{document}\n";
        let m = scan_test(src, false);
        let (open, close) = m.top_material().unwrap();
        assert_eq!(&src[open..=close], "[{\\Large a]b} c]");
    }

    #[test]
    fn a_preamble_twocolumn_carries_no_box() {
        // `\@topnewpage` runs `\@nodocument` first: this is a LaTeX error,
        // not a banner.
        let m = scan_test("\\twocolumn[Banner]\n\\begin{document}\nAaa\n\\end{document}\n", false);
        assert!(m.start());
        assert_eq!(m.top_material(), None);
    }

    #[test]
    fn a_twocolumn_after_material_carries_no_box() {
        let m = scan_test("\\begin{document}\nAaa\n\n\\twocolumn[Banner]\nBbb\n\\end{document}\n", false);
        assert_eq!(m.top_material(), None);
        assert_eq!(m.unmodelled(), vec![(22, true)]);
    }

    #[test]
    fn a_blank_line_before_the_bracket_is_a_par_and_not_an_argument() {
        // `\@ifnextchar [` skips space tokens; a blank line is a `\par`.
        let m = scan_test("\\begin{document}\n\\twocolumn\n\n[Banner]\nAaa\n\\end{document}\n", false);
        assert_eq!(m.top_material(), None);
    }

    #[test]
    fn switches_inside_uninvoked_macro_bodies_do_not_move_the_mode() {
        // Neither macro is ever invoked: real pdflatex keeps the class's
        // own column count in every case, since the body never runs.
        for class_option in [false, true] {
            for (def, switch, word) in [
                ("newcommand", "onecolumn", "wide"),
                ("newcommand", "twocolumn", "narrow"),
                ("renewcommand", "onecolumn", "wide"),
                ("renewcommand", "twocolumn", "narrow"),
                ("providecommand", "onecolumn", "wide"),
                ("providecommand", "twocolumn", "narrow"),
            ] {
                let src = format!(
                    "\\documentclass{options}{{article}}\n\\{def}{{\\{word}}}{{\\{switch}}}\n\\begin{{document}}x\\end{{document}}",
                    options = if class_option { "[twocolumn]" } else { "" },
                );
                let m = scan_test(&src, class_option);
                assert_eq!(m.start(), class_option, "{src}");
                assert!(m.unmodelled().is_empty(), "{src}");
            }
        }
    }

    #[test]
    fn def_and_newenvironment_bodies_do_not_move_the_mode_either() {
        for (class_option, src) in [
            (false, "\\documentclass{article}\n\\def\\wide{\\onecolumn}\n\\begin{document}x\\end{document}"),
            (false, "\\documentclass{article}\n\\def\\narrow{\\twocolumn}\n\\begin{document}x\\end{document}"),
            (true, "\\documentclass[twocolumn]{article}\n\\newenvironment{wide}{\\onecolumn}{}\n\\begin{document}x\\end{document}"),
            (true, "\\documentclass[twocolumn]{article}\n\\renewenvironment{narrow}{\\twocolumn}{}\n\\begin{document}x\\end{document}"),
            (false, "\\documentclass{article}\n\\newenvironment{narrow}{\\twocolumn}{\\onecolumn}\n\\begin{document}x\\end{document}"),
        ] {
            let m = scan_test(src, class_option);
            assert_eq!(m.start(), class_option, "{src}");
            assert!(m.unmodelled().is_empty(), "{src}");
        }
    }

    #[test]
    fn onecolumn_takes_no_optional_argument() {
        let m = scan_test("\\begin{document}\n\\onecolumn[Banner]\nAaa\n\\end{document}\n", true);
        assert!(!m.start());
        assert_eq!(m.top_material(), None);
    }

}
