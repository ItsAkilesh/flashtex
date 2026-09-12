//! The explanation catalog: one entry per compiler message pattern.
//!
//! Every template below is the compiler's exact message text with `*` where a
//! value is interpolated (see [`crate::pattern`]). Provenance for each pattern
//! is recorded in `sources` as `file:line@sha`, read from:
//!
//! - `MAIN` = `origin/main` 1dd26c5e0dd5e04a39f0b8e55c90abebf635c863
//!   (`crates/compiler/src/parser.rs`, `protocol.rs`)
//! - `FOUNDATION` = `origin/agent/claude/compiler-foundation` de1020c
//!   (`math.rs`, `parser.rs`) and its tip 6b13034 (preamble and macro messages).
//!
//! Wording, categories and suggestions are original to this crate. When the
//! compiler changes a message, `tests/catalog_strings.rs` fails and the entry
//! must be re-pinned; that is the intended coupling.

use crate::Category;

/// `origin/main` revision the `MAIN` patterns were read from.
pub const MAIN_SHA: &str = "1dd26c5e0dd5e04a39f0b8e55c90abebf635c863";
/// `origin/agent/claude/compiler-foundation` revision the math/display-math
/// patterns were read from.
pub const FOUNDATION_SHA: &str = "de1020c";
/// Tip of the same branch at cataloguing time (preamble and macro messages).
pub const FOUNDATION_TIP_SHA: &str = "6b13034593e04404a0cfde0f296c14360f5c4b26";

/// Which suggestion builder handles an entry (see [`crate::suggest`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    UnclosedGroup,
    StrayCloseBrace,
    UnterminatedEnvironment,
    EnvironmentMismatch,
    StrayEnd,
    UnsupportedEnvironment,
    MathModeUnavailable,
    UnsupportedCommand,
    MissingBracedArgument,
    ArgumentUnclosed,
    EmptyArgument,
    PathRejected,
    NoDocuments,
    MultiDocument,
    ProtocolError,
    // compiler-foundation math (math.rs)
    MathStrayCloseBrace,
    DuplicateScript,
    ScriptWithoutAtom,
    MathGroupUnclosed,
    ScriptMissingArgument,
    NestedMathDelimiter,
    UnsupportedMathCommand,
    MissingMathArgument,
    // compiler-foundation parser
    StrayDisplayClose,
    ScriptOutsideMath,
    DisplayMathUnclosed,
    InlineMathUnclosed,
    // compiler-foundation tip (preamble / macros)
    EmptyDocumentClass,
    EmptyPackageList,
    PackagesNotImplemented,
    MacroNameInvalid,
    MacroArgCountInvalid,
    NewcommandExists,
    RenewcommandUndefined,
    MacroRecursion,
    MacroArgumentUndeclared,
    OptionalArgumentUnclosed,
    PreambleUnsupported,
}

/// One catalog entry. `templates` are tried in order; the first that matches
/// the whole message wins.
pub struct Entry {
    pub id: &'static str,
    pub kind: Kind,
    pub category: Category,
    pub templates: &'static [&'static str],
    /// `file:line@sha` for each template, same order.
    pub sources: &'static [&'static str],
    pub title: fn(&[String]) -> String,
    pub why: fn(&[String]) -> String,
}

fn cap(captures: &[String], i: usize) -> &str {
    captures.get(i).map(String::as_str).unwrap_or("")
}

macro_rules! entry {
    ($id:literal, $kind:ident, $cat:ident, [$($tpl:literal @ $src:literal),+ $(,)?],
     title: $title:expr, why: $why:expr) => {
        Entry {
            id: $id,
            kind: Kind::$kind,
            category: Category::$cat,
            templates: &[$($tpl),+],
            sources: &[$($src),+],
            title: $title,
            why: $why,
        }
    };
}

