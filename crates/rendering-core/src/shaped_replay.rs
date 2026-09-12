//! Bounded exact offline shaped-run evidence. No font-byte verification or paint
//! permission follows from parsing. Source verification is an explicit operation.
use crate::{
    batch::ExactClip,
    mixed::BoundedOutput,
    outlines::{OutlineCoordinate, OutlinePoint, PlacedPathCommand},
    shaped_run::{PlacedShapedRun, ShapedGeometry},
    *,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Rational([String; 2]);
impl Rational {
    fn new(v: OutlineCoordinate) -> Self {
        Self([v.numerator().to_string(), v.denominator().to_string()])
    }
    fn exact(&self) -> Result<OutlineCoordinate> {
        let n: i128 = self.0[0]
            .parse()
            .map_err(|_| ValidationError("rational numerator".into()))?;
        let d: u128 = self.0[1]
            .parse()
            .map_err(|_| ValidationError("rational denominator".into()))?;
        let v = OutlineCoordinate::from_fraction(n, d)?;
        require(
            v.numerator().to_string() == self.0[0] && v.denominator().to_string() == self.0[1],
            "noncanonical exact rational",
        )?;
        Ok(v)
    }
}
type Point = [Rational; 2];
fn point(v: OutlinePoint) -> Point {
    [Rational::new(v.x), Rational::new(v.y)]
}
fn exact(p: &Point) -> Result<OutlinePoint> {
    Ok(OutlinePoint {
        x: p[0].exact()?,
        y: p[1].exact()?,
    })
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Command {
    Move(Point),
    Line(Point),
    Quad([Point; 2]),
    Cubic([Point; 3]),
    Close,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    font_sha256: String,
    engine_font_id: String,
    face_index: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    path: String,
    revision: u64,
    sha256: String,
    range: [usize; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {
    ligatures: bool,
    kerning: bool,
    compose_marks: bool,
    cmap_ligature_fallback: bool,
    fail_on_unsupported_lookups: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Note {
    table: String,
    feature: String,
    detail: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Glyph {
    original_gid: u16,
    advance_units: i32,
    x_offset: i32,
    y_offset: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cluster {
    source_range: [usize; 2],
    text: String,
    glyphs: Vec<Glyph>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CffResource {
    font_sha256: String,
    cff_sha256: String,
    face_index: u32,
    table_range: [usize; 2],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stem {
    vertical: bool,
    delta: Rational,
    width: Rational,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Mask {
    counter: bool,
    stem_count: usize,
    bytes: Vec<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CffMetadata {
    resource: CffResource,
    font_matrix: [Rational; 6],
    outline_advance: Point,
    hint_policy: String,
    stems: Vec<Stem>,
    masks: Vec<Mask>,
    flex_depths: Vec<Rational>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimitiveIdentity {
    item_index: usize,
    glyph_index: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Path {
    identity: PrimitiveIdentity,
    cluster_index: usize,
    original_gid: u16,
    source_range: [usize; 2],
    origin: Point,
    advance: Rational,
    kind: String,
    commands: Vec<Command>,
    cff: Option<CffMetadata>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    format: String,
    identity: Identity,
    source: Source,
    shape_cache_key: String,
    options: Options,
    units_per_em: u16,
    ligatures_applied: usize,
    kerning_source: String,
    notes: Vec<Note>,
    clusters: Vec<Cluster>,
    item_index: usize,
    size: Rational,
    origin: Point,
    clip: [Rational; 4],
    advance: Rational,
    primitives: Vec<Path>,
    command_count: usize,
    hinting_applied: bool,
}
#[derive(Debug, Clone, Copy)]
pub struct ReplayLimits {
    pub max_bytes: usize,
    pub max_clusters: usize,
    pub max_glyphs: usize,
    pub max_commands: usize,
}
impl Default for ReplayLimits {
    fn default() -> Self {
        Self {
            max_bytes: MAX_MESSAGE_BYTES,
            max_clusters: 65536,
            max_glyphs: 100000,
            max_commands: 2000000,
        }
    }
}
pub struct ShapedReplay {
    fixture: Fixture,
    value: Value,
    raw_sha256: String,
}
impl ShapedReplay {
    pub fn raw_sha256(&self) -> &str {
        &self.raw_sha256
    }
    pub fn metadata(&self) -> &Value {
        &self.value
    }
    pub fn cluster_count(&self) -> usize {
        self.fixture.clusters.len()
    }
    pub fn glyph_count(&self) -> usize {
        self.fixture.primitives.len()
    }
    pub fn command_count(&self) -> usize {
        self.fixture.command_count
    }
    pub fn canonical_bytes(&self, max_bytes: usize) -> Result<Vec<u8>> {
        encode(&self.fixture, max_bytes)
    }
    pub fn verify_source(&self, path: &str, snapshot: &SourceSnapshot) -> Result<()> {
        let s = &self.fixture.source;
        require(
            path == s.path
                && snapshot.revision == s.revision
                && digest(snapshot.text.as_bytes()) == s.sha256,
            "stale shaped replay source identity",
        )?;
        for c in &self.fixture.clusters {
            require(
                snapshot.text.get(c.source_range[0]..c.source_range[1]) == Some(c.text.as_str()),
                "shaped replay UTF8 source mismatch",
            )?;
        }
        require(
            snapshot.text.get(s.range[0]..s.range[1]).is_some(),
            "shaped replay source extent",
        )
    }
    pub fn parse(bytes: &[u8], limits: ReplayLimits) -> Result<Self> {
        require(
            (1..=MAX_MESSAGE_BYTES).contains(&limits.max_bytes)
                && bytes.len() <= limits.max_bytes
                && (1..=65536).contains(&limits.max_clusters)
                && (1..=100000).contains(&limits.max_glyphs)
                && (1..=2000000).contains(&limits.max_commands),
            "shaped replay budget",
        )?;
        let value =
            crate::mixed_replay::parse_unique(bytes).map_err(|e| ValidationError(e.to_string()))?;
        let f: Fixture =
            serde_json::from_value(value.clone()).map_err(|e| ValidationError(e.to_string()))?;
        require(
            f.format == "flashtex-internal-shaped-v1" && !f.hinting_applied,
            "unsupported shaped replay format/policy",
        )?;
        hash(&f.identity.font_sha256)?;
        hash(&f.identity.engine_font_id)?;
        hash(&f.shape_cache_key)?;
        hash(&f.source.sha256)?;
        path(&f.source.path)?;
        require(
            f.identity.face_index == 0 && (16..=16384).contains(&f.units_per_em),
            "unsupported shaped face/units",
        )?;
        require(
            f.source.range[0] <= f.source.range[1]
                && f.source.range[1] - f.source.range[0] <= 65536
                && f.source.range[1] <= 1024 * 1024,
            "source range limit",
        )?;
        require(
            f.clusters.len() <= limits.max_clusters
                && f.primitives.len() <= limits.max_glyphs
                && f.command_count <= limits.max_commands,
            "shaped replay extent",
        )?;
        require(
            f.notes.len() <= 65536
                && ["None", "Gpos", "KernTable", "Afm"].contains(&f.kerning_source.as_str()),
            "unsupported shaping metadata",
        )?;
        require(
            f.options.fail_on_unsupported_lookups
                || !(f.options.ligatures || f.options.kerning || f.options.compose_marks),
            "non-strict requested shaping features",
        )?;
        require(
            f.ligatures_applied <= f.source.range[1] - f.source.range[0],
            "ligature metadata extent",
        )?;
        for n in &f.notes {
            text(&n.table, 1, 128, "note table")?;
            text(&n.feature, 1, 128, "note feature")?;
            text(&n.detail, 0, 65536, "note detail")?;
        }
        let size = f.size.exact()?;
        require(size.numerator() > 0, "shaped replay positive size")?;
        let origin = exact(&f.origin)?;
        ExactClip {
            left: f.clip[0].exact()?,
            top: f.clip[1].exact()?,
            right: f.clip[2].exact()?,
            bottom: f.clip[3].exact()?,
        }
        .validate()?;
        let unit =
            size.checked_multiply(OutlineCoordinate::from_fraction(1, f.units_per_em as u128)?)?;
        let units = |n: i64| unit.checked_multiply(OutlineCoordinate::from_fraction(n as i128, 1)?);
        let mut end = f.source.range[0];
        let mut pi = 0;
        let mut pen = 0i64;
        let mut commands = 0;
        for (ci, c) in f.clusters.iter().enumerate() {
            require(
                c.source_range[0] == end
                    && c.source_range[1] > end
                    && c.source_range[1] <= f.source.range[1]
                    && c.text.len() == c.source_range[1] - end,
                "cluster source coverage",
            )?;
            end = c.source_range[1];
            for g in &c.glyphs {
                let p = f
                    .primitives
                    .get(pi)
                    .ok_or_else(|| ValidationError("missing shaped primitive".into()))?;
                require(
                    g.original_gid > 0
                        && p.original_gid == g.original_gid
                        && p.cluster_index == ci
                        && p.source_range == c.source_range
                        && p.identity.item_index == f.item_index
                        && p.identity.glyph_index == pi,
                    "shaped primitive identity/cluster mismatch",
                )?;
                let px = pen
                    .checked_add(g.x_offset as i64)
                    .ok_or_else(|| ValidationError("shaped pen overflow".into()))?;
                require(
                    exact(&p.origin)?
                        == OutlinePoint {
                            x: origin.x.checked_add(units(px)?)?,
                            y: origin.y.checked_add(units(-(g.y_offset as i64))?)?,
                        }
                        && p.advance.exact()? == units(g.advance_units as i64)?,
                    "shaped placement/advance mismatch",
                )?;
                require(
                    p.kind == "quadratic" || p.kind == "cubic",
                    "unsupported shaped primitive",
                )?;
                require(
                    (p.kind == "cubic") == p.cff.is_some(),
                    "outline resource kind mismatch",
                )?;
                if let Some(cff) = &p.cff {
                    require(
                        cff.resource.font_sha256 == f.identity.font_sha256
                            && cff.resource.face_index == f.identity.face_index
                            && cff.resource.table_range[0] < cff.resource.table_range[1]
                            && cff.resource.table_range[1] <= 64 * 1024 * 1024,
                        "CFF shaped resource mismatch",
                    )?;
                    hash(&cff.resource.cff_sha256)?;
                    require(
                        ["reject", "unhinted"].contains(&cff.hint_policy.as_str())
                            && cff.stems.len() <= 96,
                        "unsupported CFF hint metadata",
                    )?;
                    require(
                        cff.hint_policy != "reject"
                            || (cff.stems.is_empty()
                                && cff.masks.is_empty()
                                && cff.flex_depths.is_empty()),
                        "rejected hints present in replay",
                    )?;
                    for v in &cff.font_matrix {
                        v.exact()?;
                    }
                    exact(&cff.outline_advance)?;
                    for s in &cff.stems {
                        s.delta.exact()?;
                        s.width.exact()?;
                    }
                    for m in &cff.masks {
                        require(
                            m.stem_count <= cff.stems.len()
                                && m.bytes.len() == m.stem_count.div_ceil(8),
                            "invalid CFF mask extent",
                        )?;
                    }
                    for v in &cff.flex_depths {
                        v.exact()?;
                    }
                }
                commands += p.commands.len();
                require(commands <= limits.max_commands, "shaped command budget")?;
                for command in &p.commands {
                    match command {
                        Command::Move(v) | Command::Line(v) => {
                            exact(v)?;
                        }
                        Command::Quad(points) => {
                            require(p.kind == "quadratic", "quadratic command in cubic path")?;
                            for v in points {
                                exact(v)?;
                            }
                        }
                        Command::Cubic(points) => {
                            require(p.kind == "cubic", "cubic command in quadratic path")?;
                            for v in points {
                                exact(v)?;
                            }
                        }
                        Command::Close => {}
                    }
                }
                pen = pen
                    .checked_add(g.advance_units as i64)
                    .ok_or_else(|| ValidationError("shaped advance overflow".into()))?;
                pi += 1;
            }
        }
        require(
            end == f.source.range[1]
                && pi == f.primitives.len()
                && commands == f.command_count
                && f.advance.exact()? == units(pen)?,
            "shaped replay aggregate mismatch",
        )?;
        Ok(Self {
            fixture: f,
            value,
            raw_sha256: digest(bytes),
        })
    }
}
fn encode<T: Serialize>(value: &T, limit: usize) -> Result<Vec<u8>> {
    require(
        (1..=MAX_MESSAGE_BYTES).contains(&limit),
        "shaped serialization budget",
    )?;
    let mut out = BoundedOutput {
        bytes: vec![],
        limit,
    };
    serde_json::to_writer(&mut out, value).map_err(|e| ValidationError(e.to_string()))?;
    Ok(out.bytes)
}
impl PlacedShapedRun {
    pub fn replay_bytes(&self, max_bytes: usize) -> Result<Vec<u8>> {
        require(
            max_bytes <= MAX_MESSAGE_BYTES && self.command_count() <= max_bytes / 7,
            "shaped replay serialization lower bound",
        )?;
        let run = self.run();
        let options = run.options();
        let placement = self.placement();
        let rational = |v: flashtex_font_resources::Coordinate| -> Result<Rational> {
            Ok(Rational::new(OutlineCoordinate::from_fraction(
                v.numerator(),
                1u128
                    .checked_shl(v.shift())
                    .ok_or_else(|| ValidationError("outline precision".into()))?,
            )?))
        };
        let mut primitives = Vec::new();
        for g in self.glyphs() {
            let (kind, commands, cff) = match &g.geometry {
                ShapedGeometry::Quadratic(commands) => (
                    "quadratic",
                    commands
                        .iter()
                        .map(|c| match c {
                            PlacedPathCommand::MoveTo(v) => Command::Move(point(*v)),
                            PlacedPathCommand::LineTo(v) => Command::Line(point(*v)),
                            PlacedPathCommand::QuadTo { control, end } => {
                                Command::Quad([point(*control), point(*end)])
                            }
                            PlacedPathCommand::Close => Command::Close,
                        })
                        .collect(),
                    None,
                ),
                ShapedGeometry::Cubic(c) => {
                    use crate::cubic::CubicPathCommand as C;
                    let commands = c
                        .commands
                        .iter()
                        .map(|v| match v {
                            C::MoveTo(p) => Command::Move(point(*p)),
                            C::LineTo(p) => Command::Line(point(*p)),
                            C::CurveTo {
                                control1,
                                control2,
                                end,
                            } => Command::Cubic([point(*control1), point(*control2), point(*end)]),
                            C::Close => Command::Close,
                        })
                        .collect();
                    let resource = c
                        .full_font_identity
                        .as_ref()
                        .ok_or_else(|| ValidationError("unbound shaped CFF resource".into()))?;
                    let mut matrix = Vec::new();
                    for v in &c.font_matrix {
                        matrix.push(Rational::new(OutlineCoordinate::from_fraction(
                            v.numerator(),
                            v.denominator() as u128,
                        )?));
                    }
                    let metadata = CffMetadata {
                        resource: CffResource {
                            font_sha256: resource.font_sha256.clone(),
                            cff_sha256: resource.cff_sha256.clone(),
                            face_index: resource.face_index,
                            table_range: [resource.table_range.start, resource.table_range.end],
                        },
                        font_matrix: matrix
                            .try_into()
                            .map_err(|_| ValidationError("matrix extent".into()))?,
                        outline_advance: point(c.advance),
                        hint_policy: match c.hints.policy {
                            flashtex_font_resources::cff::HintPolicy::Reject => "reject",
                            flashtex_font_resources::cff::HintPolicy::Unhinted => "unhinted",
                        }
                        .into(),
                        stems: c
                            .hints
                            .stems
                            .iter()
                            .map(|s| {
                                Ok(Stem {
                                    vertical: s.vertical,
                                    delta: rational(s.delta)?,
                                    width: rational(s.width)?,
                                })
                            })
                            .collect::<Result<_>>()?,
                        masks: c
                            .hints
                            .masks
                            .iter()
                            .map(|m| Mask {
                                counter: m.counter,
                                stem_count: m.stem_count,
                                bytes: m.bytes.clone(),
                            })
                            .collect(),
                        flex_depths: c
                            .hints
                            .flex_depths
                            .iter()
                            .map(|v| rational(*v))
                            .collect::<Result<_>>()?,
                    };
                    ("cubic", commands, Some(metadata))
                }
            };
            primitives.push(Path {
                identity: PrimitiveIdentity {
                    item_index: g.primitive_id.item_index,
                    glyph_index: g
                        .primitive_id
                        .glyph_index
                        .ok_or_else(|| ValidationError("shaped glyph identity".into()))?,
                },
                cluster_index: g.cluster_index,
                original_gid: g.original_gid,
                source_range: [g.source_range.start, g.source_range.end],
                origin: point(g.origin),
                advance: Rational::new(g.advance),
                kind: kind.into(),
                commands,
                cff,
            });
        }
        let source = run.source();
        let identity = run.identity();
        let clip = placement.clip;
        let f = Fixture {
            format: "flashtex-internal-shaped-v1".into(),
            identity: Identity {
                font_sha256: identity.font_sha256.clone(),
                engine_font_id: identity.engine_font_id.content_hex(),
                face_index: identity.face_index,
            },
            source: Source {
                path: source.path.clone(),
                revision: source.revision,
                sha256: source.source_sha256.clone(),
                range: [source.range.start, source.range.end],
            },
            shape_cache_key: run.cache_key().into(),
            options: Options {
                ligatures: options.ligatures,
                kerning: options.kerning,
                compose_marks: options.compose_marks,
                cmap_ligature_fallback: options.cmap_ligature_fallback,
                fail_on_unsupported_lookups: options.fail_on_unsupported_lookups,
            },
            units_per_em: run.shaped().units_per_em,
            ligatures_applied: run.shaped().ligatures_applied,
            kerning_source: format!("{:?}", run.shaped().kerning_source),
            notes: run
                .shaped()
                .unsupported
                .iter()
                .map(|n| Note {
                    table: n.table.into(),
                    feature: n.feature.into(),
                    detail: n.detail.clone(),
                })
                .collect(),
            clusters: run
                .shaped()
                .clusters
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let range = run
                        .absolute_cluster_range(i)
                        .expect("immutable validated source cluster");
                    Cluster {
                        source_range: [range.start, range.end],
                        text: c.text.clone(),
                        glyphs: c
                            .glyphs
                            .iter()
                            .map(|g| Glyph {
                                original_gid: g.gid.0,
                                advance_units: g.advance,
                                x_offset: g.x_offset,
                                y_offset: g.y_offset,
                            })
                            .collect(),
                    }
                })
                .collect(),
            item_index: placement.item_index,
            size: Rational::new(placement.size),
            origin: point(placement.origin),
            clip: [
                Rational::new(clip.left),
                Rational::new(clip.top),
                Rational::new(clip.right),
                Rational::new(clip.bottom),
            ],
            advance: Rational::new(self.advance()),
            primitives,
            command_count: self.command_count(),
            hinting_applied: false,
        };
        encode(&f, max_bytes)
    }
}

impl ShapedReplay {
    /// Exact geometry for a consumer; parsing does not verify original font bytes.
    pub fn geometry(&self, index: usize) -> Result<crate::mixed_replay::ReplayGeometry> {
        use crate::mixed_replay::{ReplayCommand as R, ReplayGeometry as G};
        let p = self
            .fixture
            .primitives
            .get(index)
            .ok_or_else(|| ValidationError("shaped replay primitive index".into()))?;
        let commands = p
            .commands
            .iter()
            .map(|c| {
                Ok(match c {
                    Command::Move(p) => R::Move(exact(p)?),
                    Command::Line(p) => R::Line(exact(p)?),
                    Command::Quad(p) => R::Quadratic {
                        control: exact(&p[0])?,
                        end: exact(&p[1])?,
                    },
                    Command::Cubic(p) => R::Cubic {
                        control1: exact(&p[0])?,
                        control2: exact(&p[1])?,
                        end: exact(&p[2])?,
                    },
                    Command::Close => R::Close,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(if p.kind == "quadratic" {
            G::Quadratic(commands)
        } else {
            G::Cubic(commands)
        })
    }
}
