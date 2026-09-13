//! TeX's paragraph builder over `tex-boxes` nodes, in integer sp:
//! `line_break` (tex.web §813–§873), the pattern hyphenation hook
//! (§891–§899, with a simplified reconstitution) and `post_line_break`
//! (§877–§889). Written here because `crates/paragraph-layout` breaks in
//! f64 points over its own item model (seam S4); only its Liang pattern
//! matcher is reused.

use flashtex_paragraph_layout::liang::LiangHyphenator;
use flashtex_tex_boxes::node::{BoxNode, GlueNode, GlueSpec, KernKind, Node, SkipParam};
use flashtex_tex_boxes::pack::{badness, hpack, PackOrigin, PackParams, PackSpec};
use flashtex_tex_text_encoding::ligkern::{lig_kern_run, RunItem, RunOptions};

use crate::fonts::Fonts;

const AWFUL_BAD: i32 = 0o7777777777;
const INF_BAD: i32 = 10000;
const INF_PENALTY: i32 = 10000;
const EJECT_PENALTY: i32 = -10000;
const MAX_HALFWORD: i32 = 0xFFFFFFF;

/// The integer parameters `line_break` reads (LaTeX kernel defaults).
#[derive(Debug, Clone, Copy)]
pub struct BreakParams {
    pub hsize: i32,
    pub pretolerance: i32,
    pub tolerance: i32,
    pub emergency_stretch: i32,
    pub line_penalty: i32,
    pub hyphen_penalty: i32,
    pub ex_hyphen_penalty: i32,
    pub adj_demerits: i32,
    pub double_hyphen_demerits: i32,
    pub final_hyphen_demerits: i32,
    pub hang_indent: i32,
    pub hang_after: i32,
    pub uc_hyph: i32,
    pub par_fill_skip: GlueSpec,
}

impl BreakParams {
    pub fn latex(hsize: i32) -> BreakParams {
        BreakParams {
            hsize,
            pretolerance: 100,
            tolerance: 200,
            emergency_stretch: 0,
            line_penalty: 10,
            hyphen_penalty: 50,
            ex_hyphen_penalty: 50,
            adj_demerits: 10000,
            double_hyphen_demerits: 10000,
            final_hyphen_demerits: 5000,
            hang_indent: 0,
            hang_after: 1,
            uc_hyph: 1,
            par_fill_skip: GlueSpec { stretch: 65536, stretch_order: flashtex_tex_boxes::node::GlueOrder::Fil, ..GlueSpec::ZERO },
        }
    }
}

/// Counters describing what the simplified hyphenation had to skip.
#[derive(Debug, Default, Clone, Copy)]
pub struct BreakStats {
    pub paragraphs: usize,
    pub second_pass: usize,
    pub hyphenated_words: usize,
    pub hyph_skipped_word: usize,
    pub hyph_skipped_positions: usize,
}

/// One justified line ready for the vertical list.
#[derive(Debug, Clone)]
pub struct PackedLine {
    pub hbox: BoxNode,
    pub disc_break: bool,
}

type Widths = [i32; 7];

#[derive(Debug, Clone)]
struct Active {
    line_number: i32,
    fitness: usize,
    hyphenated: bool,
    total_demerits: i32,
    break_node: Option<usize>,
    /// `break_width - active_width` at creation (replaces delta nodes).
    offset: Widths,
}

#[derive(Debug, Clone, Copy)]
struct Passive {
    cur_break: Option<usize>,
    prev_break: Option<usize>,
}

fn is_char(n: &Node) -> bool {
    matches!(n, Node::Char { .. })
}

/// `precedes_break` (§148): type < math_node.
fn precedes_break(n: &Node) -> bool {
    matches!(
        n,
        Node::Box(_)
            | Node::Rule(_)
            | Node::Insert(_)
            | Node::Mark(_)
            | Node::Adjust(_)
            | Node::Ligature { .. }
            | Node::Disc { .. }
            | Node::Whatsit(_)
    )
}

