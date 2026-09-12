//! Selection uses caller-supplied exact cluster bounds/carets, never inferred
//! glyph advance rectangles. Historical inspection is separate from insertion.
use super::*;
use crate::{
    batch::ExactClip,
    outlines::{OutlineCoordinate, OutlinePoint},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionDirection {
    LeftToRight,
    RightToLeft,
}
#[derive(Debug, Clone, Copy)]
pub struct ClusterCarets {
    pub start: OutlinePoint,
    pub end: OutlinePoint,
}
#[derive(Debug, Clone, Copy)]
pub struct ClusterGeometry {
    pub cluster_index: usize,
    pub bounds: Option<ExactClip>,
    pub carets: ClusterCarets,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterSource {
    pub cluster_index: usize,
    pub source_range: std::ops::Range<usize>,
    pub text: String,
    pub glyph_count: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionHit {
    pub cluster: ClusterSource,
    pub caret_byte: usize,
    pub caret: OutlinePoint,
}
pub struct RegistrySelection {
    frame: RegistryFrame,
    instance: u64,
    epoch: u64,
    geometry: Vec<ClusterGeometry>,
}
pub struct InsertionDestination {
    frame: RegistryFrame,
    instance: u64,
    epoch: u64,
    byte: usize,
}
impl InsertionDestination {
    pub fn source_path(&self) -> &str {
        &self.frame.run().run().source().path
    }
    pub fn source_revision(&self) -> u64 {
        self.frame.run().run().source().revision
    }
    /// Informational only. Call validate_destination immediately before applying an edit.
    pub fn byte(&self) -> usize {
        self.byte
    }
}
fn source_current(frame: &RegistryFrame, path: &str, snapshot: &SourceSnapshot) -> BoundResult<()> {
    let s = frame.run().run().source();
    require(
        s.path == path
            && s.revision == snapshot.revision
            && s.source_sha256 == digest(snapshot.text.as_bytes()),
        "stale selection source",
    )?;
    for (i, c) in frame.run().run().shaped().clusters.iter().enumerate() {
        let range = frame
            .run()
            .run()
            .absolute_cluster_range(i)
            .map_err(BindingError::Font)?;
        require(
            snapshot.text.get(range) == Some(c.text.as_str()),
            "selection cluster/source disagreement",
        )?;
    }
    Ok(())
}
fn distance(a: OutlinePoint, b: OutlinePoint) -> Result<OutlineCoordinate> {
    let difference = |a: OutlineCoordinate, b: OutlineCoordinate| {
        a.checked_add(OutlineCoordinate::from_fraction(
            -b.numerator(),
            b.denominator(),
        )?)
    };
    let x = difference(a.x, b.x)?;
    let y = difference(a.y, b.y)?;
    x.checked_multiply(x)?.checked_add(y.checked_multiply(y)?)
}
impl RegistrySelection {
    /// Historical source/geometry inspection is allowed after registry replacement.
    /// This method cannot create an insertion authorization.
    pub fn inspect(&self, point: OutlinePoint) -> BoundResult<Option<SelectionHit>> {
        if !self.frame.run().placement().clip.contains(point)? {
            return Ok(None);
        }
        for geometry in self.geometry.iter().rev() {
            let Some(bounds) = geometry.bounds else {
                continue;
            };
            if !bounds.contains(point)? {
                continue;
            }
            let source = self
                .cluster(geometry.cluster_index)
                .ok_or_else(|| ValidationError("selection cluster index".into()))?;
            let start_distance = distance(point, geometry.carets.start)?;
            let end_distance = distance(point, geometry.carets.end)?;
            // Exact tie goes to logical start, including coincident empty-cluster carets.
            let end = start_distance.checked_cmp(end_distance)? == std::cmp::Ordering::Greater;
            return Ok(Some(SelectionHit {
                caret_byte: if end {
                    source.source_range.end
                } else {
                    source.source_range.start
                },
                caret: if end {
                    geometry.carets.end
                } else {
                    geometry.carets.start
                },
                cluster: source,
            }));
        }
        Ok(None)
    }
    pub fn cluster(&self, index: usize) -> Option<ClusterSource> {
        let run = self.frame.run().run();
        let cluster = run.shaped().clusters.get(index)?;
        Some(ClusterSource {
            cluster_index: index,
            source_range: run.absolute_cluster_range(index).ok()?,
            text: cluster.text.clone(),
            glyph_count: cluster.glyphs.len(),
        })
    }
}
impl RegistryRenderer {
    pub fn selection(
        &self,
        lease: &RenderLease,
        frame: &RegistryFrame,
        path: &str,
        snapshot: &SourceSnapshot,
        geometry: Vec<ClusterGeometry>,
        direction: SelectionDirection,
    ) -> BoundResult<RegistrySelection> {
        self.current(lease)?;
        require(
            Arc::ptr_eq(&lease.resource, &frame.retained_resource),
            "selection frame/lease mismatch",
        )?;
        source_current(frame, path, snapshot)?;
        require(
            direction == SelectionDirection::LeftToRight,
            "unsupported RTL selection ordering",
        )?;
        require(
            geometry.len() <= 65536 && geometry.len() == frame.run().run().shaped().clusters.len(),
            "selection geometry requires explicit complete cluster coverage",
        )?;
        let mut seen = BTreeSet::new();
        for g in &geometry {
            require(
                g.cluster_index < geometry.len() && seen.insert(g.cluster_index),
                "selection duplicate cluster",
            )?;
            if let Some(bounds) = g.bounds {
                bounds.validate()?;
            }
            require(
                g.carets.start.x.checked_cmp(g.carets.end.x)? != std::cmp::Ordering::Greater,
                "unsupported reordered cluster carets",
            )?;
        }
        Ok(RegistrySelection {
            frame: frame.clone(),
            instance: self.instance,
            epoch: self.epoch,
            geometry,
        })
    }
    pub fn destination(
        &self,
        lease: &RenderLease,
        index: &RegistrySelection,
        point: OutlinePoint,
        path: &str,
        snapshot: &SourceSnapshot,
    ) -> BoundResult<Option<InsertionDestination>> {
        self.current(lease)?;
        if index.instance != self.instance
            || index.epoch != self.epoch
            || !Arc::ptr_eq(&index.frame.retained_resource, &lease.resource)
        {
            return Err(BindingError::StaleLease);
        }
        source_current(&index.frame, path, snapshot)?;
        Ok(index.inspect(point)?.map(|hit| InsertionDestination {
            frame: index.frame.clone(),
            instance: self.instance,
            epoch: self.epoch,
            byte: hit.caret_byte,
        }))
    }
    /// Recheck at the edit boundary; a previously returned destination is not an
    /// evergreen capability. Source or registry changes invalidate it.
    pub fn validate_destination(
        &self,
        lease: &RenderLease,
        destination: &InsertionDestination,
        path: &str,
        snapshot: &SourceSnapshot,
    ) -> BoundResult<usize> {
        self.current(lease)?;
        if destination.instance != self.instance
            || destination.epoch != self.epoch
            || !Arc::ptr_eq(&destination.frame.retained_resource, &lease.resource)
        {
            return Err(BindingError::StaleLease);
        }
        source_current(&destination.frame, path, snapshot)?;
        require(
            snapshot.text.is_char_boundary(destination.byte),
            "selection UTF8 caret",
        )?;
        Ok(destination.byte)
    }
}
