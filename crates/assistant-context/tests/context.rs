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

#[test]
fn native_helper_refuses_bad_and_oversized_requests_then_recovers() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    fn invoke(input: &[u8]) -> (bool, Value) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-assistant-context"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let _ = child.stdin.take().unwrap().write_all(input);
        let output = child.wait_with_output().unwrap();
        (
            output.status.success(),
            serde_json::from_slice(&output.stdout).unwrap(),
        )
    }
    for bytes in [b"{".to_vec(), vec![b' '; 16 * 1024 * 1024 + 1]] {
        let (success, reply) = invoke(&bytes);
        assert!(!success);
        assert_eq!(reply["type"], "error");
    }
    let docs = vec![source()];
    let valid = json!({"operation":"prepare","binding":CompileBinding::capture("r","p",7,&docs).unwrap(),"sources":docs,"compiler_result":result(),"user_instruction":"Explain"});
    let (success, reply) = invoke(&serde_json::to_vec(&valid).unwrap());
    assert!(success);
    assert_eq!(reply["type"], "prepared_context");
}
#[test]
fn tampered_source_hash_and_outside_context_edits_are_refused() {
    let mut bad = source();
    bad.text.push('x');
    assert!(CompileBinding::capture("r", "p", 7, &[bad]).is_err());
    let text = format!("α \\bad{}", "x".repeat(3000));
    let docs = vec![Document::new("p".into(), "main.tex".into(), 1, text).unwrap()];
    let context = build(&docs);
    let response = json!({"context_id":context.payload().context_id,"explanation":"Change unseen text","edits":[{"location":{"path":"main.tex","start_byte":2500,"end_byte":2501},"removed_text":"x","replacement":"y"}]});
    assert!(context
        .validate_response(&serde_json::to_vec(&response).unwrap(), &docs)
        .is_err());
}

#[test]
fn registry_interleaves_routes_cancels_and_bounds_terminal_retention() {
    use flashtex_assistant_context::{ExplanationRegistry, FlightState};
    use std::time::Duration;
    let docs = vec![source()];
    let mut registry = ExplanationRegistry::new("session_a".into(), 2, 1).unwrap();
    let first = registry
        .submit(build(&docs), &docs, Duration::from_secs(10))
        .unwrap();
    let context = Context::build(
        CompileBinding::capture("r", "p", 7, &docs).unwrap(),
        &docs,
        &result(),
        "different request",
        &[],
    )
    .unwrap();
    let second = registry
        .submit(context, &docs, Duration::from_secs(10))
        .unwrap();
    assert!(registry
        .submit(build(&docs), &docs, Duration::from_secs(10))
        .is_err());
    let first_context = registry.payload(&first).unwrap().context_id.clone();
    let second_context = registry.payload(&second).unwrap().context_id.clone();
    let response = serde_json::to_vec(
        &json!({"context_id":second_context,"explanation":"Explain","edits":[]}),
    )
    .unwrap();
    assert!(registry
        .receive(&first, &second_context, &response, &docs)
        .is_err());
    assert_eq!(registry.state(&first), Some(FlightState::AwaitingResponse));
    assert!(registry
        .receive(&second, &second_context, &response, &docs)
        .is_ok());
    assert!(registry
        .receive(&second, &second_context, &response, &docs)
        .is_err());
    assert!(registry.cancel(&first));
    assert!(registry
        .receive(&first, &first_context, &response, &docs)
        .is_err());
    assert_eq!(registry.state(&second), None); // bounded tombstone eviction
    let third = registry
        .submit(build(&docs), &docs, Duration::from_secs(10))
        .unwrap();
    assert_ne!(first, third);
    assert_ne!(second, third);
    assert_eq!(registry.retained_counts(), (1, 1));
    let changed = vec![Document::new("p".into(), "main.tex".into(), 2, "changed".into()).unwrap()];
    assert!(registry
        .revoke_stale("another-project", &changed)
        .is_empty());
    assert_eq!(registry.revoke_stale("p", &changed), vec![third.clone()]);
    assert_eq!(registry.state(&third), Some(FlightState::Cancelled));
    assert_eq!(docs[0].text, "α \\bad");
}

