//! pdflatex + microtype oracle for `layout_paragraph_microtype`: every
//! fixture of `crates/microtype/tests/oracle/expected/` (pdfTeX 1.40.27
//! `\showbox` dumps: the paragraph's hlist, the fonts' widths and
//! `\lpcode`/`\rpcode`/`\efcode`, and the broken result) is converted into
//! this crate's items and broken by this crate's breaker; line breaks,
//! per-glyph expansion, margin kerns and glyph x positions must be
//! pdfTeX's. The dump parser and pdfTeX's `hlist_out` positions are the
//! microtype crate's own test support, shared by path. No TeX is run.

#[path = "../../microtype/tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use common::*;
use flashtex_paragraph_layout::{
    Algorithm, BreakMode, FORCED_BREAK, FontId, Glue, GlueOrder, Glyph, GlyphRun, Item, LineBreakParams,
    MicroGlyph, MicroItem, MicroRun, Microtype, Penalty, layout_paragraph_microtype,
};
use flashtex_microtype::pdftex::FontParams;

fn pt(sp: i32) -> f64 {
    f64::from(sp) / 65536.0
}

fn glue(g: &GlueSpec) -> Glue {
    let order = |o: Order| match o {
        Order::Normal => GlueOrder::Finite,
        Order::Fil => GlueOrder::Fil,
        Order::Fill => GlueOrder::Fill,
        Order::Filll => GlueOrder::Filll,
    };
    Glue {
        width: pt(g.width),
        stretch: pt(g.stretch),
        stretch_order: order(g.stretch_order),
        shrink: pt(g.shrink),
        shrink_order: order(g.shrink_order),
        source: None,
    }
}

struct Conv<'f> {
    fonts: &'f BTreeMap<String, FontEntry>,
    params: BTreeMap<String, Rc<FontParams>>,
    ids: BTreeMap<String, FontId>,
}

impl Conv<'_> {
    fn font(&mut self, name: &str) -> (FontId, Rc<FontParams>) {
        let n = self.ids.len() as u8;
        let id = *self.ids.entry(name.to_string()).or_insert_with(|| FontId([n; 32]));
        let p = self.params.entry(name.to_string()).or_insert_with(|| Rc::new(self.fonts[name].params.clone())).clone();
        (id, p)
    }

    /// A run of characters (with the normal kerns between them) of one font
    /// starting at `nodes[i]`; returns the run and the index after it.
    fn run(&mut self, nodes: &[Node], mut i: usize, stop: usize) -> (GlyphRun, MicroRun, usize) {
        let Node::Char { font, .. } = &nodes[i] else { unreachable!() };
        let font = font.clone();
        let (id, params) = self.font(&font);
        let size: f64 = self.fonts[&font].size.trim_end_matches("pt").parse().unwrap();
        let mut glyphs = Vec::new();
        let mut micro = Vec::new();
        while let Some(Node::Char { font: f, code, .. }) = nodes.get(i).filter(|_| i < stop) {
            if *f != font {
                break;
            }
            let w = self.fonts[&font].widths[*code as usize];
            i += 1;
            let mut kern = 0;
            if let (Some(Node::Kern { width, kind: KernKind::Normal }), Some(Node::Char { font: f2, .. })) = (nodes.get(i), nodes.get(i + 1)) {
                if *f2 == font && i + 1 < stop {
                    kern = *width;
                    i += 1;
                }
            }
            glyphs.push(Glyph { gid: u32::from(*code), advance: pt(w + kern), kern: 0.0, cluster: 0..0 });
            micro.push(MicroGlyph { code: Some(*code), width: w, kern });
        }
        let width = glyphs.iter().map(|g| g.advance).sum();
        (
            GlyphRun { font: id, size, glyphs, width, height: 0.0, depth: 0.0, source: 0..0 },
            MicroRun { params, glyphs: micro },
            i,
        )
    }

    /// Pre/post-break text as one run (characters and normal kerns only).
    fn text(&mut self, nodes: &[Node]) -> (Option<GlyphRun>, Option<MicroRun>) {
        if nodes.is_empty() {
            return (None, None);
        }
        let (r, m, end) = self.run(nodes, 0, nodes.len());
        assert_eq!(end, nodes.len(), "discretionary text beyond one run: {nodes:?}");
        (Some(r), Some(m))
    }

    fn items(&mut self, nodes: &[Node], explicit: &[bool], hyphen_penalty: i32, ex_hyphen_penalty: i32) -> (Vec<Item>, Vec<MicroItem>) {
        let mut items = Vec::new();
        let mut micro = Vec::new();
        let mut i = 0;
        let mut disc_no = 0;
        // (item index of the penalty, nodes of replace text left)
        let mut pending: Option<(usize, usize, usize)> = None;
        while i < nodes.len() {
            match &nodes[i] {
                Node::Char { .. } => {
                    // Never let a run cross the end of a replace text.
                    let stop = pending.map_or(nodes.len(), |(_, _, stop)| stop);
                    let (r, m, end) = self.run(nodes, i, stop);
                    items.push(Item::Box(r));
                    micro.push(MicroItem { run: Some(m), ..MicroItem::default() });
                    i = end;
                }
                Node::Glue(g) => {
                    items.push(Item::Glue(glue(g)));
                    micro.push(MicroItem::default());
                    i += 1;
                }
                Node::Kern { width, kind } => {
                    items.push(Item::kern(pt(*width)));
                    micro.push(MicroItem { font_kern: *kind == KernKind::Normal, ..MicroItem::default() });
                    i += 1;
                }
                Node::Math { width, .. } => {
                    items.push(Item::kern(pt(*width)));
                    micro.push(MicroItem::default());
                    i += 1;
                }
                Node::Penalty(v) => {
                    items.push(Item::penalty(*v));
                    micro.push(MicroItem::default());
                    i += 1;
                }
                Node::HBox { width, .. } | Node::VBox { width, .. } | Node::Rule { width } => {
                    items.push(Item::Box(GlyphRun { font: FontId([255; 32]), size: 0.0, glyphs: vec![], width: pt(*width), height: 0.0, depth: 0.0, source: 0..0 }));
                    micro.push(MicroItem::default());
                    i += 1;
                }
                Node::Disc { pre, post, replace } => {
                    let (pre_r, pre_m) = self.text(pre);
                    let (post_r, post_m) = self.text(post);
                    let automatic = !explicit[disc_no];
                    disc_no += 1;
                    items.push(Item::Penalty(Penalty {
                        value: if pre.is_empty() { ex_hyphen_penalty } else { hyphen_penalty },
                        flagged: true,
                        pre_break: pre_r,
                        automatic,
                        post_break: post_r,
                        replace_count: 0,
                    }));
                    micro.push(MicroItem { pre_break: pre_m, post_break: post_m, ..MicroItem::default() });
                    i += 1;
                    if *replace > 0 {
                        pending = Some((items.len() - 1, items.len(), i + replace));
                    }
                }
                Node::Other(o) => panic!("unsupported node {o}"),
            }
            if let Some((pen, first, stop)) = pending {
                if i >= stop {
                    assert_eq!(i, stop, "replace text split");
                    let count = items.len() - first;
                    let Item::Penalty(p) = &mut items[pen] else { unreachable!() };
                    p.replace_count = count;
                    pending = None;
                }
            }
        }
        (items, micro)
    }
}

