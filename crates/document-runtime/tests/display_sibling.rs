#![cfg(unix)]
use flashtex_document_runtime::{Document, Event, Limits, Request, Session};
use std::{
    os::unix::fs::PermissionsExt,
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
            text: "東京 α".into(),
        }],
    }
}
fn caps() -> Vec<String> {
    vec!["display-list-v2".into()]
}
fn session(mode: &str) -> (tempfile::TempDir, Session) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("worker.py");
    let script = format!(
        r#"#!/usr/bin/env python3
import json,sys,time,hashlib
mode={mode:?}
for line in sys.stdin:
 r=json.loads(line);p=r['payload'];cap=p.get('layout_capabilities',[])
 accepted=[] if mode=='declined' else cap
 status='failed' if mode=='failed' else 'ok'
 result={{'protocol_version':1,'type':'compile_result','id':r['id'],'payload':{{'project_id':p['project_id'],'revision':p['revision'],'status':status,'pages':[],'diagnostics':[],'layout_capabilities':accepted}}}}
 if mode=='delay': time.sleep(.04)
 print(json.dumps(result),flush=True)
 if mode=='missing': time.sleep(2);continue
 if mode in ('declined','failed'):continue
 docs=[{{'path':d['path'],'revision':p['revision'],'sha256':hashlib.sha256(d['text'].encode()).hexdigest(),'byte_length':len(d['text'].encode())}} for d in p['documents']]
 v={{'protocol_version':2,'type':'display_list','id':r['id'],'payload':{{'project_id':p['project_id'],'revision':p['revision'],'render_format':'display-list-v2','documents':docs}}}}
 if mode=='hash':docs[0]['sha256']='0'*64
 if mode=='length':docs[0]['byte_length']=len(p['documents'][0]['text'])
 if mode=='duplicate_doc':docs.append(docs[0])
 if mode=='revision':v['payload']['revision']+=1
 if mode=='id':v['id']='other'
 if mode=='interleaved':v=result
 if mode=='malformed':print('{{',flush=True);continue
 if mode=='gap':time.sleep(.1)
 print(json.dumps(v),flush=True)
 if mode=='duplicate':print(json.dumps(v),flush=True)
"#
    );
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    let limits = Limits {
        timeout: Duration::from_millis(500),
        ..Limits::default()
    };
    let s = Session::spawn(path, limits).unwrap();
    (dir, s)
}
fn poll_for(s: &mut Session, ms: u64) -> Vec<Event> {
    let start = Instant::now();
    let mut events = vec![];
    while start.elapsed() < Duration::from_millis(ms) {
        events.extend(s.poll());
        thread::sleep(Duration::from_millis(2));
    }
    events
}
#[test]
fn opt_in_exact_utf8_identity_owned_slot_and_legacy_fallback() {
    let (_d, mut s) = session("ok");
    assert!(s.submit_with_capabilities(request(1), caps()).is_err());
    s.set_display_candidates_enabled(true).unwrap();
    assert!(s.set_completed_snapshots_enabled(true).is_err());
    s.submit_with_capabilities(request(1), caps()).unwrap();
    let e = poll_for(&mut s, 100);
    assert!(e
        .iter()
        .any(|e| matches!(e, Event::Preview { revision: 1, .. })));
    let c = s.take_current_display_candidate().unwrap();
    assert_eq!(c.request_id(), "r1");
    assert_eq!(c.project_id(), "p");
    assert_eq!(c.revision(), 1);
    assert_eq!(c.sources()[0].byte_length, "東京 α".len());
    assert_eq!(
        c.sources()[0].sha256,
        flashtex_project_files::sha256_hex("東京 α".as_bytes())
    );
    assert_eq!(c.into_envelope()["protocol_version"], 2);
    assert!(s.take_current_display_candidate().is_none());
    assert!(s.is_alive());
}
#[test]
fn declined_failed_and_old_requests_finish_without_sibling() {
    for mode in ["declined", "failed"] {
        let (_d, mut s) = session(mode);
        s.set_display_candidates_enabled(true).unwrap();
        s.submit_with_capabilities(request(1), caps()).unwrap();
        poll_for(&mut s, 80);
        assert!(s.take_current_display_candidate().is_none());
        assert!(s.is_alive());
        s.submit_with_capabilities(request(2), caps()).unwrap();
        assert!(poll_for(&mut s, 80)
            .iter()
            .any(|e| matches!(e, Event::Preview { revision: 2, .. })));
    }
}
#[test]
fn malformed_mismatched_duplicate_interleaved_and_missing_fail_closed() {
    for mode in [
        "hash",
        "length",
        "duplicate_doc",
        "revision",
        "id",
        "interleaved",
        "malformed",
        "duplicate",
        "missing",
    ] {
        let (_d, mut s) = session(mode);
        s.set_display_candidates_enabled(true).unwrap();
        s.submit_with_capabilities(request(1), caps()).unwrap();
        poll_for(&mut s, if mode == "missing" { 600 } else { 120 });
        assert!(!s.is_alive(), "{mode}");
        assert!(s.take_current_display_candidate().is_none(), "{mode}");
    }
}
#[test]
fn stale_cancelled_toggle_and_submit_invalidate_candidates() {
    let (_d, mut s) = session("delay");
    s.set_display_candidates_enabled(true).unwrap();
    s.submit_with_capabilities(request(1), caps()).unwrap();
    s.submit_with_capabilities(request(2), caps()).unwrap();
    poll_for(&mut s, 160);
    assert_eq!(s.take_current_display_candidate().unwrap().revision(), 2);
    s.submit_with_capabilities(request(3), caps()).unwrap();
    s.close_project("p").unwrap();
    poll_for(&mut s, 100);
    assert!(s.take_current_display_candidate().is_none());
    assert!(s.is_alive());
    let (_d, mut s) = session("gap");
    s.set_display_candidates_enabled(true).unwrap();
    s.submit_with_capabilities(request(1), caps()).unwrap();
    poll_for(&mut s, 40);
    s.set_display_candidates_enabled(false).unwrap();
    s.set_display_candidates_enabled(true).unwrap();
    poll_for(&mut s, 140);
    assert!(s.take_current_display_candidate().is_none());
    assert!(s.is_alive());
    s.submit_with_capabilities(request(2), caps()).unwrap();
    poll_for(&mut s, 150);
    s.submit_with_capabilities(request(3), caps()).unwrap();
    assert!(s.take_current_display_candidate().is_none());
}

