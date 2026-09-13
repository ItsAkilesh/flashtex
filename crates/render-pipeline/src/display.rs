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

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    GlyphRun(GlyphRun),
    Rule(Rule),
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

    /// `json::write(&self.to_json(id))` without building the `Value` tree
    /// (FT-065: at HW1 size the tree of per-glyph maps with owned keys cost
    /// more than layout). Keys are written in the `BTreeMap` order the tree
    /// serialises in and numbers/strings go through the same json writers,
    /// so the bytes are identical (`write_json_matches_the_value_tree`).
    pub fn write_json(&self, id: &str) -> String {
        let mut o = String::with_capacity(self.estimated_json_bytes());
        o.push_str("{\"id\":");
        json::write_string_into(id, &mut o);
        o.push_str(",\"payload\":{\"color_space\":\"srgb\",\"coordinate_unit\":\"bp_2pow20\",\"diagnostics\":[");
        for (i, d) in self.diagnostics.iter().enumerate() {
            sep(&mut o, i);
            o.push_str("{\"code\":");
            json::write_string_into(&d.code, &mut o);
            o.push_str(",\"message\":");
            json::write_string_into(&d.message, &mut o);
            o.push_str(",\"severity\":");
            o.push_str(match d.severity {
                Severity::Warning => "\"warning\"",
                Severity::Error => "\"error\"",
            });
            o.push_str(",\"sources\":");
            write_sources(&mut o, &d.sources);
            o.push('}');
        }
        o.push_str("],\"documents\":[");
        for (i, d) in self.documents.iter().enumerate() {
            sep(&mut o, i);
            o.push_str("{\"byte_length\":");
            num(&mut o, d.byte_length as f64);
            o.push_str(",\"path\":");
            json::write_string_into(&d.path, &mut o);
            o.push_str(",\"revision\":");
            num(&mut o, d.revision as f64);
            o.push_str(",\"sha256\":");
            json::write_string_into(&d.sha256, &mut o);
            o.push('}');
        }
        o.push_str("],\"fonts\":[");
        for (i, f) in self.fonts.iter().enumerate() {
            sep(&mut o, i);
            o.push_str("{\"byte_length\":");
            num(&mut o, f.byte_length as f64);
            o.push_str(",\"face_index\":");
            num(&mut o, f64::from(f.face_index));
            o.push_str(",\"font_id\":");
            json::write_string_into(&f.font_id, &mut o);
            o.push_str(",\"format\":");
            json::write_string_into(&f.format, &mut o);
            o.push_str(",\"glyph_count\":");
            num(&mut o, f64::from(f.glyph_count));
            o.push_str(",\"postscript_name\":");
            json::write_string_into(&f.postscript_name, &mut o);
            o.push_str(",\"sha256\":");
            json::write_string_into(&f.sha256, &mut o);
            o.push_str(",\"units_per_em\":");
            num(&mut o, f64::from(f.units_per_em));
            o.push('}');
        }
        o.push_str("],\"pages\":[");
        for (i, p) in self.pages.iter().enumerate() {
            sep(&mut o, i);
            write_page(&mut o, p);
        }
        o.push_str("],\"project_id\":");
        json::write_string_into(&self.project_id, &mut o);
        o.push_str(",\"render_format\":\"display-list-v2\",\"required_features\":[");
        for (i, f) in self.required_features().into_iter().enumerate() {
            sep(&mut o, i);
            json::write_string_into(f, &mut o);
        }
        o.push_str("],\"revision\":");
        num(&mut o, self.revision as f64);
        o.push_str(",\"text_extraction\":\"cluster-actualtext\"},\"protocol_version\":");
        num(&mut o, PROTOCOL_VERSION as f64);
        o.push_str(",\"type\":\"display_list\"}");
        o
    }
}

fn sep(o: &mut String, i: usize) {
    if i > 0 {
        o.push(',');
    }
}

fn num(o: &mut String, n: f64) {
    json::write_number_into(n, o);
}

fn write_tick(o: &mut String, t: Tick) {
    num(o, t.0 as f64);
}

