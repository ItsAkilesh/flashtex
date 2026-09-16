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
//! `\twocolumn[<material>]`, whose argument LaTeX sets at the full
//! `\textwidth` above both columns (`\@topnewpage`); the compiler drops it
//! with a diagnostic of its own.

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
}

/// Every `\twocolumn`/`\onecolumn` in `source`, outside comments, in order,
/// as `(start byte, end byte, `\if@twocolumn` after it)`.
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
        };
        for (at, end, on) in switches(source) {
            mode.spans.push((at, end));
            if at <= material {
                mode.start = on;
            } else {
                mode.later.push((at, on));
            }
        }
        mode
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
}
