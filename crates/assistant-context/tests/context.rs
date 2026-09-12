use flashtex_assistant_context::{CompileBinding, Context};
use flashtex_edit_ledger::Document;
use serde_json::{json, Value};
fn source() -> Document {
    Document::new("p".into(), "main.tex".into(), 1, "α \\bad".into()).unwrap()
}
fn result() -> Value {
    json!({"protocol_version":1,"id":"r","type":"compile_result","payload":{"project_id":"p","revision":7,"status":"recovered","pages":[],"diagnostics":[{"severity":"error","message":"Unknown command","source":{"path":"main.tex","start_byte":3,"end_byte":7}}]}})
}
fn build(docs: &[Document]) -> Context {
    Context::build(
        CompileBinding::capture("r", "p", 7, docs).unwrap(),
        docs,
        &result(),
        "Explain simply",
        &[],
    )
    .unwrap()
}
#[test]
fn bounded_utf8_context_preserves_exact_source_and_rejects_stale_compiler() {
    let docs = vec![source()];
    let context = build(&docs);
    assert_eq!(context.payload().provider_intent, "grok");
    assert_eq!(
        context.payload().diagnostics[0]
            .snippet
            .as_ref()
            .unwrap()
            .text,
        "α \\bad"
    );
    let changed = vec![Document::new("p".into(), "main.tex".into(), 2, "changed".into()).unwrap()];
    assert!(context.check_current(&changed).is_err());
    let mut bad = result();
    bad["payload"]["revision"] = json!(8);
    assert!(Context::build(
        CompileBinding::capture("r", "p", 7, &docs).unwrap(),
        &docs,
        &bad,
        "",
        &[]
    )
    .is_err());
}
#[test]
fn response_is_only_a_source_bound_proposal() {
    let docs = vec![source()];
    let context = build(&docs);
    let response = json!({"context_id":context.payload().context_id,"explanation":"This command is unknown.","edits":[{"location":{"path":"main.tex","start_byte":3,"end_byte":7},"removed_text":"\\bad","replacement":"text"}]});
    let validated = context
        .validate_response(&serde_json::to_vec(&response).unwrap(), &docs)
        .unwrap();
    assert_eq!(validated.edits.len(), 1);
    assert_eq!(docs[0].text, "α \\bad");
    let mut stale = response.clone();
    stale["context_id"] = json!("other");
    assert!(context
        .validate_response(&serde_json::to_vec(&stale).unwrap(), &docs)
        .is_err());
    let mut malformed = response;
    malformed["edits"][0]["location"]["start_byte"] = json!(1);
    assert!(context
        .validate_response(&serde_json::to_vec(&malformed).unwrap(), &docs)
        .is_err());
}
#[test]
fn multifile_context_is_explicit_and_payload_limits_are_enforced() {
    let docs = vec![
        source(),
        Document::new("p".into(), "chapter.tex".into(), 1, "related β".into()).unwrap(),
    ];
    let binding = CompileBinding::capture("r", "p", 7, &docs).unwrap();
    let context = Context::build(
        binding.clone(),
        &docs,
        &result(),
        "use chapter",
        &["chapter.tex".into()],
    )
    .unwrap();
    assert_eq!(context.payload().related.len(), 1);
    assert!(Context::build(binding.clone(), &docs, &result(), &"x".repeat(8193), &[]).is_err());
    assert!(Context::build(binding, &docs, &result(), "", &["missing.tex".into()]).is_err());
}

#[test]
#[ignore = "requires explicitly configured original compiler"]
fn actual_compiler_diagnostic_builds_source_bound_context() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let compiler = std::env::var("FLASHTEX_TEST_COMPILER").unwrap();
    let doc = Document::new(
        "p".into(),
        "main.tex".into(),
        1,
        "\\documentclass{article}\n\\begin{document}\n\\unknowncommand\n\\end{document}".into(),
    )
    .unwrap();
    let sources = vec![doc];
    let request = json!({"protocol_version":1,"type":"compile","id":"r","payload":{"project_id":"p","revision":7,"entry_path":"main.tex","documents":[{"path":"main.tex","text":sources[0].text}]}});
    let mut child = Command::new(compiler)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{request}").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    let context = Context::build(
        CompileBinding::capture("r", "p", 7, &sources).unwrap(),
        &sources,
        &result,
        "Explain this error without editing",
        &[],
    )
    .unwrap();
    assert!(!context.payload().diagnostics.is_empty());
    assert_eq!(
        context.payload().user_instruction,
        "Explain this error without editing"
    );
}

