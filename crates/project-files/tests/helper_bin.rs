//! The `flashtex-project-files --root DIR` JSON Lines host, driven as a child
//! process the way the Mac shell drives it. Temp directories only.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use flashtex_project_files::json::Json;
use flashtex_project_files::sha256_hex;

mod common;

fn run(root: &std::path::Path, requests: &[&str]) -> Vec<Json> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_flashtex-project-files"))
        .arg("--root")
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn helper");
    {
        let mut stdin = child.stdin.take().unwrap();
        for r in requests {
            writeln!(stdin, "{r}").unwrap();
        }
    }
    let stdout = child.stdout.take().unwrap();
    let replies: Vec<Json> = BufReader::new(stdout)
        .lines()
        .map(|l| Json::parse(&l.unwrap()).expect("reply is JSON"))
        .collect();
    let status = child.wait().unwrap();
    assert!(status.success(), "helper exited {status:?}");
    replies
}

fn payload<'a>(reply: &'a Json, id: &str) -> &'a Json {
    assert_eq!(reply.get("id").and_then(Json::as_str), Some(id));
    reply
        .get("payload")
        .unwrap_or_else(|| panic!("expected payload, got {}", reply.to_string_compact()))
}

fn error_code<'a>(reply: &'a Json, id: &str) -> &'a str {
    assert_eq!(reply.get("id").and_then(Json::as_str), Some(id));
    reply
        .get("error")
        .and_then(|e| e.get("code"))
        .and_then(Json::as_str)
        .unwrap_or_else(|| panic!("expected error, got {}", reply.to_string_compact()))
}

#[test]
fn ping_read_status_save_and_conflict_over_the_wire() {
    let tmp = common::TempDir::new("helper-bin");
    let root = tmp.root();
    std::fs::write(root.join("a.tex"), "hello\n").unwrap();
    let h1 = sha256_hex(b"hello\n");
    let h2 = sha256_hex(b"hello\nworld\n");
    let replies = run(
        root,
        &[
            r#"{"id":"1","operation":"ping"}"#,
            r#"{"id":"2","operation":"read","path":"a.tex"}"#,
            &format!(
                r#"{{"id":"3","operation":"status","path":"a.tex","expected_sha256":"{h1}"}}"#
            ),
            &format!(
                r#"{{"id":"4","operation":"save","path":"a.tex","text":"hello\nworld\n","expected":"{h1}"}}"#
            ),
            &format!(
                r#"{{"id":"5","operation":"save","path":"a.tex","text":"again\n","expected":"{h1}"}}"#
            ),
            &format!(
                r#"{{"id":"6","operation":"status","path":"a.tex","expected_sha256":"{h1}"}}"#
            ),
            r#"{"id":"7","operation":"read","path":"missing.tex"}"#,
            r#"{"id":"8","operation":"status","path":"missing.tex","expected_sha256":null}"#,
            r#"{"id":"9","operation":"save","path":"new/dir/n.tex","text":"n","expected":"new"}"#,
            r#"{"id":"10","operation":"save","path":"new/dir/n.tex","text":"n2","expected":"new"}"#,
            &format!(
                r#"{{"id":"11","operation":"save","path":"a.tex","text":"forced\n","expected":"{h1}","force":true}}"#
            ),
        ],
    );
    assert_eq!(replies.len(), 11);
    let p = payload(&replies[0], "1");
    assert_eq!(
        p.get("protocol").and_then(Json::as_str),
        Some("project-files-v1")
    );
    let p = payload(&replies[1], "2");
    assert_eq!(p.get("text").and_then(Json::as_str), Some("hello\n"));
    assert_eq!(p.get("sha256").and_then(Json::as_str), Some(h1.as_str()));
    assert_eq!(
        payload(&replies[2], "3")
            .get("state")
            .and_then(Json::as_str),
        Some("unchanged")
    );
    let p = payload(&replies[3], "4");
    assert_eq!(p.get("outcome").and_then(Json::as_str), Some("saved"));
    assert_eq!(
        p.get("receipt")
            .and_then(|r| r.get("sha256"))
            .and_then(Json::as_str),
        Some(h2.as_str())
    );
    let p = payload(&replies[4], "5");
    assert_eq!(p.get("outcome").and_then(Json::as_str), Some("conflict"));
    let c = p.get("conflict").unwrap();
    assert_eq!(
        c.get("kind").and_then(Json::as_str),
        Some("modified_externally")
    );
    assert_eq!(c.get("ours").and_then(Json::as_str), Some(h1.as_str()));
    assert_eq!(c.get("theirs").and_then(Json::as_str), Some(h2.as_str()));
    assert_eq!(
        payload(&replies[5], "6")
            .get("state")
            .and_then(Json::as_str),
        Some("modified")
    );
    assert_eq!(
        payload(&replies[6], "7").get("exists"),
        Some(&Json::Bool(false))
    );
    assert_eq!(
        payload(&replies[7], "8")
            .get("state")
            .and_then(Json::as_str),
        Some("unchanged")
    );
    assert_eq!(
        payload(&replies[8], "9")
            .get("outcome")
            .and_then(Json::as_str),
        Some("saved")
    );
    let c = payload(&replies[9], "10").get("conflict").unwrap();
    assert_eq!(c.get("kind").and_then(Json::as_str), Some("already_exists"));
    assert_eq!(
        payload(&replies[10], "11")
            .get("outcome")
            .and_then(Json::as_str),
        Some("saved")
    );
    assert_eq!(
        std::fs::read_to_string(root.join("a.tex")).unwrap(),
        "forced\n"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("new/dir/n.tex")).unwrap(),
        "n"
    );
}