fn write_sources(o: &mut String, sources: &[SourceRange]) {
    o.push('[');
    for (i, s) in sources.iter().enumerate() {
        sep(o, i);
        o.push_str("{\"end_byte\":");
        num(o, s.end_byte as f64);
        o.push_str(",\"path\":");
        json::write_string_into(&s.path, o);
        o.push_str(",\"start_byte\":");
        num(o, s.start_byte as f64);
        o.push('}');
    }
    o.push(']');
}

/// `sources` or `synthetic_reason`, preceded by a comma (both sort after
/// every key written before them and before every key written after).
fn write_provenance(o: &mut String, p: &Provenance) {
    match p {
        Provenance::Source(_) | Provenance::Sources(_) => {
            o.push_str(",\"sources\":");
            write_sources(o, p.sources());
        }
        Provenance::Synthetic(reason) => {
            o.push_str(",\"synthetic_reason\":");
            json::write_string_into(reason, o);
        }
    }
}

fn write_paint(o: &mut String, p: &Paint) {
    o.push_str("{\"a\":");
    num(o, p.a);
    o.push_str(",\"b\":");
    num(o, p.b);
    o.push_str(",\"g\":");
    num(o, p.g);
    o.push_str(",\"r\":");
    num(o, p.r);
    o.push('}');
}

