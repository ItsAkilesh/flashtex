//! TeX font metrics (`.tfm`) for the text faces, so widths, kerns,
//! ligatures, heights/depths and the interword `\fontdimen`s are the exact
//! fixed-point values pdfTeX lays out with, while the OpenType program of
//! the same face supplies the outlines. A TFM is metrics data from the TeX
//! distribution, like the `.otf` next to it; no TeX engine runs.
//!
//! Parsing and the ligature/kern program are the shared `font-resources`
//! reader (`flashtex_font_resources::tfm::Tfm`, `tfm_run::GlyphRun`, pinned
//! at `vendor/font-resources/PIN`): bounded tables, boundary-character
//! programs on both sides, checked kern sums, a 4096-code run limit. This
//! module keeps the pipeline's view: fixwords (2^-20 of the design size)
//! as the advance unit with `2^20` units per em, so `fixword * size / 2^20`
//! reproduces TeX's `xn_over_d` to within one scaled point; a run's
//! `leading_kern` (a left-boundary kern) is carried as an explicit
//! zero-glyph advance before the first character, never dropped. Errors
//! from the shared interpreter propagate as [`TfmError`]; callers report
//! them and fall back to the font program's own metrics for that run.

use std::path::Path;

use flashtex_font_resources::tfm::{BoundaryOptions, Tfm as SharedTfm};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TfmError(pub String);

impl std::fmt::Display for TfmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Tfm {
    inner: SharedTfm,
    pub design_size_pt: f64,
}

impl Tfm {
    pub fn load(path: &Path) -> Result<Tfm, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Tfm::parse(&bytes).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn parse(b: &[u8]) -> Result<Tfm, String> {
        let inner = SharedTfm::parse(b).map_err(|e| format!("{e:?}"))?;
        Ok(Tfm::from_shared(inner))
    }

    /// Wraps a TFM the shared loader already verified (a digest-bound
    /// required asset).
    pub fn from_shared(inner: SharedTfm) -> Tfm {
        let design_size_pt = f64::from(inner.design_size.0) / FIX as f64;
        Tfm { inner, design_size_pt }
    }

    /// SHA-256 of the TFM bytes (font-resources' `source_sha256`), for
    /// provenance evidence.
    pub fn sha256(&self) -> &str {
        &self.inner.source_sha256
    }

    /// Whether the font declares a boundary character or a left-boundary
    /// program (both are executed by the shared interpreter).
    pub fn has_boundary(&self) -> bool {
        self.inner.boundary_character.is_some() || self.inner.left_boundary_program.is_some()
    }

    /// Metrics of a character code, `None` when the font has no such
    /// character (never a zero-width guess).
    pub fn metrics(&self, code: u8) -> Option<CharMetrics> {
        let m = self.inner.char_metrics(code)?;
        Some(CharMetrics {
            width: m.width.0,
            height: m.height.0,
            depth: m.depth.0,
            italic: m.italic.0,
        })
    }

    /// `\fontdimen n` (1-based) in fixwords.
    pub fn param(&self, n: usize) -> Option<i32> {
        self.inner.parameter(n).map(|v| v.0)
    }

    /// Fixword to points at `size_pt`.
    pub fn pt(fixword: i32, size_pt: f64) -> f64 {
        f64::from(fixword) * size_pt / FIX as f64
    }

    /// Applies the ligature/kern program (both boundaries on, as TeX does
    /// for a word between non-character nodes) to a sequence of codes.
    /// Each output glyph keeps the range of input positions it came from
    /// and the kern that follows it; a left-boundary kern is returned
    /// separately and must be advanced before the first glyph.
    pub fn ligkern(&self, codes: &[u8]) -> Result<TfmRun, TfmError> {
        let run = self
            .inner
            .glyph_run(codes, BoundaryOptions::default())
            .map_err(|e| TfmError(format!("{e:?}")))?;
        Ok(TfmRun {
            leading_kern: run.leading_kern.0,
            glyphs: run
                .glyphs
                .into_iter()
                .map(|g| TfmGlyph {
                    code: g.code,
                    input: (g.input_start, g.input_end),
                    kern_after: g.kern_after.0,
                })
                .collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TfmRun {
    /// Fixword kern before the first glyph (left boundary program).
    pub leading_kern: i32,
    pub glyphs: Vec<TfmGlyph>,
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
        assert_eq!(t.sha256().len(), 64);
        // TFtoPL: (CHARACTER C w (CHARWD R 0.707164)) in ec-lmr12.
        let w = t.metrics(b'w').unwrap();
        assert!((Tfm::pt(w.width, 12.0) - 8.486).abs() < 0.001, "{}", Tfm::pt(w.width, 12.0));
        // f f i -> ffi (T1 slot 0x1E) through two ligature steps.
        let g = t.ligkern(b"office").unwrap();
        let codes: Vec<u8> = g.glyphs.iter().map(|g| g.code).collect();
        assert_eq!(codes, vec![b'o', 0x1E, b'c', b'e']);
        assert_eq!(g.glyphs[1].input, (1, 4));
        assert_eq!(g.leading_kern, 0, "ec-lmr12 has no boundary program");
        // "wo" kerns by -0.0272em (TFtoPL: KRN C o R -0.027199).
        let g = t.ligkern(b"wo").unwrap();
        assert!((Tfm::pt(g.glyphs[0].kern_after, 12.0) + 0.326).abs() < 0.002, "{}", Tfm::pt(g.glyphs[0].kern_after, 12.0));
        // Interword glue: \fontdimen2..4 and 7.
        assert!((Tfm::pt(t.param(2).unwrap(), 12.0) - 3.916).abs() < 0.002);
        assert!(!t.has_boundary());
    }
}
