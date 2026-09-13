//! Display list v2: the pipeline's authoritative output, shaped after
//! `protocol/rendering-v2.schema.json` and `crates/rendering-core`'s model.
//!
//! Coordinates are `bp_2pow20` ticks (integer, 1 048 576 per PDF point,
//! y downward from the page's top-left corner). Every glyph carries its
//! ORIGINAL glyph id and a cluster index; every cluster carries its byte
//! range inside the run's `text` (the ActualText), exact hit rectangles
//! and source ranges with the document path. Rules are explicit rectangles.
//! Deviations from the schema are listed in `docs/proposals/rendering-abi.md`
//! and in the README, never hidden: this pipeline emits
//! `format: "opentype-cff"` for Latin Modern and `core14-afm` (no bytes) for
//! Times, and `gid: 0` never appears (missing glyphs are diagnostics).

use flashtex_compiler::json::{self, Value};

pub const PROTOCOL_VERSION: i64 = 2;
pub const TICKS_PER_BP: f64 = 1_048_576.0;
/// 1 TeX point in PDF points (big points): 72/72.27.
pub const BP_PER_TEX_PT: f64 = 72.0 / 72.27;

/// Integer ticks, 2^20 per PDF point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tick(pub i64);

impl Tick {
    /// Ticks from a length in TeX points (72.27/in). The single rounding
    /// point of the pipeline: layout is f64 TeX points, output is exact
    /// integers, so equal inputs give equal outputs.
    pub fn from_tex_pt(pt: f64) -> Tick {
        Tick((pt * BP_PER_TEX_PT * TICKS_PER_BP).round() as i64)
    }
    /// Ticks from PDF points (used for page sizes, which LaTeX declares in
    /// TeX points but consumers expect as 612x792 bp for US Letter).
    pub fn from_bp(bp: f64) -> Tick {
        Tick((bp * TICKS_PER_BP).round() as i64)
    }
    pub fn to_bp(self) -> f64 {
        self.0 as f64 / TICKS_PER_BP
    }
}