#[test]
fn unsolicited_and_closed_project_siblings_never_escape() {
    let (_d, mut s) = session("ok");
    s.submit(request(1)).unwrap();
    poll_for(&mut s, 100);
    assert!(!s.is_alive());
    assert!(s.take_current_display_candidate().is_none());
    let (_d, mut s) = session("gap");
    s.set_display_candidates_enabled(true).unwrap();
    s.submit_with_capabilities(request(1), caps()).unwrap();
    poll_for(&mut s, 40);
    s.close_project("p").unwrap();
    poll_for(&mut s, 120);
    assert!(s.is_alive());
    assert!(s.take_current_display_candidate().is_none());
}

#[test]
fn promised_sibling_blocks_next_dispatch_and_preserves_timeout() {
    let (_d, mut s) = session("missing");
    s.set_display_candidates_enabled(true).unwrap();
    s.submit_with_capabilities(request(1), caps()).unwrap();
    let mut events = poll_for(&mut s, 60);
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Preview { revision: 1, .. })));
    s.submit_with_capabilities(request(2), caps()).unwrap();
    events.extend(poll_for(&mut s, 550));
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::Preview { revision: 2, .. })));
    assert!(events.iter().any(
        |e| matches!(e, Event::Failed { id, reason } if id == "r1" && reason.contains("timeout"))
    ));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Failed { id, .. } if id == "r2")));
}
