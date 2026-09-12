//! Executable version of the rev-4 corpus gap measurement documented in
//! `coordination/daniel-title.md`: every case under `tests/tex-corpus/cases`
//! declares `\documentclass{article}` and none of them uses a title block
//! (`\title`, `\author`, `\date`, or `\maketitle`). That document previously
//! backed those published figures with a grep recipe run by hand; this file
//! reimplements the same two recipes in Rust and re-derives the counts from
//! the real corpus files on every test run, so a future corpus change that
//! adds a sectioned or titled document makes this test fail instead of
//! letting the published numbers drift silently out of date:
//!
//! ```sh
//! grep -rhoE '\\documentclass(\[[^]]*\])?\{[^}]*\}' tests/tex-corpus/cases --include='*.tex'
//! grep -rlE '\\(title|author|date|maketitle)\b' tests/tex-corpus/cases --include='*.tex'
//! ```
//!
//! Reimplemented with `std` only (no `regex` dev-dependency) using a
//! maximal-munch scan for TeX control words: after a literal `\`, take the
//! longest run of ASCII alphabetic characters as the command name and
//! compare it whole. That reproduces the grep patterns' `\b` word-boundary
//! semantics without a regex engine -- `\dateformat` and `\titlepage` do not
//! match `date`/`title`, because their maximal alphabetic run is
//! `dateformat`/`titlepage`, not the shorter word.

use std::fs;
use std::path::{Path, PathBuf};

/// Same convention as `corpus_source` in `tests/consumer_integration.rs`:
/// the corpus under `tests/tex-corpus` is a committed repo fixture, not a
/// machine-local install, so a missing path panics with a clear message
/// here too rather than skipping.
fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/tex-corpus/cases")
}

/// All case directories directly under the corpus root, sorted for a
/// deterministic report.
fn case_dirs() -> Vec<PathBuf> {
    let root = corpus_root();
    assert!(
        root.is_dir(),
        "corpus root {} is missing -- this fixture is checked into the repo and \
         other tests (tests/consumer_integration.rs) already depend on it unconditionally; \
         run from within the repo, not from an unexpected working directory",
        root.display()
    );
    let entries = fs::read_dir(&root).unwrap_or_else(|e| panic!("reading {}: {e}", root.display()));
    let mut dirs: Vec<PathBuf> = entries
        .map(|entry| {
            entry.unwrap_or_else(|e| panic!("reading an entry under {}: {e}", root.display()))
        })
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// Every `.tex` file anywhere under `dir`, recursively. A case's own tree is
/// its `main.tex` plus anything alongside it that `main.tex` (or a file it
/// includes) pulls in with `\input` -- e.g.
/// `included-file/parts/section.tex` -- so a title command hidden in an
/// included fragment is still caught, not just one in the entry file.
fn tex_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries =
            fs::read_dir(&current).unwrap_or_else(|e| panic!("reading {}: {e}", current.display()));
        for entry in entries {
            let entry = entry
                .unwrap_or_else(|e| panic!("reading an entry under {}: {e}", current.display()));
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "tex") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// Every maximal-munch TeX control word in `source`: for each literal `\`,
/// the longest following run of ASCII alphabetic characters. A `\` not
/// followed by any ASCII letter (e.g. the control symbol `\\` or `\{`) is
/// skipped -- it can never equal a named command like `documentclass`.
fn control_words(source: &str) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut words = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
                end += 1;
            }
            if end > start {
                words.push(&source[start..end]);
            }
            i = end.max(i + 1);
        } else {
            i += 1;
        }
    }
    words
}

/// Whether `source` contains the control word `name` anywhere (the Rust
/// equivalent of `grep -l '\\name\b'`).
fn has_command(source: &str, name: &str) -> bool {
    control_words(source).into_iter().any(|word| word == name)
}

