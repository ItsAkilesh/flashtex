//! Knuth–Plass optimum-fit line breaking with TeX's badness, fitness classes
//! and demerits (TeX: The Program §§851–890, in a much smaller form).
//!
//! TEMPORARY SHIM for the `paragraph-layout` sibling crate (not on any
//! branch at the time of writing). Items are boxes, glue and penalties in
//! points; badness uses TeX's integer formula over scaled points so that
//! feasibility decisions match TeX. No hyphenation: the second pass differs
//! from the first only by tolerance, and a final pass accepts overfull lines
//! when nothing fits (TeX's behaviour with `\emergencystretch = 0pt`).

/// Anything that occupies horizontal space on a line.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// `payload` is an opaque index the caller maps back to content.
    Box { width: f64, payload: usize },
    Glue { width: f64, stretch: f64, shrink: f64, fil: bool },
    /// `flagged` marks a hyphen-like penalty (unused: no hyphenation).
    Penalty { penalty: i32, width: f64, flagged: bool },
}

pub const INF_PENALTY: i32 = 10000;
pub const INF_BAD: i32 = 10000;
pub const AWFUL_BAD: i64 = 0x3FFF_FFFF;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// Index of the first node on the line.
    pub start: usize,
    /// Index of the break node (exclusive end of the line's content; the
    /// break node itself is glue or a penalty).
    pub end: usize,
    /// Glue set ratio: positive = stretch fraction, negative = shrink fraction.
    pub ratio: f64,
    /// Natural width, total stretch, total shrink of the material set.
    pub natural: f64,
    pub stretch: f64,
    pub shrink: f64,
    pub fil: bool,
    pub badness: i32,
}

pub struct Params {
    pub line_width: f64,
    pub first_indent: f64,
    pub tolerance: i32,
    pub pretolerance: i32,
    pub linepenalty: i32,
    pub adjdemerits: i32,
}

fn sp(pt: f64) -> i64 {
    (pt * 65536.0).round() as i64
}

/// TeX's `badness(t, s)` (§108).
pub fn badness(t_pt: f64, s_pt: f64) -> i32 {
    let t = sp(t_pt);
    let s = sp(s_pt);
    if t == 0 {
        return 0;
    }
    if s <= 0 {
        return INF_BAD;
    }
    let r = if t <= 7_230_584 {
        (t * 297) / s
    } else if s >= 1_663_497 {
        t / (s / 297)
    } else {
        t
    };
    if r > 1290 {
        INF_BAD
    } else {
        ((r * r * r + 0x20000) / 0x40000) as i32
    }
}

#[derive(Clone, Copy)]
struct Active {
    /// Node index of the break (0 = paragraph start).
    at: usize,
    line: usize,
    fitness: u8,
    demerits: i64,
    prev: Option<usize>, // index into `nodes_out`
    ratio: f64,
    natural: f64,
    stretch: f64,
    shrink: f64,
    fil: bool,
    badness: i32,
}

struct Sums {
    width: f64,
    stretch: f64,
    shrink: f64,
    fil: bool,
}

/// Breaks `nodes` (which must end with a forced break: glue + penalty
/// `-INF_PENALTY`) into lines.
pub fn break_paragraph(nodes: &[Node], p: &Params) -> Vec<Line> {
    for threshold in [p.pretolerance, p.tolerance] {
        if threshold < 0 {
            continue;
        }
        if let Some(lines) = try_break(nodes, p, threshold, false) {
            return lines;
        }
    }
    try_break(nodes, p, INF_BAD, true).unwrap_or_default()
}

fn line_width(p: &Params, line: usize) -> f64 {
    if line == 0 {
        p.line_width - p.first_indent
    } else {
        p.line_width
    }
}

fn fitness_class(ratio: f64, b: i32, shrinking: bool) -> u8 {
    if shrinking {
        if b > 12 {
            3
        } else {
            2
        }
    } else if b > 99 {
        0
    } else if b > 12 {
        1
    } else {
        2
    }
    .max(if ratio.is_nan() { 2 } else { 0 })
}

