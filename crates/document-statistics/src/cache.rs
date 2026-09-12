//! Caches [`Statistics`] by exact source identity, and aggregates several
//! documents into project totals without redoing work for documents that
//! did not change.
//!
//! A project is several independently-edited documents. The project total
//! *depends on* every document, but editing one document should invalidate
//! only that document's cached entry — every other document's cached
//! [`Statistics`] must survive untouched. [`ProjectCache::update`] and
//! [`ProjectCache::project_totals`] give exactly that: `update` recomputes
//! (or reuses) one document at a time, and `project_totals` sums whatever is
//! currently cached, so an untouched document never gets rescanned just
//! because a sibling document changed.

use std::collections::HashMap;

use crate::items::SourceItem;
use crate::revision::RevisionId;
use crate::scan_limit::{ScanLimit, ScanTooLarge};
use crate::statistics::{MathStats, Statistics, fingerprint};
use crate::words::WordStats;

/// One document's cached statistics, keyed by its exact source identity.
#[derive(Clone, Debug)]
struct Entry {
    revision: u64,
    content_hash: u64,
    stats: Statistics,
}

/// The result of asking a [`ProjectCache`] to bring one document up to date.
#[derive(Clone, Debug)]
pub struct Lookup {
    /// The document's current statistics, whether served from cache or
    /// freshly (re)computed.
    pub stats: Statistics,
    /// `true` if an existing cache entry answered this call directly, with
    /// no rescanning at all.
    pub hit: bool,
    /// Source bytes actually rescanned to answer this call: always `0` on a
    /// hit, and exactly [`Statistics::scanned_bytes`] of the fresh result on
    /// a miss.
    pub bytes_scanned: usize,
}

/// A statistics cache for a project made of several documents, each
/// identified by its [`RevisionId::source`].
///
/// # Exact source identity
///
/// A cache entry is keyed by `source` and answers a call to
/// [`ProjectCache::update`] as a hit only when BOTH the stored
/// `revision.revision` number AND the stored content fingerprint match
/// exactly. A `revision` number reused over changed content (e.g. a caller
/// forgetting to bump a counter) therefore MISSES the cache and is
/// recomputed — it is never served as if it were the old content, and the
/// old content is never served as if it were the new content.
#[derive(Clone, Debug, Default)]
pub struct ProjectCache {
    documents: HashMap<String, Entry>,
}

impl ProjectCache {
    /// An empty cache with no documents yet.
    pub fn new() -> Self {
        ProjectCache {
            documents: HashMap::new(),
        }
    }

    /// Number of documents currently cached.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// `true` if no document has been cached yet.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Bring one document up to date for `revision` and `items`, bound by
    /// `limit`, reusing a cached result whenever exact source identity
    /// (`revision.source`, `revision.revision`, and content) already
    /// matches.
    ///
    /// Computing the content fingerprint (via the same
    /// [`Statistics::content_hash`] algorithm) costs one cheap pass over
    /// `items` regardless of outcome — that is the identity check. What a
    /// hit actually avoids is the more expensive word/math scan: on a hit,
    /// `bytes_scanned` in the returned [`Lookup`] is `0`; on a miss, it is
    /// the full [`Statistics::scanned_bytes`] for this document, and `limit`
    /// is enforced exactly as in [`Statistics::compute_bounded`].
    ///
    /// Every OTHER document already in this cache is left completely
    /// unchanged by this call — see the module docs.
    pub fn update(
        &mut self,
        revision: RevisionId,
        items: &[SourceItem],
        limit: ScanLimit,
    ) -> Result<Lookup, ScanTooLarge> {
        let content_hash = fingerprint(items);
        if let Some(entry) = self.documents.get(&revision.source)
            && entry.revision == revision.revision
            && entry.content_hash == content_hash
        {
            return Ok(Lookup {
                stats: entry.stats.clone(),
                hit: true,
                bytes_scanned: 0,
            });
        }
        let stats = Statistics::compute_bounded(revision.clone(), items, limit)?;
        let bytes_scanned = stats.scanned_bytes;
        self.documents.insert(
            revision.source.clone(),
            Entry {
                revision: revision.revision,
                content_hash: stats.content_hash,
                stats: stats.clone(),
            },
        );
        Ok(Lookup {
            stats,
            hit: false,
            bytes_scanned,
        })
    }

    /// Currently cached statistics for one document, if any. Never computes
    /// or scans anything.
    pub fn get(&self, source: &str) -> Option<&Statistics> {
        self.documents.get(source).map(|e| &e.stats)
    }

    /// Number of documents currently cached whose source is `source` — `0`
    /// or `1`, since `source` is the cache key. Convenience for tests that
    /// want to assert a document is present without inspecting its stats.
    pub fn contains(&self, source: &str) -> bool {
        self.documents.contains_key(source)
    }