fn node_width(n: &Node, fonts: &Fonts) -> i32 {
    match n {
        Node::Char { font, ch } | Node::Ligature { font, ch, .. } => fonts.get(*font).width(*ch as u8),
        Node::Box(b) => b.width,
        Node::Rule(r) => r.width.max(0),
        Node::Kern { width, .. } | Node::MarginKern { width, .. } | Node::Math { width, .. } => *width,
        _ => 0,
    }
}

struct Breaker<'a> {
    fonts: &'a Fonts,
    p: BreakParams,
    list: Vec<Node>,
    actives: Vec<Active>,
    passive: Vec<Passive>,
    active_width: Widths,
    background: Widths,
    minimal_demerits: [i32; 4],
    minimum_demerits: i32,
    best_place: [Option<usize>; 4],
    best_pl_line: [i32; 4],
    threshold: i32,
    second_pass: bool,
    final_pass: bool,
    easy_line: i32,
    last_special_line: i32,
    first_width: i32,
    second_width: i32,
    first_indent: i32,
    second_indent: i32,
    disc_width: i32,
}

impl<'a> Breaker<'a> {
    fn width(&self, n: &Node) -> i32 {
        node_width(n, self.fonts)
    }

    /// §829 `try_break`.
    fn try_break(&mut self, mut pi: i32, hyphenated: bool, cur_p: Option<usize>) {
        if pi.abs() >= INF_PENALTY {
            if pi > 0 {
                return;
            }
            pi = EJECT_PENALTY;
        }
        let mut no_break_yet = true;
        let mut old_l = 0;
        let mut line_width = 0;
        let mut break_width: Widths = [0; 7];
        let mut i = 0usize;
        loop {
            let l = if i < self.actives.len() { self.actives[i].line_number } else { MAX_HALFWORD };
            if l > old_l {
                if self.minimum_demerits < AWFUL_BAD && (old_l != self.easy_line || i == self.actives.len()) {
                    if no_break_yet {
                        no_break_yet = false;
                        break_width = self.compute_break_width(hyphenated, cur_p);
                    }
                    if self.p.adj_demerits.abs() >= AWFUL_BAD - self.minimum_demerits {
                        self.minimum_demerits = AWFUL_BAD - 1;
                    } else {
                        self.minimum_demerits += self.p.adj_demerits.abs();
                    }
                    for fit in 0..4 {
                        if self.minimal_demerits[fit] <= self.minimum_demerits {
                            self.passive.push(Passive { cur_break: cur_p, prev_break: self.best_place[fit] });
                            let mut offset = [0; 7];
                            for k in 1..7 {
                                offset[k] = break_width[k] - self.active_width[k];
                            }
                            self.actives.insert(
                                i,
                                Active {
                                    line_number: self.best_pl_line[fit] + 1,
                                    fitness: fit,
                                    hyphenated,
                                    total_demerits: self.minimal_demerits[fit],
                                    break_node: Some(self.passive.len() - 1),
                                    offset,
                                },
                            );
                            i += 1;
                        }
                        self.minimal_demerits[fit] = AWFUL_BAD;
                    }
                    self.minimum_demerits = AWFUL_BAD;
                }
                if i == self.actives.len() {
                    return;
                }
                if l > self.easy_line {
                    line_width = self.second_width;
                    old_l = MAX_HALFWORD - 1;
                } else {
                    old_l = l;
                    line_width = if l > self.last_special_line { self.second_width } else { self.first_width };
                }
            }
            let r = self.actives[i].clone();
            let mut caw: Widths = [0; 7];
            for k in 1..7 {
                caw[k] = self.active_width[k] + r.offset[k];
            }
            // §851
            let mut artificial = false;
            let shortfall = line_width - caw[1];
            let (b, fit) = if shortfall > 0 {
                if caw[3] != 0 || caw[4] != 0 || caw[5] != 0 {
                    (0, 2)
                } else if shortfall > 7230584 && caw[2] < 1663497 {
                    (INF_BAD, 0)
                } else {
                    let b = badness(shortfall, caw[2]);
                    (b, if b > 12 { if b > 99 { 0 } else { 1 } } else { 2 })
                }
            } else {
                let b = if -shortfall > caw[6] { INF_BAD + 1 } else { badness(-shortfall, caw[6]) };
                (b, if b > 12 { 3 } else { 2 })
            };
            let node_r_stays_active;
            if b > INF_BAD || pi == EJECT_PENALTY {
                if self.final_pass && self.minimum_demerits == AWFUL_BAD && i + 1 == self.actives.len() && i == 0 {
                    artificial = true;
                } else if b > self.threshold {
                    self.actives.remove(i);
                    continue;
                }
                node_r_stays_active = false;
            } else {
                if b > self.threshold {
                    i += 1;
                    continue;
                }
                node_r_stays_active = true;
            }
            // §855 record a feasible break
            let mut d: i32 = if artificial {
                0
            } else {
                let mut d = self.p.line_penalty + b;
                d = if d.abs() >= 10000 { 100000000 } else { d * d };
                if pi != 0 {
                    if pi > 0 {
                        d += pi * pi;
                    } else if pi > EJECT_PENALTY {
                        d -= pi * pi;
                    }
                }
                if hyphenated && r.hyphenated {
                    d += if cur_p.is_some() { self.p.double_hyphen_demerits } else { self.p.final_hyphen_demerits };
                }
                if (fit as i32 - r.fitness as i32).abs() > 1 {
                    d += self.p.adj_demerits;
                }
                d
            };
            d += r.total_demerits;
            if d <= self.minimal_demerits[fit] {
                self.minimal_demerits[fit] = d;
                self.best_place[fit] = r.break_node;
                self.best_pl_line[fit] = l;
                if d < self.minimum_demerits {
                    self.minimum_demerits = d;
                }
            }
            if node_r_stays_active {
                i += 1;
                continue;
            }
            self.actives.remove(i);
        }
    }

