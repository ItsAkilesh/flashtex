//! Reproducible std-only latency measurement for incremental compilation.

use flashtex_compiler::incremental::Session;
use flashtex_compiler::layout::LayoutConstraints;
use std::hint::black_box;
use std::time::{Duration, Instant};

const PARAGRAPHS: usize = 500;
const SAMPLES: usize = 50;

fn fixture(word: &str) -> String {
    let mut source = String::from("\\newcommand{\\term}{FlashTeX}\n\n");
    for index in 0..PARAGRAPHS {
        source.push_str(&format!(
            "Paragraph {index} uses \\term and {word} prose with inline math $x_{index}^2 + \\alpha$ and a nested fraction $\\frac{{a_{index}}}{{b+1}}$. The remaining sentence makes this a representative multi-line typesetting block for incremental layout measurement.\n\n"
        ));
    }
    source
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn percentile(samples: &mut [Duration], percentile: f64) -> Duration {
    samples.sort_unstable();
    let index = ((samples.len() - 1) as f64 * percentile).ceil() as usize;
    samples[index]
}

/// Prints the summary and returns the p95 in milliseconds, so the caller can
/// state the remaining budget without recomputing it.
fn summary(name: &str, mut samples: Vec<Duration>) -> f64 {
    let median = percentile(&mut samples.clone(), 0.50);
    let p95 = percentile(&mut samples, 0.95);
    println!(
        "{name}: median={:.3} ms p95={:.3} ms",
        milliseconds(median),
        milliseconds(p95)
    );
    milliseconds(p95)
}

fn main() {
    let base = fixture("steady");
    let edited = base.replacen("Paragraph 250 uses", "Paragraph 250 now uses", 1);
    let macro_edited = base.replacen("{FlashTeX}", "{QuickTeX}", 1);
    let constraints = LayoutConstraints::default();
    println!(
        "fixture: generated-500-paragraphs, bytes={}, paragraphs={}, samples={}, math+macro per paragraph",
        base.len(), PARAGRAPHS, SAMPLES
    );

    let mut first_session = Session::new();
    let started = Instant::now();
    let first = first_session.compile(black_box(&base), constraints);
    let cold_first = started.elapsed();
    println!(
        "cold first compile: {:.3} ms, stats={:?}",
        milliseconds(cold_first),
        first.stats
    );

    let mut cold = Vec::with_capacity(SAMPLES);
    let mut warm = Vec::with_capacity(SAMPLES);
    let mut edit = Vec::with_capacity(SAMPLES);
    let mut macro_edit = Vec::with_capacity(SAMPLES);
    let mut representative_edit = None;
    let mut representative_macro_edit = None;
    for _ in 0..SAMPLES {
        let mut session = Session::new();
        let start = Instant::now();
        black_box(session.compile(black_box(&base), constraints));
        cold.push(start.elapsed());

        let start = Instant::now();
        black_box(session.compile(black_box(&base), constraints));
        warm.push(start.elapsed());

        let start = Instant::now();
        let result = session.compile(black_box(&edited), constraints);
        edit.push(start.elapsed());
        representative_edit = Some(result.stats);

        let mut macro_session = Session::new();
        black_box(macro_session.compile(black_box(&base), constraints));
        let start = Instant::now();
        let result = macro_session.compile(black_box(&macro_edited), constraints);
        macro_edit.push(start.elapsed());
        representative_macro_edit = Some(result.stats);
    }
    let _ = summary("cold", cold);
    let _ = summary("warm unchanged", warm);
    let edit_p95 = summary("one-word edit", edit);
    println!("edit stats: {:?}", representative_edit.expect("sample"));
    let _ = summary("global macro edit", macro_edit);
    println!(
        "macro edit stats: {:?}",
        representative_macro_edit.expect("sample")
    );
    // FT-002 rev 6 acceptance: warm edit timing includes compiler work and leaves
    // the native paint gate EXPLICIT. The product requirement is keystroke to
    // visible matching output. Everything measured above stops at the compiler's
    // reply; no estimate of the rest is folded in, because a folded-in guess
    // would read as a product number and it is not one.
    println!();
    println!("--- what these numbers are, and are not ---");
    println!("measured here:     compiler work only, from request line to reply line");
    println!("compiler warm-edit p95 (NOT PRODUCT NUMBER): {edit_p95:.3} ms");
    println!(
        "remaining compiler-only budget against 200 ms (NOT PRODUCT NUMBER): {:.3} ms",
        200.0 - edit_p95
    );
    println!("NOT measured here: UI paint, scheduling, IPC transport, PDF writing,");
    println!("                   and the native shell's own work. Those are outside");
    println!("                   this crate and are gated by FT-003 and FT-008.");
    println!("PRODUCT TARGET:    <200 ms keystroke-to-visible-output remains UNPROVEN.");
    println!("                   It can only be established by measuring the real app.");
}
