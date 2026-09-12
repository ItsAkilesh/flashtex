//! Source-context suggestion heuristics, one builder per catalog [`Kind`].
//!
//! Every builder receives the diagnostic, the exact document text and the
//! app's supported-command list, and returns bounded, ordered suggestions.
//! Edits are candidates for the UI to preview; nothing is applied here. When
//! the span is missing or does not fit the text (stale revision), builders
//! fall back to advice without edits rather than guessing offsets.
//!
//! Everything in this module is heuristic. The README lists what each one
//! assumes; confidence levels encode how often the assumption holds.

use crate::catalog::Kind;
use crate::text;
use crate::{Confidence, Diagnostic, Edit, Suggestion};

/// Upper bound on suggestions per diagnostic.
pub const MAX_SUGGESTIONS: usize = 4;

/// A diagnostic span validated against the document text.
struct Loc<'a> {
    path: &'a str,
    s: usize,
    e: usize,
}

fn locate<'a>(diagnostic: &'a Diagnostic, text: &str) -> Option<Loc<'a>> {
    let src = diagnostic.source.as_ref()?;
    if text.is_empty() || src.start_byte > src.end_byte || src.end_byte > text.len() {
        return None;
    }
    if !text.is_char_boundary(src.start_byte) || !text.is_char_boundary(src.end_byte) {
        return None;
    }
    Some(Loc {
        path: &src.path,
        s: src.start_byte,
        e: src.end_byte,
    })
}

pub fn suggestions(
    kind: Kind,
    captures: &[String],
    diagnostic: &Diagnostic,
    text: &str,
    supported: &[&str],
) -> Vec<Suggestion> {
    let cap = |i: usize| captures.get(i).map(String::as_str).unwrap_or("");
    let loc = locate(diagnostic, text);
    let mut out = match kind {
        Kind::UnclosedGroup => unclosed_group(loc, text),
        Kind::StrayCloseBrace | Kind::MathStrayCloseBrace => stray_close_brace(loc),
        Kind::UnterminatedEnvironment => unterminated_environment(loc, text, cap(0)),
        Kind::EnvironmentMismatch => environment_mismatch(loc, text, cap(0), cap(1)),
        Kind::StrayEnd => stray_end(loc, text, cap(0)),
        Kind::UnsupportedEnvironment => unsupported_environment(loc, text, cap(0)),
        Kind::MathModeUnavailable => math_mode_unavailable(loc, text),
        Kind::UnsupportedCommand => unsupported_command(loc, text, cap(0), supported, false),
        Kind::UnsupportedMathCommand => unsupported_command(loc, text, cap(0), supported, true),
        Kind::MissingBracedArgument => missing_braced_argument(loc, text, cap(0)),
        Kind::MissingMathArgument => missing_math_argument(loc, text, cap(0)),
        Kind::ArgumentUnclosed => argument_unclosed(loc, text, cap(0)),
        Kind::EmptyArgument | Kind::EmptyPackageList => delete_span(
            loc,
            "Remove the command and its empty argument.",
            Confidence::Medium,
        ),
        Kind::PathRejected => path_rejected(cap(0)),
        Kind::NoDocuments => vec![Suggestion::advice(
            "Include at least the entry document in the compile request's documents list.",
            Confidence::Medium,
        )],
        Kind::MultiDocument => vec![Suggestion::advice(
            "Only the entry document was typeset. Until \\input/\\include are implemented, keep the content you want rendered in the entry file.",
            Confidence::Low,
        )],
        Kind::ProtocolError => vec![Suggestion::advice(
            "Recompile from the app; if this repeats, the app and compiler builds disagree about runtime protocol v1.",
            Confidence::Low,
        )],
        Kind::DuplicateScript => duplicate_script(loc, text),
        Kind::ScriptWithoutAtom => script_without_atom(loc),
        Kind::MathGroupUnclosed => insert_at_end(
            loc,
            "}",
            "Close the group before the formula ends.",
            Confidence::Medium,
        ),
        Kind::ScriptMissingArgument => script_missing_argument(loc),
        Kind::NestedMathDelimiter => nested_math_delimiter(loc, text),
        Kind::StrayDisplayClose => stray_display_close(loc, text),
        Kind::ScriptOutsideMath => script_outside_math(loc, text),
        Kind::DisplayMathUnclosed => math_unclosed(loc, text, "\\]"),
        Kind::InlineMathUnclosed => math_unclosed(loc, text, "$"),
        Kind::EmptyDocumentClass => empty_document_class(loc, text),
        Kind::PackagesNotImplemented => packages_not_implemented(loc, text),
        Kind::MacroNameInvalid => vec![Suggestion::advice(
            format!(
                "Write the definition as \\{}{{\\name}}[count]{{body}}, with exactly one command name in the first braces.",
                cap(0)
            ),
            Confidence::Medium,
        )],
        Kind::MacroArgCountInvalid => macro_argcount_invalid(loc),
        Kind::NewcommandExists => rename_definer(loc, text, "\\newcommand", "\\renewcommand"),
        Kind::RenewcommandUndefined => rename_definer(loc, text, "\\renewcommand", "\\newcommand"),
        Kind::MacroRecursion => macro_recursion(loc, cap(0)),
        Kind::MacroArgumentUndeclared => macro_argument_undeclared(loc, text, cap(0)),
        Kind::OptionalArgumentUnclosed => optional_argument_unclosed(loc, text),
        Kind::PreambleUnsupported => preamble_unsupported(loc, text),
    };
    out.truncate(MAX_SUGGESTIONS);
    out
}