/// The catalog. Where one template is a special case of another (an exact
/// `\documentclass was given an empty argument` versus the parametrised
/// `\* was given an empty argument`), the specific entry is listed first.
pub static ENTRIES: &[Entry] = &[
    // ---- origin/main parser.rs -------------------------------------------
    entry!("unclosed-group", UnclosedGroup, UnmatchedBrace,
        ["unmatched '{' — group never closed" @ "crates/compiler/src/parser.rs:57@1dd26c5"],
        title: |_| "Opening brace never closed".into(),
        why: |_| "A '{' opens a group that must be closed by a matching '}' before the end of the document. The compiler reached the end of the input with this group still open.".into()),
    entry!("stray-close-brace", StrayCloseBrace, UnmatchedBrace,
        ["unmatched '}' — no group is open here" @ "crates/compiler/src/parser.rs:124@1dd26c5"],
        title: |_| "Closing brace with no open group".into(),
        why: |_| "This '}' closes a group, but every group opened before it was already closed. It is usually a leftover from an earlier edit, or a literal brace that needs escaping as '\\}'.".into()),
    entry!("unterminated-environment", UnterminatedEnvironment, EnvironmentMismatch,
        ["unterminated environment '*' — no matching \\end" @ "crates/compiler/src/parser.rs:64@1dd26c5"],
        title: |c| format!("\\begin{{{}}} is never closed", cap(c, 0)),
        why: |c| format!("Every \\begin{{{0}}} needs a matching \\end{{{0}}}. The compiler reached the end of the document while this environment was still open.", cap(c, 0))),
    entry!("environment-mismatch", EnvironmentMismatch, EnvironmentMismatch,
        ["\\end{*} does not match \\begin{*}" @ "crates/compiler/src/parser.rs:190@1dd26c5"],
        title: |c| format!("\\end{{{}}} closes \\begin{{{}}}", cap(c, 0), cap(c, 1)),
        why: |c| format!("Environments nest: the innermost open environment is \\begin{{{1}}}, so the next \\end must be \\end{{{1}}}, not \\end{{{0}}}. Either the name is misspelled or an \\end{{{1}}} is missing before it.", cap(c, 0), cap(c, 1))),
    entry!("stray-end", StrayEnd, EnvironmentMismatch,
        ["\\end{*} with no matching \\begin" @ "crates/compiler/src/parser.rs:197@1dd26c5"],
        title: |c| format!("\\end{{{}}} without \\begin", cap(c, 0)),
        why: |c| format!("No environment is open at this point, so \\end{{{0}}} has nothing to close. Either its \\begin{{{0}}} was removed, or this \\end is duplicated.", cap(c, 0))),
    entry!("unsupported-environment", UnsupportedEnvironment, UnsupportedEnvironment,
        ["environment '*' is not implemented; its body is typeset as plain text" @ "crates/compiler/src/parser.rs:179@1dd26c5"],
        title: |c| format!("Environment '{}' has no formatting yet", cap(c, 0)),
        why: |c| format!("This compiler version implements the 'document' environment only. \\begin{{{0}}} is recognised and its nesting is checked, but no {0}-specific layout exists yet, so its body is typeset as ordinary paragraphs.", cap(c, 0))),
    entry!("math-mode-unavailable", MathModeUnavailable, MathUnsupported,
        ["math mode is not implemented in this version" @ "crates/compiler/src/parser.rs:133@1dd26c5"],
        title: |_| "Math mode is not available".into(),
        why: |_| "This compiler version has no math typesetting. Each '$' is reported and skipped, so the formula between them is not rendered; the words inside may appear as plain text.".into()),
    entry!("unsupported-command", UnsupportedCommand, UnsupportedCommand,
        [
            "\\* is not supported by this compiler version; unrestricted TeX math mode is not implemented" @ "crates/compiler/src/parser.rs:235@de1020c",
            "\\* is not supported by this compiler version" @ "crates/compiler/src/parser.rs:212@1dd26c5",
        ],
        title: |c| format!("\\{} is not supported", cap(c, 0)),
        why: |c| format!("The compiler implements a fixed, documented set of commands and has no macro packages, so \\{} has no definition here. It may be a misspelling of a supported command, a package command, or a command this version does not implement yet.", cap(c, 0))),
    entry!("missing-braced-argument", MissingBracedArgument, MissingArgument,
        ["\\* requires a braced argument" @ "crates/compiler/src/parser.rs:234@1dd26c5"],
        title: |c| format!("\\{} needs a {{...}} argument", cap(c, 0)),
        why: |c| format!("\\{} takes its content in braces, and the next thing after it is not a '{{'. Text that should belong to the command was treated as ordinary paragraph text.", cap(c, 0))),
    entry!("argument-unclosed", ArgumentUnclosed, UnmatchedBrace,
        ["argument to \\* is missing its closing brace" @ "crates/compiler/src/parser.rs:280@1dd26c5"],
        title: |c| format!("\\{}'s argument is never closed", cap(c, 0)),
        why: |c| format!("The '{{' that opens the argument to \\{} has no matching '}}' before the next blank line or the end of the document. Arguments cannot span a paragraph break.", cap(c, 0))),
    entry!("empty-documentclass", EmptyDocumentClass, EmptyArgument,
        ["\\documentclass was given an empty argument" @ "crates/compiler/src/parser.rs:310@6b13034"],
        title: |_| "\\documentclass{} names no class".into(),
        why: |_| "The braces after \\documentclass are empty, so no document class was recorded.".into()),
    entry!("empty-package-list", EmptyPackageList, EmptyArgument,
        ["\\usepackage was given an empty package list" @ "crates/compiler/src/parser.rs:330@6b13034"],
        title: |_| "\\usepackage{} names no package".into(),
        why: |_| "The braces after \\usepackage contain no package names, so the command did nothing.".into()),
    entry!("empty-argument", EmptyArgument, EmptyArgument,
        ["\\* was given an empty argument" @ "crates/compiler/src/parser.rs:287@1dd26c5"],
        title: |c| format!("\\{}{{}} is empty", cap(c, 0)),
        why: |c| format!("The braces after \\{} contain no text, so there is nothing to typeset. This is usually an unfinished edit.", cap(c, 0))),
    // ---- origin/main protocol.rs -----------------------------------------
    entry!("path-rejected", PathRejected, PathRejected,
        [
            "rejected document path '*': paths must be project-relative with no parent traversal" @ "crates/compiler/src/protocol.rs:244@1dd26c5",
            "rejected entry_path '*': paths must be project-relative with no parent traversal" @ "crates/compiler/src/protocol.rs:255@1dd26c5",
        ],
        title: |c| format!("Path '{}' was rejected", cap(c, 0)),
        why: |c| format!("The compiler only reads documents the app hands it, addressed by paths relative to the project root. '{}' is absolute or steps outside the project with '..', so the whole compile was refused before anything was typeset.", cap(c, 0))),
    entry!("no-documents", NoDocuments, Protocol,
        ["no documents supplied to compile" @ "crates/compiler/src/protocol.rs:285@1dd26c5"],
        title: |_| "Nothing to compile".into(),
        why: |_| "The compile request carried an empty documents list, so there was no entry document to typeset.".into()),
    entry!("multi-document", MultiDocument, MultiFile,
        ["* documents were supplied; this version compiles only the entry document" @ "crates/compiler/src/protocol.rs:300@1dd26c5"],
        title: |c| format!("Only the entry document was compiled ({} supplied)", cap(c, 0)),
        why: |_| "Multi-file projects (\\input, \\include) are not implemented in this compiler version. The extra documents were accepted but not read; only the entry document contributed to the output.".into()),
    entry!("protocol-error", ProtocolError, Protocol,
        [
            "line exceeds the *-byte limit" @ "crates/compiler/src/protocol.rs:100@1dd26c5",
            "protocol version * is not supported; this build speaks version *" @ "crates/compiler/src/protocol.rs:124@1dd26c5",
            "protocol_version is required" @ "crates/compiler/src/protocol.rs:133@1dd26c5",
            "compile requires a payload" @ "crates/compiler/src/protocol.rs:144@1dd26c5",
            "message type '*' is not supported" @ "crates/compiler/src/protocol.rs:150@1dd26c5",
            "type is required" @ "crates/compiler/src/protocol.rs:152@1dd26c5",
        ],
        title: |_| "Compile request was rejected".into(),
        why: |_| "The message the app sent to the compiler did not satisfy runtime protocol v1 (one JSON object per line, protocol_version 1, a type and a payload, within the size limit). This is an app/transport problem, not a problem in the document.".into()),
    // ---- compiler-foundation math.rs (de1020c) ----------------------------
    entry!("math-stray-close-brace", MathStrayCloseBrace, UnmatchedBrace,
        ["unmatched '}' in math mode" @ "crates/compiler/src/math.rs:90@de1020c"],
        title: |_| "Closing brace inside math with no open group".into(),
        why: |_| "Inside a formula, '}' must close a group opened by '{' in the same formula. This one has no open group; it is usually a leftover or a brace meant to be '\\}'.".into()),
    entry!("duplicate-script", DuplicateScript, MathSyntax,
        ["duplicate script on a math atom" @ "crates/compiler/src/math.rs:110@de1020c"],
        title: |_| "Double superscript or subscript".into(),
        why: |_| "An atom can carry one superscript and one subscript. Writing x^a^b or x_a_b attaches two of the same kind. Group them: {x^a}^b stacks the scripts, x^{ab} makes one script.".into()),
    entry!("script-without-atom", ScriptWithoutAtom, MathSyntax,
        ["script marker has no preceding math atom" @ "crates/compiler/src/math.rs:117@de1020c"],
        title: |_| "Script marker with nothing to attach to".into(),
        why: |_| "'^' and '_' attach a script to the atom immediately before them. Here the marker is the first thing in its group, so there is no atom to attach to.".into()),
    entry!("math-group-unclosed", MathGroupUnclosed, UnmatchedBrace,
        ["math group is missing its closing brace" @ "crates/compiler/src/math.rs:133@de1020c"],
        title: |_| "Brace group inside math is never closed".into(),
        why: |_| "A '{' inside the formula was not closed before the formula ended. Groups cannot extend past the closing '$' or '\\]'.".into()),
    entry!("script-missing-argument", ScriptMissingArgument, MathSyntax,
        ["math script is missing its argument" @ "crates/compiler/src/math.rs:159@de1020c"],
        title: |_| "Superscript or subscript has no content".into(),
        why: |_| "After '^' or '_' the compiler expects a single atom or a braced group. The formula ended, or a brace closed, before any script content appeared.".into()),
    entry!("nested-math-delimiter", NestedMathDelimiter, MathSyntax,
        ["unexpected math delimiter inside math mode" @ "crates/compiler/src/math.rs:188@de1020c"],
        title: |_| "Math delimiter inside a formula".into(),
        why: |_| "A '$', '\\[' or '\\]' appeared while a formula was already open. Formulas do not nest; a literal dollar sign inside math must be written '\\$'.".into()),
    entry!("unsupported-math-command", UnsupportedMathCommand, MathUnsupported,
        ["\\* is not supported in math mode" @ "crates/compiler/src/math.rs:230@de1020c"],
        title: |c| format!("\\{} is not available in math", cap(c, 0)),
        why: |c| format!("The math subset is small: \\frac, \\sqrt, scripts, and a fixed table of Greek letters and operators. \\{} is not in that table, so the compiler typeset its name literally.", cap(c, 0))),
    entry!("missing-math-argument", MissingMathArgument, MissingArgument,
        ["\\* requires a braced math argument" @ "crates/compiler/src/math.rs:255@de1020c"],
        title: |c| format!("\\{} needs a {{...}} argument", cap(c, 0)),
        why: |c| format!("\\{} takes each operand in braces (for example \\frac{{a}}{{b}}, \\sqrt{{x}}). The next thing after it is not a '{{'.", cap(c, 0))),
    // ---- compiler-foundation parser.rs (de1020c) --------------------------
    entry!("stray-display-close", StrayDisplayClose, MathSyntax,
        ["stray \\] has no matching \\[" @ "crates/compiler/src/parser.rs:147@de1020c"],
        title: |_| "\\] without \\[".into(),
        why: |_| "'\\]' ends display math, but no '\\[' opened one. Either the opening delimiter is missing or this '\\]' is duplicated.".into()),
    entry!("script-outside-math", ScriptOutsideMath, MathSyntax,
        ["math script marker used outside math mode" @ "crates/compiler/src/parser.rs:155@de1020c"],
        title: |_| "'^' or '_' outside math".into(),
        why: |_| "Superscript and subscript markers only mean something inside a formula. In text, write the expression between '$...$', or escape a literal character as '\\_' or '\\^'.".into()),
    entry!("display-math-unclosed", DisplayMathUnclosed, MathSyntax,
        ["display math is missing its closing delimiter" @ "crates/compiler/src/parser.rs:333@de1020c"],
        title: |_| "\\[ is never closed".into(),
        why: |_| "A '\\[' opened display math and no '\\]' followed before the end of the document.".into()),
    entry!("inline-math-unclosed", InlineMathUnclosed, MathSyntax,
        ["inline math is missing its closing '$'" @ "crates/compiler/src/parser.rs:335@de1020c"],
        title: |_| "'$' is never closed".into(),
        why: |_| "A '$' opened inline math and no second '$' followed before the end of the document. Odd numbers of '$' usually mean one is missing or one should be a literal '\\$'.".into()),
    // ---- compiler-foundation tip (6b13034) preamble and macros ------------
    entry!("packages-not-implemented", PackagesNotImplemented, UnsupportedCommand,
        ["packages * are recognised but not implemented" @ "crates/compiler/src/parser.rs:339@6b13034"],
        title: |c| format!("Package(s) {} are not implemented", cap(c, 0)),
        why: |_| "The compiler records \\usepackage but ships no package code, so commands and formatting those packages would define stay unavailable. The rest of the document still compiles.".into()),
    entry!("macro-name-invalid", MacroNameInvalid, MacroDefinition,
        ["\\* requires a single command name as its first argument" @ "crates/compiler/src/parser.rs:365@6b13034"],
        title: |c| format!("\\{}'s first argument must be one command name", cap(c, 0)),
        why: |c| format!("\\{} expects the form \\{}{{\\name}}[n]{{body}}; the first braces must contain exactly one control word such as \\name.", cap(c, 0), cap(c, 0))),
    entry!("macro-argcount-invalid", MacroArgCountInvalid, MacroDefinition,
        ["\\* argument count must be an integer from 0 to 9" @ "crates/compiler/src/parser.rs:382@6b13034"],
        title: |c| format!("\\{} argument count is not 0-9", cap(c, 0)),
        why: |_| "The optional [n] after the macro name declares how many arguments the macro takes; it must be a single digit from 0 to 9.".into()),
    entry!("newcommand-exists", NewcommandExists, MacroDefinition,
        ["\\newcommand cannot redefine existing command \\*" @ "crates/compiler/src/parser.rs:402@6b13034"],
        title: |c| format!("\\{} is already defined", cap(c, 0)),
        why: |c| format!("\\newcommand refuses to overwrite an existing definition; \\{} is already a built-in or an earlier macro. Use \\renewcommand to replace it deliberately, or pick a new name.", cap(c, 0))),
    entry!("renewcommand-undefined", RenewcommandUndefined, MacroDefinition,
        ["\\renewcommand cannot redefine undefined command \\*" @ "crates/compiler/src/parser.rs:415@6b13034"],
        title: |c| format!("\\{} is not defined yet", cap(c, 0)),
        why: |c| format!("\\renewcommand replaces an existing definition, and \\{} has none. Use \\newcommand for a first definition.", cap(c, 0))),
    entry!("macro-recursion", MacroRecursion, MacroDefinition,
        ["macro \\* exceeded the expansion recursion limit of *" @ "crates/compiler/src/parser.rs:436@6b13034"],
        title: |c| format!("\\{} expands forever", cap(c, 0)),
        why: |c| format!("Expanding \\{0} produced \\{0} again (directly or through other macros) more than {1} times, so the compiler stopped instead of looping.", cap(c, 0), cap(c, 1))),
    entry!("macro-argument-undeclared", MacroArgumentUndeclared, MacroDefinition,
        ["macro replacement references #* but that argument is not declared" @ "crates/compiler/src/parser.rs:492@6b13034"],
        title: |c| format!("Macro body uses #{} without declaring it", cap(c, 0)),
        why: |c| format!("The macro body refers to argument #{0}, but its definition declares fewer arguments. Add [{0}] (or a larger count) after the name in the \\newcommand.", cap(c, 0))),
    entry!("optional-argument-unclosed", OptionalArgumentUnclosed, UnmatchedBrace,
        ["optional argument is missing its closing ']'" @ "crates/compiler/src/parser.rs:784@6b13034"],
        title: |_| "'[' option is never closed".into(),
        why: |_| "An optional argument opened with '[' has no ']' before the end of the input.".into()),
    entry!("preamble-unsupported", PreambleUnsupported, PreambleUnsupported,
        ["\\* is not supported in the document preamble" @ "crates/compiler/src/parser.rs:891@6b13034"],
        title: |c| format!("\\{} cannot appear before \\begin{{document}}", cap(c, 0)),
        why: |c| format!("Only \\documentclass, \\usepackage and macro definitions are handled in the preamble. \\{} is a body command, so it was skipped and nothing was typeset for it.", cap(c, 0))),
];

