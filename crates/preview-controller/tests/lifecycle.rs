use flashtex_document_runtime::{Event, Limits};
use flashtex_edit_ledger::{Document, Store};
use flashtex_preview_controller::{Controller, Update};
use flashtex_project_index::Category;
use std::{
    process::Command,
    thread,
    time::{Duration, Instant},
};
fn command(dir: &std::path::Path, body: &str) -> Command {
    let path = dir.join("compiler.py");
    std::fs::write(&path, body).unwrap();
    let mut command = Command::new("/usr/bin/python3");
    command.arg(path);
    command
}
const ECHO: &str = "import json,sys,time\nfor line in sys.stdin:\n r=json.loads(line);p=r['payload'];time.sleep(.02)\n print(json.dumps({'protocol_version':1,'type':'compile_result','id':r['id'],'payload':{'project_id':p['project_id'],'revision':p['revision'],'status':'ok','pages':[],'diagnostics':[]}}),flush=True)\n";
fn store(dir: &std::path::Path) -> Store {
    let mut store = Store::open(dir.join("source")).unwrap();
    if store.document().unwrap().is_none() {
        store
            .initialize(
                Document::new("p".into(), "main.tex".into(), 1, "α \\label{old}".into()).unwrap(),
            )
            .unwrap();
    }
    store
}
fn wait(controller: &mut Controller, predicate: impl Fn(&[Update]) -> bool) -> Vec<Update> {
    let start = Instant::now();
    let mut events = Vec::new();
    while start.elapsed() < Duration::from_secs(3) {
        events.extend(controller.poll());
        if predicate(&events) {
            return events;
        }
        thread::sleep(Duration::from_millis(2));
    }
    panic!("controller timed out: {events:?}");
}
#[test]
fn edit_persists_indexes_and_only_previews_current_revision() {
    let dir = tempfile::tempdir().unwrap();
    let source = store(dir.path());
    let mut controller = Controller::new(
        "p".into(),
        "main.tex".into(),
        vec![source],
        command(dir.path(), ECHO),
        Limits::default(),
    )
    .unwrap();
    controller.compile_current().unwrap();
    let before = controller.document("main.tex").unwrap().clone();
    let outcome = controller
        .replace_document(
            "main.tex",
            before.revision,
            &before.source_sha256,
            "β \\label{new}".into(),
        )
        .unwrap();
    assert!(outcome.preview_error.is_none());
    let snapshot = controller.index().snapshot();
    assert_eq!(
        controller
            .index()
            .definitions(&snapshot, Category::Label, "new")
            .unwrap()
            .len(),
        1
    );
    let events = wait(&mut controller, |events| {
        events
            .iter()
            .any(|event| matches!(event, Update::Preview(_)))
    });
    let previews: Vec<_> = events
        .iter()
        .filter_map(|event| {
            if let Update::Preview(preview) = event {
                Some(preview)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(previews.len(), 1);
    assert_eq!(previews[0].source_versions.documents["main.tex"], 2);
    drop(controller);
    assert_eq!(
        store(dir.path()).document().unwrap().unwrap().text,
        "β \\label{new}"
    );
}
#[test]
fn compiler_failure_does_not_lose_typing_and_restart_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let mut controller = Controller::new(
        "p".into(),
        "main.tex".into(),
        vec![store(dir.path())],
        command(dir.path(), "raise SystemExit(3)"),
        Limits::default(),
    )
    .unwrap();
    controller.compile_current().unwrap();
    wait(&mut controller, |events| {
        events
            .iter()
            .any(|event| matches!(event, Update::Runtime(Event::Failed { .. })))
    });
    let before = controller.document("main.tex").unwrap().clone();
    let saved = controller
        .replace_document(
            "main.tex",
            before.revision,
            &before.source_sha256,
            "saved despite crash".into(),
        )
        .unwrap();
    assert!(saved.preview_error.is_some());
    assert_eq!(
        controller.document("main.tex").unwrap().text,
        "saved despite crash"
    );
    controller
        .restart(command(dir.path(), ECHO), Limits::default())
        .unwrap();
    wait(&mut controller, |events| {
        events
            .iter()
            .any(|event| matches!(event, Update::Preview(_)))
    });
}
#[test]
fn close_never_publishes_inflight_preview_or_accepts_typing() {
    let dir = tempfile::tempdir().unwrap();
    let mut controller = Controller::new(
        "p".into(),
        "main.tex".into(),
        vec![store(dir.path())],
        command(dir.path(), ECHO),
        Limits::default(),
    )
    .unwrap();
    controller.compile_current().unwrap();
    controller.close().unwrap();
    thread::sleep(Duration::from_millis(100));
    assert!(!controller
        .poll()
        .iter()
        .any(|event| matches!(event, Update::Preview(_))));
    let before = controller.document("main.tex").unwrap().clone();
    assert!(controller
        .replace_document(
            "main.tex",
            before.revision,
            &before.source_sha256,
            "late".into()
        )
        .is_err());
}
#[test]
#[ignore = "requires explicit original compiler"]
fn original_compiler_renders_durable_updated_source() {
    let binary = std::env::var_os("FLASHTEX_TEST_COMPILER").expect("set original compiler path");
    let dir = tempfile::tempdir().unwrap();
    let mut controller = Controller::new(
        "p".into(),
        "main.tex".into(),
        vec![store(dir.path())],
        Command::new(binary),
        Limits::default(),
    )
    .unwrap();
    let before = controller.document("main.tex").unwrap().clone();
    let outcome = controller
        .replace_document(
            "main.tex",
            before.revision,
            &before.source_sha256,
            "Hello persistent preview".into(),
        )
        .unwrap();
    assert!(outcome.preview_error.is_none());
    let events = wait(&mut controller, |events| {
        events
            .iter()
            .any(|event| matches!(event, Update::Preview(_)))
    });
    let preview = events
        .into_iter()
        .find_map(|event| {
            if let Update::Preview(p) = event {
                Some(p)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(preview.source_versions.documents["main.tex"], 2);
    assert_eq!(preview.result["payload"]["status"], "ok");
    assert!(!preview.result["payload"]["pages"]
        .as_array()
        .unwrap()
        .is_empty());
}