#[test]
fn registry_expiry_reclaims_capacity_and_bad_responses_are_terminal() {
    use flashtex_assistant_context::{ExplanationRegistry, FlightState};
    use std::time::Duration;
    let docs = vec![source()];
    let mut registry = ExplanationRegistry::new("session_b".into(), 1, 2).unwrap();
    let expired = registry
        .submit(build(&docs), &docs, Duration::from_nanos(1))
        .unwrap();
    std::thread::sleep(Duration::from_millis(1));
    assert_eq!(registry.sweep(), vec![expired.clone()]);
    assert_eq!(registry.state(&expired), Some(FlightState::Expired));
    let next = registry
        .submit(build(&docs), &docs, Duration::from_secs(10))
        .unwrap();
    let context_id = registry.payload(&next).unwrap().context_id.clone();
    assert!(registry
        .receive(&next, &context_id, b"invalid", &docs)
        .is_err());
    assert_eq!(registry.state(&next), Some(FlightState::Failed));
    assert_eq!(registry.retained_counts(), (0, 2));
    assert!(ExplanationRegistry::new("".into(), 1, 1).is_err());
    assert!(ExplanationRegistry::new("ok".into(), 33, 1).is_err());
}

#[test]
fn persistent_helper_interleaves_requests_and_rejects_cancelled_callbacks() {
    use std::io::{BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    let docs = vec![source()];
    let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-assistant-context"))
        .args(["--session", "test_session"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut exchange = |mut value: Value| -> Value {
        let id = value.as_object_mut().unwrap().remove("id").unwrap();
        let value = json!({"id":id,"action":value});
        writeln!(input, "{}", value).unwrap();
        input.flush().unwrap();
        let mut line = String::new();
        assert!(output.read_line(&mut line).unwrap() > 0);
        serde_json::from_str(&line).unwrap()
    };
    let prepare = json!({"operation":"prepare","binding":CompileBinding::capture("r","p",7,&docs).unwrap(),"sources":docs,"compiler_result":result(),"user_instruction":"Explain"});
    let first =
        exchange(json!({"id":"one","operation":"submit","input":prepare,"timeout_ms":10000}));
    assert_eq!(first["result"]["type"], "submitted", "{first}");
    let second =
        exchange(json!({"id":"two","operation":"submit","input":prepare,"timeout_ms":10000}));
    let a = first["result"]["request_id"].clone();
    let b = second["result"]["request_id"].clone();
    assert_ne!(a, b);
    assert_eq!(
        exchange(json!({"id":"cancel","operation":"cancel","request_id":a}))["result"]["changed"],
        true
    );
    let context_id = first["result"]["payload"]["context_id"].clone();
    let response = json!({"context_id":context_id,"explanation":"Explanation","edits":[]});
    assert!(exchange(json!({"id":"late","operation":"receive","request_id":a,"context_id":context_id,"response":response,"current_sources":docs}))["error"].is_string());
    let valid = exchange(
        json!({"id":"valid","operation":"receive","request_id":b,"context_id":context_id,"response":response,"current_sources":docs}),
    );
    assert_eq!(valid["result"]["type"], "validated_proposal", "{valid}");
    assert_eq!(valid["result"]["applied"], false);
    assert!(
        exchange(json!({"id":"invalid","operation":"sweep","unexpected":true}))["error"]
            .is_string()
    );
    assert_eq!(
        exchange(json!({"id":"still_alive","operation":"status","request_id":b}))["result"]
            ["state"],
        "Completed"
    );
    drop(input);
    assert!(child.wait().unwrap().success());
}

#[cfg(unix)]
#[test]
fn supervised_client_handles_real_helper_and_bounds_stalled_pipes() {
    use flashtex_assistant_context::SessionClient;
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    let mut client = SessionClient::spawn(
        std::path::Path::new(env!("CARGO_BIN_EXE_flashtex-assistant-context")),
        "client_test",
    )
    .unwrap();
    assert_eq!(
        client
            .call(json!({"operation":"sweep"}), Duration::from_secs(2))
            .unwrap()["result"]["type"],
        "expired"
    );
    assert_eq!(
        client
            .call(
                json!({"operation":"cancel","request_id":"missing"}),
                Duration::from_secs(2)
            )
            .unwrap()["result"]["changed"],
        false
    );
    drop(client);
    let root =
        std::env::temp_dir().join(format!("flashtex-assistant-client-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    // No reads: command exceeds pipe capacity. Client must time out the write too.
    let fixtures = [
        ("stalled", "#!/bin/sh\nexec sleep 3\n"),
        ("eof", "#!/bin/sh\nexit 0\n"),
        ("stdout_stalled", "#!/bin/sh\nread line\nexec sleep 3\n"),
        (
            "late",
            "#!/bin/sh\nread line\nsleep 0.3\nprintf '%s\\n' '{\"id\":\"1\",\"result\":{}}'\n",
        ),
        (
            "oversized",
            "#!/bin/sh\nread line\nhead -c 140000 /dev/zero\n",
        ),
        (
            "wrong",
            "#!/bin/sh\nread line\nprintf '%s\\n' '{\"id\":\"wrong\",\"result\":{}}'\n",
        ),
        // Descendant retains stdout after direct child exit. No reader thread
        // may remain blocked; descendant exits itself after a bounded lifetime.
        ("inherited", "#!/bin/sh\nsleep 1 &\nexit 0\n"),
    ];
    for (name, script) in fixtures {
        let path = root.join(name);
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut client = SessionClient::spawn(&path, "fixture").unwrap();
        let start = Instant::now();
        let action = if name == "stalled" {
            json!({"operation":"fake","padding":"x".repeat(1024*1024)})
        } else {
            json!({"operation":"sweep"})
        };
        assert!(
            client.call(action, Duration::from_millis(100)).is_err(),
            "{name}"
        );
        assert!(start.elapsed() < Duration::from_secs(2), "{name}");
        assert!(client.is_stopped());
        assert!(client
            .call(json!({"operation":"sweep"}), Duration::from_secs(1))
            .is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dispatch_lease_is_single_use_shared_and_revoked_with_owner() {
    use flashtex_assistant_context::ExplanationRegistry;
    use std::time::Duration;
    let docs = vec![source()];
    let mut registry = ExplanationRegistry::new("lease_test".into(), 2, 2).unwrap();
    let id = registry
        .submit(build(&docs), &docs, Duration::from_secs(5))
        .unwrap();
    let lease = registry.lease(&id, &docs).unwrap();
    assert!(std::ptr::eq(
        lease.payload(),
        registry.payload(&id).unwrap()
    ));
    assert!(registry.lease(&id, &docs).is_err());
    assert!(lease.check_current(&docs).is_ok());
    assert!(registry.cancel(&id));
    assert!(lease.check_current(&docs).is_err());
    let id = registry
        .submit(build(&docs), &docs, Duration::from_secs(5))
        .unwrap();
    let lease = registry.lease(&id, &docs).unwrap();
    drop(registry);
    assert!(lease.check_current(&docs).is_err());
}

#[cfg(feature = "grok")]
#[test]
fn provider_helper_requires_explicit_startup_and_admission() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let binary = env!("CARGO_BIN_EXE_flashtex-assistant-context");
    let missing = Command::new(binary)
        .args(["--provider-session", "test", "model"])
        .env_remove("FLASHTEX_GROK_API_KEY")
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("credential missing"));
    let docs = vec![source()];
    let input = json!({"operation":"prepare","binding":CompileBinding::capture("r","p",7,&docs).unwrap(),"sources":docs,"compiler_result":result(),"user_instruction":"Explain"});
    let commands = [
        json!({"id":"snapshot","action":{"operation":"provider","command":{"operation":"snapshot"}}}),
        json!({"id":"no_consent","action":{"operation":"provider","command":{"operation":"admit","input":input,"timeout_ms":1000,"user_requested":false,"allocation":"dummy-test"}}}),
    ];
    for enabled in [false, true] {
        let mut command = Command::new(binary);
        if enabled {
            command.args(["--provider-session", "fixture", "model"]);
        } else {
            command.args(["--session", "fixture"]);
        }
        let mut child = command
            .env("FLASHTEX_GROK_API_KEY", "dummy-no-inference-secret")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        for value in &commands {
            writeln!(stdin, "{value}").unwrap();
        }
        drop(stdin);
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(!text.contains("dummy-no-inference-secret"));
        assert!(output.stderr.is_empty());
        let replies: Vec<Value> = text
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        if enabled {
            assert_eq!(
                replies[0]["result"]["payload"]["scheduler_tasks_started"],
                0
            );
        } else {
            assert!(replies[0]["error"].as_str().unwrap().contains("disabled"));
        }
        assert!(replies[1]["error"].is_string());
    }
}

#[cfg(all(unix, feature = "grok"))]
#[test]
fn supervised_provider_startup_is_explicit_and_does_not_start_calls() {
    use flashtex_assistant_context::SessionClient;
    use std::{path::Path, time::Duration};
    let binary = Path::new(env!("CARGO_BIN_EXE_flashtex-assistant-context"));
    let mut client = SessionClient::spawn_provider(
        binary,
        "native_client",
        "explicit-model",
        "dummy-local-secret",
    )
    .unwrap();
    let snapshot = client
        .call(
            json!({"operation":"provider","command":{"operation":"snapshot"}}),
            Duration::from_secs(2),
        )
        .unwrap();
    assert_eq!(snapshot["result"]["payload"]["scheduler_tasks_started"], 0);
    assert_eq!(
        snapshot["result"]["payload"]["provider_billing_known"],
        false
    );
    assert!(!snapshot.to_string().contains("dummy-local-secret"));
    assert!(SessionClient::spawn_provider(binary, "native_client", "model", "bad\nkey").is_err());
    assert!(SessionClient::spawn_provider(binary, "native_client", "", "dummy").is_err());
}
