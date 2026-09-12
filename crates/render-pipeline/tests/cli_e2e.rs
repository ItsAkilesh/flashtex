//! `flashtex-render` end to end over JSON Lines: a compile request, a
//! rejected protocol version, an unknown type, capability negotiation,
//! `--v2` output and `--pdf` output.

use std::io::Write;
use std::process::{Command, Stdio};

use flashtex_compiler::json;

fn lm_available() -> bool {
    flashtex_render_pipeline::fonts::DEFAULT_FONT_DIRS
        .iter()
        .any(|d| std::path::Path::new(d).join("lmroman12-regular.otf").is_file())
}

fn run(args: &[&str], input: &str) -> (Vec<json::Value>, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-render"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn flashtex-render");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let replies = stdout.lines().map(|l| json::parse(l).expect("reply is JSON")).collect();
    (replies, String::from_utf8_lossy(&out.stderr).into_owned())
}

fn compile_line(id: &str, text: &str, caps: Option<&[&str]>) -> String {
    let mut doc = json::Value::obj();
    doc.set("path", json::str_("main.tex"));
    doc.set("text", json::str_(text));
    let mut payload = json::Value::obj();
    payload.set("project_id", json::str_("cli"));
    payload.set("revision", json::num(3.0));
    payload.set("entry_path", json::str_("main.tex"));
    payload.set("documents", json::Value::Arr(vec![doc]));
    if let Some(c) = caps {
        payload.set("layout_capabilities", json::Value::Arr(c.iter().map(|s| json::str_(*s)).collect()));
    }
    let mut v = json::Value::obj();
    v.set("protocol_version", json::num(1.0));
    v.set("id", json::str_(id));
    v.set("type", json::str_("compile"));
    v.set("payload", payload);
    json::write(&v) + "\n"
}

#[test]
fn worker_answers_each_line_and_fails_closed_on_unknown_versions() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let dir = std::env::temp_dir().join(format!("flashtex-render-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let v2_path = dir.join("out.v2.json");
    let pdf_path = dir.join("out.pdf");
    let mut input = String::new();
    input.push_str(&compile_line("a", "\\begin{document}Hello $\\frac{1}{2}$ wörld.\\end{document}", Some(&["rules-v1", "nope"])));
    input.push_str("{\"protocol_version\":2,\"id\":\"b\",\"type\":\"compile\",\"payload\":{}}\n");
    input.push_str("{\"protocol_version\":1,\"id\":\"c\",\"type\":\"render\",\"payload\":{}}\n");
    input.push_str("not json\n");
    input.push_str(&compile_line("d", "\\begin{document}Second request.\\end{document}", None));
    let (replies, stderr) = run(&["--v2", v2_path.to_str().unwrap(), "--pdf", pdf_path.to_str().unwrap(), "--timing"], &input);
    assert_eq!(replies.len(), 5, "one reply per line: {stderr}");

    let a = &replies[0];
    assert_eq!(a.get("id").and_then(|v| v.as_str()), Some("a"));
    assert_eq!(a.get("type").and_then(|v| v.as_str()), Some("compile_result"));
    let pa = a.get("payload").unwrap();
    assert_eq!(pa.get("status").and_then(|v| v.as_str()), Some("ok"));
    let caps = pa.get("layout_capabilities").and_then(|v| v.as_arr()).unwrap();
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].as_str(), Some("rules-v1"));
    let items = pa.get("pages").and_then(|v| v.as_arr()).unwrap()[0].get("items").and_then(|v| v.as_arr()).unwrap();
    assert!(items.iter().any(|i| i.get("kind").and_then(|v| v.as_str()) == Some("rule")));
    assert!(items.iter().all(|i| i.get("font").is_none()), "font hints were not requested");
    let w = items.iter().find(|i| i.get("text").and_then(|v| v.as_str()) == Some("wörld.")).expect("wörld. item");
    let src = w.get("source").unwrap();
    assert_eq!(src.get("path").and_then(|v| v.as_str()), Some("main.tex"));
    let (s, e) = (src.get("start_byte").unwrap().as_i64().unwrap() as usize, src.get("end_byte").unwrap().as_i64().unwrap() as usize);
    assert_eq!(&"\\begin{document}Hello $\\frac{1}{2}$ wörld.\\end{document}"[s..e], "wörld.");

    let b = &replies[1];
    assert_eq!(b.get("type").and_then(|v| v.as_str()), Some("error"));
    assert_eq!(b.get("payload").unwrap().get("code").and_then(|v| v.as_str()), Some("unsupported_protocol_version"));
    let c = &replies[2];
    assert_eq!(c.get("payload").unwrap().get("code").and_then(|v| v.as_str()), Some("unsupported_type"));
    let n = &replies[3];
    assert_eq!(n.get("payload").unwrap().get("code").and_then(|v| v.as_str()), Some("malformed_json"));
    let d = &replies[4];
    assert_eq!(d.get("id").and_then(|v| v.as_str()), Some("d"));
    assert!(d.get("payload").unwrap().get("layout_capabilities").is_none(), "omitted when not requested");

    // --v2 holds the LAST successful request; --pdf too.
    let v2 = json::parse(&std::fs::read_to_string(&v2_path).unwrap()).unwrap();
    assert_eq!(v2.get("id").and_then(|v| v.as_str()), Some("d"));
    assert_eq!(v2.get("protocol_version").and_then(|v| v.as_i64()), Some(2));
    let pdf = std::fs::read(&pdf_path).unwrap();
    assert!(pdf.starts_with(b"%PDF-1."));
    assert!(stderr.contains("rendered in"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn malformed_capability_lists_fail_the_request_not_the_worker() {
    if !lm_available() {
        return;
    }
    let mut input = String::new();
    input.push_str("{\"protocol_version\":1,\"id\":\"x\",\"type\":\"compile\",\"payload\":{\"project_id\":\"p\",\"revision\":1,\"entry_path\":\"main.tex\",\"documents\":[{\"path\":\"main.tex\",\"text\":\"a\"}],\"layout_capabilities\":[\"rules-v1\",\"rules-v1\"]}}\n");
    input.push_str(&compile_line("y", "\\begin{document}ok\\end{document}", Some(&[])));
    let (replies, _) = run(&[], &input);
    assert_eq!(replies[0].get("payload").unwrap().get("status").and_then(|v| v.as_str()), Some("failed"));
    let py = replies[1].get("payload").unwrap();
    assert_eq!(py.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(py.get("layout_capabilities").and_then(|v| v.as_arr()).map(Vec::len), Some(0));
}