#[test]
fn cancelled_expired_and_stale_flights_never_accept_late_results() {
    use flashtex_assistant_context::{ExplanationFlight, FlightState};
    use std::time::Duration;
    let docs = vec![source()];
    let context = build(&docs);
    let response = serde_json::to_vec(
        &json!({"context_id":context.payload().context_id,"explanation":"Explanation","edits":[]}),
    )
    .unwrap();
    let mut cancelled = ExplanationFlight::new(context, &docs, Duration::from_secs(5)).unwrap();
    cancelled.cancel();
    assert_eq!(cancelled.state(), FlightState::Cancelled);
    assert!(cancelled.receive(&response, &docs).is_err());
    let mut expired =
        ExplanationFlight::new(build(&docs), &docs, Duration::from_millis(1)).unwrap();
    std::thread::sleep(Duration::from_millis(2));
    assert!(expired.receive(&response, &docs).is_err());
    assert_eq!(expired.state(), FlightState::Expired);
    let changed =
        vec![Document::new("p".into(), "main.tex".into(), 2, "new source".into()).unwrap()];
    let mut stale = ExplanationFlight::new(build(&docs), &docs, Duration::from_secs(5)).unwrap();
    assert!(stale.receive(&response, &changed).is_err());
    assert_eq!(stale.state(), FlightState::Failed);
    assert!(stale.receive(&response, &docs).is_err());
    let mut success = ExplanationFlight::new(build(&docs), &docs, Duration::from_secs(5)).unwrap();
    success.receive(&response, &docs).unwrap();
    assert_eq!(success.state(), FlightState::Completed);
    assert!(success.receive(&response, &docs).is_err());
}

#[test]
fn selected_diagnostics_are_bound_and_clipping_is_explicit() {
    let docs = vec![source()];
    let binding = CompileBinding::capture("r", "p", 7, &docs).unwrap();
    let mut result = result();
    let mut second = result["payload"]["diagnostics"][0].clone();
    second["message"] = json!("β".repeat(1100));
    result["payload"]["diagnostics"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let context =
        Context::build_selected(binding.clone(), &docs, &result, "second only", &[], &[1]).unwrap();
    assert_eq!(context.payload().diagnostics.len(), 1);
    assert_eq!(context.payload().diagnostics[0].diagnostic_index, 1);
    assert!(context.payload().diagnostics[0].message_truncated);
    assert_eq!(context.payload().diagnostics[0].message.len(), 2048);
    assert_eq!(context.payload().omitted_diagnostics, 1);
    assert!(Context::build_selected(binding.clone(), &docs, &result, "", &[], &[1, 1]).is_err());
    assert!(Context::build_selected(binding, &docs, &result, "", &[], &[2]).is_err());
}

#[test]
fn explicit_destination_rehashes_context_and_limits_proposals() {
    use flashtex_assistant_context::Location;
    let docs = vec![source()];
    let original = build(&docs);
    let id = original.payload().context_id.clone();
    let context = original
        .restrict_edits(
            vec![Location {
                path: "main.tex".into(),
                start_byte: 3,
                end_byte: 3,
            }],
            &docs,
        )
        .unwrap();
    assert_ne!(context.payload().context_id, id);
    let mut response = json!({"context_id":context.payload().context_id,"explanation":"Insert text","edits":[{"location":{"path":"main.tex","start_byte":3,"end_byte":3},"removed_text":"","replacement":"x"}]});
    context
        .validate_response(&serde_json::to_vec(&response).unwrap(), &docs)
        .unwrap();
    response["edits"][0]["location"]["end_byte"] = json!(7);
    response["edits"][0]["removed_text"] = json!("\\bad");
    assert!(context
        .validate_response(&serde_json::to_vec(&response).unwrap(), &docs)
        .is_err());
}
#[test]
fn native_json_helper_prepares_and_validates_without_applying() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    fn call(value: &Value) -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-assistant-context"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(value).unwrap())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    let docs = vec![source()];
    let mut request = json!({"operation":"prepare","binding":CompileBinding::capture("r","p",7,&docs).unwrap(),"sources":docs,"compiler_result":result(),"user_instruction":"Explain only","destinations":[]});
    let prepared = call(&request);
    assert_eq!(prepared["type"], "prepared_context");
    request["operation"] = json!("validate");
    request["current_sources"] = json!(docs);
    request["response"] = json!({"context_id":prepared["payload"]["context_id"],"explanation":"Explanation","edits":[]});
    let validated = call(&request);
    assert_eq!(validated["type"], "validated_proposal");
    assert_eq!(validated["applied"], false);
}