// ---- brace and group heuristics --------------------------------------------

fn unclosed_group(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Add the missing '}' where the group should end, or remove the '{'.",
            Confidence::Low,
        )];
    };
    let mut out = Vec::new();
    let after_brace = text::snap_up(text, l.s + 1);
    let eol = text::trim_end_whitespace(text, after_brace, text::line_end(text, after_brace));
    let para = text::paragraph_end(text, after_brace);
    if eol > after_brace && eol < para {
        out.push(Suggestion::new(
            "Close the group at the end of this line.",
            vec![Edit::insert(l.path, eol, "}")],
            Confidence::Medium,
        ));
    }
    if para > after_brace {
        out.push(Suggestion::new(
            "Close the group before the next paragraph break.",
            vec![Edit::insert(l.path, para, "}")],
            if out.is_empty() {
                Confidence::Medium
            } else {
                Confidence::Low
            },
        ));
    }
    out.push(Suggestion::new(
        "Remove the opening brace.",
        vec![Edit::delete(
            l.path,
            l.s,
            text::snap_up(text, l.e.max(l.s + 1)),
        )],
        Confidence::Low,
    ));
    out
}

fn stray_close_brace(loc: Option<Loc>) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Delete the extra '}', or write '\\}' for a literal brace.",
            Confidence::Low,
        )];
    };
    vec![
        Suggestion::new(
            "Remove the stray brace.",
            vec![Edit::delete(l.path, l.s, l.e)],
            Confidence::Medium,
        ),
        Suggestion::new(
            "Keep a literal brace by escaping it as '\\}'.",
            vec![Edit::new(l.path, l.s, l.e, "\\}")],
            Confidence::Low,
        ),
    ]
}

fn argument_unclosed(loc: Option<Loc>, text: &str, command: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            format!("Add the '}}' that closes the argument to \\{command}."),
            Confidence::Low,
        )];
    };
    let after = text::snap_up(text, l.s + 1);
    let para = text::paragraph_end(text, after);
    let eol = text::trim_end_whitespace(text, after, text::line_end(text, after));
    let mut out = vec![Suggestion::new(
        format!("Close \\{command}'s argument before the paragraph break."),
        vec![Edit::insert(l.path, para, "}")],
        Confidence::Medium,
    )];
    if eol > after && eol < para {
        out.push(Suggestion::new(
            format!("Close \\{command}'s argument at the end of this line."),
            vec![Edit::insert(l.path, eol, "}")],
            Confidence::Low,
        ));
    }
    out
}

// ---- environments ---------------------------------------------------------

