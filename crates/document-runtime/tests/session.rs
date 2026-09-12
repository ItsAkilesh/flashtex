use flashtex_document_runtime::{Document, Event, Limits, Request, Session};
use std::{
    thread,
    time::{Duration, Instant},
};
fn request(rev: u64) -> Request {
    Request {
        id: format!("r{rev}"),
        project_id: "p".into(),
        revision: rev,
        entry_path: "main.tex".into(),
        documents: vec![Document {
            path: "main.tex".into(),
            text: format!("α revision {rev}"),
        }],
    }
}
#[cfg(unix)]
fn executable(body: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("compiler");
    std::fs::write(&p, format!("#!/usr/bin/env python3\n{body}\n")).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o700)).unwrap();
    (d, p)
}
#[cfg(unix)]
const ECHO:&str="import json,sys,time\nfor line in sys.stdin:\n r=json.loads(line);p=r['payload'];time.sleep(0.03)\n print(json.dumps({'protocol_version':1,'id':r['id'],'type':'compile_result','payload':{'project_id':p['project_id'],'revision':p['revision'],'status':'ok','pages':[],'diagnostics':[]}}),flush=True)";
fn collect_until(session: &mut Session, predicate: impl Fn(&[Event]) -> bool) -> Vec<Event> {
    let start = Instant::now();
    let mut events = vec![];
    while start.elapsed() < Duration::from_secs(3) {
        events.extend(session.poll());
        if predicate(&events) {
            return events;
        }
        thread::sleep(Duration::from_millis(3));
    }
    panic!("events timed out: {events:?}")
}
#[test]
#[cfg(unix)]
fn coalesces_queued_edits_and_never_displays_stale_revision() {
    let (_d, path) = executable(ECHO);
    let mut s = fake_session(path, Limits::default()).unwrap();
    s.submit(request(1)).unwrap();
    s.submit(request(2)).unwrap();
    s.submit(request(3)).unwrap();
    let events = collect_until(&mut s, |es| {
        es.iter()
            .any(|e| matches!(e, Event::Preview { revision: 3, .. }))
    });
    assert!(events
        .iter()
        .any(|e| matches!(e,Event::Superseded{id,by_id}if id=="r2"&&by_id=="r3")));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Stale { revision: 1, .. })));
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Preview { .. }))
            .count(),
        1
    );
    assert!(s.is_alive());
}
#[test]
#[cfg(unix)]
fn monotonic_revisions_and_paths_are_checked_before_sending() {
    let (_d, path) = executable(ECHO);
    let mut s = fake_session(path, Limits::default()).unwrap();
    s.submit(request(2)).unwrap();
    assert!(s.submit(request(1)).is_err());
    assert!(s.submit(request(2)).is_err());
    let mut invalid = request(3);
    invalid.entry_path = "../secret.tex".into();
    assert!(s.submit(invalid).is_err());
}
#[test]
#[cfg(unix)]
fn correlation_failure_invalidates_queued_work_explicitly() {
    let (_d, path) = executable("import sys\nsys.stdin.readline()\nprint('{}',flush=True)");
    let mut s = fake_session(path, Limits::default()).unwrap();
    s.submit(request(1)).unwrap();
    s.submit(request(2)).unwrap();
    let events = collect_until(&mut s, |es| {
        es.iter()
            .filter(|e| matches!(e, Event::Failed { .. }))
            .count()
            == 2
    });
    assert!(!s.is_alive());
    assert_eq!(events.len(), 2);
    assert!(s.submit(request(3)).is_err());
}
#[test]
#[cfg(unix)]
fn timeout_and_crash_fail_without_approximate_preview() {
    for body in ["import time\ntime.sleep(10)", "raise SystemExit(3)"] {
        let (_d, path) = executable(body);
        let mut s = fake_session(
            path,
            Limits {
                timeout: Duration::from_millis(150),
                ..Limits::default()
            },
        )
        .unwrap();
        s.submit(request(1)).unwrap();
        let events = collect_until(&mut s, |es| {
            es.iter().any(|e| matches!(e, Event::Failed { .. }))
        });
        assert!(!events.iter().any(|e| matches!(e, Event::Preview { .. })));
        assert!(!s.is_alive());
    }
}
#[test]
#[cfg(unix)]
fn stderr_backpressure_does_not_block_real_reply() {
    let body = ECHO.replace(
        "r=json.loads(line);",
        "sys.stderr.write('x'*100000);sys.stderr.flush();r=json.loads(line);",
    );
    let (_d, path) = executable(&body);
    let mut s = fake_session(path, Limits::default()).unwrap();
    s.submit(request(1)).unwrap();
    let events = collect_until(&mut s, |es| {
        es.iter().any(|e| matches!(e, Event::Preview { .. }))
    });
    assert!(matches!(&events[0],Event::Preview{total_ms,..} if *total_ms>=0.0));
}
#[test]
#[ignore = "requires explicitly configured original compiler"]
fn original_persistent_outputs_equal_clean_processes() {
    let binary = std::env::var_os("FLASHTEX_TEST_COMPILER").expect("set original compiler binary");
    let mut warm = Session::spawn(&binary, Limits::default()).unwrap();
    for revision in 1..=3 {
        let r = request(revision);
        warm.submit(r.clone()).unwrap();
        let w = collect_until(&mut warm, |es| {
            es.iter().any(|e| matches!(e, Event::Preview { .. }))
        });
        let mut fresh = Session::spawn(&binary, Limits::default()).unwrap();
        fresh.submit(r).unwrap();
        let f = collect_until(&mut fresh, |es| {
            es.iter().any(|e| matches!(e, Event::Preview { .. }))
        });
        let result = |es: Vec<Event>| {
            es.into_iter()
                .find_map(|e| {
                    if let Event::Preview { result, .. } = e {
                        Some(result)
                    } else {
                        None
                    }
                })
                .unwrap()
        };
        assert_eq!(result(w), result(f));
    }
}

#[test]
#[cfg(unix)]
fn submission_backpressure_bounds_unconsumed_supersession_events() {
    let (_d, path) = executable("import time\ntime.sleep(10)");
    let mut s = fake_session(
        path,
        Limits {
            max_pending_events: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    for revision in 1..=4 {
        s.submit(request(revision)).unwrap();
    }
    assert!(s.submit(request(5)).unwrap_err().contains("poll pending"));
    let events = s.poll();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Superseded { .. }))
            .count(),
        2
    );
    s.submit(request(5)).unwrap();
}
#[test]
#[cfg(unix)]
fn malformed_unicode_source_is_never_delivered_to_preview() {
    let body=ECHO.replace("'pages':[]", "'pages':[{'number':1,'width_pt':612,'height_pt':792,'items':[{'kind':'text','text':'x','x_pt':1,'baseline_y_pt':12,'font_size_pt':12,'source':{'path':'main.tex','start_byte':1,'end_byte':2}}]}]");
    let (_d, path) = executable(&body);
    let mut s = fake_session(path, Limits::default()).unwrap();
    s.submit(request(1)).unwrap();
    let events = collect_until(&mut s, |es| {
        es.iter().any(|e| matches!(e, Event::Failed { .. }))
    });
    assert!(!events.iter().any(|e| matches!(e, Event::Preview { .. })));
}

#[cfg(unix)]
fn fake_session(path: std::path::PathBuf, limits: Limits) -> Result<Session, String> {
    let mut command = std::process::Command::new("/usr/bin/python3");
    command.arg(path);
    Session::spawn_command(command, limits)
}
