//! Document style: the `article` class geometry, sizes and spacing.
//!
//! Numbers come from the `document-style` sibling (transcribed from
//! `article.cls` / `size1x.clo` / `geometry.sty` and cross-checked against
//! pdflatex there); font-dependent quantities (`ex`, interword glue) come
//! from the TFM parameters of the face in use (`params.rs`), which is what
//! `\@startsection`'s `ex` skips evaluate to in LaTeX. All lengths are TeX
//! points (72.27/in).

use flashtex_document_style::{BaseSize, Block, ClassOptions, Geometry, Paper, Pt, Stylesheet as DsStylesheet};

use crate::fonts::Family;
use crate::params;

/// A vertical skip with TeX-style stretch and shrink, in points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Skip {
    pub natural: f64,
    pub stretch: f64,
    pub shrink: f64,
}

impl Skip {
    pub const fn fixed(natural: f64) -> Skip {
        Skip {
            natural,
            stretch: 0.0,
            shrink: 0.0,
        }
    }
    pub const fn new(natural: f64, stretch: f64, shrink: f64) -> Skip {
        Skip {
            natural,
            stretch,
            shrink,
        }
    }
    pub fn glue(self) -> flashtex_paragraph_layout::Glue {
        flashtex_paragraph_layout::Glue::finite(self.natural, self.stretch, self.shrink)
    }
}

/// One heading level, resolved to absolute points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeadingStyle {
    pub size_pt: f64,
    pub baselineskip_pt: f64,
    pub bold: bool,
    pub before: Skip,
    pub after: Skip,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub family: Family,
    pub base: BaseSize,
    /// Paper size in TeX points (US Letter: 614.295 x 794.97).
    pub page_width_pt: f64,
    pub page_height_pt: f64,
    /// Text area (`\textwidth` x `\textheight`) with its top-left corner.
    pub text_x_pt: f64,
    pub text_y_pt: f64,
    pub text_width_pt: f64,
    pub text_height_pt: f64,
    pub body_size_pt: f64,
    pub baselineskip_pt: f64,
    pub lineskip_pt: f64,
    pub lineskiplimit_pt: f64,
    pub topskip_pt: f64,
    pub maxdepth_pt: f64,
    pub parindent_pt: f64,
    pub parskip: Skip,
    pub abovedisplayskip: Skip,
    pub abovedisplayshortskip: Skip,
    pub belowdisplayskip: Skip,
    pub belowdisplayshortskip: Skip,
    pub script_size_pt: f64,
    pub scriptscript_size_pt: f64,
    pub tolerance: f64,
    pub pretolerance: f64,
    pub linepenalty: f64,
    pub adjdemerits: f64,
    /// `\raggedbottom` (article one-column default).
    pub raggedbottom: bool,
    headings: [HeadingStyle; 3],
}

impl Stylesheet {
    /// `\documentclass[<size>pt]{article}` on US Letter with an optional
    /// `geometry` override. `size` other than 10/11/12 falls back to 10
    /// (LaTeX ignores unknown options).
    pub fn article(size: u32, family: Family, geometry: Option<Geometry>) -> Stylesheet {
        let base = match size {
            11 => BaseSize::Pt11,
            12 => BaseSize::Pt12,
            _ => BaseSize::Pt10,
        };
        let mut ds = DsStylesheet::article(ClassOptions {
            paper: Paper::Letter,
            size: base,
        });
        if let Some(g) = geometry {
            ds = ds.with_geometry(g);
        }
        let page = ds.page_layout();
        let body = ds.resolve(&[Block::Document, Block::Paragraph]);
        let body_size = body.font_size.0;
        // `ex` of the body font of the *selected family* (pdflatex evaluates
        // \section's skips in the current text font).
        let design = if size == 11 { 10 } else { size };
        let ex = params::text_params(family, false, false, design).x_height * body_size;
        let (script, scriptscript) = match base {
            BaseSize::Pt12 => (8.0, 6.0),
            BaseSize::Pt11 => (8.0, 6.0),
            BaseSize::Pt10 => (7.0, 5.0),
        };
        // Display skips from size1x.clo (\normalsize).
        let (above, above_short, below_short) = match base {
            BaseSize::Pt12 => (Skip::new(12.0, 3.0, 7.0), Skip::new(0.0, 3.0, 0.0), Skip::new(6.5, 3.5, 3.0)),
            BaseSize::Pt11 => (Skip::new(11.0, 3.0, 6.0), Skip::new(0.0, 3.0, 0.0), Skip::new(6.5, 3.5, 3.0)),
            BaseSize::Pt10 => (Skip::new(10.0, 2.0, 5.0), Skip::new(0.0, 3.0, 0.0), Skip::new(6.0, 3.0, 3.0)),
        };
        let heading = |level: u8| -> HeadingStyle {
            let h = ds.resolve(&[Block::Document, Block::Heading(level)]);
            let spec = flashtex_document_style::section_spec(level).expect("levels 1..=3");
            let before = spec.before_ex.scale(ex);
            let after = match spec.after {
                flashtex_document_style::SectionAfter::VerticalEx(s) => s.scale(ex),
                flashtex_document_style::SectionAfter::RunInEm(_) => flashtex_document_style::Skip::ZERO,
            };
            HeadingStyle {
                size_pt: h.font_size.0,
                baselineskip_pt: h.baselineskip.0,
                bold: h.bold,
                before: Skip::new(before.pt, before.plus, before.minus),
                after: Skip::new(after.pt, after.plus, after.minus),
            }
        };
        let parskip = ds.parskip();
        Stylesheet {
            family,
            base,
            page_width_pt: page.paper_width.0,
            page_height_pt: page.paper_height.0,
            text_x_pt: page.text_area.x.0,
            text_y_pt: page.text_area.y.0,
            text_width_pt: page.text_area.width.0,
            text_height_pt: page.text_area.height.0,
            body_size_pt: body_size,
            baselineskip_pt: body.baselineskip.0,
            lineskip_pt: 1.0,
            lineskiplimit_pt: 0.0,
            topskip_pt: page.top_skip.0,
            maxdepth_pt: page.top_skip.0 / 2.0,
            parindent_pt: body.parindent.0,
            parskip: Skip::new(parskip.pt, parskip.plus, parskip.minus),
            abovedisplayskip: above,
            abovedisplayshortskip: above_short,
            belowdisplayskip: above,
            belowdisplayshortskip: below_short,
            script_size_pt: script,
            scriptscript_size_pt: scriptscript,
            tolerance: 200.0,
            pretolerance: 100.0,
            linepenalty: 10.0,
            adjdemerits: 10000.0,
            raggedbottom: true,
            headings: [heading(1), heading(2), heading(3)],
        }
    }