fn unterminated_environment(loc: Option<Loc>, text: &str, name: &str) -> Vec<Suggestion> {
    let closing = format!("\\end{{{name}}}");
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            format!("Add {closing} where the environment should end."),
            Confidence::Low,
        )];
    };
    let eof = text::trim_end_whitespace(text, l.e, text.len());
    if name == "document" {
        return vec![Suggestion::new(
            "Close the document at the end of the file.",
            vec![Edit::new(l.path, eof, text.len(), format!("\n{closing}\n"))],
            Confidence::High,
        )];
    }

    // After an unterminated \begin{X}, every later environment is balanced
    // (an unbalanced \end would have closed X instead), so a depth-0 scan
    // finds sibling structure: the first sectioning command or sibling
    // \begin at depth 0 is where X most plausibly should have ended.
    let mut depth = 0usize;
    let mut before_section: Option<usize> = None;
    let mut before_sibling: Option<usize> = None;
    for (i, c) in text::significant(text, l.e) {
        if c != '\\' {
            continue;
        }
        let Some((cmd, after)) = text::command_at(text, i) else {
            continue;
        };
        match cmd {
            "begin" => {
                if depth == 0 && before_sibling.is_none() {
                    before_sibling = Some(i);
                }
                if text::braced_argument(text, after).is_some() {
                    depth += 1;
                }
            }
            "end" => {
                if let Some((arg, _)) = text::braced_argument(text, after) {
                    if arg.trim() == "document" && depth == 0 {
                        before_section.get_or_insert(i);
                        break;
                    }
                    depth = depth.saturating_sub(1);
                }
            }
            "section" | "subsection" if depth == 0 => {
                before_section = Some(i);
                break;
            }
            _ => {}
        }
    }

    // Candidates in preference order; the first distinct position is Medium.
    // (insert at, delete through, why, insertion)
    let mut candidates: Vec<(usize, usize, String, String)> = Vec::new();
    if let Some(at) = before_section {
        let at = text::line_start(text, at).max(l.e);
        candidates.push((
            at,
            at,
            format!("Insert {closing} before the next section or the end of the document body."),
            format!("{closing}\n"),
        ));
    }
    let para = text::paragraph_end(text, l.e);
    if para > l.e && para < eof {
        candidates.push((
            para,
            para,
            format!("Insert {closing} after the paragraph that opens it."),
            format!("\n{closing}"),
        ));
    }
    if let Some(at) = before_sibling {
        let at = text::line_start(text, at).max(l.e);
        candidates.push((
            at,
            at,
            format!("Insert {closing} before the next \\begin at the same level."),
            format!("{closing}\n"),
        ));
    }
    candidates.push((
        eof,
        text.len(),
        format!("Insert {closing} at the end of the file."),
        format!("\n{closing}\n"),
    ));

    let mut out: Vec<Suggestion> = Vec::new();
    let mut seen: Vec<usize> = Vec::new();
    for (at, through, why, insertion) in candidates {
        if seen.contains(&at) {
            continue;
        }
        seen.push(at);
        let conf = if out.is_empty() {
            Confidence::Medium
        } else {
            Confidence::Low
        };
        out.push(Suggestion::new(
            why,
            vec![Edit::new(l.path, at, through, insertion)],
            conf,
        ));
    }
    out
}

fn environment_mismatch(
    loc: Option<Loc>,
    text: &str,
    found: &str,
    expected: &str,
) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            format!(
                "Change \\end{{{found}}} to \\end{{{expected}}}, or add \\end{{{expected}}} before it."
            ),
            Confidence::Low,
        )];
    };
    let mut out = Vec::new();
    if let Some((arg, arg_end)) = text::braced_argument(text, l.e) {
        let content_start = arg_end - 1 - arg.len();
        let close = text::edit_distance(found, expected) <= 2;
        out.push(Suggestion::new(
            format!("Rename this to \\end{{{expected}}}."),
            vec![Edit::new(l.path, content_start, arg_end - 1, expected)],
            if close {
                Confidence::High
            } else {
                Confidence::Medium
            },
        ));
    }
    out.push(Suggestion::new(
        format!(
            "Close \\begin{{{expected}}} first by inserting \\end{{{expected}}} before this line."
        ),
        vec![Edit::insert(
            l.path,
            text::line_start(text, l.s),
            format!("\\end{{{expected}}}\n"),
        )],
        Confidence::Low,
    ));
    out
}

fn stray_end(loc: Option<Loc>, text: &str, name: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            format!("Remove \\end{{{name}}} or add the \\begin{{{name}}} it should close."),
            Confidence::Low,
        )];
    };
    let end = text::braced_argument(text, l.e).map_or(l.e, |(_, e)| e);
    vec![
        Suggestion::new(
            format!("Remove the stray \\end{{{name}}}."),
            vec![Edit::delete(l.path, l.s, end)],
            Confidence::Medium,
        ),
        Suggestion::new(
            format!(
                "Open the environment: insert \\begin{{{name}}} at the start of this paragraph."
            ),
            vec![Edit::insert(
                l.path,
                text::line_start(text, l.s),
                format!("\\begin{{{name}}}\n"),
            )],
            Confidence::Low,
        ),
    ]
}

