//! Whole-list page-reference stabilization.
//!
//! A `\tableofcontents`/`\listoffigures` entry's absolute page depends on
//! how many pages the front matter (the list itself) occupies — but that
//! occupancy depends on what the list ends up containing, which depends on
//! the resolved entries. [`stabilize_toc`] drives that loop to a single,
//! bounded, deterministic fixed point (via [`crate::converge`]) and then
//! resolves every entry against it in one pass, so the same inputs always
//! produce the same [`StabilizedToc`] — never a value that depends on the
//! order entries happened to arrive in, or on how many passes were needed
//! to get there.
//!
//! Every entry is stamped with the exact [`SourceId`] it was extracted
//! from, so a [`StabilizedToc`] computed against one document revision is
//! detectably [stale](StabilizedEntry::is_stale) against another — this
//! crate never assumes an entry is still current just because its title or
//! position matches.

use crate::converge::{ConvergenceError, FrontMatterModel, converge_front_matter_pages};
use crate::entry::{EntryError, EntryRecord, RelativeEntry};

/// Exactly identifies the source revision a TOC/LOF entry was extracted
/// from: which document, and which revision of it. Two `SourceId`s that
/// differ in either field refer to different content and must never be
/// compared loosely (e.g. by title alone) or merged.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId {
    pub document: String,
    pub revision: String,
}

impl SourceId {
    pub fn new(document: impl Into<String>, revision: impl Into<String>) -> Self {
        Self {
            document: document.into(),
            revision: revision.into(),
        }
    }
}

/// One relative TOC/LOF entry plus the exact source revision it was
/// extracted from and an explicit `sequence` fixing its position in the
/// list. `sequence` — not insertion order — is what [`stabilize_toc`]
/// orders by, so entries drained from an unordered caller-side collection
/// (e.g. a hash map keyed by heading id) still stabilize deterministically.
#[derive(Debug, Clone, PartialEq)]
pub struct SourcedEntry {
    pub source: SourceId,
    pub sequence: u32,
    pub entry: RelativeEntry,
}

/// One fully resolved entry: an absolute page number, stamped with the
/// exact source revision it was computed against.
#[derive(Debug, Clone, PartialEq)]
pub struct StabilizedEntry {
    pub source: SourceId,
    pub sequence: u32,
    pub record: EntryRecord,
}

impl StabilizedEntry {
    /// True if `current` names a different source revision than the one
    /// this entry was actually computed against — i.e. this entry's page
    /// number cannot be trusted as still current.
    pub fn is_stale(&self, current: &SourceId) -> bool {
        &self.source != current
    }
}

/// A stabilized table of contents / list of figures: one shared,
/// bounded-fixed-point front-matter page count, and every entry's absolute
/// page resolved against it, ordered by [`SourcedEntry::sequence`].
#[derive(Debug, Clone, PartialEq)]
pub struct StabilizedToc {
    pub front_matter_pages: u32,
    pub entries: Vec<StabilizedEntry>,
}

/// Why whole-list page-reference stabilization failed.
#[derive(Debug, Clone, PartialEq)]
pub enum StabilizationError {
    /// The front-matter page count itself never reached a fixed point;
    /// see [`ConvergenceError`] for the cycle/unresolved distinction.
    Convergence(ConvergenceError),
    /// The front-matter page count converged, but resolving one entry's
    /// absolute page against it failed (e.g. the page counter overflowed).
    EntryResolution {
        sequence: u32,
        source: SourceId,
        error: EntryError,
    },
}

impl std::fmt::Display for StabilizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StabilizationError::Convergence(err) => write!(f, "{err}"),
            StabilizationError::EntryResolution {
                sequence,
                source,
                error,
            } => write!(
                f,
                "entry {sequence} (source {source:?}) failed to resolve: {error}"
            ),
        }
    }
}

impl std::error::Error for StabilizationError {}

/// Stabilizes a whole TOC/LOF: finds the bounded, deterministic
/// front-matter fixed point via `model` (see
/// [`converge_front_matter_pages`]), then resolves every entry's absolute
/// page against it.
///
/// `entries` may be supplied in any order — including one that varies run
/// to run, such as values drained from a hash map — because this function
/// sorts a copy by [`SourcedEntry::sequence`] before resolving anything.
/// The result depends only on `model`, `initial_guess`, and the
/// `(sequence, source, entry)` content of `entries`, never on the order
/// they were passed in.
pub fn stabilize_toc(
    entries: &[SourcedEntry],
    model: &dyn FrontMatterModel,
    initial_guess: u32,
) -> Result<StabilizedToc, StabilizationError> {
    let front_matter_pages = converge_front_matter_pages(model, initial_guess)
        .map_err(StabilizationError::Convergence)?;

    let mut ordered: Vec<&SourcedEntry> = entries.iter().collect();
    ordered.sort_by_key(|sourced| sourced.sequence);

    let mut resolved = Vec::with_capacity(ordered.len());
    for sourced in ordered {
        let record = sourced.entry.resolve(front_matter_pages).map_err(|error| {
            StabilizationError::EntryResolution {
                sequence: sourced.sequence,
                source: sourced.source.clone(),
                error,
            }
        })?;
        resolved.push(StabilizedEntry {
            source: sourced.source.clone(),
            sequence: sourced.sequence,
            record,
        });
    }

    Ok(StabilizedToc {
        front_matter_pages,
        entries: resolved,
    })
}
