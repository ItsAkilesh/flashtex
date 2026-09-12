//! Measured, pinned gap-and-convergence report against the repo's real
//! `.tex` corpus (`tests/tex-corpus`, FT-011) -- not a description, an
//! assertion. `crates/compiler/tests/corpus_gate.rs` pins this same corpus's
//! compile status; this file pins what `toc-layout` needs from it: how many
//! documents this crate's supported model (`\section`/`\subsection`, the
//! only sectioning `Block::Heading` the compiler ever emits -- see
//! `producer_fixture.rs`'s module docs) can actually find in it, how many
//! use a construct outside that model, how many entries per document on
//! average, and how many convergence passes stabilizing those entries
//! actually takes against the crate's real bounded solver.
//!
//! A failing assertion here means the corpus changed, matching
//! `corpus_gate.rs`'s own philosophy: an improvement (or regression) is
//! visible in the diff, never a silently drifting average.

mod support;

use flashtex_toc_layout::{MAX_CONVERGENCE_ITERATIONS, converge_front_matter_pages};
use support::{
    CorpusCase, CountingModel, GeometryFrontMatterModel, corpus_cases, heading_records,
    uses_unsupported_sectioning,
};

struct CaseMeasurement {
    id: String,
    entry_count: u32,
    uses_unsupported: bool,
    convergence_passes: u32,
    hit_bound: bool,
}

fn measure(case: &CorpusCase) -> CaseMeasurement {
    let parsed = case.parse();
    let entry_count = heading_records(&parsed.blocks).len() as u32;

    let base_model = GeometryFrontMatterModel { entry_count };
    let counting = CountingModel::new(&base_model);
    // initial_guess = 0: "assume no front matter until the real entry count
    // proves otherwise", the same bootstrap a first LaTeX pass makes.
    let result = converge_front_matter_pages(&counting, 0);
    let passes = counting.calls.get();

    CaseMeasurement {
        id: case.id.clone(),
        entry_count,
        uses_unsupported: uses_unsupported_sectioning(case),
        convergence_passes: passes,
        hit_bound: result.is_err() && passes as usize == MAX_CONVERGENCE_ITERATIONS,
    }
}

#[test]
fn measured_sectioning_and_convergence_gaps_across_the_real_corpus() {
    let cases = corpus_cases();
    assert_eq!(
        cases.len(),
        14,
        "tests/tex-corpus/manifest.json case count changed -- update this pin deliberately"
    );

    let measurements: Vec<CaseMeasurement> = cases.iter().map(measure).collect();

    let total_documents = measurements.len();
    let documents_with_supported_sectioning =
        measurements.iter().filter(|m| m.entry_count > 0).count();
    let documents_with_unsupported_sectioning =
        measurements.iter().filter(|m| m.uses_unsupported).count();
    let total_entries: u32 = measurements.iter().map(|m| m.entry_count).sum();
    let entries_per_document_avg = f64::from(total_entries) / total_documents as f64;
    let max_convergence_passes = measurements
        .iter()
        .map(|m| m.convergence_passes)
        .max()
        .expect("corpus is non-empty");
    let any_hit_bound = measurements.iter().any(|m| m.hit_bound);

    eprintln!(
        "--- toc-layout measured gaps against tests/tex-corpus ({total_documents} documents) ---"
    );
    for m in &measurements {
        eprintln!(
            "{:<20} entries={} unsupported_construct={} convergence_passes={}",
            m.id, m.entry_count, m.uses_unsupported, m.convergence_passes
        );
    }
    eprintln!(
        "documents_with_supported_sectioning={documents_with_supported_sectioning}/{total_documents}"
    );
    eprintln!(
        "documents_with_unsupported_sectioning={documents_with_unsupported_sectioning}/{total_documents}"
    );
    eprintln!(
        "total_entries={total_entries} entries_per_document_avg={entries_per_document_avg:.3}"
    );
    eprintln!(
        "max_convergence_passes={max_convergence_passes} any_hit_bound_of_{MAX_CONVERGENCE_ITERATIONS}={any_hit_bound}"
    );

    // Pinned measurements (2026-09-12, main_sha see coordination/daniel-contents.md):
    // tests/tex-corpus is FT-011's 14 small recovery/compatibility projects.
    // None contains any sectioning command at all -- confirmed both by this
    // real parse (Block::Heading count is 0 for every case) and by a raw
    // grep across every corpus file for \section/\subsection/\part/
    // \chapter/\appendix (zero matches; see producer_fixture.rs). The
    // biggest measured gap is therefore not a specific unsupported
    // construct -- it's that the repo's only .tex corpus has no sectioned
    // document at all to exercise this crate's consumer path against yet.
    assert_eq!(total_documents, 14);
    assert_eq!(documents_with_supported_sectioning, 0);
    assert_eq!(documents_with_unsupported_sectioning, 0);
    assert_eq!(total_entries, 0);
    assert!((entries_per_document_avg - 0.0).abs() < f64::EPSILON);
    // Every document converges in exactly 2 passes: pages_for is a constant
    // function of a real (here: zero) entry count, so pass 1 (guess 0 ->
    // corrected count) never matches, and pass 2 (corrected count -> same
    // count) always does. None of the 14 real documents come anywhere near
    // the 8-pass bound; only the adversarial synthetic models in
    // tests/adversarial_bounds.rs actually reach it.
    assert_eq!(max_convergence_passes, 2);
    assert!(!any_hit_bound);
}
