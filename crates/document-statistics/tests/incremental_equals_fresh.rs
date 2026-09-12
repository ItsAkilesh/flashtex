//! The load-bearing property of this crate's rev-2 cache: computing
//! statistics incrementally through [`ProjectCache`] must always agree,
//! exactly, with recomputing everything from scratch with
//! [`Statistics::compute`] — for every document, at every step, over many
//! varied edit sequences. If these two paths ever disagree, that is a real
//! cache bug (stale content served, a document's cache entry corrupted by an
//! unrelated update, ...), not something to loosen the assertion around.
//!
//! This also measures the thing the cache is FOR: across a full sequence,
//! the incremental path must scan strictly fewer source bytes than
//! recomputing every document from scratch at every step.

use flashtex_document_statistics::{ProjectCache, RevisionId, ScanLimit, SourceItem, Statistics};

/// Ground-truth model of one project: `documents[i]` is document `i`'s
/// current item list, and `revisions[i]` is the revision number the test
/// harness will claim for it on its NEXT edit.
struct Model {
    documents: Vec<Vec<SourceItem>>,
    revisions: Vec<u64>,
}

impl Model {
    fn new(n: usize) -> Self {
        Model {
            documents: vec![Vec::new(); n],
            revisions: vec![0; n],
        }
    }

    fn source(i: usize) -> String {
        format!("doc-{i}.tex")
    }

    /// Recompute every document from scratch (no cache involved at all) and
    /// sum the results. This is "fresh".
    fn fresh_totals(&self) -> (usize, usize, usize, u64) {
        let mut words = 0usize;
        let mut math = 0usize;
        let mut pages = 0usize;
        let mut scanned = 0u64;
        for (i, items) in self.documents.iter().enumerate() {
            let stats =
                Statistics::compute(RevisionId::new(Model::source(i), self.revisions[i]), items);
            words += stats.words.words;
            math += stats.math.total;
            pages += stats.pages;
            scanned += stats.scanned_bytes as u64;
        }
        (words, math, pages, scanned)
    }

    fn fresh_one(&self, i: usize) -> Statistics {
        Statistics::compute(
            RevisionId::new(Model::source(i), self.revisions[i]),
            &self.documents[i],
        )
    }
}

/// A tiny deterministic PRNG (xorshift64*) so the property test is
/// reproducible without an external `rand` dependency.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// One mutation applied to the model AND fed through the cache in lockstep.
enum Edit {
    AppendText(String),
    AppendInlineMath(String),
    AppendDisplayMath(String),
    AppendPageMark,
    /// Replace the whole document with new content, WITHOUT bumping the
    /// revision number: same revision id claimed over genuinely different
    /// content. The cache must miss (see `cache.rs`'s dedicated unit test);
    /// this exercises the same rule inside a long, varied sequence instead
    /// of in isolation.
    ReplaceContentSameRevision(Vec<SourceItem>),
    /// A pure no-op re-application of the exact same revision and content:
    /// must be a cache hit that still agrees with fresh.
    Repeat,
}

fn random_text(rng: &mut Rng) -> String {
    const WORDS: &[&str] = &[
        "alpha",
        "well-known",
        "don't",
        "beta,",
        "...",
        "42",
        "你好世界",
        "gamma-delta",
    ];
    let n = 1 + rng.below(5);
    (0..n)
        .map(|_| WORDS[rng.below(WORDS.len())])
        .collect::<Vec<_>>()
        .join(" ")
}

fn random_edit(rng: &mut Rng, current: &[SourceItem]) -> Edit {
    match rng.below(6) {
        0 => Edit::AppendText(random_text(rng)),
        1 => Edit::AppendInlineMath(random_text(rng)),
        2 => Edit::AppendDisplayMath(random_text(rng)),
        3 => Edit::AppendPageMark,
        4 => {
            let mut replaced = current.to_vec();
            replaced.push(SourceItem::text(random_text(rng)));
            Edit::ReplaceContentSameRevision(replaced)
        }
        _ => Edit::Repeat,
    }
}

