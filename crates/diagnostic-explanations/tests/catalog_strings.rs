//! Every catalog pattern must match at least one message string the compiler
//! actually emits. The strings below are copied verbatim (with representative
//! interpolations) from the compiler sources at the cited revisions:
//!
//! - MAIN: origin/main 1dd26c5e0dd5e04a39f0b8e55c90abebf635c863
//!   crates/compiler/src/parser.rs, crates/compiler/src/protocol.rs
//! - FOUNDATION: origin/agent/claude/compiler-foundation de1020c
//!   crates/compiler/src/math.rs, crates/compiler/src/parser.rs
//! - FOUNDATION TIP: 6b13034593e04404a0cfde0f296c14360f5c4b26
//!   crates/compiler/src/parser.rs (preamble and macro messages)
//!
//! If the compiler rewords a message, the corresponding assertion fails and
//! the catalog entry must be re-pinned. That coupling is deliberate.

use flashtex_diagnostic_explanations::catalog::{self, ENTRIES};

/// (expected catalog id, exact message, provenance)
const REAL_MESSAGES: &[(&str, &str, &str)] = &[
    // ---- MAIN parser.rs ------------------------------------------------------
    (
        "unclosed-group",
        "unmatched '{' — group never closed",
        "parser.rs:57@1dd26c5",
    ),
    (
        "unterminated-environment",
        "unterminated environment 'itemize' — no matching \\end",
        "parser.rs:64@1dd26c5 (format! with name)",
    ),
    (
        "stray-close-brace",
        "unmatched '}' — no group is open here",
        "parser.rs:124@1dd26c5",
    ),
    (
        "math-mode-unavailable",
        "math mode is not implemented in this version",
        "parser.rs:133@1dd26c5",
    ),
    (
        "unsupported-environment",
        "environment 'itemize' is not implemented; its body is typeset as plain text",
        "parser.rs:179@1dd26c5 (format! with env)",
    ),
    (
        "environment-mismatch",
        "\\end{enumerate} does not match \\begin{itemize}",
        "parser.rs:190@1dd26c5 (format! with env, open)",
    ),
    (
        "stray-end",
        "\\end{itemize} with no matching \\begin",
        "parser.rs:197@1dd26c5 (format! with env)",
    ),
    (
        "unsupported-command",
        "\\flashtexUnknownCommand is not supported by this compiler version",
        "parser.rs:212@1dd26c5 (format! with other); name from tests/tex-corpus unknown-command",
    ),
    (
        "missing-braced-argument",
        "\\section requires a braced argument",
        "parser.rs:234@1dd26c5 (format! with cmd)",
    ),
    (
        "argument-unclosed",
        "argument to \\textbf is missing its closing brace",
        "parser.rs:280@1dd26c5 (format! with cmd)",
    ),
    (
        "empty-argument",
        "\\emph was given an empty argument",
        "parser.rs:287@1dd26c5 (format! with cmd)",
    ),
    // ---- MAIN protocol.rs -----------------------------------------------------
    (
        "path-rejected",
        "rejected document path '../etc/passwd': paths must be project-relative with no parent traversal",
        "protocol.rs:244@1dd26c5",
    ),
    (
        "path-rejected",
        "rejected entry_path '/abs/main.tex': paths must be project-relative with no parent traversal",
        "protocol.rs:255@1dd26c5",
    ),
    (
        "no-documents",
        "no documents supplied to compile",
        "protocol.rs:285@1dd26c5",
    ),
    (
        "multi-document",
        "3 documents were supplied; this version compiles only the entry document",
        "protocol.rs:300@1dd26c5",
    ),
    (
        "protocol-error",
        "line exceeds the 8388608-byte limit",
        "protocol.rs:100@1dd26c5",
    ),
    (
        "protocol-error",
        "protocol version 2 is not supported; this build speaks version 1",
        "protocol.rs:124@1dd26c5",
    ),
    (
        "protocol-error",
        "protocol_version is required",
        "protocol.rs:133@1dd26c5",
    ),
    (
        "protocol-error",
        "compile requires a payload",
        "protocol.rs:144@1dd26c5",
    ),
    (
        "protocol-error",
        "message type 'capture_submit' is not supported",
        "protocol.rs:150@1dd26c5",
    ),
    (
        "protocol-error",
        "type is required",
        "protocol.rs:152@1dd26c5",
    ),
    // ---- FOUNDATION math.rs (de1020c) --------------------------------------
    (
        "math-stray-close-brace",
        "unmatched '}' in math mode",
        "math.rs:90@de1020c",
    ),
    (
        "duplicate-script",
        "duplicate script on a math atom",
        "math.rs:110@de1020c",
    ),
    (
        "script-without-atom",
        "script marker has no preceding math atom",
        "math.rs:117@de1020c",
    ),
    (
        "math-group-unclosed",
        "math group is missing its closing brace",
        "math.rs:133@de1020c",
    ),
    (
        "script-missing-argument",
        "math script is missing its argument",
        "math.rs:159@de1020c",
    ),
    (
        "nested-math-delimiter",
        "unexpected math delimiter inside math mode",
        "math.rs:188@de1020c",
    ),
    (
        "unsupported-math-command",
        "\\mathbb is not supported in math mode",
        "math.rs:230@de1020c (format! with name)",
    ),
    (
        "missing-math-argument",
        "\\frac requires a braced math argument",
        "math.rs:255@de1020c (format! with command)",
    ),
    // ---- FOUNDATION parser.rs (de1020c) -------------------------------------
    (
        "stray-display-close",
        "stray \\] has no matching \\[",
        "parser.rs:147@de1020c",
    ),
    (
        "script-outside-math",
        "math script marker used outside math mode",
        "parser.rs:155@de1020c",
    ),
    (
        "unsupported-command",
        "\\tikz is not supported by this compiler version; unrestricted TeX math mode is not implemented",
        "parser.rs:235@de1020c (format! with other)",
    ),
    (
        "display-math-unclosed",
        "display math is missing its closing delimiter",
        "parser.rs:333@de1020c",
    ),
    (
        "inline-math-unclosed",
        "inline math is missing its closing '$'",
        "parser.rs:335@de1020c",
    ),
    // ---- FOUNDATION TIP parser.rs (6b13034) ---------------------------------
    (
        "empty-documentclass",
        "\\documentclass was given an empty argument",
        "parser.rs:310@6b13034",
    ),
    (
        "empty-package-list",
        "\\usepackage was given an empty package list",
        "parser.rs:330@6b13034",
    ),
    (
        "packages-not-implemented",
        "packages amsmath, graphicx are recognised but not implemented",
        "parser.rs:339@6b13034 (format! with packages.join(\", \"))",
    ),
    (
        "macro-name-invalid",
        "\\newcommand requires a single command name as its first argument",
        "parser.rs:365@6b13034 (format! with kind)",
    ),
    (
        "macro-argcount-invalid",
        "\\renewcommand argument count must be an integer from 0 to 9",
        "parser.rs:382@6b13034 (format! with kind)",
    ),
    (
        "newcommand-exists",
        "\\newcommand cannot redefine existing command \\same",
        "parser.rs:402@6b13034; literal asserted in parser test at 6b13034:1073",
    ),
    (
        "renewcommand-undefined",
        "\\renewcommand cannot redefine undefined command \\missing",
        "parser.rs:415@6b13034; literal asserted in parser test at 6b13034:1076",
    ),
    (
        "macro-recursion",
        "macro \\loop exceeded the expansion recursion limit of 64",
        "parser.rs:436@6b13034 (format! with name, MACRO_RECURSION_LIMIT)",
    ),
    (
        "macro-argument-undeclared",
        "macro replacement references #2 but that argument is not declared",
        "parser.rs:492@6b13034 (format! with argument_index + 1)",
    ),
    (
        "optional-argument-unclosed",
        "optional argument is missing its closing ']'",
        "parser.rs:784@6b13034",
    ),
    (
        "preamble-unsupported",
        "\\section is not supported in the document preamble",
        "parser.rs:891@6b13034 (format! with name)",
    ),
];