fn unsupported_environment(loc: Option<Loc>, text: &str, name: &str) -> Vec<Suggestion> {
    let mut out = vec![Suggestion::advice(
        format!(
            "Keep it: the body of '{name}' is typeset as plain paragraphs, and the environment will pick up its formatting when the compiler implements it."
        ),
        Confidence::Low,
    )];
    let Some(l) = loc else { return out };
    let Some((_, begin_end)) = text::braced_argument(text, l.e) else {
        return out;
    };
    // Find the matching \end{name} with a depth counter over same-named markers.
    let mut depth = 0usize;
    for m in text::environments(text, begin_end) {
        if m.name != name {
            continue;
        }
        if m.is_begin {
            depth += 1;
        } else if depth == 0 {
            out.push(Suggestion::new(
                format!("Remove the \\begin{{{name}}} / \\end{{{name}}} pair and keep its body."),
                vec![
                    Edit::delete(l.path, l.s, begin_end),
                    Edit::delete(l.path, m.start, m.end),
                ],
                Confidence::Low,
            ));
            break;
        } else {
            depth -= 1;
        }
    }
    out
}

// ---- math ---------------------------------------------------------------

fn math_mode_unavailable(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let mut out = Vec::new();
    if let Some(l) = loc {
        if let Some(close) = text::next_dollar(text, l.e) {
            let inner = text[l.e..close].trim().to_string();
            out.push(Suggestion::new(
                "Keep the formula's text as plain words by removing the '$' delimiters.",
                vec![Edit::new(l.path, l.s, close + 1, inner)],
                Confidence::Low,
            ));
            out.push(Suggestion::new(
                "Remove the formula entirely.",
                vec![Edit::delete(l.path, l.s, close + 1)],
                Confidence::Low,
            ));
        } else {
            out.push(Suggestion::new(
                "Remove the unpaired '$'.",
                vec![Edit::delete(l.path, l.s, l.e)],
                Confidence::Low,
            ));
        }
    }
    out.push(Suggestion::advice(
        "Math typesetting is outstanding in this compiler build; the formula will render once a build with math support is installed.",
        Confidence::Low,
    ));
    out
}

fn duplicate_script(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let advice = Suggestion::advice(
        "Group the scripts: write {x^a}^b to stack them or x^{ab} for one script.",
        Confidence::Low,
    );
    let Some(l) = loc else { return vec![advice] };
    let marker = &text[l.s..l.e];
    // Walk backwards: <atom> <marker> <script> | <marker at l.s>
    let mut j = back_over_ws(text, l.s);
    j = back_over_operand(text, j);
    j = back_over_ws(text, j);
    if !text[..j].ends_with(marker) {
        return vec![advice];
    }
    j -= marker.len();
    j = back_over_ws(text, j);
    let atom_start = back_over_operand(text, j);
    if atom_start >= l.s {
        return vec![advice];
    }
    vec![
        Suggestion::new(
            format!(
                "Stack the scripts by grouping the first: {{{}}}{}...",
                text[atom_start..l.s].trim(),
                marker
            ),
            vec![
                Edit::insert(l.path, atom_start, "{"),
                Edit::insert(l.path, l.s, "}"),
            ],
            Confidence::Low,
        ),
        advice,
    ]
}

/// Byte offset before any spaces/tabs that end `text[..j]`.
fn back_over_ws(text: &str, mut j: usize) -> usize {
    while j > 0 && matches!(text.as_bytes()[j - 1], b' ' | b'\t') {
        j -= 1;
    }
    j
}

/// Start of the operand that ends at `j`: a balanced `{...}` group, a control
/// word, or a single character.
fn back_over_operand(text: &str, j: usize) -> usize {
    if j == 0 {
        return 0;
    }
    if text.as_bytes()[j - 1] == b'}' {
        let mut depth = 0usize;
        let mut k = j;
        while k > 0 {
            k -= 1;
            match text.as_bytes()[k] {
                b'}' => depth += 1,
                b'{' => {
                    depth -= 1;
                    if depth == 0 {
                        return k;
                    }
                }
                _ => {}
            }
        }
        return 0;
    }
    let prev = text[..j].chars().next_back().unwrap();
    let mut start = j - prev.len_utf8();
    if prev.is_alphabetic() {
        while start > 0 {
            let c = text[..start].chars().next_back().unwrap();
            if c.is_alphabetic() {
                start -= c.len_utf8();
            } else if c == '\\' {
                return start - 1;
            } else {
                break;
            }
        }
        // A bare letter run in math is one atom per letter; only a control
        // word counts as one operand.
        return j - prev.len_utf8();
    }
    start = text::snap_down(text, start);
    start
}

