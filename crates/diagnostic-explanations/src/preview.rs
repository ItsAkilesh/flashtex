//! Fix previews: apply a suggestion's edits to a copy of the text and report
//! before/after excerpts plus whether the diagnostic's original span survives.
//!
//! Nothing here touches the document the app holds; the caller gets a new
//! `String` and decides. Offset rebasing mirrors the Mac app's `SourceMapping`
//! (common prefix/suffix of the old and new bytes, widened to scalar
//! boundaries): [`changed_region`] and [`rebase`] are byte-for-byte the same
//! rules, so a span this crate reports as still valid is one the Mac would map
//! the same way.

use crate::text;
use crate::{Edit, Suggestion};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewError {
    /// A suggestion without edits (advice) has nothing to preview.
    NoEdits,
    /// Edits name more than one document.
    MixedPaths,
    /// An edit lies outside the text or inside a multi-byte scalar.
    OutOfBounds { start_byte: usize, end_byte: usize },
    /// Two edits overlap; applying them is ambiguous.
    Overlap,
}

impl std::fmt::Display for PreviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreviewError::NoEdits => f.write_str("suggestion has no edits to preview"),
            PreviewError::MixedPaths => f.write_str("edits span more than one document"),
            PreviewError::OutOfBounds {
                start_byte,
                end_byte,
            } => {
                write!(
                    f,
                    "edit {start_byte}..{end_byte} is outside the text or splits a character"
                )
            }
            PreviewError::Overlap => f.write_str("edits overlap"),
        }
    }
}

impl std::error::Error for PreviewError {}

/// The single byte region that differs between two texts (Mac
/// `SourceMapping.ChangedRegion`): `old[start_byte..old_end_byte]` became
/// `new[start_byte..new_end_byte]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedRegion {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub replacement: String,
}

/// Outcome of rebasing a span across an edit (Mac `SourceMapping.Outcome`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rebase {
    Unchanged,
    Rebased { start: usize, end: usize },
    OverlapsEdit,
}

pub fn changed_region(old: &str, new: &str) -> ChangedRegion {
    let o = old.as_bytes();
    let n = new.as_bytes();
    let is_cont = |b: u8| b & 0xC0 == 0x80;
    let mut prefix = 0;
    while prefix < o.len() && prefix < n.len() && o[prefix] == n[prefix] {
        prefix += 1;
    }
    while prefix > 0
        && ((prefix < o.len() && is_cont(o[prefix])) || (prefix < n.len() && is_cont(n[prefix])))
    {
        prefix -= 1;
    }
    let mut suffix = 0;
    while suffix < o.len() - prefix
        && suffix < n.len() - prefix
        && o[o.len() - 1 - suffix] == n[n.len() - 1 - suffix]
    {
        suffix += 1;
    }
    while suffix > 0 && is_cont(o[o.len() - suffix]) {
        suffix -= 1;
    }
    let old_end = o.len() - suffix;
    let new_end = n.len() - suffix;
    ChangedRegion {
        start_byte: prefix,
        old_end_byte: old_end,
        new_end_byte: new_end,
        replacement: new[prefix..new_end].to_string(),
    }
}

pub fn rebase(start: usize, end: usize, old: &str, new: &str) -> Rebase {
    if old == new {
        return Rebase::Unchanged;
    }
    let region = changed_region(old, new);
    let delta = new.len() as isize - old.len() as isize;
    if end <= region.start_byte {
        return Rebase::Rebased { start, end };
    }
    if start >= region.old_end_byte {
        let shift = |v: usize| (v as isize + delta) as usize;
        return Rebase::Rebased {
            start: shift(start),
            end: shift(end),
        };
    }
    Rebase::OverlapsEdit
}

/// Result of previewing one suggestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    pub path: String,
    /// The whole document after the edits.
    pub after_text: String,
    /// Bounded excerpts (whole lines) around the edited region, before/after.
    pub before_excerpt: String,
    pub after_excerpt: String,
    /// Absolute byte ranges of the excerpts in their respective texts.
    pub before_excerpt_range: (usize, usize),
    pub after_excerpt_range: (usize, usize),
    /// The edited region as the Mac's prefix/suffix rule sees it.
    pub changed: ChangedRegion,
    /// Where the diagnostic's original span lands in `after_text`, or `None`
    /// when it overlapped the edit and cannot be mapped.
    pub original_span_after: Option<(usize, usize)>,
    /// `true` when the original span maps cleanly and still spells the same
    /// bytes; the UI may keep the underline, otherwise it must recompile.
    pub span_still_valid: bool,
}