/// A byte range in one source document. `path` is shared: a page carries
/// one range per cluster, so the string is reference-counted rather than
/// copied a hundred thousand times per compile.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceRange {
    pub path: std::rc::Rc<str>,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paint {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Paint {
    pub const BLACK: Paint = Paint {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: Tick,
    pub top: Tick,
    pub width: Tick,
    pub height: Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caret {
    pub text_byte: usize,
    pub x: Tick,
    pub top: Tick,
    pub height: Tick,
}

/// Where a cluster's bytes came from: exact source ranges, or a stated
/// reason when the pipeline synthesised it.
#[derive(Debug, Clone, PartialEq)]
pub enum Provenance {
    /// One source range (the common case; no allocation per cluster).
    Source(SourceRange),
    /// Several ranges (macro expansion); reserved, unused today.
    Sources(Vec<SourceRange>),
    Synthetic(String),
}

impl Provenance {
    /// The source ranges, in order (empty for synthetic content).
    pub fn sources(&self) -> &[SourceRange] {
        match self {
            Provenance::Source(s) => std::slice::from_ref(s),
            Provenance::Sources(v) => v,
            Provenance::Synthetic(_) => &[],
        }
    }
}

/// One or two carets per cluster (its start, and the run end on the last
/// cluster), stored inline: a page carries a caret pair per cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Carets {
    pub first: Caret,
    pub last: Option<Caret>,
}

impl Carets {
    pub fn iter(&self) -> impl Iterator<Item = &Caret> {
        std::iter::once(&self.first).chain(self.last.iter())
    }
    pub fn len(&self) -> usize {
        1 + usize::from(self.last.is_some())
    }
    pub fn is_empty(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    pub text_start_byte: usize,
    pub text_end_byte: usize,
    /// The cluster's hit rectangle (the wire format is a list; this
    /// pipeline emits exactly one per cluster).
    pub hit_rect: Rect,
    pub carets: Carets,
    pub provenance: Provenance,
}

impl Cluster {
    pub fn hit_rects(&self) -> &[Rect] {
        std::slice::from_ref(&self.hit_rect)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    /// Original glyph id in the run's font. Never 0.
    pub gid: u16,
    pub origin_x: Tick,
    pub baseline_y: Tick,
    pub advance_x: Tick,
    pub advance_y: Tick,
    pub cluster: u32,
}

/// What a run is, for the v1 fallback (not on the v2 wire).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunRole {
    /// A word segment: one v1 text item.
    Text,
    /// Math glyphs with individual positions: one v1 text item per glyph.
    Math,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    /// Content-addressed font id (SHA-256 hex of the program).
    pub font_id: std::rc::Rc<str>,
    pub font_size: Tick,
    /// ActualText of the whole run; clusters partition it.
    pub text: String,
    pub glyphs: Vec<Glyph>,
    pub clusters: Vec<Cluster>,
    pub paint: Paint,
    pub role: RunRole,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub x: Tick,
    pub top: Tick,
    pub width: Tick,
    pub height: Tick,
    pub paint: Paint,
    pub provenance: Provenance,
}

/// One vector path command, in ticks (top-left origin, y down). Proposal
/// `path-v0` (`docs/proposals/display-list-paths.md`), not in the frozen
/// rendering-v2 schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathCmd {
    Move(Tick, Tick),
    Line(Tick, Tick),
    /// Cubic Bézier: first control, second control, end.
    Cubic(Tick, Tick, Tick, Tick, Tick, Tick),
    Close,
}

impl PathCmd {
    /// The command with every y coordinate mapped by `f`.
    pub fn map_y(self, f: &dyn Fn(Tick) -> Tick) -> PathCmd {
        match self {
            PathCmd::Move(x, y) => PathCmd::Move(x, f(y)),
            PathCmd::Line(x, y) => PathCmd::Line(x, f(y)),
            PathCmd::Cubic(a, b, c, d, e, g) => PathCmd::Cubic(a, f(b), c, f(d), e, f(g)),
            PathCmd::Close => PathCmd::Close,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub width: Tick,
    pub cap: LineCap,
    pub join: LineJoin,
    pub miter_limit: f64,
    /// Alternating on/off lengths; empty for a solid line.
    pub dash: Vec<Tick>,
    pub dash_phase: Tick,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathPaintOp {
    Fill { even_odd: bool },
    Stroke(Stroke),
}

/// A clip in page space; an item is visible only inside every clip.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipPath {
    pub commands: Vec<PathCmd>,
    pub even_odd: bool,
}

/// A filled or stroked vector path (TikZ pictures).
#[derive(Debug, Clone, PartialEq)]
pub struct PathItem {
    pub op: PathPaintOp,
    pub commands: Vec<PathCmd>,
    pub clips: Vec<ClipPath>,
    pub paint: Paint,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    GlyphRun(GlyphRun),
    Rule(Rule),
    Path(PathItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub number: u32,
    pub width: Tick,
    pub height: Tick,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontResource {
    pub font_id: std::rc::Rc<str>,
    pub sha256: String,
    pub byte_length: u64,
    /// `opentype-cff`, `static-truetype` or `core14-afm`.
    pub format: String,
    pub face_index: u32,
    pub units_per_em: u32,
    pub glyph_count: u32,
    pub postscript_name: String,
    /// Not on the wire; where the bytes came from, for diagnostics/PDF.
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentResource {
    pub path: String,
    pub revision: u64,
    pub sha256: String,
    pub byte_length: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub sources: Vec<SourceRange>,
    /// The compiler's recovery note, when it produced this diagnostic.
    pub recovery: Option<String>,
}

impl Diagnostic {
    pub fn error(code: &str, message: impl Into<String>, sources: Vec<SourceRange>) -> Diagnostic {
        Diagnostic {
            code: code.into(),
            message: message.into(),
            severity: Severity::Error,
            sources,
            recovery: None,
        }
    }
    pub fn warning(code: &str, message: impl Into<String>, sources: Vec<SourceRange>) -> Diagnostic {
        Diagnostic {
            code: code.into(),
            message: message.into(),
            severity: Severity::Warning,
            sources,
            recovery: None,
        }
    }

    /// Converts a compiler diagnostic; `paths` is indexed by `DocumentId`.
    pub fn from_compiler(d: &flashtex_compiler::diagnostics::Diagnostic, paths: &[&str]) -> Diagnostic {
        use flashtex_compiler::diagnostics::Severity as S;
        Diagnostic {
            code: "compiler".into(),
            message: d.message.clone(),
            severity: match d.severity {
                S::Error => Severity::Error,
                _ => Severity::Warning,
            },
            sources: d
                .span
                .map(|s| {
                    vec![SourceRange {
                        path: std::rc::Rc::from(paths.get(s.document.0).copied().unwrap_or("")),
                        start_byte: s.start,
                        end_byte: s.end,
                    }]
                })
                .unwrap_or_default(),
            recovery: d.recovery.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayList {
    pub project_id: String,
    pub revision: u64,
    pub documents: Vec<DocumentResource>,
    pub fonts: Vec<FontResource>,
    pub pages: Vec<Page>,
    pub diagnostics: Vec<Diagnostic>,
}

impl DisplayList {
    /// An upper-bound estimate of the serialised envelope size, so a
    /// producer can decline `display-list-v2` for a request without first
    /// serialising a line it would then throw away (the exact check still
    /// runs on the serialised line when the estimate is under the limit).
    pub fn estimated_json_bytes(&self) -> usize {
        let mut n = 512 + self.fonts.len() * 400 + self.documents.len() * 200;
        for d in &self.diagnostics {
            n += 160 + d.message.len() + d.sources.len() * 80;
        }
        for p in &self.pages {
            n += 64;
            for it in &p.items {
                n += match it {
                    Item::GlyphRun(r) => 220 + 2 * r.text.len() + 120 * r.glyphs.len() + 280 * r.clusters.len(),
                    Item::Rule(_) => 240,
                    Item::Path(p) => 240 + 64 * (p.commands.len() + p.clips.iter().map(|c| c.commands.len()).sum::<usize>()),
                };
            }
        }
        n
    }

    pub fn required_features(&self) -> Vec<&'static str> {
        let mut f = vec!["glyph_run", "rgba-srgb", "cluster-actualtext"];
        if self.pages.iter().any(|p| p.items.iter().any(|i| matches!(i, Item::Rule(_)))) {
            f.insert(1, "rule");
        }
        let paths = || self.pages.iter().flat_map(|p| p.items.iter()).filter_map(|i| if let Item::Path(p) = i { Some(p) } else { None });
        if paths().any(|p| matches!(p.op, PathPaintOp::Fill { .. })) {
            f.push("path_fill");
        }
        if paths().any(|p| matches!(p.op, PathPaintOp::Stroke(_))) {
            f.push("path_stroke");
        }
        if paths().any(|p| !p.clips.is_empty()) {
            f.push("clip");
        }
        if self.fonts.iter().any(|r| r.format == "static-truetype") {
            f.push("static-truetype");
        }
        f
    }

    /// The `display_list` envelope of rendering-v2 as a JSON value.
    pub fn to_json(&self, id: &str) -> Value {
        let mut payload = Value::obj();
        payload.set("render_format", json::str_("display-list-v2"));
        payload.set("coordinate_unit", json::str_("bp_2pow20"));
        payload.set("color_space", json::str_("srgb"));
        payload.set("text_extraction", json::str_("cluster-actualtext"));
        payload.set("project_id", json::str_(self.project_id.clone()));
        payload.set("revision", json::num(self.revision as f64));
        payload.set(
            "required_features",
            Value::Arr(self.required_features().into_iter().map(json::str_).collect()),
        );
        payload.set(
            "documents",
            Value::Arr(
                self.documents
                    .iter()
                    .map(|d| {
                        let mut o = Value::obj();
                        o.set("path", json::str_(d.path.clone()));
                        o.set("revision", json::num(d.revision as f64));
                        o.set("sha256", json::str_(d.sha256.clone()));
                        o.set("byte_length", json::num(d.byte_length as f64));
                        o
                    })
                    .collect(),
            ),
        );
        payload.set(
            "fonts",
            Value::Arr(
                self.fonts
                    .iter()
                    .map(|f| {
                        let mut o = Value::obj();
                        o.set("font_id", json::str_(f.font_id.to_string()));
                        o.set("sha256", json::str_(f.sha256.clone()));
                        o.set("byte_length", json::num(f.byte_length as f64));
                        o.set("format", json::str_(f.format.clone()));
                        o.set("face_index", json::num(f64::from(f.face_index)));
                        o.set("units_per_em", json::num(f64::from(f.units_per_em)));
                        o.set("glyph_count", json::num(f64::from(f.glyph_count)));
                        o.set("postscript_name", json::str_(f.postscript_name.clone()));
                        o
                    })
                    .collect(),
            ),
        );
        payload.set("pages", Value::Arr(self.pages.iter().map(page_json).collect()));
        payload.set(
            "diagnostics",
            Value::Arr(self.diagnostics.iter().map(diagnostic_json).collect()),
        );
        let mut v = Value::obj();
        v.set("protocol_version", json::num(PROTOCOL_VERSION as f64));
        v.set("id", json::str_(id));
        v.set("type", json::str_("display_list"));
        v.set("payload", payload);
        v
    }
}

fn tick(t: Tick) -> Value {
    json::num(t.0 as f64)
}

fn source_json(s: &SourceRange) -> Value {
    let mut o = Value::obj();
    o.set("path", json::str_(s.path.to_string()));
    o.set("start_byte", json::num(s.start_byte as f64));
    o.set("end_byte", json::num(s.end_byte as f64));
    o
}

fn provenance_into(o: &mut Value, p: &Provenance) {
    match p {
        Provenance::Source(_) | Provenance::Sources(_) => o.set("sources", Value::Arr(p.sources().iter().map(source_json).collect())),
        Provenance::Synthetic(reason) => o.set("synthetic_reason", json::str_(reason.clone())),
    }
}

fn paint_json(p: &Paint) -> Value {
    let mut o = Value::obj();
    o.set("r", json::num(p.r));
    o.set("g", json::num(p.g));
    o.set("b", json::num(p.b));
    o.set("a", json::num(p.a));
    o
}

/// `[["m",x,y],["l",x,y],["c",x1,y1,x2,y2,x,y],["z"]]` in ticks.
fn path_json(cmds: &[PathCmd]) -> Value {
    Value::Arr(
        cmds.iter()
            .map(|c| {
                Value::Arr(match *c {
                    PathCmd::Move(x, y) => vec![json::str_("m"), tick(x), tick(y)],
                    PathCmd::Line(x, y) => vec![json::str_("l"), tick(x), tick(y)],
                    PathCmd::Cubic(a, b, cc, d, e, f) => vec![json::str_("c"), tick(a), tick(b), tick(cc), tick(d), tick(e), tick(f)],
                    PathCmd::Close => vec![json::str_("z")],
                })
            })
            .collect(),
    )
}

fn rect_json(r: &Rect) -> Value {
    let mut o = Value::obj();
    o.set("x", tick(r.x));
    o.set("top", tick(r.top));
    o.set("width", tick(r.width));
    o.set("height", tick(r.height));
    o
}

pub fn diagnostic_json(d: &Diagnostic) -> Value {
    let mut o = Value::obj();
    o.set("code", json::str_(d.code.clone()));
    o.set("message", json::str_(d.message.clone()));
    o.set(
        "severity",
        json::str_(match d.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        }),
    );
    o.set("sources", Value::Arr(d.sources.iter().map(source_json).collect()));
    o
}

fn page_json(p: &Page) -> Value {
    let mut o = Value::obj();
    o.set("number", json::num(f64::from(p.number)));
    o.set("width", tick(p.width));
    o.set("height", tick(p.height));
    o.set(
        "items",
        Value::Arr(
            p.items
                .iter()
                .map(|it| match it {
                    Item::GlyphRun(r) => {
                        let mut o = Value::obj();
                        o.set("kind", json::str_("glyph_run"));
                        o.set("font_id", json::str_(r.font_id.to_string()));
                        o.set("font_size", tick(r.font_size));
                        o.set("text", json::str_(r.text.clone()));
                        o.set(
                            "glyphs",
                            Value::Arr(
                                r.glyphs
                                    .iter()
                                    .map(|g| {
                                        let mut o = Value::obj();
                                        o.set("gid", json::num(f64::from(g.gid)));
                                        o.set("origin_x", tick(g.origin_x));
                                        o.set("baseline_y", tick(g.baseline_y));
                                        o.set("advance_x", tick(g.advance_x));
                                        o.set("advance_y", tick(g.advance_y));
                                        o.set("cluster", json::num(f64::from(g.cluster)));
                                        o
                                    })
                                    .collect(),
                            ),
                        );
                        o.set(
                            "clusters",
                            Value::Arr(
                                r.clusters
                                    .iter()
                                    .map(|c| {
                                        let mut o = Value::obj();
                                        o.set("text_start_byte", json::num(c.text_start_byte as f64));
                                        o.set("text_end_byte", json::num(c.text_end_byte as f64));
                                        o.set("hit_rects", Value::Arr(c.hit_rects().iter().map(rect_json).collect()));
                                        o.set(
                                            "carets",
                                            Value::Arr(
                                                c.carets
                                                    .iter()
                                                    .map(|k| {
                                                        let mut o = Value::obj();
                                                        o.set("text_byte", json::num(k.text_byte as f64));
                                                        o.set("x", tick(k.x));
                                                        o.set("top", tick(k.top));
                                                        o.set("height", tick(k.height));
                                                        o
                                                    })
                                                    .collect(),
                                            ),
                                        );
                                        provenance_into(&mut o, &c.provenance);
                                        o
                                    })
                                    .collect(),
                            ),
                        );
                        o.set("paint", paint_json(&r.paint));
                        o
                    }
                    Item::Path(p) => {
                        let mut o = Value::obj();
                        match &p.op {
                            PathPaintOp::Fill { even_odd } => {
                                o.set("kind", json::str_("path_fill"));
                                o.set("fill_rule", json::str_(if *even_odd { "evenodd" } else { "nonzero" }));
                            }
                            PathPaintOp::Stroke(s) => {
                                o.set("kind", json::str_("path_stroke"));
                                let mut so = Value::obj();
                                so.set("width", tick(s.width));
                                so.set(
                                    "cap",
                                    json::str_(match s.cap {
                                        LineCap::Butt => "butt",
                                        LineCap::Round => "round",
                                        LineCap::Square => "square",
                                    }),
                                );
                                so.set(
                                    "join",
                                    json::str_(match s.join {
                                        LineJoin::Miter => "miter",
                                        LineJoin::Round => "round",
                                        LineJoin::Bevel => "bevel",
                                    }),
                                );
                                so.set("miter_limit", json::num(s.miter_limit));
                                if !s.dash.is_empty() {
                                    let mut d = Value::obj();
                                    d.set("array", Value::Arr(s.dash.iter().map(|t| tick(*t)).collect()));
                                    d.set("phase", tick(s.dash_phase));
                                    so.set("dash", d);
                                }
                                o.set("stroke", so);
                            }
                        }
                        o.set("path", path_json(&p.commands));
                        if !p.clips.is_empty() {
                            o.set(
                                "clips",
                                Value::Arr(
                                    p.clips
                                        .iter()
                                        .map(|c| {
                                            let mut co = Value::obj();
                                            co.set("kind", json::str_("path"));
                                            co.set("path", path_json(&c.commands));
                                            co.set("fill_rule", json::str_(if c.even_odd { "evenodd" } else { "nonzero" }));
                                            co
                                        })
                                        .collect(),
                                ),
                            );
                        }
                        o.set("paint", paint_json(&p.paint));
                        provenance_into(&mut o, &p.provenance);
                        o
                    }
                    Item::Rule(r) => {
                        let mut o = Value::obj();
                        o.set("kind", json::str_("rule"));
                        o.set("x", tick(r.x));
                        o.set("top", tick(r.top));
                        o.set("width", tick(r.width));
                        o.set("height", tick(r.height));
                        o.set("paint", paint_json(&r.paint));
                        provenance_into(&mut o, &r.provenance);
                        o
                    }
                })
                .collect(),
        ),
    );
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_round_once_from_tex_points() {
        // 72.27 TeX pt = 72 bp = 72 * 2^20 ticks exactly.
        assert_eq!(Tick::from_tex_pt(72.27), Tick(72 * 1_048_576));
        assert_eq!(Tick::from_bp(612.0).0, 612 * 1_048_576);
        assert_eq!(Tick::from_tex_pt(0.0), Tick(0));
    }
}
