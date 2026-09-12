//! Burst-edit and recovery gates.
//!
//! One measured edit says little about a live editor. Real typing is a burst of
//! consecutive edits against a session that never resets, and the failure this
//! guards against is degradation: reuse state that accumulates, caches that grow,
//! or work that creeps back toward a full recompile as the burst goes on.
//!
//! It also guards the harder case — typing through broken syntax. An author
//! opens a brace, types for a while, then closes it. Every intermediate state is
//! malformed, and the compiler must stay correct and fast across all of them.

use flashtex_compiler::incremental::{compile_full, Session};
use flashtex_compiler::layout::LayoutConstraints;
use std::time::{Duration, Instant};

fn base_document(paragraphs: usize) -> String {
    let mut out = String::from("\\documentclass{article}\n\\newcommand{\\proj}{FlashTeX}\n");
    out.push_str("\\begin{document}\n\\section{Burst}\n");
    for i in 0..paragraphs {
        out.push_str(&format!(
            "Paragraph {i} of \\proj{{}} with $x^{{{i}}}$ and enough words to wrap.\n\n"
        ));
    }
    out.push_str("\\end{document}\n");
    out
}

fn percentile(samples: &mut [Duration], q: f64) -> f64 {
    samples.sort_unstable();
    let idx = (((samples.len() - 1) as f64) * q).round() as usize;
    samples[idx].as_secs_f64() * 1000.0
}

/// Typing a word one character at a time, as an editor actually sends it.
#[test]
fn a_burst_of_edits_does_not_degrade() {
    let constraints = LayoutConstraints::default();
    let base = base_document(300);
    let anchor = base.find("Paragraph 150").expect("anchor");

    let mut session = Session::new();
    session.compile(&base, constraints);

    let mut typed = String::new();
    let mut samples = Vec::new();
    for c in "incrementally typed sentence here".chars() {
        typed.push(c);
        let text = format!("{}{typed} {}", &base[..anchor], &base[anchor..]);
        let start = Instant::now();
        let result = session.compile(&text, constraints);
        samples.push(start.elapsed());
        assert!(
            result.stats.blocks_reused > 0,
            "burst editing fell back to a full recompile after {:?}",
            typed
        );
    }

    let first_half: Vec<Duration> = samples[..samples.len() / 2].to_vec();
    let second_half: Vec<Duration> = samples[samples.len() / 2..].to_vec();
    let early = percentile(&mut first_half.clone(), 0.95);
    let late = percentile(&mut second_half.clone(), 0.95);

    // Degradation is the failure mode: later keystrokes must not cost
    // dramatically more than earlier ones in the same session.
    assert!(
        late < early * 3.0 + 1.0,
        "burst editing degraded: early p95 {early:.3} ms, late p95 {late:.3} ms"
    );

    // And the end state must still be exactly a clean build.
    let text = format!("{}{typed} {}", &base[..anchor], &base[anchor..]);
    assert_eq!(
        format!("{:#?}", session.compile(&text, constraints).output),
        format!("{:#?}", compile_full(&text, constraints)),
        "after a burst of edits the incremental result diverged from a clean build"
    );
}

/// Typing through a broken construct: every intermediate state is malformed.
#[test]
fn typing_through_broken_syntax_stays_correct_and_recovers() {
    let constraints = LayoutConstraints::default();
    let base = base_document(120);
    let anchor = base.find("Paragraph 60").expect("anchor");

    let mut session = Session::new();
    session.compile(&base, constraints);

    // Open a group, type inside it, then close it. Only the last state is valid.
    let mut fragment = String::from("{");
    let mut states = Vec::new();
    for c in "emphasised text".chars() {
        fragment.push(c);
        states.push(fragment.clone());
    }
    states.push(format!("{fragment}}}"));

    for (step, fragment) in states.iter().enumerate() {
        let text = format!("{}{fragment} {}", &base[..anchor], &base[anchor..]);
        let result = session.compile(&text, constraints);
        assert_eq!(
            format!("{:#?}", result.output),
            format!("{:#?}", compile_full(&text, constraints)),
            "step {step}: incremental diverged from clean while syntax was broken"
        );
        // Output must still be produced: a half-typed group cannot blank the preview.
        assert!(
            result.output.pages.iter().any(|p| !p.items.is_empty()),
            "step {step}: the preview went empty while the author was mid-typing"
        );
    }
}