/// Apply `edits` (any order, must not overlap) to a copy of `text`.
pub fn apply_edits(text: &str, edits: &[Edit]) -> Result<String, PreviewError> {
    if edits.is_empty() {
        return Err(PreviewError::NoEdits);
    }
    if edits.iter().any(|e| e.path != edits[0].path) {
        return Err(PreviewError::MixedPaths);
    }
    for e in edits {
        if e.start_byte > e.end_byte
            || e.end_byte > text.len()
            || !text.is_char_boundary(e.start_byte)
            || !text.is_char_boundary(e.end_byte)
        {
            return Err(PreviewError::OutOfBounds {
                start_byte: e.start_byte,
                end_byte: e.end_byte,
            });
        }
    }
    let mut order: Vec<&Edit> = edits.iter().collect();
    // Stable by start so two insertions at one offset keep suggestion order.
    order.sort_by_key(|e| (e.start_byte, e.end_byte));
    for pair in order.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let overlap = a.end_byte > b.start_byte
            || (a.start_byte == b.start_byte
                && a.end_byte != a.start_byte
                && b.end_byte != b.start_byte);
        if overlap {
            return Err(PreviewError::Overlap);
        }
    }
    let mut out = String::with_capacity(text.len() + 16);
    let mut cursor = 0;
    for e in order {
        out.push_str(&text[cursor..e.start_byte]);
        out.push_str(&e.replacement);
        cursor = e.end_byte;
    }
    out.push_str(&text[cursor..]);
    Ok(out)
}

/// Preview `suggestion` against `text`. `original_span` is the diagnostic's
/// byte range in `text`, used to report whether it survives the edit.
pub fn preview(
    text: &str,
    suggestion: &Suggestion,
    original_span: Option<(usize, usize)>,
) -> Result<Preview, PreviewError> {
    let after_text = apply_edits(text, &suggestion.edits)?;
    let path = suggestion.edits[0].path.clone();

    let lo = suggestion
        .edits
        .iter()
        .map(|e| e.start_byte)
        .min()
        .unwrap_or(0);
    let hi = suggestion
        .edits
        .iter()
        .map(|e| e.end_byte)
        .max()
        .unwrap_or(0);
    let before_range = excerpt_range(text, lo, hi);
    let delta_total: isize = suggestion
        .edits
        .iter()
        .map(|e| e.replacement.len() as isize - (e.end_byte - e.start_byte) as isize)
        .sum();
    let after_hi = (hi as isize + delta_total).max(lo as isize) as usize;
    let after_range = excerpt_range(&after_text, lo, after_hi.min(after_text.len()));

    let changed = changed_region(text, &after_text);
    let (original_span_after, span_still_valid) = match original_span {
        Some((s, e)) if s <= e && e <= text.len() => match rebase(s, e, text, &after_text) {
            Rebase::Unchanged => (Some((s, e)), true),
            Rebase::Rebased { start, end } => {
                let same = after_text.get(start..end) == text.get(s..e);
                (Some((start, end)), same)
            }
            Rebase::OverlapsEdit => (None, false),
        },
        _ => (None, false),
    };

    Ok(Preview {
        path,
        before_excerpt: text[before_range.0..before_range.1].to_string(),
        after_excerpt: after_text[after_range.0..after_range.1].to_string(),
        before_excerpt_range: before_range,
        after_excerpt_range: after_range,
        after_text,
        changed,
        original_span_after,
        span_still_valid,
    })
}

/// Whole lines covering `lo..hi`, capped like the context window.
fn excerpt_range(text: &str, lo: usize, hi: usize) -> (usize, usize) {
    let (lo, hi) = text::clamp_span(text, lo, hi);
    let start = text::line_start(text, lo).max(lo.saturating_sub(crate::context::MAX_SIDE_BYTES));
    let end = text::line_end(text, hi).min(hi.saturating_add(crate::context::MAX_SIDE_BYTES));
    (
        text::snap_up(text, start),
        text::snap_down(text, end.max(hi)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_region_matches_mac_rules() {
        let r = changed_region("héllo world", "héllo brave world");
        assert_eq!((r.start_byte, r.old_end_byte, r.new_end_byte), (7, 7, 13));
        assert_eq!(r.replacement, "brave ");
        // Identical strings: empty region at the end.
        let r = changed_region("abc", "abc");
        assert_eq!((r.start_byte, r.old_end_byte, r.new_end_byte), (3, 3, 3));
        // Region ends widen to scalar boundaries.
        let r = changed_region("aé", "aè");
        assert_eq!((r.start_byte, r.old_end_byte), (1, 3));
    }

    #[test]
    fn rebase_before_after_overlap() {
        let old = "aaa bbb ccc";
        let new = "aaa XX ccc";
        assert_eq!(rebase(0, 3, old, new), Rebase::Rebased { start: 0, end: 3 });
        assert_eq!(
            rebase(8, 11, old, new),
            Rebase::Rebased { start: 7, end: 10 }
        );
        assert_eq!(rebase(4, 7, old, new), Rebase::OverlapsEdit);
        assert_eq!(rebase(0, 1, old, old), Rebase::Unchanged);
    }

    #[test]
    fn apply_rejects_bad_edits() {
        let t = "aé b";
        assert_eq!(apply_edits(t, &[]), Err(PreviewError::NoEdits));
        assert!(matches!(
            apply_edits(t, &[Edit::insert("m", 2, "x")]),
            Err(PreviewError::OutOfBounds { .. })
        ));
        assert_eq!(
            apply_edits(t, &[Edit::delete("m", 0, 3), Edit::delete("m", 1, 4)]),
            Err(PreviewError::Overlap)
        );
        assert_eq!(
            apply_edits(t, &[Edit::insert("m", 0, "x"), Edit::insert("n", 1, "y")]),
            Err(PreviewError::MixedPaths)
        );
        assert_eq!(
            apply_edits(t, &[Edit::insert("m", 5, "!"), Edit::insert("m", 0, "<")]).unwrap(),
            "<aé b!"
        );
    }
}