#[test]
fn every_real_message_matches_its_catalog_entry() {
    for (id, message, provenance) in REAL_MESSAGES {
        let (entry, _) = catalog::lookup(message)
            .unwrap_or_else(|| panic!("no catalog match for {message:?} ({provenance})"));
        assert_eq!(
            entry.id, *id,
            "{message:?} ({provenance}) matched the wrong entry"
        );
    }
}

#[test]
fn every_catalog_template_is_exercised_by_a_real_message() {
    for entry in ENTRIES {
        for template in entry.templates {
            let hit = REAL_MESSAGES.iter().any(|(_, message, _)| {
                flashtex_diagnostic_explanations::pattern::match_template(template, message)
                    .is_some()
            });
            assert!(
                hit,
                "catalog template {template:?} ({}) has no real message in this test",
                entry.id
            );
        }
    }
}

#[test]
fn catalog_size_is_reported() {
    // Documented in README; bump both when adding entries.
    assert_eq!(
        ENTRIES.len(),
        38,
        "catalog entry count changed; update README"
    );
    assert_eq!(
        catalog::template_count(),
        45,
        "template count changed; update README"
    );
}

#[test]
fn captures_come_back_verbatim() {
    let (entry, caps) =
        catalog::lookup("\\end{enumerate} does not match \\begin{itemize}").unwrap();
    assert_eq!(entry.id, "environment-mismatch");
    assert_eq!(caps, vec!["enumerate".to_string(), "itemize".to_string()]);

    let (entry, caps) =
        catalog::lookup("macro \\loop exceeded the expansion recursion limit of 64").unwrap();
    assert_eq!(entry.id, "macro-recursion");
    assert_eq!(caps, vec!["loop".to_string(), "64".to_string()]);
}