    /// §837–§840.
    fn compute_break_width(&self, hyphenated: bool, cur_p: Option<usize>) -> Widths {
        let mut bw = self.background;
        let mut s = cur_p;
        if hyphenated {
            if let Some(c) = cur_p {
                if let Node::Disc { post, replace_count, .. } = &self.list[c] {
                    let mut v = c;
                    for _ in 0..*replace_count {
                        v += 1;
                        bw[1] -= self.width(&self.list[v]);
                    }
                    for n in post {
                        bw[1] += self.width(n);
                    }
                    bw[1] += self.disc_width;
                    s = if post.is_empty() { Some(v + 1) } else { None };
                }
            }
        }
        while let Some(k) = s {
            if k >= self.list.len() {
                break;
            }
            match &self.list[k] {
                Node::Glue(g) => {
                    bw[1] -= g.spec.width;
                    bw[2 + g.spec.stretch_order.index()] -= g.spec.stretch;
                    bw[6] -= g.spec.shrink;
                }
                Node::Penalty(_) => {}
                Node::Math { width, .. } => bw[1] -= width,
                Node::Kern { width, kind } => {
                    if *kind != KernKind::Explicit {
                        break;
                    }
                    bw[1] -= width;
                }
                _ => break,
            }
            s = Some(k + 1);
        }
        bw
    }

    fn add_glue(&mut self, g: &GlueNode) {
        self.active_width[1] += g.spec.width;
        self.active_width[2 + g.spec.stretch_order.index()] += g.spec.stretch;
        self.active_width[6] += g.spec.shrink;
    }

    fn kern_break(&mut self, cur: usize, auto_breaking: bool) {
        if let Some(next) = self.list.get(cur + 1) {
            if !is_char(next) && auto_breaking && matches!(next, Node::Glue(_)) {
                self.try_break(0, false, Some(cur));
            }
        }
    }

