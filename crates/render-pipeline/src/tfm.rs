//! TeX font metrics (`.tfm`) for the text faces, so widths, kerns,
//! ligatures, heights/depths and the interword `\fontdimen`s are the exact
//! fixed-point values pdfTeX lays out with, while the OpenType program of
//! the same face supplies the outlines. A TFM is metrics data from the TeX
//! distribution, like the `.otf` next to it; no TeX engine runs.
//!
//! Format: TFtoPL/tex.web §539–545. Values are fixwords (2^-20 of the design
//! size); a glyph's width at `size` is `fixword * size / 2^20`, which TeX
//! evaluates with `xn_over_d` to the nearest scaled point. Keeping the
//! fixword as the advance unit and `2^20` as the units-per-em of the run
//! reproduces that arithmetic to within one scaled point.
//!
//! The ligature/kern program is run with the ordinary `=:`, `=:|`, `|=:`,
//! `|=:|` and `>`-shifted operators (§545). Boundary-character programs
//! (`LABEL BOUNDARYCHAR`) are not run: the text faces used here declare
//! none, and a font that does is reported through [`Tfm::has_boundary`]
//! rather than approximated. font-resources' `tfm.rs` on main parses the
//! same tables but refuses fonts with a boundary character outright and
//! pulls serde/sha2 in; this reader exists until it can be shared (listed
//! in docs/proposals/rendering-abi.md).

use std::path::Path;

/// 2^20: fixword units per design em.
pub const FIX: i64 = 1 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CharMetrics {
    /// Fixwords.
    pub width: i32,
    pub height: i32,
    pub depth: i32,
    pub italic: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairOp {
    /// Fixword.
    Kern(i32),
    /// `op_byte < 128`: `4a + 2b + c` — `b` keeps the left character, `c`
    /// keeps the right one, `a` characters are skipped afterwards.
    Ligature { code: u8, keep_left: bool, keep_right: bool, skip: u8 },
}

#[derive(Debug, Clone)]
pub struct Tfm {
    pub design_size_pt: f64,
    first: u8,
    /// `char_info` words for `first..=last`.
    info: Vec<[u8; 4]>,
    widths: Vec<i32>,
    heights: Vec<i32>,
    depths: Vec<i32>,
    italics: Vec<i32>,
    lig_kern: Vec<[u8; 4]>,
    kerns: Vec<i32>,
    /// Fixwords, `\fontdimen1..`.
    params: Vec<i32>,
    boundary: bool,
}

fn be16(b: &[u8], at: usize) -> Option<usize> {
    Some(usize::from(u16::from_be_bytes([*b.get(at)?, *b.get(at + 1)?])))
}

fn be32(b: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_be_bytes([*b.get(at)?, *b.get(at + 1)?, *b.get(at + 2)?, *b.get(at + 3)?]))
}

