//! Bounded LRU of immutable unhinted, expanded glyph paths. Cache measurements
//! count font expansion only, not native painting or end-to-end preview latency.
use crate::*;
use flashtex_font_resources::{Error as FontError, FontResource, GlyphInstance, PathCommand};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExpansionPolicy {
    UnhintedExactComponentsV1,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlyphPathKey {
    pub font_sha256: String,
    pub face_index: u32,
    pub original_gid: u16,
    pub policy: ExpansionPolicy,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathFailure {
    Font(FontError),
    PayloadBudget { required: usize, limit: usize },
}
#[derive(Debug)]
pub struct GlyphPathData {
    pub key: GlyphPathKey,
    pub instances: Vec<GlyphInstance>,
    pub commands: Vec<PathCommand>,
    pub point_count: usize,
    pub contour_count: usize,
}
#[derive(Debug, Clone)]
pub enum PathOutcome {
    Ready(Arc<GlyphPathData>),
    Unavailable(Arc<PathFailure>),
}
#[derive(Debug, Clone)]
pub struct PathLookup {
    pub outcome: PathOutcome,
    pub cache_hit: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub expansions: u64,
    pub evictions: u64,
    pub retained_payload_bytes: usize,
    pub entries: usize,
}
struct Entry {
    outcome: PathOutcome,
    stamp: u64,
    payload_bytes: usize,
}
pub struct GlyphPathCache {
    entries: BTreeMap<GlyphPathKey, Entry>,
    age: BTreeMap<u64, GlyphPathKey>,
    clock: u64,
    max_entries: usize,
    max_payload_bytes: usize,
    stats: CacheStats,
}
impl GlyphPathCache {
    pub fn new(max_entries: usize, max_payload_bytes: usize) -> Result<Self> {
        require(
            (1..=65536).contains(&max_entries) && max_payload_bytes >= 256,
            "invalid glyph cache capacity",
        )?;
        Ok(Self {
            entries: BTreeMap::new(),
            age: BTreeMap::new(),
            clock: 0,
            max_entries,
            max_payload_bytes,
            stats: CacheStats::default(),
        })
    }
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.stats;
        stats.entries = self.entries.len();
        stats
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.age.clear();
        self.stats.retained_payload_bytes = 0;
    }
    fn stamp(&mut self) -> Result<u64> {
        self.clock = self
            .clock
            .checked_add(1)
            .ok_or_else(|| ValidationError("glyph cache clock exhausted".into()))?;
        Ok(self.clock)
    }
    pub fn lookup(&mut self, font: &FontResource, gid: u16) -> Result<PathLookup> {
        let descriptor = font.descriptor();
        let key = GlyphPathKey {
            font_sha256: descriptor.sha256.clone(),
            face_index: descriptor.face_index,
            original_gid: gid,
            policy: ExpansionPolicy::UnhintedExactComponentsV1,
        };
        let stamp = self.stamp()?;
        if let Some(entry) = self.entries.get_mut(&key) {
            self.age.remove(&entry.stamp);
            entry.stamp = stamp;
            self.age.insert(stamp, key);
            self.stats.hits += 1;
            return Ok(PathLookup {
                outcome: entry.outcome.clone(),
                cache_hit: true,
            });
        }
        self.stats.misses += 1;
        self.stats.expansions += 1;
        let expanded = font.expanded_outline(gid).and_then(|outline| {
            let commands = outline.quadratic_path()?.collect();
            Ok(GlyphPathData {
                key: key.clone(),
                instances: outline.instances,
                commands,
                point_count: outline.points.len(),
                contour_count: outline.contour_ends.len(),
            })
        });
        let (outcome, size) = match expanded {
            Ok(data) => {
                let size = payload_size(&data)?;
                if size > self.max_payload_bytes {
                    let error = PathFailure::PayloadBudget {
                        required: size,
                        limit: self.max_payload_bytes,
                    };
                    (PathOutcome::Unavailable(Arc::new(error)), 128)
                } else {
                    (PathOutcome::Ready(Arc::new(data)), size)
                }
            }
            Err(error) => {
                let size = 128usize.saturating_add(format!("{error:?}").len());
                (
                    PathOutcome::Unavailable(Arc::new(PathFailure::Font(error))),
                    size,
                )
            }
        };
        // Extremely long diagnostics can be returned without retaining them; the
        // cache limit is never expanded implicitly to accommodate an error.
        if size <= self.max_payload_bytes {
            while self.entries.len() >= self.max_entries
                || size > self.max_payload_bytes - self.stats.retained_payload_bytes
            {
                let (old_stamp, old_key) = self
                    .age
                    .pop_first()
                    .expect("occupied cache under capacity pressure");
                let entry = self
                    .entries
                    .remove(&old_key)
                    .expect("age index matches cache");
                debug_assert_eq!(entry.stamp, old_stamp);
                self.stats.retained_payload_bytes -= entry.payload_bytes;
                self.stats.evictions += 1;
            }
            self.entries.insert(
                key.clone(),
                Entry {
                    outcome: outcome.clone(),
                    stamp,
                    payload_bytes: size,
                },
            );
            self.age.insert(stamp, key);
            self.stats.retained_payload_bytes += size;
        }
        Ok(PathLookup {
            outcome,
            cache_hit: false,
        })
    }
}
fn payload_size(data: &GlyphPathData) -> Result<usize> {
    data.commands
        .len()
        .checked_mul(std::mem::size_of::<PathCommand>())
        .and_then(|size| {
            data.instances
                .len()
                .checked_mul(std::mem::size_of::<GlyphInstance>())
                .and_then(|instances| size.checked_add(instances))
        })
        .and_then(|size| size.checked_add(128))
        .ok_or_else(|| ValidationError("glyph cache payload size overflow".into()))
}