    /// One pass (§863–§873). Returns the best active node if breaks were found.
    fn pass(&mut self, hyph: &LiangHyphenator, stats: &mut BreakStats) -> Option<Active> {
        self.actives = vec![Active {
            line_number: 1,
            fitness: 2,
            hyphenated: false,
            total_demerits: 0,
            break_node: None,
            offset: [0; 7],
        }];
        self.passive.clear();
        self.active_width = self.background;
        self.minimal_demerits = [AWFUL_BAD; 4];
        self.minimum_demerits = AWFUL_BAD;
        let mut auto_breaking = true;
        let mut cur = 0usize;
        let mut prev_p = 0usize;
        while cur < self.list.len() && !self.actives.is_empty() {
            if is_char(&self.list[cur]) {
                prev_p = cur;
                while cur < self.list.len() && is_char(&self.list[cur]) {
                    self.active_width[1] += self.width(&self.list[cur]);
                    cur += 1;
                }
                if cur >= self.list.len() {
                    break;
                }
            }
            match self.list[cur].clone() {
                Node::Box(_) | Node::Rule(_) | Node::Ligature { .. } | Node::MarginKern { .. } => {
                    self.active_width[1] += self.width(&self.list[cur]);
                }
                Node::Glue(g) => {
                    if auto_breaking {
                        let pp = &self.list[prev_p];
                        let legal = is_char(pp)
                            || precedes_break(pp)
                            || matches!(pp, Node::Kern { kind, .. } if *kind != KernKind::Explicit);
                        if legal {
                            self.try_break(0, false, Some(cur));
                        }
                    }
                    self.add_glue(&g);
                    if self.second_pass && auto_breaking {
                        try_hyphenate(&mut self.list, cur, self.fonts, hyph, self.p.uc_hyph, stats);
                    }
                }
                Node::Kern { kind, width } => {
                    if kind == KernKind::Explicit {
                        self.kern_break(cur, auto_breaking);
                    }
                    self.active_width[1] += width;
                }
                Node::Disc { pre, replace_count, .. } => {
                    self.disc_width = pre.iter().map(|n| self.width(n)).sum();
                    if pre.is_empty() {
                        self.try_break(self.p.ex_hyphen_penalty, true, Some(cur));
                    } else {
                        self.active_width[1] += self.disc_width;
                        self.try_break(self.p.hyphen_penalty, true, Some(cur));
                        self.active_width[1] -= self.disc_width;
                    }
                    let mut s = cur + 1;
                    for _ in 0..replace_count {
                        self.active_width[1] += self.width(&self.list[s]);
                        s += 1;
                    }
                    prev_p = cur;
                    cur = s;
                    continue;
                }
                Node::Math { on, width } => {
                    auto_breaking = !on;
                    self.kern_break(cur, auto_breaking);
                    self.active_width[1] += width;
                }
                Node::Penalty(p) => self.try_break(p, false, Some(cur)),
                Node::Char { .. } | Node::Whatsit(_) | Node::Mark(_) | Node::Insert(_) | Node::Adjust(_) => {}
            }
            prev_p = cur;
            cur += 1;
        }
        if cur >= self.list.len() {
            self.try_break(EJECT_PENALTY, true, None);
            if !self.actives.is_empty() {
                let mut best: Option<Active> = None;
                for a in &self.actives {
                    if best.as_ref().is_none_or(|b| a.total_demerits < b.total_demerits) {
                        best = Some(a.clone());
                    }
                }
                return best;
            }
        }
        None
    }
}