fn script_without_atom(loc: Option<Loc>) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Put the base of the script before '^' or '_', or remove the marker.",
            Confidence::Low,
        )];
    };
    vec![
        Suggestion::new(
            "Remove the marker.",
            vec![Edit::delete(l.path, l.s, l.e)],
            Confidence::Medium,
        ),
        Suggestion::new(
            "Attach the script to an empty base: insert {} before it.",
            vec![Edit::insert(l.path, l.s, "{}")],
            Confidence::Low,
        ),
    ]
}

fn insert_at_end(loc: Option<Loc>, what: &str, why: &str, conf: Confidence) -> Vec<Suggestion> {
    match loc {
        Some(l) => vec![Suggestion::new(
            why,
            vec![Edit::insert(l.path, l.e, what)],
            conf,
        )],
        None => vec![Suggestion::advice(
            format!("{why} (insert '{what}')"),
            Confidence::Low,
        )],
    }
}

fn script_missing_argument(loc: Option<Loc>) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Give the script content, e.g. x^{2}, or remove the marker.",
            Confidence::Low,
        )];
    };
    vec![
        Suggestion::new(
            "Add an empty group to fill in: ^{}.",
            vec![Edit::insert(l.path, l.e, "{}")],
            Confidence::Medium,
        ),
        Suggestion::new(
            "Remove the marker.",
            vec![Edit::delete(l.path, l.s, l.e)],
            Confidence::Low,
        ),
    ]
}

fn nested_math_delimiter(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Write a literal dollar sign as '\\$', or end the formula before opening another.",
            Confidence::Low,
        )];
    };
    let mut out = Vec::new();
    if &text[l.s..l.e] == "$" {
        out.push(Suggestion::new(
            "Make it a literal dollar sign: '\\$'.",
            vec![Edit::new(l.path, l.s, l.e, "\\$")],
            Confidence::Medium,
        ));
    }
    out.push(Suggestion::new(
        "Remove the delimiter.",
        vec![Edit::delete(l.path, l.s, l.e)],
        Confidence::Low,
    ));
    out
}

fn stray_display_close(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Remove the '\\]' or add the '\\[' that opens it.",
            Confidence::Low,
        )];
    };
    vec![
        Suggestion::new(
            "Remove the stray '\\]'.",
            vec![Edit::delete(l.path, l.s, l.e)],
            Confidence::Medium,
        ),
        Suggestion::new(
            "Open display math at the start of this paragraph.",
            vec![Edit::insert(l.path, text::line_start(text, l.s), "\\[\n")],
            Confidence::Low,
        ),
    ]
}

fn script_outside_math(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Put the expression between $...$, or escape the character as '\\_' or '\\^'.",
            Confidence::Low,
        )];
    };
    let marker = &text[l.s..l.e];
    let mut out = Vec::new();
    // Expression = word before the marker + marker + one operand after it.
    let mut w_start = l.s;
    while w_start > 0 {
        let c = text[..w_start].chars().next_back().unwrap();
        if c.is_alphanumeric() {
            w_start -= c.len_utf8();
        } else {
            break;
        }
    }
    let w_end = match text.as_bytes().get(l.e) {
        Some(b'{') => text::matching_close_brace(text, l.e),
        Some(_) if !text[l.e..].starts_with(char::is_whitespace) => {
            text[l.e..].chars().next().map(|c| l.e + c.len_utf8())
        }
        _ => None,
    };
    if let Some(w_end) = w_end
        && w_start < l.s
    {
        out.push(Suggestion::new(
            format!("Typeset it as math: ${}$.", &text[w_start..w_end]),
            vec![
                Edit::insert(l.path, w_start, "$"),
                Edit::insert(l.path, w_end, "$"),
            ],
            Confidence::Medium,
        ));
    }
    out.push(Suggestion::new(
        format!("Keep a literal '{marker}' in text by escaping it as '\\{marker}'."),
        vec![Edit::new(l.path, l.s, l.e, format!("\\{marker}"))],
        if marker == "_" {
            Confidence::Medium
        } else {
            Confidence::Low
        },
    ));
    out
}

