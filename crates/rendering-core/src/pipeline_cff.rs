//! Explicit pipeline CFF consumer. This is not a negotiated wire-profile change.
//! Registry resources verify font, face, table and license bytes upstream; this
//! adapter binds their immutable identity to each producer declaration and source.
use crate::{
    batch::{ExactClip, PrimitiveId},
    cubic::CachedCffConsumer,
    mixed::*,
    outlines::{OutlineCoordinate, OutlinePoint},
    *,
};
use flashtex_font_resources::{
    cff::{CacheLimits, HintPolicy},
    registry::CffFontResource,
};
use std::sync::Arc;

pub struct PipelineCff {
    list: DisplayList,
    fonts: BTreeMap<String, CachedCffConsumer>,
}
impl PipelineCff {
    pub fn bind(
        bytes: &[u8],
        capabilities: &Capabilities,
        documents: &BTreeMap<String, SourceSnapshot>,
        resources: &BTreeMap<String, Arc<CffFontResource>>,
    ) -> Result<Self> {
        let Message::DisplayList(list) = parse(bytes)?.message else {
            return Err(ValidationError("display list required".into()));
        };
        list.validate_profile(capabilities, true)?;
        for d in &list.documents {
            let source = documents
                .get(&d.path)
                .ok_or_else(|| ValidationError("missing source snapshot".into()))?;
            require(
                source.revision == d.revision
                    && source.text.len() as u64 == d.byte_length
                    && digest(source.text.as_bytes()) == d.sha256,
                "source identity mismatch",
            )?;
        }
        let mut fonts = BTreeMap::new();
        for f in &list.fonts {
            require(
                f.format == "opentype-cff",
                "pipeline CFF adapter requires explicit CFF resource",
            )?;
            let r = resources
                .get(&f.font_id)
                .ok_or_else(|| ValidationError("missing immutable CFF resource".into()))?;
            let d = r.descriptor();
            require(
                f.sha256 == d.sha256
                    && f.face_index == d.face_index
                    && f.byte_length == d.byte_length
                    && f.units_per_em == d.units_per_em
                    && f.glyph_count == d.glyph_count
                    && f.postscript_name == d.postscript_name,
                "CFF resource metadata mismatch",
            )?;
            let cache = r
                .outline_cache(CacheLimits {
                    max_entries: 64,
                    max_bytes: 8 * 1024 * 1024,
                })
                .map_err(|e| ValidationError(format!("CFF resource: {e:?}")))?;
            fonts.insert(f.font_id.clone(), CachedCffConsumer::new(cache));
        }
        for page in &list.pages {
            for item in &page.items {
                match item {
                    Item::GlyphRun(run) => {
                        for c in &run.clusters {
                            if let Some(r) = &c.sources {
                                validate_source_bytes(r, documents)?;
                            }
                        }
                    }
                    Item::Rule(rule) => {
                        if let Some(r) = &rule.sources {
                            validate_source_bytes(r, documents)?;
                        }
                    }
                }
            }
        }
        for d in &list.diagnostics {
            validate_source_bytes(&d.sources, documents)?;
        }
        Ok(Self { list, fonts })
    }
    /// Producer advances remain distinct from CFF outline advances; absolute origins
    /// drive painting and the immutable display retains the original metrics.
    pub fn display(&self) -> &DisplayList {
        &self.list
    }
    pub fn page(
        &self,
        index: usize,
        policy: HintPolicy,
        limits: MixedLimits,
    ) -> MixedResult<MixedBatch> {
        let p = self.list.pages.get(index).ok_or(MixedError::Identity)?;
        let clip = ExactClip::from_rect(&HitRect {
            x: Tick(0),
            top: Tick(0),
            width: p.width,
            height: p.height,
        })?;
        let mut primitives = Vec::new();
        let mut commands = 0usize;
        let q = |t: Tick| OutlineCoordinate::from_fraction(t.0 as i128, 1);
        for (item_index, item) in p.items.iter().enumerate() {
            match item {
                Item::GlyphRun(run) => {
                    for (glyph_index, g) in run.glyphs.iter().enumerate() {
                        if primitives.len() >= limits.max_primitives {
                            return Err(MixedError::Budget);
                        }
                        let c = &run.clusters[g.cluster as usize];
                        let outline = self.fonts[&run.font_id]
                            .place_cached(
                                g.gid as u16,
                                policy,
                                q(run.font_size)?,
                                OutlinePoint {
                                    x: q(g.origin_x)?,
                                    y: q(g.baseline_y)?,
                                },
                                limits.max_commands.saturating_sub(commands),
                            )?
                            .outline;
                        commands = commands
                            .checked_add(outline.commands.len())
                            .ok_or(MixedError::Budget)?;
                        primitives.push(MixedPrimitive {
                            identity: PrimitiveId {
                                item_index,
                                glyph_index: Some(glyph_index),
                            },
                            geometry: MixedGeometry::Cubic(Box::new(outline)),
                            clip,
                            paint: run.paint.clone(),
                            source_chain: Vec::new(),
                            sources: c.sources.clone().unwrap_or_default(),
                            synthetic_reason: c.synthetic_reason.clone(),
                            logical_interval: Some((c.text_start_byte, c.text_end_byte)),
                            font_sha256: Some(
                                self.fonts[&run.font_id].identity().font_sha256.clone(),
                            ),
                            original_gid: Some(g.gid),
                        });
                    }
                }
                Item::Rule(r) => {
                    if primitives.len() >= limits.max_primitives {
                        return Err(MixedError::Budget);
                    }
                    primitives.push(MixedPrimitive {
                        identity: PrimitiveId {
                            item_index,
                            glyph_index: None,
                        },
                        geometry: MixedGeometry::Rule(ExactClip::from_rect(&HitRect {
                            x: r.x,
                            top: r.top,
                            width: r.width,
                            height: r.height,
                        })?),
                        clip,
                        paint: r.paint.clone(),
                        source_chain: Vec::new(),
                        sources: r.sources.clone().unwrap_or_default(),
                        synthetic_reason: r.synthetic_reason.clone(),
                        logical_interval: None,
                        font_sha256: None,
                        original_gid: None,
                    });
                }
            }
        }
        MixedBatch::assembled(
            MixedContext {
                project_id: &self.list.project_id,
                revision: self.list.revision,
                page: p.number,
                page_width: p.width,
                page_height: p.height,
                clip,
            },
            primitives,
            limits,
        )
    }
}