fn try_break(nodes: &[Node], p: &Params, threshold: i32, final_pass: bool) -> Option<Vec<Line>> {
    // Prefix sums from paragraph start.
    let n = nodes.len();
    let mut pre_w = vec![0.0; n + 1];
    let mut pre_st = vec![0.0; n + 1];
    let mut pre_sh = vec![0.0; n + 1];
    let mut pre_fil = vec![0u32; n + 1];
    for (i, node) in nodes.iter().enumerate() {
        let (w, st, sh, fil) = match node {
            Node::Box { width, .. } => (*width, 0.0, 0.0, 0),
            Node::Glue { width, stretch, shrink, fil } => (*width, *stretch, *shrink, u32::from(*fil)),
            Node::Penalty { .. } => (0.0, 0.0, 0.0, 0),
        };
        pre_w[i + 1] = pre_w[i] + w;
        pre_st[i + 1] = pre_st[i] + st;
        pre_sh[i + 1] = pre_sh[i] + sh;
        pre_fil[i + 1] = pre_fil[i] + fil;
    }
    // Material of a line from break `a` to break `b`: glue right after `a`
    // is discarded; the break node `b` contributes its width if it is a
    // penalty (hyphen) and nothing if glue.
    let sums = |a: usize, b: usize| -> Sums {
        let mut s = a;
        // Skip discardable items after a break.
        while s < b && matches!(nodes[s], Node::Glue { .. } | Node::Penalty { .. }) {
            s += 1;
        }
        let pen_w = match nodes.get(b) {
            Some(Node::Penalty { width, .. }) => *width,
            _ => 0.0,
        };
        Sums {
            width: pre_w[b] - pre_w[s] + pen_w,
            stretch: pre_st[b] - pre_st[s],
            shrink: pre_sh[b] - pre_sh[s],
            fil: pre_fil[b] > pre_fil[s],
        }
    };

    let mut actives: Vec<Active> = vec![Active {
        at: 0,
        line: 0,
        fitness: 2,
        demerits: 0,
        prev: None,
        ratio: 0.0,
        natural: 0.0,
        stretch: 0.0,
        shrink: 0.0,
        fil: false,
        badness: 0,
    }];
    let mut passive: Vec<Active> = Vec::new();

    for b in 0..n {
        let legal = match &nodes[b] {
            Node::Penalty { penalty, .. } => *penalty < INF_PENALTY,
            Node::Glue { .. } => b > 0 && matches!(nodes[b - 1], Node::Box { .. }),
            Node::Box { .. } => false,
        };
        if !legal {
            continue;
        }
        let penalty = match &nodes[b] {
            Node::Penalty { penalty, .. } => *penalty,
            _ => 0,
        };
        let mut best: [Option<(i64, usize, f64, i32, Sums)>; 4] = [None, None, None, None];
        let mut still_active = Vec::with_capacity(actives.len());
        for (ai, a) in actives.iter().enumerate() {
            let s = sums(a.at, b);
            let lw = line_width(p, a.line);
            let shortfall = lw - s.width;
            let (ratio, bad, shrinking) = if shortfall > 0.0 {
                if s.fil {
                    (0.0, 0, false)
                } else if s.stretch > 0.0 {
                    (shortfall / s.stretch, badness(shortfall, s.stretch), false)
                } else {
                    (f64::INFINITY, INF_BAD, false)
                }
            } else if shortfall < 0.0 {
                if s.shrink > 0.0 && -shortfall <= s.shrink + 1e-9 {
                    (shortfall / s.shrink, badness(-shortfall, s.shrink), true)
                } else if -shortfall <= s.shrink + 1e-9 {
                    (0.0, 0, true)
                } else {
                    (-1.0, AWFUL_BAD as i32, true)
                }
            } else {
                (0.0, 0, false)
            };
            let overfull = bad > INF_BAD;
            // Deactivate when the line is overfull or the break is forced.
            let forced = penalty <= -INF_PENALTY;
            let keep = !overfull && !forced;
            let feasible = if final_pass {
                !overfull || true
            } else {
                !overfull && bad <= threshold
            };
            if feasible {
                let bad_capped = bad.min(INF_BAD);
                let fit = fitness_class(ratio, bad_capped, shrinking);
                let d = {
                    let lb = i64::from(p.linepenalty) + i64::from(bad_capped);
                    let mut d = lb * lb;
                    if penalty >= 0 {
                        d += i64::from(penalty) * i64::from(penalty);
                    } else if penalty > -INF_PENALTY {
                        d -= i64::from(penalty) * i64::from(penalty);
                    }
                    if (i32::from(fit) - i32::from(a.fitness)).abs() > 1 {
                        d += i64::from(p.adjdemerits);
                    }
                    if overfull {
                        d += AWFUL_BAD;
                    }
                    d
                };
                let total = a.demerits + d;
                let slot = &mut best[usize::from(fit)];
                if slot.as_ref().map_or(true, |(td, ..)| total < *td) {
                    *slot = Some((total, ai, ratio, bad_capped, s));
                }
            }
            if keep {
                still_active.push(a.clone());
            } else {
                passive.push(*a);
            }
        }
        // Create new active nodes at `b` for each fitness class winner,
        // then drop those dominated by a better total in another class
        // (TeX keeps one per class; that is what we do).
        let mut new_nodes = Vec::new();
        let mut min_total = i64::MAX;
        for slot in best.iter().flatten() {
            min_total = min_total.min(slot.0);
        }
        for (fit, slot) in best.iter().enumerate() {
            if let Some((total, ai, ratio, bad, s)) = slot {
                if *total > min_total + i64::from(p.adjdemerits) {
                    continue;
                }
                let a = &actives[*ai];
                passive.push(*a);
                let prev_index = passive.len() - 1;
                new_nodes.push(Active {
                    at: b,
                    line: a.line + 1,
                    fitness: fit as u8,
                    demerits: *total,
                    prev: Some(prev_index),
                    ratio: *ratio,
                    natural: s.width,
                    stretch: s.stretch,
                    shrink: s.shrink,
                    fil: s.fil,
                    badness: *bad,
                });
            }
        }
        actives = still_active;
        actives.extend(new_nodes);
        if actives.is_empty() {
            return None;
        }
    }
    // The paragraph must end with a forced break at the last node, so the
    // remaining actives are the candidates ending there.
    let end = actives.iter().filter(|a| a.at == n - 1).min_by_key(|a| a.demerits)?;
    let mut chain = vec![*end];
    let mut cur = end.prev;
    while let Some(pi) = cur {
        let a = passive[pi];
        if a.at == 0 && a.line == 0 {
            break;
        }
        chain.push(a);
        cur = a.prev;
    }
    chain.reverse();
    let mut lines = Vec::with_capacity(chain.len());
    let mut start = 0;
    for a in chain {
        lines.push(Line {
            start,
            end: a.at,
            ratio: a.ratio,
            natural: a.natural,
            stretch: a.stretch,
            shrink: a.shrink,
            fil: a.fil,
            badness: a.badness,
        });
        start = a.at;
    }
    Some(lines)
}