fn math_unclosed(loc: Option<Loc>, text: &str, closer: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            format!("Add the closing '{closer}' where the formula ends."),
            Confidence::Low,
        )];
    };
    let eol = text::trim_end_whitespace(text, l.e, text::line_end(text, l.e));
    let para = text::paragraph_end(text, l.e);
    let mut out = Vec::new();
    if closer == "$" && eol > l.e {
        out.push(Suggestion::new(
            "Close the formula at the end of this line.",
            vec![Edit::insert(l.path, eol, closer)],
            Confidence::Medium,
        ));
    }
    if para > l.e && para != eol || out.is_empty() {
        let (at, sep) = if closer == "$" {
            (para, "")
        } else {
            (para, "\n")
        };
        out.push(Suggestion::new(
            "Close the formula before the paragraph break.",
            vec![Edit::insert(l.path, at, format!("{sep}{closer}"))],
            if out.is_empty() {
                Confidence::Medium
            } else {
                Confidence::Low
            },
        ));
    }
    if closer == "$" {
        out.push(Suggestion::new(
            "Make the opening '$' a literal dollar sign: '\\$'.",
            vec![Edit::new(l.path, l.s, l.e, "\\$")],
            Confidence::Low,
        ));
    }
    out
}

// ---- commands and arguments -------------------------------------------------

fn unsupported_command(
    loc: Option<Loc>,
    text: &str,
    name: &str,
    supported: &[&str],
    math: bool,
) -> Vec<Suggestion> {
    let mut out = Vec::new();
    let candidates = text::closest_commands(name, supported, 3);
    for (cand, d) in &candidates {
        let conf = match d {
            0 | 1 => Confidence::High,
            2 => Confidence::Medium,
            _ => Confidence::Low,
        };
        let edits = loc
            .as_ref()
            .map(|l| vec![Edit::new(l.path, l.s, l.e, format!("\\{cand}"))])
            .unwrap_or_default();
        out.push(Suggestion::new(
            format!("Did you mean \\{cand}?"),
            edits,
            conf,
        ));
    }
    if let Some(l) = &loc {
        match text::braced_argument(text, l.e) {
            Some((arg, arg_end)) if !math => out.push(Suggestion::new(
                format!("Remove \\{name} and keep its argument text."),
                vec![Edit::new(l.path, l.s, arg_end, arg.to_string())],
                Confidence::Low,
            )),
            _ => out.push(Suggestion::new(
                format!("Remove \\{name}."),
                vec![Edit::delete(l.path, l.s, l.e)],
                Confidence::Low,
            )),
        }
    }
    if candidates.is_empty() {
        out.push(Suggestion::advice(
            if math {
                "Supported in math: \\frac, \\sqrt, ^ and _ scripts, Greek letters and the operator table (\\times, \\leq, \\sum, ...). Anything else is typeset as its literal name.".to_string()
            } else {
                "No similar supported command; this compiler version has no packages or macro expansion, so the command's effect is unavailable.".to_string()
            },
            Confidence::Low,
        ));
    }
    out
}

fn missing_braced_argument(loc: Option<Loc>, text: &str, command: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            format!("Write the argument in braces: \\{command}{{...}}."),
            Confidence::Low,
        )];
    };
    let eol = text::line_end(text, l.e);
    let content_start = l.e + text[l.e..eol].len() - text[l.e..eol].trim_start().len();
    let heading = matches!(command, "section" | "subsection");
    let content_end = if heading {
        text::trim_end_whitespace(text, content_start, eol)
    } else {
        content_start
            + text[content_start..eol]
                .chars()
                .take_while(|c| !c.is_whitespace() && !matches!(c, '\\' | '{' | '}' | '$' | '%'))
                .map(char::len_utf8)
                .sum::<usize>()
    };
    let mut out = Vec::new();
    if content_end > content_start {
        out.push(Suggestion::new(
            format!(
                "Wrap the following {} in braces: \\{command}{{{}}}.",
                if heading { "text" } else { "word" },
                &text[content_start..content_end]
            ),
            vec![
                Edit::new(l.path, l.e, content_start, "{"),
                Edit::insert(l.path, content_end, "}"),
            ],
            Confidence::Medium,
        ));
    }
    out.push(Suggestion::new(
        format!("Insert an empty argument to fill in: \\{command}{{}}."),
        vec![Edit::insert(l.path, l.e, "{}")],
        Confidence::Low,
    ));
    out
}

