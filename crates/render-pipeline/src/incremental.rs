//! Incremental layout reuse: one typeset block per cache entry.
//!
//! A block's layout (shaped runs, line breaks, box records, math boxes,
//! diagnostics) depends only on its own items, the flags the page builder
//! hands it, and the document style. The key therefore hashes the items
//! with source offsets made relative to the block's first byte, plus those
//! flags and a stylesheet fingerprint. On a hit the cached records are
//! cloned back with every source offset shifted by the block's new base,
//! so the output is byte-identical to a fresh compile: nothing in layout
//! depends on an absolute offset except the provenance fields, and those
//! are all relocated (`relocate_block`). Blocks whose items straddle two
//! documents are never cached. Diagnostics captured while a block was
//! built are replayed at the same point with the same once-only keys.
//!
//! The cache is bounded: past `MAX_BLOCKS` entries it is cleared.

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use flashtex_compiler::math::{MathList, Nucleus};
use flashtex_compiler::{DocumentId, Span};

use crate::adapter::{CharSrc, Item};
use crate::display::Diagnostic;
use crate::typeset::{BoxRec, BuiltBlock, MathRec};

pub const MAX_BLOCKS: usize = 50_000;

/// A block as built, with its records relative to the block.
#[derive(Clone)]
pub struct CachedBlock {
    pub block: BuiltBlock,
    /// The block's `recs` indices are absolute into the context that built
    /// it; `rec_base`/`math_base` are what to subtract.
    pub rec_base: usize,
    pub math_base: usize,
    pub recs: Vec<BoxRec>,
    pub maths: Vec<MathRec>,
    /// `(once-only key, diagnostic)` emitted while building the block.
    pub diagnostics: Vec<(Option<String>, Diagnostic)>,
    /// The document and first source byte the offsets are relative to.
    pub document: DocumentId,
    pub base: usize,
    pub path: String,
}

#[derive(Default)]
pub struct RenderCache {
    blocks: RefCell<HashMap<u64, Rc<CachedBlock>>>,
    hits: RefCell<u64>,
    misses: RefCell<u64>,
}

impl RenderCache {
    pub fn new() -> RenderCache {
        RenderCache::default()
    }

    pub fn get(&self, key: u64) -> Option<Rc<CachedBlock>> {
        let hit = self.blocks.borrow().get(&key).cloned();
        if hit.is_some() {
            *self.hits.borrow_mut() += 1;
        } else {
            *self.misses.borrow_mut() += 1;
        }
        hit
    }

    pub fn insert(&self, key: u64, block: CachedBlock) {
        let mut b = self.blocks.borrow_mut();
        if b.len() >= MAX_BLOCKS {
            b.clear();
        }
        b.insert(key, Rc::new(block));
    }

    pub fn len(&self) -> usize {
        self.blocks.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.borrow().is_empty()
    }

    /// `(hits, misses)` since creation.
    pub fn stats(&self) -> (u64, u64) {
        (*self.hits.borrow(), *self.misses.borrow())
    }
}

/// The document and lowest source byte of a block's items; `None` when
/// the items come from more than one document or carry no source.
pub fn block_origin(items: &[Item]) -> Option<(DocumentId, usize)> {
    let mut origin: Option<(DocumentId, usize)> = None;
    let mut note = |c: &CharSrc| -> bool {
        match origin {
            None => {
                origin = Some((c.document, c.start));
                true
            }
            Some((d, s)) => {
                if d != c.document {
                    return false;
                }
                if c.start < s {
                    origin = Some((d, c.start));
                }
                true
            }
        }
    };
    for it in items {
        match it {
            Item::Word(w) => {
                for seg in &w.segments {
                    for c in &seg.chars {
                        if !note(c) {
                            return None;
                        }
                    }
                }
            }
            Item::Math { span, .. } => {
                if !note(&CharSrc {
                    document: span.document,
                    start: span.start,
                    end: span.end,
                }) {
                    return None;
                }
            }
            _ => {}
        }
    }
    origin
}

