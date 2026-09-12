//! \left / \right delimiters and the \nu control word (FT-002 rev 12).
//!
//! Reference fixtures 13 and 14 were blocked on these. The tests check the
//! blockers are gone, that malformed input still recovers, that nesting is
//! bounded, and — the part that matters most — that source ranges still slice.

use flashtex_compiler::json::{self, Value};
use flashtex_compiler::protocol::handle_line;

fn compile(project: &str, text: &str) -> Value {
    let mut doc = Value::obj();
    doc.set("path", json::str_("main.tex"));
    doc.set("text", json::str_(text));
    let mut payload = Value::obj();
    payload.set("project_id", json::str_(project));
    payload.set("revision", Value::Num(1.0));
    payload.set("entry_path", json::str_("main.tex"));
    payload.set("documents", Value::Arr(vec![doc]));
    let mut env = Value::obj();
    env.set("protocol_version", Value::Num(1.0));
    env.set("id", json::str_(project));
    env.set("type", json::str_("compile"));
    env.set("payload", payload);
    json::parse(&handle_line(&json::write(&env))).expect("valid reply")
}

fn items(v: &Value) -> Vec<Value> {
    v.get("payload")
        .and_then(|p| p.get("pages"))
        .and_then(|p| p.as_arr())
        .map(|pages| {
            pages
                .iter()
                .flat_map(|pg| {
                    pg.get("items")
                        .and_then(|i| i.as_arr())
                        .cloned()
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rendered(v: &Value) -> String {
    items(v)
        .iter()
        .map(|i| {
            i.get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn status(v: &Value) -> String {
    v.get("payload")
        .and_then(|p| p.get("status"))
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string()
}

fn messages(v: &Value) -> Vec<String> {
    v.get("payload")
        .and_then(|p| p.get("diagnostics"))
        .and_then(|d| d.as_arr())
        .map(|ds| {
            ds.iter()
                .map(|d| {
                    d.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("")
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Every span must slice the source it names. This is the invariant the whole
/// editor depends on, and a new construct is exactly where it tends to break.
fn assert_spans_slice(v: &Value, text: &str) {
    let raw = text.as_bytes();
    for item in items(v) {
        let src = item.get("source").expect("source");
        let a = src.get("start_byte").unwrap().as_i64().unwrap() as usize;
        let b = src.get("end_byte").unwrap().as_i64().unwrap() as usize;
        assert!(a <= b && b <= raw.len(), "span {a}..{b} out of range");
        assert!(
            text.is_char_boundary(a) && text.is_char_boundary(b),
            "span {a}..{b} is not on a character boundary"
        );
        let _ = &text[a..b];
    }
}

#[test]
fn balanced_delimiters_render_without_a_diagnostic() {
    let text = "$\\left( \\frac{a}{b} \\right)$\n";
    let reply = compile("delim-balanced", text);
    let out = rendered(&reply);
    assert!(out.starts_with('('), "left delimiter missing: {out:?}");
    assert!(out.ends_with(')'), "right delimiter missing: {out:?}");
    assert!(
        !out.contains("\\left") && !out.contains("\\right"),
        "the command name leaked into rendered output: {out:?}"
    );
    assert!(
        !messages(&reply)
            .iter()
            .any(|m| m.contains("not supported in math mode")),
        "delimiters still reported unsupported: {:?}",
        messages(&reply)
    );
    assert_spans_slice(&reply, text);
}

#[test]
fn nu_renders_as_greek_and_is_exportable() {
    let text = "$\\nu + \\mu$\n";
    let reply = compile("delim-nu", text);
    assert_eq!(rendered(&reply), "\u{3BD} + \u{3BC}");
    assert_eq!(status(&reply), "ok", "nu should not need recovery");
    assert!(
        !messages(&reply)
            .iter()
            .any(|m| m.contains("will not survive PDF export")),
        "nu must be exportable: {:?}",
        messages(&reply)
    );
    assert_spans_slice(&reply, text);
}

#[test]
fn an_unmatched_left_is_diagnosed_and_still_typesets() {
    let text = "$\\left( x$\n";
    let reply = compile("delim-unmatched", text);
    assert_eq!(status(&reply), "recovered");
    assert!(
        messages(&reply)
            .iter()
            .any(|m| m.contains("without a matching")),
        "unmatched \\left not reported: {:?}",
        messages(&reply)
    );
    assert!(rendered(&reply).contains('x'), "content was lost");
    assert_spans_slice(&reply, text);
}

#[test]
fn a_stray_right_is_diagnosed_and_still_typesets() {
    let text = "$x \\right)$\n";
    let reply = compile("delim-stray", text);
    assert_eq!(status(&reply), "recovered");
    assert!(
        messages(&reply).iter().any(|m| m.contains("no matching")),
        "stray \\right not reported: {:?}",
        messages(&reply)
    );
    assert!(rendered(&reply).contains('x'));
    assert_spans_slice(&reply, text);
}

#[test]
fn the_null_delimiter_pairs_without_rendering() {
    let text = "$\\left. x \\right|$\n";
    let reply = compile("delim-null", text);
    assert_eq!(status(&reply), "ok", "a null delimiter is legal TeX");
    assert!(rendered(&reply).contains('|'), "right delimiter missing");
    assert_spans_slice(&reply, text);
}

#[test]
fn deep_delimiter_nesting_is_bounded_not_a_crash() {
    for depth in [8usize, 64, 200, 2000] {
        let text = format!(
            "${}x{}$\n",
            "\\left( ".repeat(depth),
            " \\right)".repeat(depth)
        );
        let reply = compile(&format!("delim-deep-{depth}"), &text);
        assert_eq!(
            reply.get("type").and_then(|t| t.as_str()),
            Some("compile_result"),
            "depth {depth} did not produce a reply"
        );
        assert_spans_slice(&reply, &text);
    }
}

/// GH38: a command that is not supported must say so wherever it appears,
/// including inside a section title. This arm used to be a silent drop, which
/// lost fourteen diagnostics on the HW1 source.
#[test]
fn unsupported_commands_inside_a_title_are_still_diagnosed() {
    let text = "\\section{Title \\hfill with \\normalfont commands}\nBody.\n";
    let reply = compile("gh38-title", text);
    let msgs = messages(&reply);
    assert!(
        msgs.iter().any(|m| m.contains("\\hfill")),
        "\\hfill in a title was dropped silently: {msgs:?}"
    );
    assert!(
        msgs.iter().any(|m| m.contains("\\normalfont")),
        "\\normalfont in a title was dropped silently: {msgs:?}"
    );
    // The title must still typeset the words around the unsupported commands.
    let out = rendered(&reply);
    assert!(
        out.contains("Title") && out.contains("commands"),
        "title text lost: {out:?}"
    );
    assert_spans_slice(&reply, text);
}

/// Math in a section title is ordinary LaTeX and must NOT be reported as a
/// problem. Guarding the fix above from over-reaching.
#[test]
fn math_inside_a_title_is_not_reported_as_unsupported() {
    let text = "\\section{Measured $x^2$ heading}\nBody.\n";
    let reply = compile("gh38-title-math", text);
    let msgs = messages(&reply);
    assert!(
        !msgs
            .iter()
            .any(|m| m.contains("math is not supported inside")),
        "legal math in a title was reported as a problem: {msgs:?}"
    );
    assert_spans_slice(&reply, text);
}

/// GH40: the HW1 set-membership commands must render, and must be exportable,
/// adding no new diagnostics of their own.
#[test]
fn set_membership_commands_render_and_export_cleanly() {
    let text = "$\\in \\ni \\notin \\subset \\subseteq \\supset \\supseteq \\cup \\cap \\emptyset \\forall \\exists$\n";
    let reply = compile("gh40-membership", text);
    assert_eq!(
        status(&reply),
        "ok",
        "membership commands should need no recovery"
    );
    assert_eq!(
        rendered(&reply),
        "\u{2208} \u{220B} \u{2209} \u{2282} \u{2286} \u{2283} \u{2287} \u{222A} \u{2229} \u{2205} \u{2200} \u{2203}"
    );
    assert!(
        messages(&reply).is_empty(),
        "membership commands introduced diagnostics: {:?}",
        messages(&reply)
    );
    assert_spans_slice(&reply, text);
}