fn missing_math_argument(loc: Option<Loc>, text: &str, command: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            format!("Give \\{command} its operand(s) in braces, e.g. \\{command}{{...}}."),
            Confidence::Low,
        )];
    };
    let mut out = Vec::new();
    let rest = &text[l.e..];
    let skipped = rest.len() - rest.trim_start_matches([' ', '\t']).len();
    let at = l.e + skipped;
    if let Some(c) = text[at..].chars().next()
        && !c.is_whitespace()
        && !matches!(c, '{' | '}' | '$' | '\\' | '%')
    {
        out.push(Suggestion::new(
            format!("Brace the next character: \\{command}{{{c}}}."),
            vec![
                Edit::insert(l.path, at, "{"),
                Edit::insert(l.path, at + c.len_utf8(), "}"),
            ],
            Confidence::Medium,
        ));
    }
    out.push(Suggestion::new(
        format!("Insert an empty argument to fill in: \\{command}{{}}."),
        vec![Edit::insert(l.path, l.e, "{}")],
        Confidence::Low,
    ));
    out
}

fn delete_span(loc: Option<Loc>, why: &str, conf: Confidence) -> Vec<Suggestion> {
    match loc {
        Some(l) => vec![Suggestion::new(
            why,
            vec![Edit::delete(l.path, l.s, l.e)],
            conf,
        )],
        None => vec![Suggestion::advice(why, Confidence::Low)],
    }
}

// ---- protocol / paths ---------------------------------------------------------

fn path_rejected(path: &str) -> Vec<Suggestion> {
    let cleaned: Vec<&str> = path
        .split(['/', '\\'])
        .filter(|seg| !seg.is_empty() && *seg != "." && *seg != "..")
        .collect();
    let mut out = Vec::new();
    if !cleaned.is_empty() {
        out.push(Suggestion::advice(
            format!(
                "Keep the file inside the project and refer to it by a project-relative path such as '{}'.",
                cleaned.join("/")
            ),
            Confidence::Medium,
        ));
    }
    out.push(Suggestion::advice(
        "Absolute paths and '..' segments are refused for every document, so fix the path in the app's project settings rather than in the document.",
        Confidence::Low,
    ));
    out
}

// ---- preamble and macros ----------------------------------------------------------

fn empty_document_class(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Name a class, e.g. \\documentclass{article}.",
            Confidence::Low,
        )];
    };
    let mut out = Vec::new();
    // The span is the command; its argument follows (possibly after [options]).
    let mut at = l.e;
    if text.as_bytes().get(at) == Some(&b'[')
        && let Some(close) = text[at..].find(']')
    {
        at += close + 1;
    }
    if let Some((arg, arg_end)) = text::braced_argument(text, at) {
        let inner_start = arg_end - 1 - arg.len();
        out.push(Suggestion::new(
            "Use the article class.",
            vec![Edit::new(l.path, inner_start, arg_end - 1, "article")],
            Confidence::Medium,
        ));
    }
    out.push(Suggestion::advice(
        "Name a class, e.g. \\documentclass{article}; this compiler records the name but applies no class-specific layout yet.",
        Confidence::Low,
    ));
    out
}

fn packages_not_implemented(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let mut out = vec![Suggestion::advice(
        "Keep it: the rest of the document compiles; package commands stay unsupported until the compiler implements them.",
        Confidence::Low,
    )];
    if let Some(l) = loc {
        let mut end = l.e;
        if text.as_bytes().get(end) == Some(&b'\n') {
            end += 1;
        }
        out.push(Suggestion::new(
            "Remove the \\usepackage line if the document does not rely on it.",
            vec![Edit::delete(
                l.path,
                text::line_start(text, l.s).min(l.s),
                end,
            )],
            Confidence::Low,
        ));
    }
    out
}

fn macro_argcount_invalid(loc: Option<Loc>) -> Vec<Suggestion> {
    let mut out = vec![Suggestion::advice(
        "Declare the argument count as a single digit, e.g. [1] for one argument.",
        Confidence::Medium,
    )];
    if let Some(l) = loc {
        out.push(Suggestion::new(
            "Drop the option so the macro takes no arguments.",
            vec![Edit::delete(l.path, l.s, l.e)],
            Confidence::Low,
        ));
    }
    out
}

