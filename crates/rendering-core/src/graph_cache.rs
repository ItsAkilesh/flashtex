//! Bounded expansion cache tied to one immutably borrowed resource graph. Cache
//! entries cannot cross graphs with different encoding/local-font bindings even
//! when their file hash keys happen to match. No device rounding or native paint.
use crate::{require, Result, ValidationError};
use flashtex_font_resources::{
    vf_graph::{NestedPacket, NestedPlacement, ResourceGraph, ResourceKey},
    Error,
};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    Resource(Error),
    PayloadBudget { required: usize, limit: usize },
}
#[derive(Debug, Clone)]
pub enum Outcome {
    Ready(Arc<NestedPacket>),
    Unavailable(Arc<Failure>),
}
#[derive(Debug, Clone)]
pub struct Lookup {
    pub outcome: Outcome,
    pub cache_hit: bool,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    pub hits: u64,
    pub expansions: u64,
    pub evictions: u64,
    pub entries: usize,
    pub retained_payload_bytes: usize,
}
struct Entry {
    outcome: Outcome,
    stamp: u64,
    bytes: usize,
}
pub struct GraphCache<'g, 'r> {
    graph: &'g ResourceGraph<'r>,
    entries: BTreeMap<(ResourceKey, u8), Entry>,
    age: BTreeMap<u64, (ResourceKey, u8)>,
    max_entries: usize,
    max_bytes: usize,
    clock: u64,
    stats: Stats,
}
impl<'g, 'r> GraphCache<'g, 'r> {
    pub fn new(graph: &'g ResourceGraph<'r>, max_entries: usize, max_bytes: usize) -> Result<Self> {
        require(
            (1..=65536).contains(&max_entries) && max_bytes >= 1024,
            "invalid graph cache budget",
        )?;
        Ok(Self {
            graph,
            entries: BTreeMap::new(),
            age: BTreeMap::new(),
            max_entries,
            max_bytes,
            clock: 0,
            stats: Stats::default(),
        })
    }
    pub fn stats(&self) -> Stats {
        Stats {
            entries: self.entries.len(),
            ..self.stats
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.age.clear();
        self.stats.retained_payload_bytes = 0;
    }
    pub fn lookup(&mut self, resource: &ResourceKey, code: u8) -> Result<Lookup> {
        validate_key(resource)?;
        self.clock = self
            .clock
            .checked_add(1)
            .ok_or_else(|| ValidationError("graph cache clock exhausted".into()))?;
        let key = (resource.clone(), code);
        if let Some(entry) = self.entries.get_mut(&key) {
            self.age.remove(&entry.stamp);
            entry.stamp = self.clock;
            self.age.insert(self.clock, key);
            self.stats.hits = self.stats.hits.saturating_add(1);
            return Ok(Lookup {
                outcome: entry.outcome.clone(),
                cache_hit: true,
            });
        }
        self.stats.expansions = self.stats.expansions.saturating_add(1);
        let result = self.graph.expand(resource, code);
        let (outcome, bytes) = match result {
            Ok(packet) => {
                let required = estimate(&packet);
                if required > self.max_bytes {
                    (
                        Outcome::Unavailable(Arc::new(Failure::PayloadBudget {
                            required,
                            limit: self.max_bytes,
                        })),
                        512,
                    )
                } else {
                    (Outcome::Ready(Arc::new(packet)), required)
                }
            }
            Err(error) => {
                let bytes = 512usize.saturating_add(error.to_string().len());
                (
                    Outcome::Unavailable(Arc::new(Failure::Resource(error))),
                    bytes,
                )
            }
        };
        if bytes <= self.max_bytes {
            while self.entries.len() >= self.max_entries
                || self.stats.retained_payload_bytes > self.max_bytes - bytes
            {
                let Some((stamp, key)) = self.age.pop_first() else {
                    break;
                };
                let entry = self
                    .entries
                    .remove(&key)
                    .ok_or_else(|| ValidationError("graph cache LRU mismatch".into()))?;
                debug_assert_eq!(stamp, entry.stamp);
                self.stats.retained_payload_bytes -= entry.bytes;
                self.stats.evictions = self.stats.evictions.saturating_add(1);
            }
            self.stats.retained_payload_bytes += bytes;
            self.age.insert(self.clock, key.clone());
            self.entries.insert(
                key,
                Entry {
                    outcome: outcome.clone(),
                    stamp: self.clock,
                    bytes,
                },
            );
        }
        Ok(Lookup {
            outcome,
            cache_hit: false,
        })
    }
}
fn validate_key(key: &ResourceKey) -> Result<()> {
    match key {
        ResourceKey::Physical {
            font_sha256,
            tfm_sha256,
            face_index,
        } => {
            crate::hash(font_sha256)?;
            crate::hash(tfm_sha256)?;
            require(*face_index == 0, "unsupported graph face")
        }
        ResourceKey::Virtual {
            vf_sha256,
            tfm_sha256,
        } => {
            crate::hash(vf_sha256)?;
            crate::hash(tfm_sha256)
        }
    }
}
/// Conservative retained payload estimate, excluding map/allocator overhead and
/// external Arcs. Expansion transient memory is bounded by the loader separately.
fn estimate(packet: &NestedPacket) -> usize {
    let mut bytes = 512usize;
    for placement in &packet.placements {
        let source = match placement {
            NestedPlacement::Glyph { source, .. } | NestedPlacement::Rule { source, .. } => source,
        };
        bytes = bytes
            .saturating_add(512)
            .saturating_add(source.len().saturating_mul(384));
    }
    bytes
}
