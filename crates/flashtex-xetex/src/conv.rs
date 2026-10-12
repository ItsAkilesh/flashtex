//! Convergence (DESIGN.md §5.3, docs/design/xetex/INCREMENTAL.md §5 item
//! 5): a run restarted from S₀ or a page checkpoint compares its state
//! after each page with the last complete run's after the same page; where
//! they are the same, the rest of the old run stands.
//!
//! This first stage measures: at each page checkpoint of a restarted run,
//! the word space against the old run's checkpoint after the same page
//! (`Arena::diff_branch`), by array, with `FLASHTEX_UNICODE_CONV_DEBUG=1`.

use crate::generated::Globals;
use flashtex_engine::arena::{Branch, CheckpointId, CHUNK_BYTES};
use std::collections::BTreeMap;

/// The last complete run's state after the restart point: the word space's
/// branch and its page checkpoints (pages shipped, id).
pub struct Old {
    pub branch: Branch,
    pub pages: Vec<(usize, CheckpointId)>,
    debug: bool,
    notes: Vec<String>,
}

impl Old {
    pub fn new(branch: Branch, pages: Vec<(usize, CheckpointId)>) -> Old {
        Old {
            branch,
            pages,
            debug: std::env::var_os("FLASHTEX_UNICODE_CONV_DEBUG").is_some(),
            notes: vec![],
        }
    }

    /// What the tests found (the debug notes).
    pub fn report(&self) -> Option<String> {
        (!self.notes.is_empty()).then(|| self.notes.join("; "))
    }
}

impl Globals {
    /// After the page checkpoint of page `pages`: the test against the old
    /// run's.
    pub(crate) fn conv_test(&mut self, pages: usize) {
        let Some(r) = self.host.resident.as_ref() else {
            return;
        };
        let Some(c) = r.conv.as_ref() else {
            return;
        };
        if !c.debug {
            return;
        }
        let Some(&(_, old)) = c.pages.iter().find(|(p, _)| *p == pages) else {
            return;
        };
        let note = match self.arena.diff_branch(&c.branch, old) {
            Err(e) => format!("page {pages}: {e}"),
            Ok(d) => {
                let mut by: BTreeMap<&'static str, (usize, Vec<usize>)> = BTreeMap::new();
                for &(ch, new, oldp) in &d.differing {
                    let words = CHUNK_BYTES / 8;
                    for w in 0..words {
                        // SAFETY: both point at whole chunks the diff holds.
                        let (a, b) = unsafe { (*new.add(w), *oldp.add(w)) };
                        if a != b {
                            let off = (ch as usize) * CHUNK_BYTES + w * 8;
                            let (info, at) = self.arena.region_at(off);
                            let e = by.entry(info.name).or_default();
                            e.0 += 1;
                            if e.1.len() < 4 {
                                e.1.push(at / info.elem.max(1));
                            }
                        }
                    }
                }
                let parts: Vec<String> = by
                    .iter()
                    .map(|(k, (n, ex))| format!("{k} {n} {ex:?}"))
                    .collect();
                format!(
                    "page {pages}: {} chunks: {}",
                    d.differing.len(),
                    parts.join(", ")
                )
            }
        };
        eprintln!("[conv] {note}");
        if let Some(c) = self.host.resident.as_mut().and_then(|r| r.conv.as_mut()) {
            c.notes.push(note);
        }
    }
}