    /// Sum [`Statistics`] over every document currently cached: words,
    /// math, and pages all added together.
    ///
    /// This sums whatever is currently cached, not some externally-tracked
    /// document list — a document that was never `update`d is simply
    /// absent from the total, the same way it would be absent from a fresh
    /// from-scratch computation that never saw it.
    pub fn project_totals(&self) -> ProjectTotals {
        let mut totals = ProjectTotals::default();
        for entry in self.documents.values() {
            totals.words = totals.words + entry.stats.words;
            totals.math.total += entry.stats.math.total;
            totals.math.inline += entry.stats.math.inline;
            totals.math.display += entry.stats.math.display;
            totals.pages += entry.stats.pages;
        }
        totals
    }
}

/// Sums of [`Statistics`] across every document currently held by a
/// [`ProjectCache`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectTotals {
    /// Word/char totals summed across every cached document.
    pub words: WordStats,
    /// Math totals summed across every cached document.
    pub math: MathStats,
    /// Page totals summed across every cached document.
    pub pages: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::SourceItem;

    fn rev(source: &str, n: u64) -> RevisionId {
        RevisionId::new(source, n)
    }

    #[test]
    fn first_update_is_always_a_miss() {
        let mut cache = ProjectCache::new();
        let items = vec![SourceItem::text("hello world")];
        let lookup = cache
            .update(rev("a.tex", 1), &items, ScanLimit::UNBOUNDED)
            .unwrap();
        assert!(!lookup.hit);
        assert_eq!(lookup.bytes_scanned, "hello world".len());
    }

    #[test]
    fn identical_revision_and_content_is_a_hit_with_zero_bytes_scanned() {
        let mut cache = ProjectCache::new();
        let items = vec![SourceItem::text("hello world")];
        cache
            .update(rev("a.tex", 1), &items, ScanLimit::UNBOUNDED)
            .unwrap();
        let lookup = cache
            .update(rev("a.tex", 1), &items, ScanLimit::UNBOUNDED)
            .unwrap();
        assert!(lookup.hit);
        assert_eq!(lookup.bytes_scanned, 0);
        assert_eq!(lookup.stats.words.words, 2);
    }

    #[test]
    fn reused_revision_id_over_changed_content_is_a_miss_not_a_stale_hit() {
        let mut cache = ProjectCache::new();
        let original = vec![SourceItem::text("hello world")];
        let changed = vec![SourceItem::text("hello world, edited further")];

        cache
            .update(rev("a.tex", 1), &original, ScanLimit::UNBOUNDED)
            .unwrap();
        // SAME revision id (1), DIFFERENT content: must miss, not return the
        // stale "hello world" statistics under revision 1.
        let lookup = cache
            .update(rev("a.tex", 1), &changed, ScanLimit::UNBOUNDED)
            .unwrap();
        assert!(
            !lookup.hit,
            "reused revision id over changed content must miss"
        );
        assert_eq!(lookup.stats.words.words, 4);
        assert_eq!(cache.get("a.tex").unwrap().words.words, 4);
    }

    #[test]
    fn unrelated_document_survives_a_sibling_update() {
        let mut cache = ProjectCache::new();
        cache
            .update(
                rev("a.tex", 1),
                &[SourceItem::text("alpha document text")],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();
        cache
            .update(
                rev("b.tex", 1),
                &[SourceItem::text("bravo document text")],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();
        let b_before = cache.get("b.tex").cloned().unwrap();

        // Edit only a.tex.
        let lookup = cache
            .update(
                rev("a.tex", 2),
                &[SourceItem::text("alpha document text, revised")],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();
        assert!(!lookup.hit);

        // b.tex's cached entry must be byte-for-byte the same value as
        // before a.tex changed.
        let b_after = cache.get("b.tex").cloned().unwrap();
        assert_eq!(b_before, b_after);
    }

    #[test]
    fn project_totals_sum_every_cached_document() {
        let mut cache = ProjectCache::new();
        cache
            .update(
                rev("a.tex", 1),
                &[
                    SourceItem::text("one two three"),
                    SourceItem::inline_math("x"),
                ],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();
        cache
            .update(
                rev("b.tex", 1),
                &[SourceItem::text("four five"), SourceItem::PageMark],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();
        let totals = cache.project_totals();
        assert_eq!(totals.words.words, 5);
        assert_eq!(totals.math.total, 1);
        assert_eq!(totals.pages, 1);
    }

    #[test]
    fn update_rejects_scan_past_the_limit_and_leaves_cache_unchanged() {
        let mut cache = ProjectCache::new();
        cache
            .update(
                rev("a.tex", 1),
                &[SourceItem::text("short")],
                ScanLimit::UNBOUNDED,
            )
            .unwrap();

        let err = cache
            .update(
                rev("a.tex", 2),
                &[SourceItem::text(
                    "this text is way too long for the tiny limit",
                )],
                ScanLimit::new(4),
            )
            .unwrap_err();
        assert!(err.scanned > err.limit);

        // The rejected update must not have clobbered the existing entry.
        assert_eq!(cache.get("a.tex").unwrap().revision.revision, 1);
    }
}