/// Positions the boxes of one line: returns `(payload, x)` for every box,
/// with `x` relative to the line's left edge.
pub fn place(nodes: &[Node], line: &Line) -> Vec<(usize, f64)> {
    let mut x = 0.0;
    let mut out = Vec::new();
    let mut s = line.start;
    while s < line.end && matches!(nodes[s], Node::Glue { .. } | Node::Penalty { .. }) {
        s += 1;
    }
    for node in &nodes[s..line.end] {
        match node {
            Node::Box { width, payload } => {
                out.push((*payload, x));
                x += width;
            }
            Node::Glue { width, stretch, shrink, fil } => {
                let mut w = *width;
                if line.fil {
                    if *fil {
                        // fil glue absorbs everything; finite glue is natural.
                    }
                } else if line.ratio > 0.0 {
                    w += stretch * line.ratio;
                } else if line.ratio < 0.0 {
                    w += shrink * line.ratio;
                }
                x += w;
            }
            Node::Penalty { .. } => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badness_matches_tex_examples() {
        assert_eq!(badness(0.0, 1.0), 0);
        assert_eq!(badness(1.0, 0.0), INF_BAD);
        // t = s gives 100.
        assert_eq!(badness(1.0, 1.0), 100);
        assert_eq!(badness(0.5, 1.0), 12);
    }

    #[test]
    fn breaks_a_paragraph_into_feasible_lines() {
        let mut nodes = Vec::new();
        for i in 0..20 {
            nodes.push(Node::Box { width: 30.0, payload: i });
            nodes.push(Node::Glue { width: 4.0, stretch: 2.0, shrink: 1.0, fil: false });
        }
        nodes.pop();
        nodes.push(Node::Penalty { penalty: INF_PENALTY, width: 0.0, flagged: false });
        nodes.push(Node::Glue { width: 0.0, stretch: 0.0, shrink: 0.0, fil: true });
        nodes.push(Node::Penalty { penalty: -INF_PENALTY, width: 0.0, flagged: false });
        let p = Params { line_width: 200.0, first_indent: 0.0, tolerance: 200, pretolerance: 100, linepenalty: 10, adjdemerits: 10000 };
        let lines = break_paragraph(&nodes, &p);
        assert!(lines.len() >= 3);
        for l in &lines[..lines.len() - 1] {
            assert!(l.natural <= 200.0 + l.shrink + 1e-9);
            assert!(l.badness <= 200);
        }
        assert!(lines.last().unwrap().fil);
    }
}