#[test]
fn refusals_and_bad_requests_are_errors_and_write_nothing() {
    let tmp = common::TempDir::new("helper-bin-refuse");
    let root = tmp.root();
    let outside = common::TempDir::new("helper-bin-outside");
    let secret = outside.root().join("secret.tex");
    std::fs::write(&secret, "secret\n").unwrap();
    common::symlink_file(&secret, &root.join("link.tex"));
    let replies = run(
        root,
        &[
            r#"{"id":"1","operation":"save","path":"link.tex","text":"x","expected":"any","force":true}"#,
            r#"{"id":"2","operation":"read","path":"link.tex"}"#,
            r#"{"id":"3","operation":"read","path":"../secret.tex"}"#,
            r#"{"id":"4","operation":"read","path":"/etc/hosts"}"#,
            r#"{"id":"5","operation":"nope"}"#,
            r#"{"id":"6","operation":"save","path":"a.tex","text":"x","expected":"zz"}"#,
            r#"{"id":"7","operation":"save","path":"a.tex","text":"x","expected":"new","force":"yes"}"#,
            r#"not json"#,
            r#"{"id":"8","operation":"read"}"#,
        ],
    );
    assert_eq!(replies.len(), 9);
    assert_eq!(error_code(&replies[0], "1"), "refused");
    assert_eq!(error_code(&replies[1], "2"), "refused");
    assert_eq!(error_code(&replies[2], "3"), "invalid_path");
    assert_eq!(error_code(&replies[3], "4"), "invalid_path");
    assert_eq!(error_code(&replies[4], "5"), "unsupported_operation");
    assert_eq!(error_code(&replies[5], "6"), "invalid_request");
    assert_eq!(error_code(&replies[6], "7"), "invalid_request");
    assert_eq!(replies[7].get("id"), Some(&Json::Null));
    assert_eq!(
        replies[7]
            .get("error")
            .and_then(|e| e.get("code"))
            .and_then(Json::as_str),
        Some("invalid_request")
    );
    assert_eq!(error_code(&replies[8], "8"), "invalid_request");
    assert_eq!(std::fs::read_to_string(&secret).unwrap(), "secret\n");
    assert!(!root.join("a.tex").exists());
}