fn rename_definer(loc: Option<Loc>, text: &str, from: &str, to: &str) -> Vec<Suggestion> {
    let why = format!("Use {to} instead of {from}.");
    let Some(l) = loc else {
        return vec![Suggestion::advice(why, Confidence::Medium)];
    };
    if text[l.s..].starts_with(from) {
        vec![Suggestion::new(
            why,
            vec![Edit::new(l.path, l.s, l.s + from.len(), to)],
            Confidence::High,
        )]
    } else {
        vec![Suggestion::advice(why, Confidence::Medium)]
    }
}

fn macro_recursion(loc: Option<Loc>, name: &str) -> Vec<Suggestion> {
    let mut out = vec![Suggestion::advice(
        format!("Change the definition of \\{name} so its body does not expand to \\{name} again."),
        Confidence::Medium,
    )];
    if let Some(l) = loc {
        out.push(Suggestion::new(
            format!("Remove this use of \\{name}."),
            vec![Edit::delete(l.path, l.s, l.e)],
            Confidence::Low,
        ));
    }
    out
}

fn macro_argument_undeclared(loc: Option<Loc>, text: &str, number: &str) -> Vec<Suggestion> {
    let advice = Suggestion::advice(
        format!(
            "Declare at least {number} argument(s) in the definition: \\newcommand{{\\name}}[{number}]{{...}}."
        ),
        Confidence::Medium,
    );
    let Some(l) = loc else { return vec![advice] };
    let Some((name, _)) = text::command_at(text, l.s) else {
        return vec![advice];
    };
    let needle_a = format!("\\newcommand{{\\{name}}}");
    let needle_b = format!("\\renewcommand{{\\{name}}}");
    let def_end = text
        .find(&needle_a)
        .map(|i| i + needle_a.len())
        .or_else(|| text.find(&needle_b).map(|i| i + needle_b.len()));
    let Some(def_end) = def_end else {
        return vec![advice];
    };
    let edit = if text.as_bytes().get(def_end) == Some(&b'[') {
        match text[def_end..].find(']') {
            Some(close) => Edit::new(l.path, def_end, def_end + close + 1, format!("[{number}]")),
            None => return vec![advice],
        }
    } else {
        Edit::insert(l.path, def_end, format!("[{number}]"))
    };
    vec![
        Suggestion::new(
            format!("Declare {number} argument(s) on \\{name}'s definition."),
            vec![edit],
            Confidence::Medium,
        ),
        advice,
    ]
}

fn optional_argument_unclosed(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Add the ']' that closes the option.",
            Confidence::Low,
        )];
    };
    let eol = text::trim_end_whitespace(text, l.s + 1, text::line_end(text, l.s));
    vec![Suggestion::new(
        "Close the option at the end of this line.",
        vec![Edit::insert(
            l.path,
            text::snap_up(text, eol.max(l.s + 1)),
            "]",
        )],
        Confidence::Medium,
    )]
}

fn preamble_unsupported(loc: Option<Loc>, text: &str) -> Vec<Suggestion> {
    let Some(l) = loc else {
        return vec![Suggestion::advice(
            "Move the command after \\begin{document}.",
            Confidence::Medium,
        )];
    };
    let cmd_end = text::braced_argument(text, l.e).map_or(l.e, |(_, e)| e);
    let snippet = text[l.s..cmd_end].to_string();
    let mut out = Vec::new();
    if let Some(begin) = text::environments(text, cmd_end)
        .into_iter()
        .find(|m| m.is_begin && m.name == "document")
    {
        let mut delete_end = cmd_end;
        if text.as_bytes().get(delete_end) == Some(&b'\n') {
            delete_end += 1;
        }
        out.push(Suggestion::new(
            "Move it into the document body, right after \\begin{document}.",
            vec![
                Edit::delete(l.path, text::line_start(text, l.s).min(l.s), delete_end),
                Edit::insert(l.path, begin.end, format!("\n{snippet}")),
            ],
            Confidence::Medium,
        ));
    } else {
        out.push(Suggestion::advice(
            "Add \\begin{document} ... \\end{document} and put body commands inside it.",
            Confidence::Medium,
        ));
    }
    out.push(Suggestion::new(
        "Remove it from the preamble.",
        vec![Edit::delete(l.path, l.s, cmd_end)],
        Confidence::Low,
    ));
    out
}