fn write_page(o: &mut String, p: &Page) {
    o.push_str("{\"height\":");
    write_tick(o, p.height);
    o.push_str(",\"items\":[");
    for (i, it) in p.items.iter().enumerate() {
        sep(o, i);
        match it {
            Item::GlyphRun(r) => {
                o.push_str("{\"clusters\":[");
                for (j, c) in r.clusters.iter().enumerate() {
                    sep(o, j);
                    o.push_str("{\"carets\":[");
                    for (k, caret) in c.carets.iter().enumerate() {
                        sep(o, k);
                        o.push_str("{\"height\":");
                        write_tick(o, caret.height);
                        o.push_str(",\"text_byte\":");
                        num(o, caret.text_byte as f64);
                        o.push_str(",\"top\":");
                        write_tick(o, caret.top);
                        o.push_str(",\"x\":");
                        write_tick(o, caret.x);
                        o.push('}');
                    }
                    o.push_str("],\"hit_rects\":[");
                    for (k, rect) in c.hit_rects().iter().enumerate() {
                        sep(o, k);
                        o.push_str("{\"height\":");
                        write_tick(o, rect.height);
                        o.push_str(",\"top\":");
                        write_tick(o, rect.top);
                        o.push_str(",\"width\":");
                        write_tick(o, rect.width);
                        o.push_str(",\"x\":");
                        write_tick(o, rect.x);
                        o.push('}');
                    }
                    o.push(']');
                    write_provenance(o, &c.provenance);
                    o.push_str(",\"text_end_byte\":");
                    num(o, c.text_end_byte as f64);
                    o.push_str(",\"text_start_byte\":");
                    num(o, c.text_start_byte as f64);
                    o.push('}');
                }
                o.push_str("],\"font_id\":");
                json::write_string_into(&r.font_id, o);
                o.push_str(",\"font_size\":");
                write_tick(o, r.font_size);
                o.push_str(",\"glyphs\":[");
                for (j, g) in r.glyphs.iter().enumerate() {
                    sep(o, j);
                    o.push_str("{\"advance_x\":");
                    write_tick(o, g.advance_x);
                    o.push_str(",\"advance_y\":");
                    write_tick(o, g.advance_y);
                    o.push_str(",\"baseline_y\":");
                    write_tick(o, g.baseline_y);
                    o.push_str(",\"cluster\":");
                    num(o, f64::from(g.cluster));
                    o.push_str(",\"gid\":");
                    num(o, f64::from(g.gid));
                    o.push_str(",\"origin_x\":");
                    write_tick(o, g.origin_x);
                    o.push('}');
                }
                o.push_str("],\"kind\":\"glyph_run\",\"paint\":");
                write_paint(o, &r.paint);
                o.push_str(",\"text\":");
                json::write_string_into(&r.text, o);
                o.push('}');
            }
            Item::Rule(r) => {
                o.push_str("{\"height\":");
                write_tick(o, r.height);
                o.push_str(",\"kind\":\"rule\",\"paint\":");
                write_paint(o, &r.paint);
                write_provenance(o, &r.provenance);
                o.push_str(",\"top\":");
                write_tick(o, r.top);
                o.push_str(",\"width\":");
                write_tick(o, r.width);
                o.push_str(",\"x\":");
                write_tick(o, r.x);
                o.push('}');
            }
        }
    }
    o.push_str("],\"number\":");
    num(o, f64::from(p.number));
    o.push_str(",\"width\":");
    write_tick(o, p.width);
    o.push('}');
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

    #[test]
    fn write_json_matches_the_value_tree() {
        let src = |a, b| SourceRange {
            path: std::rc::Rc::from("dir/ma\"in.tex"),
            start_byte: a,
            end_byte: b,
        };
        let caret = |x| Caret {
            text_byte: 3,
            x: Tick(x),
            top: Tick(-7),
            height: Tick(1 << 40),
        };
        let cluster = |provenance| Cluster {
            text_start_byte: 0,
            text_end_byte: 4,
            hit_rect: Rect {
                x: Tick(1),
                top: Tick(-2),
                width: Tick(3),
                height: Tick(4),
            },
            carets: Carets {
                first: caret(5),
                last: Some(caret(9)),
            },
            provenance,
        };
        let run = Item::GlyphRun(GlyphRun {
            font_id: std::rc::Rc::from("abc"),
            font_size: Tick(12 << 20),
            text: "ﬁ \"q\"\\\n\t\u{1}é".into(),
            glyphs: vec![
                Glyph {
                    gid: 65535,
                    origin_x: Tick(-1),
                    baseline_y: Tick(2),
                    advance_x: Tick(3),
                    advance_y: Tick(0),
                    cluster: 1,
                };
                2
            ],
            clusters: vec![
                cluster(Provenance::Source(src(1, 2))),
                cluster(Provenance::Sources(vec![src(3, 4), src(5, 6)])),
                cluster(Provenance::Synthetic("heading number".into())),
            ],
            paint: Paint {
                r: 0.25,
                g: 0.1,
                b: 1.0 / 3.0,
                a: 1.0,
            },
            role: RunRole::Text,
        });
        let rule = |provenance| {
            Item::Rule(Rule {
                x: Tick(10),
                top: Tick(20),
                width: Tick(30),
                height: Tick(40),
                paint: Paint::BLACK,
                provenance,
            })
        };
        let list = DisplayList {
            project_id: "p\\1".into(),
            revision: 42,
            documents: vec![DocumentResource {
                path: "main.tex".into(),
                revision: 42,
                sha256: "00ff".into(),
                byte_length: 5126,
            }],
            fonts: vec![FontResource {
                font_id: std::rc::Rc::from("abc"),
                sha256: "abc".into(),
                byte_length: 1 << 33,
                format: "opentype-cff".into(),
                face_index: 0,
                units_per_em: 1000,
                glyph_count: 821,
                postscript_name: "LMRoman12-Regular".into(),
                path: Some("/x".into()),
            }],
            pages: vec![
                Page {
                    number: 1,
                    width: Tick(612 << 20),
                    height: Tick(792 << 20),
                    items: vec![run, rule(Provenance::Source(src(7, 8))), rule(Provenance::Synthetic("frac".into()))],
                },
                Page {
                    number: 2,
                    width: Tick(1),
                    height: Tick(2),
                    items: Vec::new(),
                },
            ],
            diagnostics: vec![
                Diagnostic::warning("overfull_hbox", "line \"3\" is 1.5pt too wide", vec![src(1, 9)]),
                Diagnostic::error("compiler", "x", Vec::new()),
            ],
        };
        assert_eq!(list.write_json("id\"1"), json::write(&list.to_json("id\"1")));
        let empty = DisplayList {
            project_id: String::new(),
            revision: 0,
            documents: Vec::new(),
            fonts: Vec::new(),
            pages: Vec::new(),
            diagnostics: Vec::new(),
        };
        assert_eq!(empty.write_json(""), json::write(&empty.to_json("")));
    }
}