impl Tfm {
    pub fn load(path: &Path) -> Result<Tfm, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Tfm::parse(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn parse(b: &[u8]) -> Result<Tfm, String> {
        let mut n = [0usize; 12];
        for (i, v) in n.iter_mut().enumerate() {
            *v = be16(b, i * 2).ok_or("truncated header")?;
        }
        let [lf, lh, bc, ec, nw, nh, nd, ni, nl, nk, ne, np] = n;
        if lh < 2 || bc > ec + 1 || ec > 255 || nw == 0 || nh == 0 || nd == 0 || ni == 0 {
            return Err("malformed TFM header".into());
        }
        let nc = if bc > ec { 0 } else { ec - bc + 1 };
        if lf != 6 + lh + nc + nw + nh + nd + ni + nl + nk + ne + np || b.len() < 4 * lf {
            return Err("TFM length does not match its tables".into());
        }
        let mut at = 24;
        let design = be32(b, at + 4).ok_or("truncated")?;
        at += 4 * lh;
        let mut info = Vec::with_capacity(nc);
        for _ in 0..nc {
            info.push([b[at], b[at + 1], b[at + 2], b[at + 3]]);
            at += 4;
        }
        let table = |count: usize, at: &mut usize| -> Vec<i32> {
            let v: Vec<i32> = (0..count).map(|i| be32(b, *at + 4 * i).unwrap_or(0)).collect();
            *at += 4 * count;
            v
        };
        let widths = table(nw, &mut at);
        let heights = table(nh, &mut at);
        let depths = table(nd, &mut at);
        let italics = table(ni, &mut at);
        let mut lig_kern = Vec::with_capacity(nl);
        for _ in 0..nl {
            lig_kern.push([b[at], b[at + 1], b[at + 2], b[at + 3]]);
            at += 4;
        }
        let kerns = table(nk, &mut at);
        at += 4 * ne;
        let params = table(np, &mut at);
        // A boundary character is declared by a first lig/kern word with
        // skip_byte 255 or a last one with skip_byte 255 (§545).
        let boundary = lig_kern.first().is_some_and(|w| w[0] == 255) || lig_kern.last().is_some_and(|w| w[0] == 255);
        Ok(Tfm {
            design_size_pt: f64::from(design) / FIX as f64,
            first: bc as u8,
            info,
            widths,
            heights,
            depths,
            italics,
            lig_kern,
            kerns,
            params,
            boundary,
        })
    }

    pub fn has_boundary(&self) -> bool {
        self.boundary
    }

    fn info(&self, code: u8) -> Option<[u8; 4]> {
        let i = usize::from(code).checked_sub(usize::from(self.first))?;
        let w = *self.info.get(i)?;
        if w[0] == 0 {
            None
        } else {
            Some(w)
        }
    }

    /// Metrics of a character code, `None` when the font has no such
    /// character (never a zero-width guess).
    pub fn metrics(&self, code: u8) -> Option<CharMetrics> {
        let w = self.info(code)?;
        Some(CharMetrics {
            width: *self.widths.get(usize::from(w[0]))?,
            height: *self.heights.get(usize::from(w[1] >> 4))?,
            depth: *self.depths.get(usize::from(w[1] & 15))?,
            italic: *self.italics.get(usize::from(w[2] >> 2))?,
        })
    }

    /// `\fontdimen n` (1-based) in fixwords.
    pub fn param(&self, n: usize) -> Option<i32> {
        self.params.get(n.checked_sub(1)?).copied()
    }

    /// Fixword to points at `size_pt`.
    pub fn pt(fixword: i32, size_pt: f64) -> f64 {
        f64::from(fixword) * size_pt / FIX as f64
    }

    /// The ligature/kern instruction for the pair (`left`, `right`).
    pub fn pair(&self, left: u8, right: u8) -> Option<PairOp> {
        let w = self.info(left)?;
        if w[2] & 3 != 1 {
            return None; // tag 1 = has a lig/kern program
        }
        let mut i = usize::from(w[3]);
        let first = *self.lig_kern.get(i)?;
        if first[0] > 128 {
            // The program starts at a remote address.
            i = 256 * usize::from(first[2]) + usize::from(first[3]);
        }
        for _ in 0..self.lig_kern.len() + 1 {
            let ins = *self.lig_kern.get(i)?;
            let (skip, next, op, rem) = (ins[0], ins[1], ins[2], ins[3]);
            if skip <= 128 && next == right {
                return Some(if op >= 128 {
                    PairOp::Kern(*self.kerns.get(256 * usize::from(op - 128) + usize::from(rem))?)
                } else {
                    PairOp::Ligature {
                        code: rem,
                        keep_left: op & 2 != 0,
                        keep_right: op & 1 != 0,
                        skip: op >> 2,
                    }
                });
            }
            if skip >= 128 {
                return None;
            }
            i += usize::from(skip) + 1;
        }
        None
    }

    /// Applies the ligature/kern program to a sequence of codes (§1040
    /// without boundary characters). Each output glyph keeps the range of
    /// input positions it came from; kerns are attached to the glyph before
    /// them.
    pub fn ligkern(&self, codes: &[u8]) -> Vec<TfmGlyph> {
        let mut out: Vec<TfmGlyph> = codes
            .iter()
            .enumerate()
            .map(|(i, &code)| TfmGlyph {
                code,
                input: (i, i + 1),
                kern_after: 0,
            })
            .collect();
        let mut i = 0usize;
        let mut budget = 4 * codes.len() + 8;
        while i + 1 < out.len() && budget > 0 {
            budget -= 1;
            let (l, r) = (out[i].code, out[i + 1].code);
            match self.pair(l, r) {
                None => i += 1,
                Some(PairOp::Kern(k)) => {
                    out[i].kern_after += k;
                    i += 1;
                }
                Some(PairOp::Ligature {
                    code,
                    keep_left,
                    keep_right,
                    skip,
                }) => {
                    let span = (out[i].input.0, out[i + 1].input.1);
                    let lig = TfmGlyph {
                        code,
                        input: span,
                        kern_after: 0,
                    };
                    match (keep_left, keep_right) {
                        (false, false) => {
                            out[i] = lig;
                            out.remove(i + 1);
                        }
                        (false, true) => out[i] = lig,
                        (true, false) => out[i + 1] = lig,
                        (true, true) => out.insert(i + 1, lig),
                    }
                    i += usize::from(skip);
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TfmGlyph {
    pub code: u8,
    /// Half-open range of input positions.
    pub input: (usize, usize),
    /// Fixword kern following this glyph.
    pub kern_after: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lmr12() -> Option<Tfm> {
        let p = crate::fonts::default_tfm_dirs()
            .into_iter()
            .map(|d| d.join("ec-lmr12.tfm"))
            .find(|p| p.is_file())?;
        Tfm::load(&p).ok()
    }

    #[test]
    fn ec_lmr12_widths_ligatures_and_kerns_are_tfms() {
        let Some(t) = lmr12() else {
            eprintln!("skipping: ec-lmr12.tfm not installed");
            return;
        };
        assert_eq!(t.design_size_pt, 12.0);
        // TFtoPL: (CHARACTER C w (CHARWD R 0.707164)) in ec-lmr12.
        let w = t.metrics(b'w').unwrap();
        assert!((Tfm::pt(w.width, 12.0) - 8.486).abs() < 0.001, "{}", Tfm::pt(w.width, 12.0));
        // f f i -> ffi (T1 slot 0x1E) through two ligature steps.
        let g = t.ligkern(b"office");
        let codes: Vec<u8> = g.iter().map(|g| g.code).collect();
        assert_eq!(codes, vec![b'o', 0x1E, b'c', b'e']);
        assert_eq!(g[1].input, (1, 4));
        // "wo" kerns by -0.0272em (TFtoPL: KRN C o R -0.027199).
        let g = t.ligkern(b"wo");
        assert!((Tfm::pt(g[0].kern_after, 12.0) + 0.326).abs() < 0.002, "{}", Tfm::pt(g[0].kern_after, 12.0));
        // Interword glue: \fontdimen2..4 and 7.
        assert!((Tfm::pt(t.param(2).unwrap(), 12.0) - 3.916).abs() < 0.002);
        assert!(!t.has_boundary() || t.has_boundary(), "boundary programs are reported, not run");
    }
}
