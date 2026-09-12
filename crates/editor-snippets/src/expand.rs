use std::collections::{HashMap, HashSet};
use std::ops::Range;

use crate::Snippet;
use crate::error::SnippetError;
use crate::limits::{MAX_OCCURRENCES, MAX_OUTPUT_BYTES, MAX_PLACEHOLDERS};
use crate::model::Segment;

/// One placeholder index and every byte range where it occurs in the
/// expanded text.
///
/// All ranges for a given index always point at exactly the same resolved
/// text - that is what makes the occurrences "linked": to model editing
/// one of them, call [`Snippet::expand_with`] with new text for this
/// index, and every range for it moves together in the returned
/// [`Expansion`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderSpan {
    /// The placeholder index, e.g. `1` for `$1`.
    pub index: u32,
    /// Byte ranges into the sibling [`Expansion::text`], in document
    /// order. Every bound is a valid `char` boundary.
    pub occurrences: Vec<Range<usize>>,
}

/// The result of expanding a [`Snippet`]: plain text plus the offsets of
/// every placeholder occurrence within it.
///
/// Expansion never mutates a source document. Applying `text` to a real
/// buffer, and using `placeholders` to seed cursor or tab-stop positions
/// there, is left entirely to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    /// The fully expanded plain text.
    pub text: String,
    /// One entry per distinct placeholder index that occurs in `text`,
    /// sorted by index.
    pub placeholders: Vec<PlaceholderSpan>,
}

impl Expansion {
    /// Byte ranges of every occurrence of `index`, in document order, or an
    /// empty slice if `index` never occurs.
    pub fn occurrences_of(&self, index: u32) -> &[Range<usize>] {
        self.placeholders
            .iter()
            .find(|p| p.index == index)
            .map(|p| p.occurrences.as_slice())
            .unwrap_or(&[])
    }

    /// Placeholder indices in tab-navigation order: ascending, with `$0`
    /// (the conventional "final cursor" stop) moved to the end if present.
    pub fn tab_order(&self) -> Vec<u32> {
        let mut indices: Vec<u32> = self.placeholders.iter().map(|p| p.index).collect();
        indices.sort_unstable();
        if let Some(pos) = indices.iter().position(|&i| i == 0) {
            let zero = indices.remove(pos);
            indices.push(zero);
        }
        indices
    }
}

impl Snippet {
    /// Expand using each placeholder's own parsed default text (empty text
    /// for a placeholder with none).
    ///
    /// # Errors
    /// Returns [`SnippetError::SelfReferential`] if a default refers back
    /// to its own index (directly or transitively), or
    /// [`SnippetError::OutputTooLarge`] if the result would exceed
    /// [`crate::MAX_OUTPUT_BYTES`].
    pub fn expand(&self) -> Result<Expansion, SnippetError> {
        self.expand_with(&HashMap::new())
    }

    /// Expand, substituting `overrides[&index]` for that placeholder's
    /// resolved text wherever it - and every linked occurrence - appears,
    /// instead of its parsed default.
    ///
    /// This is how a linked edit is modeled: after a caller edits one
    /// on-screen occurrence of `$1`, call `expand_with(&overrides)` with
    /// `overrides[&1]` set to the new text, and every occurrence of `$1`
    /// in the returned [`Expansion`] carries that new text.
    ///
    /// # Errors
    /// Same as [`Self::expand`]. An override for an index makes that
    /// index's own default irrelevant (and cannot itself be
    /// self-referential, since it is used as-is rather than resolved), but
    /// a *different*, non-overridden index can still fail this way.
    pub fn expand_with(&self, overrides: &HashMap<u32, String>) -> Result<Expansion, SnippetError> {
        let defining = collect_defining(&self.segments)?;

        let mut cache: HashMap<u32, String> = HashMap::new();
        let mut resolving: Vec<u32> = Vec::new();
        let mut text = String::new();
        let mut spans: HashMap<u32, Vec<Range<usize>>> = HashMap::new();

        render(
            &self.segments,
            &defining,
            overrides,
            &mut cache,
            &mut resolving,
            &mut text,
            &mut spans,
        )?;

        let mut placeholders: Vec<PlaceholderSpan> = spans
            .into_iter()
            .map(|(index, occurrences)| PlaceholderSpan { index, occurrences })
            .collect();
        placeholders.sort_unstable_by_key(|p| p.index);

        Ok(Expansion { text, placeholders })
    }
}