    /// Derives the stylesheet from the parse result: class option size,
    /// font-selecting packages and a `geometry` `margin=` option.
    pub fn from_document(class_options: &str, packages: &[String], geometry: Option<Geometry>, parindent_pt: f64) -> Stylesheet {
        let size = class_options
            .split(',')
            .filter_map(|o| o.trim().strip_suffix("pt"))
            .filter_map(|n| n.parse::<u32>().ok())
            .find(|n| matches!(n, 10 | 11 | 12))
            .unwrap_or(10);
        let family = if packages.iter().any(|p| matches!(p.as_str(), "times" | "mathptmx" | "newtxtext" | "txfonts")) {
            Family::Times
        } else {
            Family::LatinModern
        };
        let mut s = Stylesheet::article(size, family, geometry);
        s.parindent_pt = parindent_pt;
        s
    }

    pub fn heading(&self, level: u8) -> HeadingStyle {
        self.headings[usize::from(level.clamp(1, 3) - 1)]
    }

    /// `\usepackage[margin=1in]{geometry}` as the sibling's `Geometry`; only
    /// `margin=`, `left/right/top/bottom=` and `textwidth/textheight=` are read.
    pub fn geometry_from_options(options: &str) -> Geometry {
        let mut g = Geometry::default();
        for opt in options.split(',') {
            let Some((k, v)) = opt.split_once('=') else { continue };
            let Ok(pt) = Pt::parse(v.trim()) else { continue };
            match k.trim() {
                "margin" => g.margin = Some(pt),
                "left" | "lmargin" | "inner" => g.left = Some(pt),
                "right" | "rmargin" | "outer" => g.right = Some(pt),
                "top" | "tmargin" => g.top = Some(pt),
                "bottom" | "bmargin" => g.bottom = Some(pt),
                "textwidth" | "width" => g.textwidth = Some(pt),
                "textheight" | "height" => g.textheight = Some(pt),
                _ => {}
            }
        }
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_geometry_matches_paragraph_layout_and_document_style() {
        let s = Stylesheet::article(12, Family::Times, Some(Geometry::margin(Pt::inches(1.0))));
        assert!((s.page_width_pt - 614.295).abs() < 1e-3);
        assert!((s.page_height_pt - 794.97).abs() < 1e-3);
        assert!((s.text_width_pt - 469.755).abs() < 1e-2, "{}", s.text_width_pt);
        assert!((s.text_x_pt - 72.27).abs() < 1e-3);
        assert_eq!(s.body_size_pt, 12.0);
        assert_eq!(s.baselineskip_pt, 14.5);
        assert_eq!(s.topskip_pt, 12.0);
        // \section in 12pt Times: 3.5ex = 18.9pt before (paragraph-layout doc).
        let h = s.heading(1);
        assert!((h.before.natural - 18.9).abs() < 1e-9, "{}", h.before.natural);
        assert!((h.after.natural - 12.42).abs() < 1e-9);
        assert_eq!(h.size_pt, 17.28);
        assert_eq!(h.baselineskip_pt, 22.0);
        // Latin Modern's ex differs (0.430556 em): 3.5ex = 18.083pt.
        let lm = Stylesheet::article(12, Family::LatinModern, Some(Geometry::margin(Pt::inches(1.0))));
        assert!((lm.heading(1).before.natural - 18.0834).abs() < 1e-3);
        // Plain article without geometry: 390pt text width at 12pt.
        let plain = Stylesheet::article(12, Family::LatinModern, None);
        assert_eq!(plain.text_width_pt, 390.0);
        assert!((plain.parindent_pt - 17.62482).abs() < 1e-5);
    }
}
