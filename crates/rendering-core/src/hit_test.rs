//! Exact fixed-point hit testing of compiler-supplied cluster/rule geometry.
//! No font measurement or glyph-width division is performed. Source ranges stay
//! intact when a caret's logical text offset has no one-to-one TeX mapping.
use crate::transform::ExactPoint;
use crate::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: Tick,
    pub y: Tick,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalSelection {
    Caret { text_byte: u64 },
    WholeCluster { start_byte: u64, end_byte: u64 },
    Rule,
}
#[derive(Debug, Clone)]
pub struct Hit {
    pub page: u32,
    pub item_index: usize,
    pub cluster_index: Option<usize>,
    pub selection: LogicalSelection,
    /// Original ranges, not inferred from glyph widths or logical caret indices.
    pub sources: Vec<SourceRange>,
    pub synthetic_reason: Option<String>,
}
#[derive(Debug, Clone)]
struct Entry {
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
    item_index: usize,
    cluster_index: Option<usize>,
    details: Arc<Details>,
}
#[derive(Debug, Clone)]
struct Details {
    text_range: Option<(u64, u64)>,
    carets: Vec<Caret>,
    sources: Vec<SourceRange>,
    synthetic_reason: Option<String>,
}
#[derive(Debug, Clone)]
struct IndexedPage {
    width: Tick,
    height: Tick,
    entries: Vec<Entry>,
    prefix_bottom: Vec<i64>,
}
#[derive(Debug, Clone)]
pub struct PageIndex {
    pub project_id: String,
    pub revision: u64,
    pages: BTreeMap<u32, IndexedPage>,
}
impl PageIndex {
    pub fn build(list: &DisplayList, capabilities: &Capabilities) -> Result<Self> {
        list.validate(capabilities)?;
        Ok(Self::build_validated(list))
    }
    /// Private validated CFF path: PipelineCff cannot be constructed without
    /// complete profile/source/resource validation and exposes no mutable list.
    pub(crate) fn from_pipeline(pipeline: &crate::pipeline_cff::PipelineCff) -> Self {
        Self::build_validated(pipeline.display())
    }
    fn build_validated(list: &DisplayList) -> Self {
        let mut pages = BTreeMap::new();
        for page in &list.pages {
            let mut entries = Vec::new();
            for (item_index, item) in page.items.iter().enumerate() {
                match item {
                    Item::GlyphRun(run) => {
                        for (cluster_index, cluster) in run.clusters.iter().enumerate() {
                            let details = Arc::new(Details {
                                text_range: Some((cluster.text_start_byte, cluster.text_end_byte)),
                                carets: cluster.carets.clone(),
                                sources: cluster.sources.clone().unwrap_or_default(),
                                synthetic_reason: cluster.synthetic_reason.clone(),
                            });
                            for rect in &cluster.hit_rects {
                                if let Some((left, top, right, bottom)) = clipped(rect, page) {
                                    entries.push(Entry {
                                        left,
                                        top,
                                        right,
                                        bottom,
                                        item_index,
                                        cluster_index: Some(cluster_index),
                                        details: Arc::clone(&details),
                                    });
                                }
                            }
                        }
                    }
                    Item::Rule(rule) => {
                        let rect = HitRect {
                            x: rule.x,
                            top: rule.top,
                            width: rule.width,
                            height: rule.height,
                        };
                        if let Some((left, top, right, bottom)) = clipped(&rect, page) {
                            entries.push(Entry {
                                left,
                                top,
                                right,
                                bottom,
                                item_index,
                                cluster_index: None,
                                details: Arc::new(Details {
                                    text_range: None,
                                    carets: vec![],
                                    sources: rule.sources.clone().unwrap_or_default(),
                                    synthetic_reason: rule.synthetic_reason.clone(),
                                }),
                            });
                        }
                    }
                }
            }
            entries.sort_by_key(|entry| (entry.top, entry.item_index, entry.cluster_index));
            let mut maximum = 0;
            let prefix_bottom = entries
                .iter()
                .map(|entry| {
                    maximum = maximum.max(entry.bottom);
                    maximum
                })
                .collect();
            pages.insert(
                page.number,
                IndexedPage {
                    width: page.width,
                    height: page.height,
                    entries,
                    prefix_bottom,
                },
            );
        }
        Self {
            project_id: list.project_id.clone(),
            revision: list.revision,
            pages,
        }
    }
    /// Half-open rectangles and page bounds; later paint order wins overlaps.
    /// An absent caret list selects the entire cluster, never an invented caret.
    pub fn hit_test(
        &self,
        project_id: &str,
        revision: u64,
        page: u32,
        point: Point,
    ) -> Result<Option<Hit>> {
        self.hit_test_exact(project_id, revision, page, ExactPoint::from_point(point)?)
    }
    pub fn hit_test_exact(
        &self,
        project_id: &str,
        revision: u64,
        page: u32,
        exact: ExactPoint,
    ) -> Result<Option<Hit>> {
        let point = exact.floor();
        require(
            project_id == self.project_id && revision == self.revision,
            "stale hit-test index",
        )?;
        point.x.validate()?;
        point.y.validate()?;
        let indexed = self
            .pages
            .get(&page)
            .ok_or_else(|| ValidationError("unknown hit-test page".into()))?;
        if point.x.0 < 0
            || point.y.0 < 0
            || point.x.0 >= indexed.width.0
            || point.y.0 >= indexed.height.0
        {
            return Ok(None);
        }
        let end = indexed
            .entries
            .partition_point(|entry| entry.top <= point.y.0);
        let mut winner: Option<&Entry> = None;
        for i in (0..end).rev() {
            if indexed.prefix_bottom[i] <= point.y.0 {
                break;
            }
            let entry = &indexed.entries[i];
            if point.x.0 >= entry.left
                && point.x.0 < entry.right
                && point.y.0 < entry.bottom
                && winner.is_none_or(|old| {
                    (entry.item_index, entry.cluster_index) > (old.item_index, old.cluster_index)
                })
            {
                winner = Some(entry);
            }
        }
        winner
            .map(|entry| {
                Ok(Hit {
                    page,
                    item_index: entry.item_index,
                    cluster_index: entry.cluster_index,
                    selection: selection(entry, exact)?,
                    sources: entry.details.sources.clone(),
                    synthetic_reason: entry.details.synthetic_reason.clone(),
                })
            })
            .transpose()
    }
}
fn clipped(rect: &HitRect, page: &Page) -> Option<(i64, i64, i64, i64)> {
    // DisplayList::validate has already checked these exact additions.
    let left = rect.x.0.max(0);
    let top = rect.top.0.max(0);
    let right = (rect.x.0 + rect.width.0).min(page.width.0);
    let bottom = (rect.top.0 + rect.height.0).min(page.height.0);
    (left < right && top < bottom).then_some((left, top, right, bottom))
}
fn selection(entry: &Entry, point: ExactPoint) -> Result<LogicalSelection> {
    let mut nearest: Option<(i128, u64)> = None;
    for caret in &entry.details.carets {
        let candidate = (point.caret_distance(caret)?, caret.text_byte);
        if nearest.is_none_or(|old| candidate < old) {
            nearest = Some(candidate);
        }
    }
    if let Some((_, text_byte)) = nearest {
        return Ok(LogicalSelection::Caret { text_byte });
    }
    Ok(match entry.details.text_range {
        Some((start_byte, end_byte)) => LogicalSelection::WholeCluster {
            start_byte,
            end_byte,
        },
        None => LogicalSelection::Rule,
    })
}