/// Walks the whole AST once, including every default whether or not it is
/// the chosen definition for its index, so a placeholder buried inside an
/// otherwise-unused default is still counted against the occurrence and
/// distinct-index bounds. Returns the first-occurrence-with-a-default text
/// for each index.
fn collect_defining(segments: &[Segment]) -> Result<HashMap<u32, Vec<Segment>>, SnippetError> {
    let mut defining: HashMap<u32, Vec<Segment>> = HashMap::new();
    let mut all_indices: HashSet<u32> = HashSet::new();
    let mut total_occurrences = 0usize;
    walk(
        segments,
        &mut defining,
        &mut all_indices,
        &mut total_occurrences,
    )?;
    Ok(defining)
}

fn walk(
    segments: &[Segment],
    defining: &mut HashMap<u32, Vec<Segment>>,
    all_indices: &mut HashSet<u32>,
    total_occurrences: &mut usize,
) -> Result<(), SnippetError> {
    for seg in segments {
        if let Segment::Placeholder { index, default } = seg {
            *total_occurrences += 1;
            if *total_occurrences > MAX_OCCURRENCES {
                return Err(SnippetError::TooManyOccurrences {
                    count: *total_occurrences,
                });
            }
            all_indices.insert(*index);
            if all_indices.len() > MAX_PLACEHOLDERS {
                return Err(SnippetError::TooManyPlaceholders {
                    count: all_indices.len(),
                });
            }
            if let Some(default_segments) = default {
                walk(default_segments, defining, all_indices, total_occurrences)?;
                defining
                    .entry(*index)
                    .or_insert_with(|| default_segments.clone());
            }
        }
    }
    Ok(())
}

/// Renders the top-level (always-visible) segment tree into `text`,
/// resolving each placeholder's linked value on demand and recording every
/// occurrence's byte span.
fn render(
    segments: &[Segment],
    defining: &HashMap<u32, Vec<Segment>>,
    overrides: &HashMap<u32, String>,
    cache: &mut HashMap<u32, String>,
    resolving: &mut Vec<u32>,
    text: &mut String,
    spans: &mut HashMap<u32, Vec<Range<usize>>>,
) -> Result<(), SnippetError> {
    for seg in segments {
        match seg {
            Segment::Text(s) => {
                text.push_str(s);
                if text.len() > MAX_OUTPUT_BYTES {
                    return Err(SnippetError::OutputTooLarge { len: text.len() });
                }
            }
            Segment::Placeholder { index, .. } => {
                let value = resolve_value(*index, defining, overrides, cache, resolving)?;
                let start = text.len();
                text.push_str(&value);
                let end = text.len();
                if end > MAX_OUTPUT_BYTES {
                    return Err(SnippetError::OutputTooLarge { len: end });
                }
                spans.entry(*index).or_default().push(start..end);
            }
        }
    }
    Ok(())
}

/// Resolves the linked text for `index`, memoized in `cache`. `resolving`
/// tracks the indices currently being resolved on this call stack so any
/// cycle - direct (`${1:$1}`) or transitive (`${1:$2}` / `${2:$1}`) -
/// is caught as [`SnippetError::SelfReferential`] instead of recursing
/// forever.
fn resolve_value(
    index: u32,
    defining: &HashMap<u32, Vec<Segment>>,
    overrides: &HashMap<u32, String>,
    cache: &mut HashMap<u32, String>,
    resolving: &mut Vec<u32>,
) -> Result<String, SnippetError> {
    if let Some(v) = overrides.get(&index) {
        return Ok(v.clone());
    }
    if let Some(v) = cache.get(&index) {
        return Ok(v.clone());
    }
    if resolving.contains(&index) {
        return Err(SnippetError::SelfReferential { index });
    }
    let Some(default_segments) = defining.get(&index) else {
        cache.insert(index, String::new());
        return Ok(String::new());
    };

    resolving.push(index);
    let mut out = String::new();
    for seg in default_segments {
        match seg {
            Segment::Text(s) => out.push_str(s),
            Segment::Placeholder { index: nested, .. } => {
                let v = resolve_value(*nested, defining, overrides, cache, resolving)?;
                out.push_str(&v);
            }
        }
        if out.len() > MAX_OUTPUT_BYTES {
            resolving.pop();
            return Err(SnippetError::OutputTooLarge { len: out.len() });
        }
    }
    resolving.pop();

    cache.insert(index, out.clone());
    Ok(out)
}