/// Alternating break and repair many times: state must not accumulate.
#[test]
fn repeated_break_and_repair_cycles_do_not_accumulate_state() {
    let constraints = LayoutConstraints::default();
    let good = base_document(60);
    let broken = good.replacen("Paragraph 30", "Paragraph {30", 1);

    let mut session = Session::new();
    let first_good = format!("{:#?}", session.compile(&good, constraints).output);

    for cycle in 0..25 {
        session.compile(&broken, constraints);
        let repaired = format!("{:#?}", session.compile(&good, constraints).output);
        assert_eq!(
            repaired, first_good,
            "cycle {cycle}: repaired output differed from the original good state"
        );
    }

    assert_eq!(
        format!("{:#?}", session.compile(&good, constraints).output),
        format!("{:#?}", compile_full(&good, constraints)),
        "after 25 break/repair cycles the session diverged from a clean build"
    );
}

/// Rapid capability switching must not leak results between capability sets.
#[test]
fn alternating_capability_requests_never_leak_between_them() {
    use flashtex_compiler::json::{self, Value};
    use flashtex_compiler::protocol::handle_line;

    let text = "$\\frac{a}{b}$\n";
    let build = |caps: &[&str]| {
        let mut doc = Value::obj();
        doc.set("path", json::str_("m.tex"));
        doc.set("text", json::str_(text));
        let mut payload = Value::obj();
        payload.set("project_id", json::str_("burst-caps"));
        payload.set("revision", Value::Num(1.0));
        payload.set("entry_path", json::str_("m.tex"));
        payload.set("documents", Value::Arr(vec![doc]));
        if !caps.is_empty() {
            payload.set(
                "layout_capabilities",
                Value::Arr(caps.iter().map(|c| json::str_(*c)).collect()),
            );
        }
        let mut env = Value::obj();
        env.set("protocol_version", Value::Num(1.0));
        env.set("id", json::str_("c"));
        env.set("type", json::str_("compile"));
        env.set("payload", payload);
        json::write(&env)
    };

    for cycle in 0..20 {
        let with = handle_line(&build(&["rules-v1"]));
        let without = handle_line(&build(&[]));
        assert!(
            with.contains("\"rule\""),
            "cycle {cycle}: rules-v1 was requested but no rule primitive was emitted"
        );
        assert!(
            !without.contains("\"rule\""),
            "cycle {cycle}: a rule primitive leaked to a client that did not request it"
        );
    }
}

/// Issue #21: a valid large project must not produce a reply the consumer will
/// reject. It may deliver fewer pages, but only while saying so explicitly.
#[test]
fn a_large_project_reply_fits_the_transport_frame_and_says_what_it_dropped() {
    use flashtex_compiler::json::{self, Value};
    use flashtex_compiler::protocol::{handle_line, MAX_RESULT_BYTES};

    let mut text = String::from("\\documentclass{article}\n\\begin{document}\n");
    while text.len() < 500_000 {
        text.push_str("Paragraph with a reasonable number of words that wrap across a line.\n\n");
    }
    text.push_str("\\end{document}\n");

    let mut doc = Value::obj();
    doc.set("path", json::str_("main.tex"));
    doc.set("text", json::str_(text));
    let mut payload = Value::obj();
    payload.set("project_id", json::str_("oversized"));
    payload.set("revision", Value::Num(1.0));
    payload.set("entry_path", json::str_("main.tex"));
    payload.set("documents", Value::Arr(vec![doc]));
    let mut env = Value::obj();
    env.set("protocol_version", Value::Num(1.0));
    env.set("id", json::str_("oversized"));
    env.set("type", json::str_("compile"));
    env.set("payload", payload);

    let reply = handle_line(&json::write(&env));
    assert!(
        reply.len() <= MAX_RESULT_BYTES,
        "reply of {} bytes exceeds the {MAX_RESULT_BYTES}-byte transport frame; \
         the consumer would reject it as malformed",
        reply.len()
    );

    let parsed = json::parse(&reply).expect("reply must still be valid JSON");
    let payload = parsed.get("payload").expect("payload");

    let pages = payload
        .get("pages")
        .and_then(|v| v.as_arr())
        .cloned()
        .unwrap_or_default();
    assert!(
        !pages.is_empty(),
        "a valid document must still deliver pages"
    );

    // Dropping pages silently is what the issue explicitly rules out.
    let diagnostics = payload
        .get("diagnostics")
        .and_then(|v| v.as_arr())
        .cloned()
        .unwrap_or_default();
    let announced = diagnostics.iter().any(|d| {
        d.get("message")
            .and_then(|m| m.as_str())
            .is_some_and(|m| m.contains("transport frame") && m.contains("were not delivered"))
    });
    assert!(
        announced,
        "pages were truncated without an explicit diagnostic saying so"
    );
}