/// `remove` over the wire: a real unlink reports `removed:true`, a path that
/// was never there reports `removed:false` rather than failing (the library's
/// own `Ok(false)`), and a second `remove` of the same file converges on
/// `removed:false` instead of erroring.
#[test]
fn remove_over_the_wire_reports_whether_anything_was_there() {
    let tmp = common::TempDir::new("helper-bin-remove");
    let root = tmp.root();
    std::fs::write(root.join("a.tex"), "bye\n").unwrap();
    std::fs::create_dir(root.join("sub")).unwrap();
    std::fs::write(root.join("sub/b.tex"), "nested\n").unwrap();
    let replies = run(
        root,
        &[
            r#"{"id":"1","operation":"remove","path":"a.tex"}"#,
            r#"{"id":"2","operation":"remove","path":"a.tex"}"#,
            r#"{"id":"3","operation":"remove","path":"never-existed.tex"}"#,
            r#"{"id":"4","operation":"remove","path":"sub/b.tex"}"#,
            r#"{"id":"5","operation":"read","path":"a.tex"}"#,
            r#"{"id":"6","operation":"remove","path":"../outside.tex"}"#,
            r#"{"id":"7","operation":"remove"}"#,
        ],
    );
    assert_eq!(replies.len(), 7);
    for (index, id, removed) in [(0, "1", true), (1, "2", false), (2, "3", false), (3, "4", true)] {
        let p = payload(&replies[index], id);
        assert_eq!(p.get("removed"), Some(&Json::Bool(removed)), "id={id}");
    }
    assert_eq!(
        payload(&replies[0], "1").get("path").and_then(Json::as_str),
        Some("a.tex")
    );
    assert_eq!(
        payload(&replies[4], "5").get("exists"),
        Some(&Json::Bool(false))
    );
    assert_eq!(error_code(&replies[5], "6"), "invalid_path");
    assert_eq!(error_code(&replies[6], "7"), "invalid_request");
    assert!(!root.join("a.tex").exists());
    assert!(!root.join("sub/b.tex").exists());
    assert!(root.join("sub").is_dir(), "the directory itself stays");
}