/// Hashes the items with offsets relative to `base`.
pub fn hash_items(items: &[Item], base: usize, h: &mut DefaultHasher) {
    for it in items {
        match it {
            Item::Word(w) => {
                0u8.hash(h);
                for seg in &w.segments {
                    seg.text.hash(h);
                    seg.style.bold.hash(h);
                    seg.style.italic.hash(h);
                    for c in &seg.chars {
                        (c.start.wrapping_sub(base)).hash(h);
                        (c.end.wrapping_sub(base)).hash(h);
                    }
                }
            }
            Item::Space { style, factor, no_break } => {
                1u8.hash(h);
                style.bold.hash(h);
                style.italic.hash(h);
                factor.hash(h);
                no_break.hash(h);
            }
            Item::Math { list, span } => {
                2u8.hash(h);
                hash_math(list, h);
                (span.start.wrapping_sub(base)).hash(h);
                (span.end.wrapping_sub(base)).hash(h);
            }
            Item::LineBreak => 3u8.hash(h),
            Item::Quad { em } => {
                4u8.hash(h);
                em.to_bits().hash(h);
            }
            Item::Label { key } => {
                5u8.hash(h);
                key.hash(h);
            }
            Item::ItalicCorrection => 6u8.hash(h),
        }
    }
}

/// Hashes a math list's structure (spans are not part of layout).
pub fn hash_math(list: &MathList, h: &mut DefaultHasher) {
    list.atoms.len().hash(h);
    for a in &list.atoms {
        match &a.nucleus {
            Nucleus::Symbol(s) => {
                0u8.hash(h);
                s.hash(h);
            }
            Nucleus::Fraction { numerator, denominator } => {
                1u8.hash(h);
                hash_math(numerator, h);
                hash_math(denominator, h);
            }
            Nucleus::Radical(r) => {
                2u8.hash(h);
                hash_math(r, h);
            }
        }
        match &a.superscript {
            Some(s) => {
                1u8.hash(h);
                hash_math(s, h);
            }
            None => 0u8.hash(h),
        }
        match &a.subscript {
            Some(s) => {
                1u8.hash(h);
                hash_math(s, h);
            }
            None => 0u8.hash(h),
        }
    }
}

fn shift(v: usize, delta: isize) -> usize {
    (v as isize + delta) as usize
}

fn shift_range(r: &mut std::ops::Range<usize>, delta: isize) {
    r.start = shift(r.start, delta);
    r.end = shift(r.end, delta);
}

fn shift_span(s: &mut Span, delta: isize) {
    *s = Span::in_document(s.document, shift(s.start, delta), shift(s.end, delta));
}

/// Moves every source offset of a cached block by `delta` bytes.
pub fn relocate_block(b: &mut BuiltBlock, recs: &mut [BoxRec], maths: &mut [MathRec], diags: &mut [(Option<String>, Diagnostic)], path: &str, delta: isize) {
    if delta == 0 {
        return;
    }
    for line in &mut b.block.lines.lines {
        for run in &mut line.runs {
            shift_range(&mut run.source, delta);
            for g in &mut run.glyphs {
                shift_range(&mut g.cluster, delta);
            }
        }
    }
    for d in &mut b.block.lines.diagnostics {
        if let Some(s) = &mut d.source {
            shift_range(s, delta);
        }
        for r in &mut d.boxes {
            shift_range(r, delta);
        }
    }
    for item in &mut b.items {
        if let flashtex_paragraph_layout::Item::Box(run) = item {
            shift_range(&mut run.source, delta);
            for g in &mut run.glyphs {
                shift_range(&mut g.cluster, delta);
            }
        }
    }
    for rec in recs {
        if let BoxRec::Text { clusters, .. } = rec {
            for c in clusters {
                shift_span(&mut c.span, delta);
            }
        }
    }
    for m in maths {
        shift_span(&mut m.span, delta);
    }
    for (_, d) in diags {
        for s in &mut d.sources {
            if &*s.path == path {
                s.start_byte = shift(s.start_byte, delta);
                s.end_byte = shift(s.end_byte, delta);
            }
        }
    }
}

/// A stable fingerprint of everything outside the items that a block's
/// layout depends on.
pub fn style_fingerprint(style: &crate::style::Stylesheet) -> u64 {
    let mut h = DefaultHasher::new();
    format!("{style:?}").hash(&mut h);
    h.finish()
}