/// Find the entry whose template matches `message` exactly, with captures.
pub fn lookup(message: &str) -> Option<(&'static Entry, Vec<String>)> {
    for entry in ENTRIES {
        for template in entry.templates {
            if let Some(captures) = crate::pattern::match_template(template, message) {
                return Some((entry, captures));
            }
        }
    }
    None
}

/// Number of distinct message templates in the catalog.
pub fn template_count() -> usize {
    ENTRIES.iter().map(|e| e.templates.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_sources_align() {
        let mut ids: Vec<&str> = ENTRIES.iter().map(|e| e.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ENTRIES.len(), "duplicate catalog id");
        for e in ENTRIES {
            assert_eq!(
                e.templates.len(),
                e.sources.len(),
                "{}: sources must align",
                e.id
            );
            assert!(!e.templates.is_empty());
        }
    }

    #[test]
    fn templates_do_not_shadow_each_other() {
        // A template must never match another entry's template text (with `*`
        // standing in for a value), or lookup order would silently decide.
        for (i, a) in ENTRIES.iter().enumerate() {
            for ta in a.templates {
                for (j, b) in ENTRIES.iter().enumerate() {
                    // A later, more specific entry may legitimately be a
                    // special case of an earlier one only if it is listed
                    // first; an earlier generic entry must never steal it.
                    if j <= i {
                        continue;
                    }
                    for tb in b.templates {
                        let sample = tb.replace('*', "X");
                        assert!(
                            crate::pattern::match_template(ta, &sample).is_none(),
                            "{} template {:?} also matches {} sample {:?}",
                            a.id,
                            ta,
                            b.id,
                            sample
                        );
                    }
                }
            }
        }
    }
}