/// Character codes of the top-level chars before each discretionary.
fn disc_contexts(nodes: &[Node]) -> Vec<Vec<u8>> {
    let mut text = Vec::new();
    let mut out = Vec::new();
    for n in nodes {
        match n {
            Node::Char { code, .. } => text.push(*code),
            Node::Disc { .. } => out.push(text.clone()),
            _ => {}
        }
    }
    out
}

struct Outcome {
    name: String,
    breaks_ok: bool,
    expansion_ok: bool,
    margins_ok: bool,
    max_dx_pt: f64,
    detail: String,
}

fn run(path: &Path) -> Outcome {
    let name = path.file_stem().unwrap().to_string_lossy().to_string();
    let fx = load_fixture(&name, &fs::read_to_string(path).unwrap());
    let env = Env::from_fixture(&fx);
    let mut list = strip_list2(fx.list2.clone());
    if !matches!(list.last(), Some(Node::Glue(g)) if g.name.as_deref() == Some("parfillskip")) {
        list = finish_list1(list, &env);
    }
    // Discretionaries already in the pass-1 list are explicit.
    let explicit_ctx = disc_contexts(&fx.list1);
    let explicit: Vec<bool> = disc_contexts(&list).iter().map(|c| explicit_ctx.contains(c)).collect();
    let mut conv = Conv { fonts: &fx.fonts, params: BTreeMap::new(), ids: BTreeMap::new() };
    let (mut items, mut micro) = conv.items(&list, &explicit, fx.int("hyphenpenalty"), fx.int("exhyphenpenalty"));
    items.push(Item::penalty(FORCED_BREAK));
    micro.push(MicroItem::default());
    let params = LineBreakParams {
        line_width: pt(env.hsize),
        mode: BreakMode::Justified,
        algorithm: Algorithm::TotalFit,
        pretolerance: f64::from(env.pretolerance),
        tolerance: f64::from(env.tolerance),
        emergency_stretch: pt(env.emergency_stretch),
        line_penalty: f64::from(env.line_penalty),
        adj_demerits: f64::from(env.adj_demerits),
        double_hyphen_demerits: f64::from(env.double_hyphen_demerits),
        final_hyphen_demerits: f64::from(env.final_hyphen_demerits),
        parindent: 0.0,
        left_skip: glue(&env.left_skip),
        right_skip: glue(&env.right_skip),
        baselineskip: 12.0,
        lineskip: 1.0,
        lineskiplimit: 0.0,
        hfuzz: 0.1,
        hbadness: 1000.0,
    };
    let mt = Microtype { protrude_chars: env.protrude_chars, adjust_spacing: env.adjust_spacing, items: micro };
    let (lines, mlines) = layout_paragraph_microtype(&items, &params, &mt).expect("layout");
    let expected: Vec<&Node> = fx.result.iter().filter(|n| matches!(n, Node::HBox { .. })).collect();
    let mut o = Outcome { name, breaks_ok: lines.lines.len() == expected.len(), expansion_ok: true, margins_ok: true, max_dx_pt: 0.0, detail: String::new() };
    for (k, (line, exp)) in lines.lines.iter().zip(&expected).enumerate() {
        let Node::HBox { children, sign, set, order, .. } = exp else { unreachable!() };
        let got: Vec<(u8, i32, f64)> = line
            .runs
            .iter()
            .enumerate()
            .flat_map(|(ri, r)| {
                let e = &mlines[k].expansion[ri];
                r.glyphs.iter().enumerate().filter(|_| !r.glyphs.is_empty() && !e.is_empty()).map(move |(gi, g)| (g.gid as u8, e[gi], (r.x + g.x_offset) * 65536.0)).collect::<Vec<_>>()
            })
            .collect();
        let want_chars: Vec<(u8, i32)> = children.iter().filter_map(|n| match n {
            Node::Char { code, expansion, .. } => Some((*code, *expansion)),
            _ => None,
        }).collect();
        let want_x = glyph_positions(&env, children, *sign, *order, *set);
        let codes = |v: &[(u8, i32)]| v.iter().map(|c| c.0).collect::<Vec<_>>();
        let got_ce: Vec<(u8, i32)> = got.iter().map(|g| (g.0, g.1)).collect();
        if codes(&got_ce) != codes(&want_chars) {
            o.breaks_ok = false;
            if o.detail.is_empty() {
                let s = |v: Vec<u8>| v.into_iter().map(char::from).collect::<String>();
                o.detail = format!("line {k}: got {:?} want {:?}", s(codes(&got_ce)), s(codes(&want_chars)));
            }
            continue;
        }
        if got_ce != want_chars {
            o.expansion_ok = false;
            if o.detail.is_empty() {
                o.detail = format!("line {k}: ratio {} got {:?} want {:?}", mlines[k].expand_ratio, &got_ce[..3], &want_chars[..3]);
            }
        }
        let margins: Vec<i32> = children.iter().filter_map(|n| match n {
            Node::Kern { width, kind: KernKind::LeftMargin | KernKind::RightMargin } => Some(*width),
            _ => None,
        }).collect();
        let got_m: Vec<i32> = [mlines[k].left_margin_kern, mlines[k].right_margin_kern].into_iter().filter(|w| *w != 0).collect();
        if got_m != margins {
            o.margins_ok = false;
            if o.detail.is_empty() {
                o.detail = format!("line {k}: margin kerns got {got_m:?} want {margins:?}");
            }
        }
        for (g, w) in got.iter().zip(&want_x) {
            o.max_dx_pt = o.max_dx_pt.max((g.2 - w.1).abs() / 65536.0);
        }
    }
    if !o.breaks_ok && o.detail.is_empty() {
        o.detail = format!("{} lines, want {}", lines.lines.len(), expected.len());
    }
    o
}

#[test]
fn microtype_breaks_expansion_and_margin_kerns_match_pdftex() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../microtype/tests/oracle/expected");
    let mut paths: Vec<_> = fs::read_dir(&dir).expect("microtype oracle fixtures").map(|e| e.unwrap().path()).collect();
    paths.sort();
    assert!(paths.len() >= 32, "found {}", paths.len());
    let mut failures = Vec::new();
    let mut maxdx = 0f64;
    for p in &paths {
        let o = run(p);
        maxdx = maxdx.max(o.max_dx_pt);
        let ok = o.breaks_ok && o.expansion_ok && o.margins_ok && o.max_dx_pt <= 0.001;
        println!("{:<34} breaks={} expansion={} margins={} max_dx={:.5}pt {}", o.name, o.breaks_ok, o.expansion_ok, o.margins_ok, o.max_dx_pt, o.detail);
        if !ok {
            failures.push(o.name);
        }
    }
    println!("SUMMARY fixtures={} failures={} max_glyph_dx={maxdx:.5}pt", paths.len(), failures.len());
    assert!(failures.is_empty(), "{failures:?}");
}