/// `rename` over the wire. The successful case moves the entry with no second
/// copy left behind; an occupied `to` and a missing `from` are *conflicts*
/// (payloads the shell must show), not errors; a cross-directory pair is an
/// `invalid_request`; and a malformed request is rejected before anything is
/// touched.
#[test]
fn rename_over_the_wire_moves_or_reports_a_conflict() {
    let tmp = common::TempDir::new("helper-bin-rename");
    let root = tmp.root();
    std::fs::write(root.join("old.tex"), "chapter\n").unwrap();
    std::fs::write(root.join("taken.tex"), "someone else\n").unwrap();
    std::fs::create_dir(root.join("sub")).unwrap();
    std::fs::write(root.join("sub/n.tex"), "nested\n").unwrap();
    let h = sha256_hex(b"chapter\n");
    let replies = run(
        root,
        &[
            r#"{"id":"1","operation":"rename","from":"old.tex","to":"new.tex"}"#,
            r#"{"id":"2","operation":"read","path":"new.tex"}"#,
            r#"{"id":"3","operation":"read","path":"old.tex"}"#,
            r#"{"id":"4","operation":"rename","from":"new.tex","to":"taken.tex"}"#,
            r#"{"id":"5","operation":"rename","from":"gone.tex","to":"other.tex"}"#,
            r#"{"id":"6","operation":"rename","from":"sub/n.tex","to":"sub/m.tex"}"#,
            r#"{"id":"7","operation":"rename","from":"new.tex","to":"sub/new.tex"}"#,
            r#"{"id":"8","operation":"rename","from":"new.tex"}"#,
            r#"{"id":"9","operation":"rename","from":"new.tex","to":"../escape.tex"}"#,
        ],
    );
    assert_eq!(replies.len(), 9);

    let p = payload(&replies[0], "1");
    assert_eq!(p.get("outcome").and_then(Json::as_str), Some("renamed"));
    assert_eq!(p.get("from").and_then(Json::as_str), Some("old.tex"));
    assert_eq!(p.get("to").and_then(Json::as_str), Some("new.tex"));
    let p = payload(&replies[1], "2");
    assert_eq!(p.get("text").and_then(Json::as_str), Some("chapter\n"));
    assert_eq!(
        p.get("sha256").and_then(Json::as_str),
        Some(h.as_str()),
        "content identity is carried over unchanged"
    );
    assert_eq!(
        payload(&replies[2], "3").get("exists"),
        Some(&Json::Bool(false)),
        "no second copy at the old name"
    );

    let p = payload(&replies[3], "4");
    assert_eq!(p.get("outcome").and_then(Json::as_str), Some("conflict"));
    let c = p.get("conflict").unwrap();
    assert_eq!(c.get("kind").and_then(Json::as_str), Some("already_exists"));
    assert_eq!(c.get("path").and_then(Json::as_str), Some("taken.tex"));
    assert_eq!(c.get("ours"), Some(&Json::Null));
    assert_eq!(c.get("theirs"), Some(&Json::Null));

    let c = payload(&replies[4], "5").get("conflict").unwrap();
    assert_eq!(
        c.get("kind").and_then(Json::as_str),
        Some("deleted_externally")
    );
    assert_eq!(c.get("path").and_then(Json::as_str), Some("gone.tex"));

    assert_eq!(
        payload(&replies[5], "6")
            .get("outcome")
            .and_then(Json::as_str),
        Some("renamed")
    );
    assert_eq!(error_code(&replies[6], "7"), "invalid_request");
    assert_eq!(error_code(&replies[7], "8"), "invalid_request");
    assert_eq!(error_code(&replies[8], "9"), "invalid_path");

    assert_eq!(
        std::fs::read_to_string(root.join("new.tex")).unwrap(),
        "chapter\n"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("taken.tex")).unwrap(),
        "someone else\n",
        "the refused target was never clobbered"
    );
    assert!(!root.join("old.tex").exists());
    assert!(!root.join("sub/n.tex").exists());
    assert_eq!(
        std::fs::read_to_string(root.join("sub/m.tex")).unwrap(),
        "nested\n"
    );
    assert!(!root.join("sub/new.tex").exists());
}

/// `remove` and `rename` inherit the same refusals every other operation has:
/// a symlink is neither followed nor moved, and the file it points to outside
/// the root is untouched.
#[test]
fn remove_and_rename_refuse_symlinks_over_the_wire() {
    let tmp = common::TempDir::new("helper-bin-rename-refuse");
    let root = tmp.root();
    let outside = common::TempDir::new("helper-bin-rename-outside");
    let secret = outside.root().join("secret.tex");
    std::fs::write(&secret, "secret\n").unwrap();
    common::symlink_file(&secret, &root.join("link.tex"));
    let replies = run(
        root,
        &[
            r#"{"id":"1","operation":"remove","path":"link.tex"}"#,
            r#"{"id":"2","operation":"rename","from":"link.tex","to":"moved.tex"}"#,
        ],
    );
    assert_eq!(replies.len(), 2);
    assert_eq!(error_code(&replies[0], "1"), "refused");
    assert_eq!(error_code(&replies[1], "2"), "refused");
    assert_eq!(std::fs::read_to_string(&secret).unwrap(), "secret\n");
    assert!(std::fs::symlink_metadata(root.join("link.tex")).is_ok());
    assert!(!root.join("moved.tex").exists());
}

#[test]
fn symlinked_root_is_refused_at_startup() {
    let real = common::TempDir::new("helper-bin-realroot");
    let holder = common::TempDir::new("helper-bin-holder");
    let link = holder.root().join("root-link");
    common::symlink_dir(real.root(), &link);
    let out = Command::new(env!("CARGO_BIN_EXE_flashtex-project-files"))
        .arg("--root")
        .arg(&link)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot open root"));
}

