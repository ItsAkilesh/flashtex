//! Bounded, measured dot-leader layout for one contents/list-of-figures
//! line: `<indent><title><leader dots><page number>`, right-aligned to a
//! fixed line width using real measured widths (never a fixed dot count).

use crate::entry::EntryRecord;
use crate::measure::{InvalidWidth, TextMeasure, checked_width};

/// The fixed geometry a contents/list-of-figures line is laid out into.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineBox {
    /// Full usable width of one line, in the measurer's unit.
    pub width: f64,
    /// Width added per nesting level (see [`EntryRecord::level`]).
    pub indent_unit: f64,
}

/// A malformed [`LineBox`]: non-finite, or a width/indent that cannot hold
/// any entry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineBoxError {
    pub width: f64,
    pub indent_unit: f64,
}

impl std::fmt::Display for LineBoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid line box: width {} indent_unit {}",
            self.width, self.indent_unit
        )
    }
}

impl std::error::Error for LineBoxError {}

impl LineBox {
    /// Validates a line box: `width` must be finite and positive,
    /// `indent_unit` finite and non-negative.
    pub fn new(width: f64, indent_unit: f64) -> Result<Self, LineBoxError> {
        if width.is_finite() && width > 0.0 && indent_unit.is_finite() && indent_unit >= 0.0 {
            Ok(Self { width, indent_unit })
        } else {
            Err(LineBoxError { width, indent_unit })
        }
    }
}

/// A fully measured, laid-out contents/list-of-figures line. Every width
/// field is a real measured value; `indent + title_width + leader_width +
/// gap_before_page + page_label_width == line width` holds exactly (see
/// the `sum_reaches_line_width` test), so the page number lands flush at
/// the right margin.
#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutEntry {
    pub title: String,
    pub level: u8,
    pub indent: f64,
    pub title_width: f64,
    pub page_label: String,
    pub page_label_width: f64,
    /// Number of whole leader units (e.g. dots) that fit in the gap.
    pub leader_count: u32,
    /// `leader_count * leader_unit_width`.
    pub leader_width: f64,
    /// Sub-leader-unit remainder, absorbed as a small gap before the page
    /// number rather than as an extra partial dot.
    pub gap_before_page: f64,
}

/// Why an entry could not be laid out into a given [`LineBox`].
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    /// `indent + title width + page-label width` alone exceeds the line
    /// width — there is no room even for zero leader dots.
    Overflow { title: String, deficit: f64 },
    /// The measurer returned a non-finite or negative width.
    InvalidMeasurement { text: String, source: InvalidWidth },
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::Overflow { title, deficit } => write!(
                f,
                "entry {title:?} overflows its line by {deficit} measurement units"
            ),
            LayoutError::InvalidMeasurement { text, source } => {
                write!(f, "measuring {text:?}: {source}")
            }
        }
    }
}

impl std::error::Error for LayoutError {}

/// Lays out one entry, computing a real leader-dot count from measured
/// widths so the page number is flush against `line.width`.
pub fn layout_entry(
    record: &EntryRecord,
    line: LineBox,
    measure: &dyn TextMeasure,
) -> Result<LaidOutEntry, LayoutError> {
    let indent = line.indent_unit * f64::from(record.level);

    let title_width = checked_width(measure.width(&record.title)).map_err(|source| {
        LayoutError::InvalidMeasurement {
            text: record.title.clone(),
            source,
        }
    })?;

    let page_label = record.page.to_string();
    let page_label_width = checked_width(measure.width(&page_label)).map_err(|source| {
        LayoutError::InvalidMeasurement {
            text: page_label.clone(),
            source,
        }
    })?;

    let used = indent + title_width + page_label_width;
    if used > line.width {
        return Err(LayoutError::Overflow {
            title: record.title.clone(),
            deficit: used - line.width,
        });
    }

    let available = line.width - used;
    let leader_unit = checked_width(measure.leader_unit_width()).map_err(|source| {
        LayoutError::InvalidMeasurement {
            text: "<leader unit>".to_string(),
            source,
        }
    })?;

    let (leader_count, leader_width, gap_before_page) = if leader_unit > 0.0 {
        let count = (available / leader_unit).floor();
        // `available` is finite and non-negative and `leader_unit` > 0, so
        // `count` is finite, non-negative, and at most `available /
        // leader_unit`; clamp defensively against a `u32` overflow on an
        // absurdly large line box rather than panic.
        let count = count.min(f64::from(u32::MAX)) as u32;
        let leader_width = f64::from(count) * leader_unit;
        (count, leader_width, available - leader_width)
    } else {
        (0, 0.0, available)
    };

    Ok(LaidOutEntry {
        title: record.title.clone(),
        level: record.level,
        indent,
        title_width,
        page_label,
        page_label_width,
        leader_count,
        leader_width,
        gap_before_page,
    })
}

/// Lays out a whole contents/list-of-figures list against one shared
/// [`LineBox`], failing on the first entry that does not fit.
pub fn layout_entries(
    records: &[EntryRecord],
    line: LineBox,
    measure: &dyn TextMeasure,
) -> Result<Vec<LaidOutEntry>, LayoutError> {
    records
        .iter()
        .map(|record| layout_entry(record, line, measure))
        .collect()
}
