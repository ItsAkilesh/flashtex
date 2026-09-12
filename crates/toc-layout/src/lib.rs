//! FlashTeX table-of-contents / list-of-figures entry layout.
//!
//! Lays out `\tableofcontents`/`\listoffigures`-style entries —
//! `<indent><title><leader dots><page number>` — from **explicit**
//! section/page records, with a real measured leader-dot count (never a
//! fixed guess) and a bounded, revision-safe fixed-point solver for the
//! classic circularity where an entry's absolute page depends on how many
//! pages the contents/figures list itself occupies.
//!
//! No font, shaping, or TeX engine is linked. Real widths come from a
//! caller-supplied [`measure::TextMeasure`] — the typed adapter contract a
//! consumer (font-engine, the compiler, native layout) implements; this
//! crate ships [`measure::CharWidthMeasure`] as a documented, non-shaping
//! reference implementation for tests and callers without a font yet.
//!
//! ```
//! use flashtex_toc_layout::{CharWidthMeasure, EntryRecord, LineBox, layout_entry};
//!
//! // 1pt per character, 1pt per leader dot, 40pt-wide line.
//! let measure = CharWidthMeasure::new(1.0, 1.0);
//! let line = LineBox::new(40.0, 12.0).unwrap();
//! let entry = EntryRecord::new("Introduction", 3, 0).unwrap();
//! let laid_out = layout_entry(&entry, line, &measure).unwrap();
//!
//! // indent(0) + title(12) + leader + gap + page("3", width 1) == 40.
//! assert_eq!(
//!     laid_out.indent + laid_out.title_width + laid_out.leader_width
//!         + laid_out.gap_before_page + laid_out.page_label_width,
//!     40.0
//! );
//! assert!(laid_out.leader_count > 0);
//! ```

pub mod converge;
pub mod entry;
pub mod leader;
pub mod measure;
pub mod stabilize;

pub use converge::{
    ConvergenceError, FrontMatterModel, MAX_CONVERGENCE_ITERATIONS, converge_front_matter_pages,
};
pub use entry::{EntryError, EntryRecord, MAX_LEVEL, PageNumber, RelativeEntry};
pub use leader::{LaidOutEntry, LayoutError, LineBox, LineBoxError, layout_entries, layout_entry};
pub use measure::{CharWidthMeasure, InvalidWidth, TextMeasure};
pub use stabilize::{
    SourceId, SourcedEntry, StabilizationError, StabilizedEntry, StabilizedToc, stabilize_toc,
};
