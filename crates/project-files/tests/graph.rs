mod common;

use common::{TempDir, pp};
use flashtex_project_files::{
    DiagnosticKind, DiscoverError, FileKind, FileSource, Overlay, PathError, ProjectGraph,
    ProjectPath, ReferenceKind, Severity, json::Json,
};

fn paths(g: &ProjectGraph) -> Vec<&str> {
    g.files().iter().map(|f| f.path.as_str()).collect()
}

#[test]
fn nested_includes_diamond_and_cycle() {
    let t = TempDir::new("graph");
    t.write("main.tex", "\\documentclass{article}\n\\input{chapters/one}\n\\include{chapters/two}\n\\input{shared}\n");
    t.write(
        "chapters/one.tex",
        "One. \\input{shared} \\input{chapters/deep/three}",
    );
    t.write(
        "chapters/two.tex",
        "Two. \\input{main} % cycle back to the entry",
    );
    t.write("chapters/deep/three.tex", "Three.");
    t.write("shared.tex", "Shared.");

    let g = ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    // Depth-first in reference order; shared appears once (diamond).
    assert_eq!(
        paths(&g),
        [
            "main.tex",
            "chapters/one.tex",
            "shared.tex",
            "chapters/deep/three.tex",
            "chapters/two.tex"
        ]
    );
    assert_eq!(
        g.edges().len(),
        6,
        "every resolved reference is an edge, including the cycle-closing one"
    );

    let cycles: Vec<_> = g
        .diagnostics()
        .iter()
        .filter(|d| matches!(d.kind, DiagnosticKind::Cycle { .. }))
        .collect();
    assert_eq!(cycles.len(), 1);
    let c = cycles[0];
    assert_eq!(c.severity, Severity::Error);
    assert_eq!(c.path, pp("chapters/two.tex"));
    let two = t.read("chapters/two.tex");
    let span = c.span.unwrap();
    assert_eq!(&two[span.start..span.end], "\\input{main}");
    match &c.kind {
        DiagnosticKind::Cycle { chain } => {
            assert_eq!(chain, &[pp("main.tex"), pp("chapters/two.tex")])
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(g.has_errors());
}

#[test]
fn path_escape_is_rejected_with_span() {
    let t = TempDir::new("escape");
    t.write("main.tex", "x \\input{../outside} y \\include{/abs/file} z");
    let g = ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    assert_eq!(paths(&g), ["main.tex"]);
    let diags = g.diagnostics();
    assert_eq!(diags.len(), 2);
    assert!(
        matches!(&diags[0].kind, DiagnosticKind::InvalidPath { target, error: PathError::EscapesRoot } if target == "../outside")
    );
    assert!(matches!(
        &diags[1].kind,
        DiagnosticKind::InvalidPath {
            error: PathError::Absolute,
            ..
        }
    ));
    let text = t.read("main.tex");
    let s = diags[0].argument_span.unwrap();
    assert_eq!(&text[s.start..s.end], "../outside");
    assert!(g.edges().is_empty());
}

#[test]
fn missing_file_diagnostic_has_multibyte_spans() {
    let t = TempDir::new("missing");
    let text =
        "Café résumé — \\input{naïve/chapitre} fin. \\includegraphics{fig} \\bibliography{refs}";
    t.write("main.tex", text);
    let g = ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    let diags = g.diagnostics();
    assert_eq!(diags.len(), 3);

    let d = &diags[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.path, pp("main.tex"));
    let span = d.span.unwrap();
    let arg = d.argument_span.unwrap();
    assert_eq!(&text[span.start..span.end], "\\input{naïve/chapitre}");
    assert_eq!(&text[arg.start..arg.end], "naïve/chapitre");
    assert_eq!(
        span.start,
        "Café résumé — ".len(),
        "byte offset, not char offset"
    );
    match &d.kind {
        DiagnosticKind::MissingFile { target, tried } => {
            assert_eq!(target, "naïve/chapitre");
            assert_eq!(tried, &[pp("naïve/chapitre.tex"), pp("naïve/chapitre")]);
        }
        other => panic!("unexpected {other:?}"),
    }
    // Missing graphics are warnings; missing bibliographies are errors.
    assert_eq!(diags[1].severity, Severity::Warning);
    assert!(
        matches!(&diags[1].kind, DiagnosticKind::MissingFile { tried, .. } if tried.len() == 6 && tried[0] == pp("fig.pdf"))
    );
    assert_eq!(diags[2].severity, Severity::Error);
    assert!(
        matches!(&diags[2].kind, DiagnosticKind::MissingFile { tried, .. } if tried == &[pp("refs.bib")])
    );
}

#[test]
fn documents_export_entry_first_tex_only() {
    let t = TempDir::new("docs");
    t.write(
        "main.tex",
        "\\input{b}\\input{a}\\bibliography{refs}\\includegraphics{pic.png}",
    );
    t.write("a.tex", "A");
    t.write("b.tex", "B \\input{a}");
    t.write("refs.bib", "@book{k, title={T}}");
    t.write("pic.png", "not really a png");
    let g = ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    assert!(g.diagnostics().is_empty(), "{:?}", g.diagnostics());
    assert_eq!(
        paths(&g),
        ["main.tex", "b.tex", "a.tex", "refs.bib", "pic.png"]
    );
    assert_eq!(
        g.file(&pp("refs.bib")).unwrap().kind,
        FileKind::Bibliography
    );
    let pic = g.file(&pp("pic.png")).unwrap();
    assert_eq!(pic.kind, FileKind::Graphic);
    assert!(pic.text.is_none());
    assert_eq!(pic.bytes, 16);

    let docs = g.documents();
    let doc_paths: Vec<&str> = docs.iter().map(|d| d.path.as_str()).collect();
    assert_eq!(doc_paths, ["main.tex", "b.tex", "a.tex"]);
    assert_eq!(docs[2].text, "A");
    let with_bib: Vec<String> = g
        .documents_including_bibliography()
        .into_iter()
        .map(|d| d.path)
        .collect();
    assert_eq!(with_bib, ["main.tex", "b.tex", "a.tex", "refs.bib"]);

    let payload = g.compile_payload("demo", 7);
    assert_eq!(
        payload.get("entry_path").unwrap().as_str(),
        Some("main.tex")
    );
    assert_eq!(payload.get("revision").unwrap().as_u64(), Some(7));
    let line = g.compile_envelope("req-1", "demo", 7);
    let parsed = Json::parse(&line).unwrap();
    assert_eq!(parsed.get("type").unwrap().as_str(), Some("compile"));
    assert_eq!(parsed.get("protocol_version").unwrap().as_u64(), Some(1));
    let docs_json = parsed.get("payload").unwrap().get("documents").unwrap();
    match docs_json {
        Json::Array(items) => assert_eq!(items[0].get("path").unwrap().as_str(), Some("main.tex")),
        _ => panic!("documents must be an array"),
    }
    assert!(!line.contains('\n'));
}

#[test]
fn overlay_buffers_take_precedence_and_are_hashed() {
    let t = TempDir::new("overlay");
    t.write("main.tex", "disk \\input{a}");
    t.write("a.tex", "disk a");
    let mut overlay = Overlay::new();
    overlay.insert(pp("main.tex"), "buffer \\input{a} \\input{unsaved}");
    overlay.insert(pp("unsaved.tex"), "never saved");
    let g = ProjectGraph::discover_with(t.root(), &pp("main.tex"), &overlay).unwrap();
    assert!(g.diagnostics().is_empty(), "{:?}", g.diagnostics());
    assert_eq!(paths(&g), ["main.tex", "a.tex", "unsaved.tex"]);
    let main = g.file(&pp("main.tex")).unwrap();
    assert_eq!(main.source, FileSource::Overlay);
    assert_eq!(
        main.sha256,
        flashtex_project_files::sha256(b"buffer \\input{a} \\input{unsaved}")
    );
    assert_eq!(g.file(&pp("a.tex")).unwrap().source, FileSource::Disk);
    assert_eq!(g.documents()[2].text, "never saved");
}

#[test]
fn discovery_is_deterministic() {
    let t = TempDir::new("determinism");
    t.write("main.tex", "\\input{z}\\input{y}\\input{x}");
    t.write("x.tex", "\\input{y}");
    t.write("y.tex", "y");
    t.write("z.tex", "\\input{x}");
    let a = ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    let b = ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    assert_eq!(paths(&a), paths(&b));
    assert_eq!(paths(&a), ["main.tex", "z.tex", "x.tex", "y.tex"]);
    assert_eq!(
        a.compile_envelope("id", "p", 1),
        b.compile_envelope("id", "p", 1)
    );
    assert_eq!(a.edges(), b.edges());
    assert_eq!(a.diagnostics(), b.diagnostics());
}

#[test]
fn unresolvable_macro_argument_and_bare_input() {
    let t = TempDir::new("macro");
    t.write("main.tex", "\\input{\\jobname-extra} \\input bare.tex\n");
    t.write("bare.tex", "bare");
    let g = ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    assert_eq!(paths(&g), ["main.tex", "bare.tex"]);
    assert_eq!(g.diagnostics().len(), 1);
    assert_eq!(g.diagnostics()[0].severity, Severity::Warning);
    assert!(matches!(
        &g.diagnostics()[0].kind,
        DiagnosticKind::UnresolvableReference { .. }
    ));
    assert_eq!(g.edges()[0].reference.kind, ReferenceKind::Input);
}

#[test]
fn entry_errors() {
    let t = TempDir::new("entry");
    assert!(matches!(
        ProjectGraph::discover(t.root(), &pp("nope.tex")),
        Err(DiscoverError::EntryMissing(_))
    ));
    assert!(matches!(
        ProjectGraph::discover(&t.root().join("missing"), &pp("main.tex")),
        Err(DiscoverError::RootNotDirectory(_))
    ));
    std::fs::write(t.root().join("bad.tex"), [0xff, 0xfe, b'x']).unwrap();
    assert!(matches!(
        ProjectGraph::discover(t.root(), &pp("bad.tex")),
        Err(DiscoverError::EntryNotUtf8(_))
    ));
}

#[cfg(unix)]
#[test]
fn symlink_escaping_root_is_rejected() {
    let outside = TempDir::new("outside");
    outside.write("secret.tex", "secret");
    let t = TempDir::new("symlink");
    t.write("main.tex", "\\input{link}");
    std::os::unix::fs::symlink(outside.root().join("secret.tex"), t.root().join("link.tex"))
        .unwrap();
    let g = ProjectGraph::discover(t.root(), &pp("main.tex")).unwrap();
    assert_eq!(paths(&g), ["main.tex"]);
    assert!(
        matches!(&g.diagnostics()[0].kind, DiagnosticKind::EscapesRootViaSymlink { target } if target == &pp("link.tex"))
    );
}

#[test]
fn project_path_display_and_ordering() {
    let mut v = [pp("b/a.tex"), pp("a.tex"), pp("a/z.tex")];
    v.sort();
    assert_eq!(
        v.iter().map(ProjectPath::as_str).collect::<Vec<_>>(),
        ["a.tex", "a/z.tex", "b/a.tex"]
    );
    assert_eq!(format!("{}", pp("./x/../y.tex")), "y.tex");
}
