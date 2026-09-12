//! Consumer-neutral exact draw batches. Every operation retains paint order and
//! source provenance. Curves are never flattened or clipped approximately: a
//! consumer must apply the exact common rectangular clip when painting the paths.
use crate::{
    font_adapter::validate_with_collection,
    glyph_cache::GlyphPathCache,
    outlines::{PositionedGlyph, PreparedOutlines},
    *,
};
use flashtex_font_resources::FontCollection;
#[derive(Debug, Clone)]
pub enum DrawOperation {
    ExactRule {
        geometry: ExactClip,
        paint: Paint,
        sources: Vec<SourceRange>,
        synthetic_reason: Option<String>,
    },
    Glyph {
        path: Box<PositionedGlyph>,
        paint: Paint,
    },
    Rule {
        geometry: HitRect,
        paint: Paint,
        sources: Vec<SourceRange>,
        synthetic_reason: Option<String>,
    },
}
#[derive(Debug, Clone)]
pub struct DrawBatch {
    pub project_id: String,
    pub revision: u64,
    pub page: u32,
    pub page_width: Tick,
    pub page_height: Tick,
    /// None means an empty visible region, never an unbounded clip.
    pub visible_clip: Option<HitRect>,
    pub operations: Vec<DrawOperation>,
    pub path_commands: usize,
    pub color_space: &'static str,
    pub compositing: &'static str,
    pub hinting_applied: bool,
}
#[derive(Debug, Clone, Copy)]
pub struct BatchLimits {
    pub max_operations: usize,
    pub max_path_commands: usize,
}
impl Default for BatchLimits {
    fn default() -> Self {
        Self {
            max_operations: 100000,
            max_path_commands: 2_000_000,
        }
    }
}
/// Resources and exact source snapshots are verified once for this immutable
/// revision. No mutable list/resource handles are exposed by the prepared source.
pub struct PreparedBatchSource<'a> {
    list: &'a DisplayList,
    outlines: PreparedOutlines<'a>,
}
impl<'a> PreparedBatchSource<'a> {
    pub fn new(
        list: &'a DisplayList,
        capabilities: &Capabilities,
        documents: &BTreeMap<String, SourceSnapshot>,
        fonts: &'a FontCollection,
    ) -> Result<Self> {
        validate_with_collection(list, capabilities, documents, fonts)?;
        Ok(Self {
            list,
            outlines: PreparedOutlines::new(list, capabilities, fonts)?,
        })
    }
    /// Build atomically: on an unsupported outline or exceeded limit no partial
    /// batch is returned. Font-cache work may remain reusable after a failure.
    pub fn page(
        &self,
        project_id: &str,
        revision: u64,
        page: u32,
        requested_clip: Option<&HitRect>,
        limits: BatchLimits,
        cache: &mut GlyphPathCache,
    ) -> Result<DrawBatch> {
        require(
            project_id == self.list.project_id && revision == self.list.revision,
            "stale batch revision",
        )?;
        require(
            (1..=100000).contains(&limits.max_operations)
                && (1..=2_000_000).contains(&limits.max_path_commands),
            "invalid batch budget",
        )?;
        let index =
            page.checked_sub(1)
                .ok_or_else(|| ValidationError("unknown batch page".into()))? as usize;
        let page_data = self
            .list
            .pages
            .get(index)
            .ok_or_else(|| ValidationError("unknown batch page".into()))?;
        let page_clip = HitRect {
            x: Tick(0),
            top: Tick(0),
            width: page_data.width,
            height: page_data.height,
        };
        let visible_clip = if let Some(clip) = requested_clip {
            clip.validate()?;
            intersection(&page_clip, clip)
        } else {
            Some(page_clip)
        };
        let mut batch = DrawBatch {
            project_id: self.list.project_id.clone(),
            revision: self.list.revision,
            page,
            page_width: page_data.width,
            page_height: page_data.height,
            visible_clip,
            operations: vec![],
            path_commands: 0,
            color_space: "srgb",
            compositing: "source-over",
            hinting_applied: false,
        };
        let Some(clip) = batch.visible_clip.as_ref() else {
            return Ok(batch);
        };
        for (item_index, item) in page_data.items.iter().enumerate() {
            match item {
                Item::GlyphRun(run) => {
                    for glyph_index in 0..run.glyphs.len() {
                        require(
                            batch.operations.len() < limits.max_operations,
                            "batch operation budget exceeded",
                        )?;
                        let path =
                            self.outlines
                                .glyph_cached(page, item_index, glyph_index, cache)?;
                        let count = batch
                            .path_commands
                            .checked_add(path.commands.len())
                            .ok_or_else(|| {
                                ValidationError("batch command count overflow".into())
                            })?;
                        require(
                            count <= limits.max_path_commands,
                            "batch path command budget exceeded",
                        )?;
                        batch.path_commands = count;
                        batch.operations.push(DrawOperation::Glyph {
                            path: Box::new(path),
                            paint: run.paint.clone(),
                        });
                    }
                }
                Item::Rule(rule) => {
                    let geometry = HitRect {
                        x: rule.x,
                        top: rule.top,
                        width: rule.width,
                        height: rule.height,
                    };
                    // Exact rectangle intersection can discard invisible rules.
                    // Glyphs are not culled using hit boxes; ink can extend beyond them.
                    if intersection(&geometry, clip).is_some() {
                        require(
                            batch.operations.len() < limits.max_operations,
                            "batch operation budget exceeded",
                        )?;
                        batch.operations.push(DrawOperation::Rule {
                            geometry,
                            paint: rule.paint.clone(),
                            sources: rule.sources.clone().unwrap_or_default(),
                            synthetic_reason: rule.synthetic_reason.clone(),
                        });
                    }
                }
            }
        }
        Ok(batch)
    }
}
pub(crate) fn intersection(a: &HitRect, b: &HitRect) -> Option<HitRect> {
    let left = a.x.0.max(b.x.0);
    let top = a.top.0.max(b.top.0);
    let right = (a.x.0 + a.width.0).min(b.x.0 + b.width.0);
    let bottom = (a.top.0 + a.height.0).min(b.top.0 + b.height.0);
    (left < right && top < bottom).then_some(HitRect {
        x: Tick(left),
        top: Tick(top),
        width: Tick(right - left),
        height: Tick(bottom - top),
    })
}