/// `line_break` + `post_line_break` for one paragraph. `list` is the
/// horizontal list as `\par` finds it (§816 preprocessing is done here).
pub fn break_paragraph(
    mut list: Vec<Node>,
    p: BreakParams,
    fonts: &Fonts,
    hyph: &LiangHyphenator,
    stats: &mut BreakStats,
) -> Vec<PackedLine> {
    stats.paragraphs += 1;
    if matches!(list.last(), Some(Node::Glue(_))) {
        list.pop();
    }
    list.push(Node::Penalty(INF_PENALTY));
    list.push(Node::param_glue(SkipParam::ParFillSkip, p.par_fill_skip, false));

    // §848–§849
    let (last_special_line, first_width, second_width, first_indent, second_indent) = if p.hang_indent == 0 {
        (0, p.hsize, p.hsize, 0, 0)
    } else {
        let lsl = p.hang_after.abs();
        if p.hang_after < 0 {
            let (w, ind) = (p.hsize - p.hang_indent.abs(), if p.hang_indent >= 0 { p.hang_indent } else { 0 });
            (lsl, w, p.hsize, ind, 0)
        } else {
            let (w, ind) = (p.hsize - p.hang_indent.abs(), if p.hang_indent >= 0 { p.hang_indent } else { 0 });
            (lsl, p.hsize, w, 0, ind)
        }
    };
    let mut b = Breaker {
        fonts,
        p,
        list,
        actives: Vec::new(),
        passive: Vec::new(),
        active_width: [0; 7],
        background: [0; 7],
        minimal_demerits: [AWFUL_BAD; 4],
        minimum_demerits: AWFUL_BAD,
        best_place: [None; 4],
        best_pl_line: [0; 4],
        threshold: p.pretolerance,
        second_pass: false,
        final_pass: false,
        easy_line: last_special_line,
        last_special_line,
        first_width,
        second_width,
        first_indent,
        second_indent,
        disc_width: 0,
    };
    if b.threshold < 0 {
        b.threshold = p.tolerance;
        b.second_pass = true;
        b.final_pass = p.emergency_stretch <= 0;
    }
    let best = loop {
        if b.threshold > INF_BAD {
            b.threshold = INF_BAD;
        }
        if let Some(best) = b.pass(hyph, stats) {
            break best;
        }
        if !b.second_pass {
            stats.second_pass += 1;
            b.threshold = p.tolerance;
            b.second_pass = true;
            b.final_pass = p.emergency_stretch <= 0;
        } else {
            b.background[2] += p.emergency_stretch;
            b.final_pass = true;
        }
    };
    // Chain of breaks.
    let mut breaks = Vec::new();
    let mut q = best.break_node;
    while let Some(i) = q {
        breaks.push(b.passive[i].cur_break);
        q = b.passive[i].prev_break;
    }
    breaks.reverse();
    post_line_break(b.list, &breaks, &b, fonts)
}

fn discardable(n: &Node) -> bool {
    match n {
        Node::Glue(_) | Node::Penalty(_) | Node::Math { .. } => true,
        Node::Kern { kind, .. } => *kind == KernKind::Explicit,
        _ => false,
    }
}