fn list_files_json(payload: &Json) -> Vec<String> {
    payload
        .get("files")
        .and_then(|f| match f {
            Json::Array(items) => Some(items),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected files array, got {}", payload.to_string_compact()))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// `list` over the wire: every project file under the root (or a given
/// subdirectory), recursively, sorted, with non-project files, a hidden
/// `.flashtex/` directory (created for real by taking the project lock, not
/// hand-placed) and a symlinked entry all silently excluded.
#[test]
fn list_over_the_wire_finds_project_files_and_excludes_the_rest() {
    let tmp = common::TempDir::new("helper-bin-list");
    let root = tmp.root();
    std::fs::write(root.join("main.tex"), "main\n").unwrap();
    std::fs::write(root.join("refs.bib"), "@misc{x}\n").unwrap();
    std::fs::write(root.join("notes.md"), "not latex\n").unwrap();
    std::fs::create_dir_all(root.join("chapters")).unwrap();
    std::fs::write(root.join("chapters/ch1.tex"), "one\n").unwrap();
    std::fs::create_dir_all(root.join(".hidden")).unwrap();
    std::fs::write(root.join(".hidden/skip.tex"), "skip\n").unwrap();
    let outside = common::TempDir::new("helper-bin-list-outside");
    let secret = outside.root().join("secret.tex");
    std::fs::write(&secret, "secret\n").unwrap();
    common::symlink_file(&secret, &root.join("alias.tex"));
    // A real lock/journal write, so `.flashtex/` genuinely exists on disk,
    // not just as a name this test happens to also use.
    {
        let real_root = flashtex_project_files::ProjectRoot::open(root).unwrap();
        drop(real_root.lock().unwrap());
    }

    let replies = run(
        root,
        &[
            r#"{"id":"1","operation":"list"}"#,
            r#"{"id":"2","operation":"list","path":"chapters"}"#,
            r#"{"id":"3","operation":"list","path":"../escape"}"#,
        ],
    );
    assert_eq!(replies.len(), 3);

    let p = payload(&replies[0], "1");
    assert_eq!(p.get("path").and_then(Json::as_str), Some(""));
    assert_eq!(p.get("truncated"), Some(&Json::Bool(false)));
    assert_eq!(
        list_files_json(p),
        vec!["chapters/ch1.tex", "main.tex", "refs.bib"]
    );
    assert!(root.join(".flashtex/project.lock").exists());

    let p = payload(&replies[1], "2");
    assert_eq!(p.get("path").and_then(Json::as_str), Some("chapters"));
    assert_eq!(list_files_json(p), vec!["chapters/ch1.tex"]);

    assert_eq!(error_code(&replies[2], "3"), "invalid_path");

    assert_eq!(std::fs::read_to_string(&secret).unwrap(), "secret\n");
}

/// The helper's `LIST_LIMIT` is not adjustable over the wire (unlike the
/// library's own `list_files`, which takes an explicit `limit`), so the
/// over-the-cap behavior itself is covered directly against the library in
/// `list_files_respects_the_limit_and_reports_truncation`
/// (`tests/rooted.rs`) — the helper calls the exact same function with the
/// same constant. This just confirms the ordinary, well-under-the-cap case
/// reports `truncated:false` over the wire.
#[test]
fn list_over_the_wire_reports_truncated_false_when_under_the_cap() {
    let tmp = common::TempDir::new("helper-bin-list-small");
    let root = tmp.root();
    std::fs::write(root.join("a.tex"), "a\n").unwrap();
    let replies = run(root, &[r#"{"id":"1","operation":"list"}"#]);
    let p = payload(&replies[0], "1");
    assert_eq!(p.get("truncated"), Some(&Json::Bool(false)));
    assert_eq!(list_files_json(p), vec!["a.tex"]);
}