/// Exact half-open internal rectangle; no wire fields are added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactClip {
    pub left: outlines::OutlineCoordinate,
    pub top: outlines::OutlineCoordinate,
    pub right: outlines::OutlineCoordinate,
    pub bottom: outlines::OutlineCoordinate,
}
impl ExactClip {
    pub fn validate(&self) -> Result<()> {
        require(
            self.left.checked_cmp(self.right)?.is_lt()
                && self.top.checked_cmp(self.bottom)?.is_lt(),
            "empty or reversed exact clip",
        )
    }
    pub fn from_rect(rect: &HitRect) -> Result<Self> {
        rect.validate()?;
        let c = |value: Tick| outlines::OutlineCoordinate::from_fraction(value.0 as i128, 1);
        Ok(Self {
            left: c(rect.x)?,
            top: c(rect.top)?,
            right: c(rect.x.checked_add(rect.width)?)?,
            bottom: c(rect.top.checked_add(rect.height)?)?,
        })
    }
    pub fn intersect(self, other: Self) -> Result<Option<Self>> {
        self.validate()?;
        other.validate()?;
        let min = |a: outlines::OutlineCoordinate, b: outlines::OutlineCoordinate| -> Result<_> {
            Ok(if a.checked_cmp(b)?.is_lt() { a } else { b })
        };
        let max = |a: outlines::OutlineCoordinate, b: outlines::OutlineCoordinate| -> Result<_> {
            Ok(if a.checked_cmp(b)?.is_gt() { a } else { b })
        };
        let result = Self {
            left: max(self.left, other.left)?,
            top: max(self.top, other.top)?,
            right: min(self.right, other.right)?,
            bottom: min(self.bottom, other.bottom)?,
        };
        Ok((result.left.checked_cmp(result.right)?.is_lt()
            && result.top.checked_cmp(result.bottom)?.is_lt())
        .then_some(result))
    }
    pub fn contains(self, point: outlines::OutlinePoint) -> Result<bool> {
        self.validate()?;
        Ok(!point.x.checked_cmp(self.left)?.is_lt()
            && point.x.checked_cmp(self.right)?.is_lt()
            && !point.y.checked_cmp(self.top)?.is_lt()
            && point.y.checked_cmp(self.bottom)?.is_lt())
    }
}
/// The exact clip is authoritative and must be applied by the consumer; an empty
/// clip returns no operations. The inner batch retains the integer page clip.
#[derive(Debug, Clone)]
pub struct ExactDrawBatch {
    pub batch: DrawBatch,
    pub visible_clip: Option<ExactClip>,
}