fn skip_ascii_whitespace(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// The class name declared by the first `\documentclass` in `source`, i.e.
/// the `{...}` group's contents after skipping an optional `[...]` options
/// group -- the same grammar as
/// `\\documentclass(\[[^]]*\])?\{[^}]*\}`. Returns `None` if the document
/// declares no `\documentclass` at all, or its argument groups are
/// malformed.
fn documentclass_value(source: &str) -> Option<&str> {
    let bytes = source.as_bytes();

    // Find where the matched `documentclass` control word ends, by
    // re-deriving control-word spans the same way `control_words` does
    // (byte offsets, not the borrowed word itself, so we can keep parsing
    // forward from that position).
    let mut i = 0;
    let mut after_command = None;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_alphabetic() {
                end += 1;
            }
            if end > start && &source[start..end] == "documentclass" {
                after_command = Some(end);
                break;
            }
            i = end.max(i + 1);
        } else {
            i += 1;
        }
    }
    let mut pos = after_command?;

    pos = skip_ascii_whitespace(bytes, pos);
    if bytes.get(pos) == Some(&b'[') {
        let close = source[pos..].find(']')? + pos;
        pos = skip_ascii_whitespace(bytes, close + 1);
    }
    if bytes.get(pos) != Some(&b'{') {
        return None;
    }
    let open = pos + 1;
    let close = source[open..].find('}')? + open;
    Some(&source[open..close])
}

#[test]
fn corpus_has_exactly_14_case_directories() {
    let cases = case_dirs();
    let names: Vec<String> = cases
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        cases.len(),
        14,
        "expected 14 corpus case directories under {}, found {}: {names:?}",
        corpus_root().display(),
        cases.len()
    );
}

#[test]
fn every_corpus_case_declares_documentclass_article_and_none_declare_another_class() {
    let cases = case_dirs();

    let mut missing_declaration = Vec::new();
    let mut article_count = 0usize;
    let mut non_article = Vec::new();

    for case_dir in &cases {
        let case_name = case_dir.file_name().unwrap().to_string_lossy().into_owned();
        let main_tex = case_dir.join("main.tex");
        assert!(
            main_tex.is_file(),
            "case {case_name} has no main.tex entry file at {}",
            main_tex.display()
        );
        let source = read(&main_tex);

        match documentclass_value(&source) {
            Some("article") => article_count += 1,
            Some(other) => non_article.push(format!("{case_name} ({other})")),
            None => missing_declaration.push(case_name),
        }
    }

    assert_eq!(
        missing_declaration.len(),
        0,
        "expected every corpus case to declare \\documentclass, but {} did not: {missing_declaration:?}",
        missing_declaration.len()
    );
    assert_eq!(
        article_count, 14,
        "expected 14 of 14 corpus cases to declare \\documentclass{{article}}, found {article_count}; \
         non-article cases: {non_article:?}",
    );
    assert_eq!(
        non_article.len(),
        0,
        "expected 0 corpus cases to declare an unsupported (non-article) document class, \
         found {}: {non_article:?}",
        non_article.len()
    );
}

#[test]
fn no_corpus_case_uses_a_title_block_command_anywhere_in_its_tree() {
    let cases = case_dirs();
    let title_commands = ["title", "author", "date", "maketitle"];

    let mut titled = Vec::new();
    for case_dir in &cases {
        let case_name = case_dir.file_name().unwrap().to_string_lossy().into_owned();
        let files = tex_files_under(case_dir);
        assert!(
            !files.is_empty(),
            "case {case_name} has no .tex files at all under {}",
            case_dir.display()
        );

        for file in &files {
            let source = read(file);
            if let Some(cmd) = title_commands
                .iter()
                .find(|name| has_command(&source, name))
            {
                titled.push(format!("{case_name} ({} uses \\{cmd})", file.display()));
                break;
            }
        }
    }

    assert_eq!(
        titled.len(),
        0,
        "expected 0 of 14 corpus cases to use a title-block command \
         (\\title/\\author/\\date/\\maketitle) anywhere in their tree, found {}: {titled:?}",
        titled.len()
    );
}

#[test]
fn control_word_scan_reproduces_grep_word_boundary_semantics() {
    // A same-crate sanity check on the scanner itself (not the corpus):
    // `\dateformat` and `\titlepage` must not be mistaken for `\date` /
    // `\title`, exactly like the grep recipes' `\b` anchors.
    assert!(!has_command(r"\dateformat{x}", "date"));
    assert!(!has_command(r"\titlepage", "title"));
    assert!(has_command(r"\date{2026}", "date"));
    assert!(has_command(r"\title{T}", "title"));
    assert_eq!(
        documentclass_value(r"\documentclass{article}"),
        Some("article")
    );
    assert_eq!(
        documentclass_value(r"\documentclass[12pt,letterpaper]{article}"),
        Some("article")
    );
    assert_eq!(documentclass_value(r"\relax"), None);
}
