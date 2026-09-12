//! In-process compile cache for incremental reuse.
//!
//! Maps `(project_id, content_hash)` → `CachedResult` so that
//! re-submitting an identical document body skips the full
//! parse → layout → PDF pipeline.
//!
//! Design constraints:
//!   - Zero external dependencies: uses a bare FNV-1a hash.
//!   - Single-threaded by design: the JSON Lines main loop is
//!     sequential; no locks required.
//!   - Size-bounded: evicts the least-recently-used entry beyond
//!     MAX_ENTRIES. A hit promotes the entry to the back of the list so
//!     the front always holds the least-recently-accessed entry.
//!   - Cache is per-process; no persistence across restarts.
//!
//! Incremental correctness guarantee: a cache hit is returned only when
//! the hash of the full document text matches the stored entry's hash.
//! Because FNV-1a has a negligible collision probability for the
//! kilobyte-scale inputs FlashTeX processes, this is a safe proxy for
//! content equality. A full byte comparison guard is available via the
//! `exact_text` field when higher assurance is needed in tests.

use crate::layout::Page;

/// Maximum cached entries before LRU eviction.
const MAX_ENTRIES: usize = 64;

/// A single cached compile result.
#[derive(Clone)]
pub struct CachedResult {
    /// FNV-1a hash of the exact source text that produced this result.
    pub content_hash: u64,
    /// Exact source text — kept for collision-resistance verification in tests.
    pub exact_text: String,
    /// Laid-out pages produced by the compile pipeline.
    pub pages: Vec<Page>,
    /// Path to the written PDF file, if one was produced.
    pub pdf_path: Option<String>,
    /// How many times this entry has been returned as a cache hit.
    pub hit_count: u64,
}

/// In-process cache keyed by `(project_id, content_hash)`.
pub struct CompileCache {
    entries: Vec<(String, u64, CachedResult)>, // (project_id, hash, result)
}

impl CompileCache {
    pub fn new() -> Self {
        CompileCache { entries: Vec::with_capacity(MAX_ENTRIES) }
    }

    /// Look up a cached result.
    ///
    /// Returns `Some(&mut entry)` on a hit (hash match + exact text match),
    /// `None` on a miss.
    pub fn get(&mut self, project_id: &str, text: &str) -> Option<&mut CachedResult> {
        let hash = fnv1a(text.as_bytes());
        let pos = self.entries.iter().position(|(pid, h, entry)| {
            *h == hash && pid.as_str() == project_id && entry.exact_text == text
        });
        if let Some(idx) = pos {
            // Promote hit to the back so the front remains least-recently-used.
            let mut entry = self.entries.remove(idx);
            entry.2.hit_count += 1;
            self.entries.push(entry);
            self.entries.last_mut().map(|(_, _, r)| r)
        } else {
            None
        }
    }

    /// Insert a new result. Evicts the oldest entry when the cache is full.
    pub fn insert(&mut self, project_id: &str, text: &str, result: CachedResult) {
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.remove(0); // FIFO eviction — oldest first
        }
        let hash = fnv1a(text.as_bytes());
        self.entries.push((project_id.to_string(), hash, result));
    }

    /// Number of entries currently stored.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Clear all entries (used in tests).
    #[cfg(test)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for CompileCache {
    fn default() -> Self {
        Self::new()
    }
}

/// FNV-1a 64-bit hash — fast, no external deps, suitable for cache keys.
/// Reference: http://www.isthe.com/chongo/tech/comp/fnv/
pub fn fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 14695981039346656037;
    const PRIME: u64 = 1099511628211;
    let mut hash = OFFSET_BASIS;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Page, TextItem};
    use crate::Span;

    fn dummy_page() -> Page {
        Page {
            number: 1,
            width_pt: 612.0,
            height_pt: 792.0,
            items: vec![TextItem {
                text: "hello".into(),
                x_pt: 72.0,
                baseline_y_pt: 720.0,
                font_size_pt: 12.0,
                span: Span::new(0, 5),
            }],
        }
    }

    fn make_result(text: &str) -> CachedResult {
        CachedResult {
            content_hash: fnv1a(text.as_bytes()),
            exact_text: text.to_string(),
            pages: vec![dummy_page()],
            pdf_path: Some("/tmp/flashtex/test/rev1.pdf".into()),
            hit_count: 0,
        }
    }

    #[test]
    fn miss_on_empty_cache() {
        let mut cache = CompileCache::new();
        assert!(cache.get("proj", "\\documentclass{article}").is_none());
    }

    #[test]
    fn hit_after_insert() {
        let mut cache = CompileCache::new();
        let text = "\\documentclass{article}\\begin{document}hello\\end{document}";
        cache.insert("proj", text, make_result(text));
        let hit = cache.get("proj", text);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().hit_count, 1);
    }

    #[test]
    fn miss_on_different_project() {
        let mut cache = CompileCache::new();
        let text = "hello world";
        cache.insert("proj-a", text, make_result(text));
        assert!(cache.get("proj-b", text).is_none());
    }

    #[test]
    fn miss_on_changed_text() {
        let mut cache = CompileCache::new();
        let text1 = "\\documentclass{article}\\begin{document}hello\\end{document}";
        let text2 = "\\documentclass{article}\\begin{document}world\\end{document}";
        cache.insert("proj", text1, make_result(text1));
        assert!(cache.get("proj", text2).is_none(), "changed text must be a cache miss");
    }

    #[test]
    fn hit_increments_count() {
        let mut cache = CompileCache::new();
        let text = "some content";
        cache.insert("proj", text, make_result(text));
        cache.get("proj", text); // hit 1
        cache.get("proj", text); // hit 2
        let entry = cache.get("proj", text).unwrap(); // hit 3
        assert_eq!(entry.hit_count, 3);
    }

    #[test]
    fn evicts_when_full() {
        let mut cache = CompileCache::new();
        for i in 0..super::MAX_ENTRIES {
            let text = format!("document {}", i);
            cache.insert("proj", &text, make_result(&text));
        }
        assert_eq!(cache.len(), super::MAX_ENTRIES);
        // Insert one more — should evict oldest (document 0)
        cache.insert("proj", "document extra", make_result("document extra"));
        assert_eq!(cache.len(), super::MAX_ENTRIES);
        assert!(cache.get("proj", "document 0").is_none(), "evicted entry must be gone");
        assert!(cache.get("proj", "document extra").is_some(), "new entry must be present");
    }

    #[test]
    fn lru_not_fifo_eviction() {
        // Fill the cache completely.
        let mut cache = CompileCache::new();
        for i in 0..super::MAX_ENTRIES {
            let text = format!("doc_{}", i);
            cache.insert("proj", &text, make_result(&text));
        }
        // Touch entry "doc_0" — it becomes the most-recently-used.
        assert!(cache.get("proj", "doc_0").is_some());
        // Insert one more entry to trigger eviction.
        cache.insert("proj", "doc_extra", make_result("doc_extra"));
        assert_eq!(cache.len(), super::MAX_ENTRIES);
        // "doc_0" was promoted past "doc_1", so "doc_1" is the LRU victim.
        assert!(cache.get("proj", "doc_0").is_some(), "promoted entry must survive eviction");
        assert!(cache.get("proj", "doc_1").is_none(), "least-recently-used must be evicted");
    }

    #[test]
    fn fnv1a_deterministic() {
        let h1 = fnv1a(b"hello");
        let h2 = fnv1a(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn fnv1a_distinct_inputs() {
        let h1 = fnv1a(b"hello");
        let h2 = fnv1a(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn fnv1a_empty() {
        // Should not panic
        let _ = fnv1a(b"");
    }
}