/// §877–§889.
fn post_line_break(list: Vec<Node>, breaks: &[Option<usize>], b: &Breaker, fonts: &Fonts) -> Vec<PackedLine> {
    let rightskip = Node::param_glue(SkipParam::RightSkip, GlueSpec::ZERO, true);
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut pending: Vec<Node> = Vec::new();
    for (idx, brk) in breaks.iter().enumerate() {
        let cur_line = idx as i32 + 1;
        let mut line = std::mem::take(&mut pending);
        let mut disc_break = false;
        let mut post_disc = false;
        let next_pos;
        match brk {
            Some(k) => {
                let k = *k;
                match &list[k] {
                    Node::Glue(_) => {
                        line.extend(list[pos..k].iter().cloned());
                        line.push(rightskip.clone());
                        next_pos = k + 1;
                    }
                    Node::Disc { pre, post, replace_count } => {
                        line.extend(list[pos..k].iter().cloned());
                        line.push(Node::Disc { pre: Vec::new(), post: Vec::new(), replace_count: 0 });
                        line.extend(pre.iter().cloned());
                        line.push(rightskip.clone());
                        next_pos = k + 1 + *replace_count as usize;
                        if !post.is_empty() {
                            pending = post.clone();
                            post_disc = true;
                        }
                        disc_break = true;
                    }
                    Node::Kern { kind, .. } => {
                        line.extend(list[pos..k].iter().cloned());
                        line.push(Node::Kern { width: 0, kind: *kind });
                        line.push(rightskip.clone());
                        next_pos = k + 1;
                    }
                    Node::Math { on, .. } => {
                        line.extend(list[pos..k].iter().cloned());
                        line.push(Node::Math { on: *on, width: 0 });
                        line.push(rightskip.clone());
                        next_pos = k + 1;
                    }
                    _ => {
                        line.extend(list[pos..=k].iter().cloned());
                        line.push(rightskip.clone());
                        next_pos = k + 1;
                    }
                }
            }
            None => {
                line.extend(list[pos..].iter().cloned());
                line.push(rightskip.clone());
                next_pos = list.len();
            }
        }
        pos = next_pos;
        // §879 prune the start of the next line
        if idx + 1 < breaks.len() && !post_disc {
            let next_break = breaks[idx + 1];
            while pos < list.len() && Some(pos) != next_break && discardable(&list[pos]) {
                pos += 1;
            }
        }
        let (width, indent) = if cur_line > b.last_special_line {
            (b.second_width, b.second_indent)
        } else {
            (b.first_width, b.first_indent)
        };
        let packed = hpack(line, PackSpec::Exactly(width), true, &PackParams::default(), PackOrigin::default(), fonts);
        let mut hbox = packed.node;
        hbox.shift = indent;
        out.push(PackedLine { hbox, disc_break });
    }
    out
}

/// `\lccode` for OT1 text fonts: ASCII letters only.
fn lc_code(c: u32) -> u32 {
    match c {
        0x61..=0x7a => c,
        0x41..=0x5a => c + 32,
        _ => 0,
    }
}

/// Turns a lig/kern run into `tex-boxes` nodes.
pub fn run_nodes(fonts: &Fonts, font: u32, codes: &[u8], unrestricted: bool) -> Vec<Node> {
    let f = fonts.get(font);
    let opts = RunOptions { unrestricted_hmode: unrestricted, ..RunOptions::default() };
    lig_kern_run(f, codes, opts)
        .into_iter()
        .map(|it| match it {
            RunItem::Char { code, ligature: None } => Node::Char { font, ch: code as u32 },
            RunItem::Char { code, ligature: Some(orig) } => Node::Ligature {
                font,
                ch: code as u32,
                original: orig.iter().map(|c| Node::Char { font, ch: *c as u32 }).collect(),
                left_boundary: false,
                right_boundary: false,
            },
            RunItem::Kern(w) => Node::Kern { width: w, kind: KernKind::Normal },
            RunItem::Disc => Node::Disc { pre: Vec::new(), post: Vec::new(), replace_count: 0 },
        })
        .collect()
}

