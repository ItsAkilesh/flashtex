//! Content identity: per-file revision counters keyed by SHA-256, and a
//! project revision derived deterministically from the observed sequence of
//! content changes (the same sequence of `observe`/`remove` calls always
//! yields the same numbers, whatever the wall clock says).

use std::collections::BTreeMap;

use crate::path::ProjectPath;
use crate::sha256::{Digest, sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileRevision {
    /// Starts at 1 when a file is first observed; increments only when the
    /// content hash changes.
    pub revision: u64,
    pub sha256: Digest,
    pub bytes: u64,
}

/// Tracks revisions for a set of project files.
#[derive(Debug, Clone, Default)]
pub struct RevisionTracker {
    files: BTreeMap<ProjectPath, FileRevision>,
    project: u64,
}

impl RevisionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current project revision: 0 before anything is observed, then +1 for
    /// every file content change or removal. Suitable as the runtime-v1
    /// integer `revision`.
    pub fn project_revision(&self) -> u64 {
        self.project
    }

    pub fn file(&self, path: &ProjectPath) -> Option<&FileRevision> {
        self.files.get(path)
    }

    pub fn files(&self) -> impl Iterator<Item = (&ProjectPath, &FileRevision)> {
        self.files.iter()
    }

    /// Records the current text of `path`. Returns the file revision and
    /// whether it changed (a first observation counts as a change).
    pub fn observe(&mut self, path: &ProjectPath, text: &str) -> (FileRevision, bool) {
        self.observe_bytes(path, text.as_bytes())
    }

    pub fn observe_bytes(&mut self, path: &ProjectPath, bytes: &[u8]) -> (FileRevision, bool) {
        let digest = sha256(bytes);
        self.observe_digest(path, digest, bytes.len() as u64)
    }

    /// Like [`observe`](Self::observe) with a precomputed digest.
    pub fn observe_digest(
        &mut self,
        path: &ProjectPath,
        digest: Digest,
        bytes: u64,
    ) -> (FileRevision, bool) {
        match self.files.get_mut(path) {
            Some(existing) if existing.sha256 == digest => (*existing, false),
            Some(existing) => {
                existing.revision += 1;
                existing.sha256 = digest;
                existing.bytes = bytes;
                self.project += 1;
                (*existing, true)
            }
            None => {
                let rev = FileRevision {
                    revision: 1,
                    sha256: digest,
                    bytes,
                };
                self.files.insert(path.clone(), rev);
                self.project += 1;
                (rev, true)
            }
        }
    }

    /// Forgets `path`; bumps the project revision if it was tracked.
    pub fn remove(&mut self, path: &ProjectPath) -> Option<FileRevision> {
        let removed = self.files.remove(path);
        if removed.is_some() {
            self.project += 1;
        }
        removed
    }

    /// Observes every text file of a discovered graph. Returns the paths whose
    /// content changed.
    pub fn observe_graph(&mut self, graph: &crate::graph::ProjectGraph) -> Vec<ProjectPath> {
        let mut changed = Vec::new();
        for f in graph.files() {
            if f.kind == crate::graph::FileKind::Graphic {
                continue;
            }
            let (_, did_change) = self.observe_digest(&f.path, f.sha256, f.bytes);
            if did_change {
                changed.push(f.path.clone());
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revisions_follow_content_changes() {
        let mut t = RevisionTracker::new();
        let a = ProjectPath::normalize("a.tex").unwrap();
        let b = ProjectPath::normalize("b.tex").unwrap();
        assert_eq!(t.project_revision(), 0);
        assert_eq!(t.observe(&a, "x").0.revision, 1);
        assert_eq!(t.observe(&a, "x"), (t.file(&a).copied().unwrap(), false));
        assert_eq!(t.observe(&a, "y").0.revision, 2);
        assert_eq!(t.observe(&b, "z").0.revision, 1);
        assert_eq!(t.project_revision(), 3);
        assert!(t.remove(&b).is_some());
        assert_eq!(t.project_revision(), 4);
        assert!(t.remove(&b).is_none());
        assert_eq!(t.project_revision(), 4);
    }
}