/// Apply one edit sequence of `steps` steps over `n_docs` documents, checking
/// exact agreement between the incremental (cached) path and the fresh path
/// after EVERY step. Returns (incremental_bytes_scanned, fresh_bytes_scanned)
/// summed over the whole sequence.
fn run_sequence(seed: u64, n_docs: usize, steps: usize) -> (u64, u64) {
    let mut rng = Rng::new(seed);
    let mut model = Model::new(n_docs);
    let mut cache = ProjectCache::new();
    let mut incremental_bytes = 0u64;
    let mut fresh_bytes_running = 0u64;

    for step in 0..steps {
        let doc = rng.below(n_docs);
        let edit = random_edit(&mut rng, &model.documents[doc]);

        let claimed_revision = match &edit {
            Edit::ReplaceContentSameRevision(new_items) => {
                model.documents[doc] = new_items.clone();
                model.revisions[doc] // NOT bumped: deliberate stale-id reuse.
            }
            Edit::Repeat => model.revisions[doc],
            other => {
                match other {
                    Edit::AppendText(t) => model.documents[doc].push(SourceItem::text(t.clone())),
                    Edit::AppendInlineMath(t) => {
                        model.documents[doc].push(SourceItem::inline_math(t.clone()))
                    }
                    Edit::AppendDisplayMath(t) => {
                        model.documents[doc].push(SourceItem::display_math(t.clone()))
                    }
                    Edit::AppendPageMark => model.documents[doc].push(SourceItem::PageMark),
                    Edit::ReplaceContentSameRevision(_) | Edit::Repeat => unreachable!(),
                }
                model.revisions[doc] += 1;
                model.revisions[doc]
            }
        };

        let revision = RevisionId::new(Model::source(doc), claimed_revision);
        let lookup = cache
            .update(revision, &model.documents[doc], ScanLimit::UNBOUNDED)
            .unwrap_or_else(|e| panic!("seed {seed} step {step}: unexpected scan rejection: {e}"));
        incremental_bytes += lookup.bytes_scanned as u64;

        // --- The core assertion: incremental must equal fresh, for THIS
        // document, right now. ---
        let fresh_doc = model.fresh_one(doc);
        assert_eq!(
            lookup.stats.words, fresh_doc.words,
            "seed {seed} step {step} doc {doc}: word stats diverged from fresh"
        );
        assert_eq!(
            lookup.stats.math, fresh_doc.math,
            "seed {seed} step {step} doc {doc}: math stats diverged from fresh"
        );
        assert_eq!(
            lookup.stats.pages, fresh_doc.pages,
            "seed {seed} step {step} doc {doc}: page count diverged from fresh"
        );
        assert_eq!(
            lookup.stats.content_hash, fresh_doc.content_hash,
            "seed {seed} step {step} doc {doc}: content hash diverged from fresh"
        );

        // --- And the project-wide total, summed over every document's
        // current cached entry, must equal a from-scratch total over every
        // document's current true content. ---
        let (fresh_words, fresh_math, fresh_pages, fresh_scanned) = model.fresh_totals();
        fresh_bytes_running += fresh_scanned;
        let totals = cache.project_totals();
        assert_eq!(
            totals.words.words, fresh_words,
            "seed {seed} step {step}: project word total diverged from fresh"
        );
        assert_eq!(
            totals.math.total, fresh_math,
            "seed {seed} step {step}: project math total diverged from fresh"
        );
        assert_eq!(
            totals.pages, fresh_pages,
            "seed {seed} step {step}: project page total diverged from fresh"
        );
        assert!(
            cache.len() <= n_docs,
            "cache should hold at most one entry per document, got {} for {n_docs} documents",
            cache.len()
        );
    }

    (incremental_bytes, fresh_bytes_running)
}

#[test]
fn incremental_equals_fresh_over_many_varied_sequences() {
    // 40 independent seeds, each a sequence of edits across several
    // documents. Different seeds, document counts, and lengths so that a
    // stale-cache bug tied to a specific shape (single doc, short sequence,
    // no replacements, ...) is unlikely to hide.
    let mut total_incremental = 0u64;
    let mut total_fresh = 0u64;
    for seed in 0..40u64 {
        let n_docs = 1 + (seed as usize % 5); // 1..=5 documents
        let steps = 20 + (seed as usize % 30); // 20..=49 steps
        let (inc, fresh) = run_sequence(seed * 7919 + 17, n_docs, steps);
        total_incremental += inc;
        total_fresh += fresh;
    }

    // The property itself already asserted equality at every single step of
    // every sequence above; this is the "cache is doing something" proof.
    assert!(
        total_incremental < total_fresh,
        "incremental path scanned {total_incremental} bytes, fresh path scanned \
         {total_fresh} bytes: the cache should scan strictly less"
    );
    // Sanity: with 40 sequences of 20-49 steps each, most steps touch only
    // one changed document while `fresh_totals` rescans every document in
    // the project, so the saving should be substantial, not marginal.
    assert!(
        total_incremental * 2 < total_fresh,
        "expected the incremental path to scan well under half of what \
         fresh scanning costs; incremental={total_incremental} fresh={total_fresh}"
    );
}

#[test]
fn single_document_repeat_updates_are_hits_and_agree_with_fresh() {
    let mut cache = ProjectCache::new();
    let items = vec![
        SourceItem::text("hello world"),
        SourceItem::inline_math("x^2"),
        SourceItem::PageMark,
    ];
    let revision = RevisionId::new("only.tex", 1);
    let first = cache
        .update(revision.clone(), &items, ScanLimit::UNBOUNDED)
        .unwrap();
    assert!(!first.hit);

    for _ in 0..5 {
        let repeat = cache
            .update(revision.clone(), &items, ScanLimit::UNBOUNDED)
            .unwrap();
        assert!(repeat.hit);
        assert_eq!(repeat.bytes_scanned, 0);
        let fresh = Statistics::compute(revision.clone(), &items);
        assert_eq!(repeat.stats.words, fresh.words);
        assert_eq!(repeat.stats.math, fresh.math);
        assert_eq!(repeat.stats.pages, fresh.pages);
    }
}