/// §894–§899 word selection, Liang positions, and a reconstitution that
/// handles breaks at item boundaries only (a break inside a ligature is
/// skipped and counted).
fn try_hyphenate(
    list: &mut Vec<Node>,
    glue: usize,
    fonts: &Fonts,
    hyph: &LiangHyphenator,
    uc_hyph: i32,
    stats: &mut BreakStats,
) {
    let mut s = glue + 1;
    let hf;
    loop {
        let Some(n) = list.get(s) else { return };
        let (font, c) = match n {
            Node::Char { font, ch } => (*font, *ch),
            Node::Ligature { font, original, .. } => match original.first() {
                Some(Node::Char { ch, .. }) => (*font, *ch),
                _ => {
                    s += 1;
                    continue;
                }
            },
            Node::Kern { kind: KernKind::Normal, .. } | Node::Whatsit(_) => {
                s += 1;
                continue;
            }
            _ => return,
        };
        let lc = lc_code(c);
        if lc != 0 {
            if lc == c || uc_hyph > 0 {
                hf = font;
                break;
            }
            return;
        }
        s += 1;
    }
    let ha = s;
    let mut hb = ha;
    let mut hu: Vec<u8> = Vec::new();
    while let Some(n) = list.get(s) {
        match n {
            Node::Char { font, ch } => {
                if *font != hf || lc_code(*ch) == 0 || hu.len() == 63 {
                    break;
                }
                hu.push(*ch as u8);
                hb = s;
            }
            Node::Ligature { font, original, .. } => {
                if *font != hf {
                    break;
                }
                let chars: Vec<u32> =
                    original.iter().filter_map(|o| if let Node::Char { ch, .. } = o { Some(*ch) } else { None }).collect();
                if chars.iter().any(|c| lc_code(*c) == 0) || hu.len() + chars.len() > 63 {
                    break;
                }
                hu.extend(chars.iter().map(|c| *c as u8));
                hb = s;
            }
            Node::Kern { kind: KernKind::Normal, .. } => hb = s,
            _ => break,
        }
        s += 1;
    }
    if hu.len() < 5 {
        return;
    }
    // §899
    while let Some(n) = list.get(s) {
        match n {
            Node::Char { .. } | Node::Ligature { .. } => {}
            Node::Kern { kind, .. } => {
                if *kind != KernKind::Normal {
                    break;
                }
            }
            Node::Whatsit(_) | Node::Glue(_) | Node::Penalty(_) | Node::Insert(_) | Node::Adjust(_) | Node::Mark(_) => break,
            _ => return,
        }
        s += 1;
    }
    let word: String = hu.iter().map(|c| (*c as char).to_ascii_lowercase()).collect();
    let positions = hyph.positions(&word);
    if positions.is_empty() {
        return;
    }
    let mut orig: Vec<Node> = list[ha..=hb].to_vec();
    let mut trailing_kern = false;
    let full = run_nodes(fonts, hf, &hu, false);
    if full != orig {
        if matches!(orig.last(), Some(Node::Kern { .. })) && full[..] == orig[..orig.len() - 1] {
            orig.pop();
            trailing_kern = true;
        } else {
            stats.hyph_skipped_word += 1;
            return;
        }
    }
    // Char coverage per item.
    let mut new_nodes: Vec<Node> = Vec::new();
    let mut covered = 0usize;
    let mut inserted = 0;
    let hyphen = Node::Char { font: hf, ch: 45 };
    let mut k = 0;
    while k < full.len() {
        let n = &full[k];
        let span = match n {
            Node::Char { .. } => 1,
            Node::Ligature { original, .. } => original.len(),
            _ => 0,
        };
        let before = covered;
        covered += span;
        new_nodes.push(n.clone());
        if span > 0 {
            for &pos in &positions {
                if pos > before && pos < covered {
                    stats.hyph_skipped_positions += 1;
                }
            }
        }
        if span > 0 && positions.contains(&covered) && covered < hu.len() {
            let has_kern = matches!(full.get(k + 1), Some(Node::Kern { .. }));
            let pre_run = run_nodes(fonts, hf, &[hu[covered - 1], 45], false);
            let clean_pre = pre_run == vec![Node::Char { font: hf, ch: hu[covered - 1] as u32 }, hyphen.clone()];
            if clean_pre {
                new_nodes.push(Node::Disc { pre: vec![hyphen.clone()], post: Vec::new(), replace_count: u16::from(has_kern) });
                inserted += 1;
            } else {
                stats.hyph_skipped_positions += 1;
            }
        }
        k += 1;
    }
    if inserted == 0 {
        return;
    }
    stats.hyphenated_words += 1;
    let end = if trailing_kern { hb } else { hb + 1 };
    list.splice(ha..end, new_nodes);
}
