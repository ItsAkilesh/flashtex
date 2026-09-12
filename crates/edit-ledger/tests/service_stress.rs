use flashtex_edit_ledger::{
    service::{BackgroundService, ServiceOptions, ServiceReply, ShutdownWatch},
    Document, Store,
};
use serde_json::{json, Value};
use std::{
    thread,
    time::{Duration, Instant},
};

fn ask(service: &BackgroundService, request: Value) -> ServiceReply {
    let frame = format!("{request}\n").into_bytes();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match service.try_submit(frame.clone()) {
            Ok(reply) => return reply.wait().unwrap(),
            Err(error) if error.code == "busy" && Instant::now() < deadline => thread::yield_now(),
            Err(error) => panic!("admission failed: {error}"),
        }
    }
}
fn stopped(watch: ShutdownWatch) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !watch.is_stopped() {
        assert!(Instant::now() < deadline, "service failed to stop");
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn concurrent_sessions_and_writers_preserve_all_guarded_groups() {
    let sessions:Vec<_>=(0..4).map(|session|thread::spawn(move||{
        let dir=tempfile::tempdir().unwrap();
        let service=BackgroundService::start(dir.path().into(),ServiceOptions::default()).unwrap();
        let document=Document::new(format!("project-{session}"),"main.tex".into(),1,"é😀".into()).unwrap();
        assert!(ask(&service,json!({"id":"init","operation":"initialize","document":document})).command_succeeded);
        let writers:Vec<_>=(0..4).map(|writer|{
            let client=service.clone();
            thread::spawn(move||{
                for index in 0..10 {
                    let id=format!("s{session}-w{writer}-e{index}");let marker=format!("[{id}]");
                    let deadline=Instant::now()+Duration::from_secs(10);
                    loop {
                        assert!(Instant::now()<deadline,"guarded edit made no progress");
                        let state=ask(&client,json!({"id":format!("read-{id}"),"operation":"status"}));
                        let doc=&state.payload.as_ref().unwrap()["document"];
                        let text=doc["text"].as_str().unwrap();
                        let result=ask(&client,json!({"id":id,"operation":"apply_group","group":{
                            "command_id":id,"expected_revision":doc["revision"],"expected_sha256":doc["source_sha256"],"label":"Concurrent append",
                            "edits":[{"start_byte":text.len(),"end_byte":text.len(),"removed_text":"","replacement":marker}]
                        }}));
                        assert_eq!(result.session_id,client.session_id());
                        if result.command_succeeded {break;}
                        assert_eq!(result.error.unwrap().code,"document_conflict");
                    }
                }
            })
        }).collect();
        for writer in writers {writer.join().unwrap();}
        let result=ask(&service,json!({"id":"final","operation":"status"}));
        assert_eq!(result.document_revision,Some(41));
        let text=result.payload.unwrap()["document"]["text"].as_str().unwrap().to_owned();
        for writer in 0..4 {for index in 0..10 {assert_eq!(text.matches(&format!("[s{session}-w{writer}-e{index}]")).count(),1);}}
        stopped(service.shutdown());
        let reopened=Store::open(dir.path()).unwrap();
        assert_eq!(reopened.document().unwrap().unwrap().text,text);
        assert_eq!(reopened.history_status().unwrap().permanent_command_ids,40);
    })).collect();
    for session in sessions {
        session.join().unwrap();
    }
}

#[test]
fn shutdown_waits_for_all_handles_then_allows_new_session() {
    let dir = tempfile::tempdir().unwrap();
    let first = BackgroundService::start(dir.path().into(), ServiceOptions::default()).unwrap();
    let clone = first.clone();
    let first_id = first.session_id().to_owned();
    assert!(ask(&first, json!({"id":"open","operation":"status"})).command_succeeded);
    let watch = first.shutdown();
    assert!(!watch.is_stopped());
    drop(clone);
    stopped(watch);
    let second = BackgroundService::start(dir.path().into(), ServiceOptions::default()).unwrap();
    assert_ne!(second.session_id(), first_id);
    assert!(ask(&second, json!({"id":"reopen","operation":"status"})).command_succeeded);
    stopped(second.shutdown());
}

#[test]
fn long_sequence_dropped_replies_backpressure_and_retention_keep_exactly_once_ids() {
    let dir = tempfile::tempdir().unwrap();
    let service = BackgroundService::start(
        dir.path().into(),
        ServiceOptions {
            capacity: 1,
            ..ServiceOptions::default()
        },
    )
    .unwrap();
    let document = Document::new("long-sequence".into(), "main.tex".into(), 1, "x".into()).unwrap();
    assert!(
        ask(
            &service,
            json!({"id":"init","operation":"initialize","document":document})
        )
        .command_succeeded
    );
    let mut replay = Vec::new();
    for index in 0..120 {
        let before = ask(&service, json!({"id":"before","operation":"status"}));
        let document = &before.payload.as_ref().unwrap()["document"];
        let edit = json!({"capture_id":format!("capture-{index}"),"edit_id":format!("edit-{index}"),
            "project_id":"long-sequence","path":"main.tex","expected_revision":document["revision"],
            "start_byte":0,"end_byte":1,"removed_text":document["text"],"replacement":if index%2==0 {"y"}else{"x"},
            "document_before_sha256":document["source_sha256"]});
        let frame = format!(
            "{}\n",
            json!({"id":format!("apply-{index}"),"operation":"apply","edit":edit})
        )
        .into_bytes();
        let pending = service.try_submit(frame).unwrap();
        // An undrained reply occupies the one admission slot even if committed.
        let rejected=format!("{}\n",json!({"id":"must-not-run","operation":"replace_document",
            "expected_revision":document["revision"],"expected_sha256":document["source_sha256"],"text":"rejected"})).into_bytes();
        assert!(matches!(service.try_submit(rejected),Err(error) if error.code=="busy"));
        // UI cancellation only abandons delivery; an accepted transaction must
        // remain durable. FIFO status/retry reconciles its actual outcome.
        drop(pending);
        let retried = ask(
            &service,
            json!({"id":"retry","operation":"apply","edit":edit}),
        );
        assert!(retried.command_succeeded, "{retried:?}");
        let receipt = retried.payload.as_ref().unwrap()["receipt"].clone();
        assert_eq!(retried.document_revision, Some(index + 2));
        assert!(
            ask(
                &service,
                json!({"id":"confirm","operation":"confirm","receipt":receipt})
            )
            .command_succeeded
        );
        replay.push((edit, receipt));
        if index % 10 == 9 {
            let recovery = ask(
                &service,
                json!({"id":"token","operation":"recovery_export"}),
            );
            let token = &recovery.payload.as_ref().unwrap()["snapshot_token"];
            assert!(ask(&service,json!({"id":"retention","operation":"retain_history","policy":{
                "snapshot_token":token,"acknowledge_undo_redo_loss":true,"keep_latest_undo":4,"keep_latest_redo":0}})).command_succeeded);
            let recovery = ask(
                &service,
                json!({"id":"token","operation":"recovery_export"}),
            );
            let token = &recovery.payload.as_ref().unwrap()["snapshot_token"];
            assert!(ask(&service,json!({"id":"compact","operation":"compact","policy":{
                "snapshot_token":token,"acknowledge_permanent_id_retention":true,"acknowledged_through_revision":index+2,"keep_latest_confirmed":2}})).command_succeeded);
        }
    }
    let final_state = ask(&service, json!({"id":"final","operation":"status"}));
    let final_document = final_state.payload.unwrap()["document"].clone();
    stopped(service.shutdown());
    let mut store = Store::open(dir.path()).unwrap();
    assert_eq!(
        serde_json::to_value(store.document().unwrap()).unwrap(),
        final_document
    );
    assert_eq!(store.history_status().unwrap().undo_labels.len(), 4);
    for (edit, receipt) in replay {
        let actual = store.apply(serde_json::from_value(edit).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(), receipt);
    }
    assert_eq!(
        serde_json::to_value(store.document().unwrap()).unwrap(),
        final_document
    );
}
