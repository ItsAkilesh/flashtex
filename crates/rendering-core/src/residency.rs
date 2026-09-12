//! Generation-bound mixed page residency. Jobs capture an immutable source/resource
//! lease before building; completion under a superseded lease cannot publish.
use crate::{
    mixed::{MixedBatch, MixedContext, MixedGeometry, MixedInput, MixedLimits},
    *,
};
use serde::Serialize;
use std::sync::Arc;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum ResourceIdentity {
    TrueType {
        sha256: String,
    },
    CffTable {
        sha256: String,
    },
    CffFont {
        font_sha256: String,
        table_sha256: String,
        face_index: u32,
        table_start: usize,
        table_end: usize,
    },
}
#[derive(Clone)]
pub struct ResidencyInputs {
    project_id: String,
    revision: u64,
    fingerprint: String,
    documents: Vec<DocumentResource>,
    snapshots: BTreeMap<String, SourceSnapshot>,
    resources: BTreeSet<ResourceIdentity>,
}
impl ResidencyInputs {
    pub fn new(
        project_id: &str,
        revision: u64,
        configuration_sha256: &str,
        documents: Vec<DocumentResource>,
        snapshots: BTreeMap<String, SourceSnapshot>,
        resources: BTreeSet<ResourceIdentity>,
    ) -> Result<Self> {
        id(project_id)?;
        hash(configuration_sha256)?;
        require(
            documents.len() <= 4096 && resources.len() <= 4096,
            "residency input count budget",
        )?;
        let mut documents = documents;
        documents.sort_by(|a, b| a.path.cmp(&b.path));
        let mut previous = None;
        let mut total = 0usize;
        for doc in &documents {
            path(&doc.path)?;
            hash(&doc.sha256)?;
            require(
                previous != Some(doc.path.as_str()),
                "duplicate residency document",
            )?;
            previous = Some(&doc.path);
            let snapshot = snapshots
                .get(&doc.path)
                .ok_or_else(|| ValidationError("missing residency source snapshot".into()))?;
            total = total
                .checked_add(snapshot.text.len())
                .ok_or_else(|| ValidationError("source byte overflow".into()))?;
            require(total <= MAX_MESSAGE_BYTES, "residency source byte budget")?;
            require(
                snapshot.revision == doc.revision
                    && snapshot.text.len() as u64 == doc.byte_length
                    && digest(snapshot.text.as_bytes()) == doc.sha256,
                "residency source identity mismatch",
            )?;
        }
        require(
            snapshots.len() == documents.len(),
            "undeclared residency snapshot",
        )?;
        for resource in &resources {
            match resource {
                ResourceIdentity::TrueType { sha256 } | ResourceIdentity::CffTable { sha256 } => {
                    hash(sha256)?
                }
                ResourceIdentity::CffFont {
                    font_sha256,
                    table_sha256,
                    face_index,
                    table_start,
                    table_end,
                } => {
                    hash(font_sha256)?;
                    hash(table_sha256)?;
                    require(
                        *face_index == 0 && table_start < table_end,
                        "invalid CFF residency identity",
                    )?;
                }
            }
        }
        let fingerprint = digest(
            &serde_json::to_vec(&(
                project_id,
                revision,
                configuration_sha256,
                &documents,
                &resources,
            ))
            .map_err(|e| ValidationError(e.to_string()))?,
        );
        Ok(Self {
            project_id: project_id.into(),
            revision,
            fingerprint,
            documents,
            snapshots,
            resources,
        })
    }
}
#[derive(Clone)]
pub struct MixedLease {
    generation: u64,
    inputs: Arc<ResidencyInputs>,
}
pub struct PreparedMixed {
    generation: u64,
    fingerprint: String,
    batch: MixedBatch,
}
impl MixedLease {
    /// Compile against these captured snapshots; obtaining a fresh lease after
    /// compilation does not retroactively bind old output to new source bytes.
    pub fn snapshots(&self) -> &BTreeMap<String, SourceSnapshot> {
        &self.inputs.snapshots
    }
    pub fn build(
        &self,
        context: MixedContext<'_>,
        inputs: &[MixedInput<'_>],
        limits: MixedLimits,
    ) -> Result<PreparedMixed> {
        require(
            context.project_id == self.inputs.project_id
                && context.revision == self.inputs.revision,
            "mixed lease identity mismatch",
        )?;
        let batch = MixedBatch::build(context, inputs, limits)
            .map_err(|e| ValidationError(format!("mixed preparation: {e:?}")))?;
        let docs: BTreeMap<_, _> = self
            .inputs
            .documents
            .iter()
            .map(|d| (d.path.as_str(), d))
            .collect();
        for primitive in batch.primitives() {
            for range in &primitive.sources {
                source(range, &docs)?;
            }
            validate_source_bytes(&primitive.sources, &self.inputs.snapshots)?;
            let identity = match &primitive.geometry {
                MixedGeometry::Quadratic(_) => Some(ResourceIdentity::TrueType {
                    sha256: primitive.font_sha256.clone().ok_or_else(|| {
                        ValidationError("quadratic residency font missing".into())
                    })?,
                }),
                MixedGeometry::Cubic(cubic) => {
                    Some(if let Some(full) = &cubic.full_font_identity {
                        ResourceIdentity::CffFont {
                            font_sha256: full.font_sha256.clone(),
                            table_sha256: full.cff_sha256.clone(),
                            face_index: full.face_index,
                            table_start: full.table_range.start,
                            table_end: full.table_range.end,
                        }
                    } else {
                        ResourceIdentity::CffTable {
                            sha256: cubic.cff_table_sha256.clone(),
                        }
                    })
                }
                MixedGeometry::Rule(_) => None,
            };
            if let Some(identity) = identity {
                require(
                    self.inputs.resources.contains(&identity),
                    "unbound mixed residency resource",
                )?;
            }
        }
        Ok(PreparedMixed {
            generation: self.generation,
            fingerprint: self.inputs.fingerprint.clone(),
            batch,
        })
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ResidencyLimits {
    pub max_pages: usize,
    pub max_encoded_bytes: usize,
    pub max_commands: usize,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResidencyStats {
    pub pages: usize,
    pub encoded_bytes: usize,
    pub commands: usize,
    pub evictions: u64,
}
struct Page {
    batch: Arc<MixedBatch>,
    used: u64,
}
pub struct MixedResidency {
    project_id: String,
    limits: ResidencyLimits,
    generation: u64,
    revision: Option<u64>,
    current: Option<Arc<ResidencyInputs>>,
    pages: BTreeMap<u32, Page>,
    clock: u64,
    stats: ResidencyStats,
}
impl MixedResidency {
    pub fn new(project_id: &str, limits: ResidencyLimits) -> Result<Self> {
        id(project_id)?;
        require(
            (1..=4096).contains(&limits.max_pages)
                && (1..=MAX_MESSAGE_BYTES).contains(&limits.max_encoded_bytes)
                && (1..=2_000_000).contains(&limits.max_commands),
            "invalid residency budget",
        )?;
        Ok(Self {
            project_id: project_id.into(),
            limits,
            generation: 0,
            revision: None,
            current: None,
            pages: BTreeMap::new(),
            clock: 0,
            stats: ResidencyStats::default(),
        })
    }
    pub fn begin(&mut self, inputs: ResidencyInputs) -> Result<MixedLease> {
        require(
            inputs.project_id == self.project_id
                && self.revision.is_none_or(|r| inputs.revision >= r),
            "stale residency revision",
        )?;
        if self
            .current
            .as_ref()
            .is_none_or(|v| v.fingerprint != inputs.fingerprint)
        {
            self.invalidate()?;
            self.revision = Some(inputs.revision);
            self.current = Some(Arc::new(inputs));
        }
        Ok(MixedLease {
            generation: self.generation,
            inputs: self.current.as_ref().unwrap().clone(),
        })
    }
    pub fn invalidate(&mut self) -> Result<()> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| ValidationError("residency generation exhausted".into()))?;
        self.current = None;
        self.pages.clear();
        self.stats.pages = 0;
        self.stats.encoded_bytes = 0;
        self.stats.commands = 0;
        Ok(())
    }
    pub fn stats(&self) -> ResidencyStats {
        self.stats
    }
    fn valid(&self, lease: &MixedLease) -> Result<()> {
        require(
            lease.generation == self.generation
                && self
                    .current
                    .as_ref()
                    .is_some_and(|v| v.fingerprint == lease.inputs.fingerprint),
            "stale mixed lease",
        )
    }
    pub fn page(&mut self, lease: &MixedLease, page: u32) -> Result<Option<Arc<MixedBatch>>> {
        self.valid(lease)?;
        self.clock = self
            .clock
            .checked_add(1)
            .ok_or_else(|| ValidationError("residency LRU exhausted".into()))?;
        Ok(self.pages.get_mut(&page).map(|entry| {
            entry.used = self.clock;
            entry.batch.clone()
        }))
    }
    pub fn install(&mut self, prepared: PreparedMixed) -> Result<Arc<MixedBatch>> {
        require(
            prepared.generation == self.generation
                && self
                    .current
                    .as_ref()
                    .is_some_and(|v| v.fingerprint == prepared.fingerprint),
            "stale mixed completion",
        )?;
        let (project, revision, page) = prepared.batch.identity();
        require(
            project == self.project_id && Some(revision) == self.revision,
            "resident batch identity mismatch",
        )?;
        let bytes = prepared.batch.fixture_bytes().len();
        let commands = prepared.batch.command_count();
        require(
            bytes <= self.limits.max_encoded_bytes && commands <= self.limits.max_commands,
            "resident batch exceeds budget",
        )?;
        if let Some(existing) = self.pages.get(&page) {
            require(
                existing.batch.fixture_bytes() == prepared.batch.fixture_bytes(),
                "conflicting mixed result for one source/resource/configuration identity",
            )?;
            return Ok(existing.batch.clone());
        }
        self.clock = self
            .clock
            .checked_add(1)
            .ok_or_else(|| ValidationError("residency LRU exhausted".into()))?;
        while self.pages.len() >= self.limits.max_pages
            || self.stats.encoded_bytes > self.limits.max_encoded_bytes - bytes
            || self.stats.commands > self.limits.max_commands - commands
        {
            let key = *self
                .pages
                .iter()
                .min_by_key(|(_, v)| v.used)
                .map(|(k, _)| k)
                .ok_or_else(|| ValidationError("residency eviction invariant".into()))?;
            let old = self.pages.remove(&key).unwrap();
            self.stats.encoded_bytes -= old.batch.fixture_bytes().len();
            self.stats.commands -= old.batch.command_count();
            self.stats.evictions = self.stats.evictions.saturating_add(1);
        }
        let batch = Arc::new(prepared.batch);
        self.pages.insert(
            page,
            Page {
                batch: batch.clone(),
                used: self.clock,
            },
        );
        self.stats.encoded_bytes += bytes;
        self.stats.commands += commands;
        self.stats.pages = self.pages.len();
        Ok(batch)
    }
}
