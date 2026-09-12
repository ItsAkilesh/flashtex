//! Document style: the `article` class geometry, sizes and spacing.
//!
//! TEMPORARY SHIM for the `document-style` sibling crate (not on any branch
//! at the time of writing). Values are transcribed from LaTeX's
//! `article.cls` / `size10.clo` / `size11.clo` / `size12.clo` and the
//! `geometry` package's `margin=` semantics; the font-dependent quantities
//! (`ex`, space glue) come from the Latin Modern TFM parameters recorded in
//! `params.rs`. This is an original implementation reading no TeX at run time.

use crate::fonts::Family;

/// A vertical skip with TeX-style stretch and shrink, in points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Skip {
    pub natural: f64,
    pub stretch: f64,
    pub shrink: f64,
    /// Infinite (`fil`) stretch.
    pub fil: bool,
}

impl Skip {
    pub const fn fixed(natural: f64) -> Skip {
        Skip {
            natural,
            stretch: 0.0,
            shrink: 0.0,
            fil: false,
        }
    }
    pub const fn new(natural: f64, stretch: f64, shrink: f64) -> Skip {
        Skip {
            natural,
            stretch,
            shrink,
            fil: false,
        }
    }
}

/// `\@startsection` parameters for one heading level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SectionStyle {
    /// Skip before, in `ex` of the body font (negative in LaTeX means "no
    /// indent after"; the magnitude is used here).
    pub before_ex: (f64, f64, f64),
    /// Skip after, in `ex`.
    pub after_ex: (f64, f64),
    pub size_pt: f64,
    pub baselineskip_pt: f64,
    pub bold: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub family: Family,
    pub page_width_pt: f64,
    pub page_height_pt: f64,
    pub margin_left_pt: f64,
    pub margin_top_pt: f64,
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
    pub section: SectionStyle,
    pub subsection: SectionStyle,
    pub script_size_pt: f64,
    pub scriptscript_size_pt: f64,
    pub tolerance: i32,
    pub pretolerance: i32,
    pub linepenalty: i32,
    pub adjdemerits: i32,
    pub clubpenalty: i32,
    pub widowpenalty: i32,
    pub secpenalty: i32,
    /// `\raggedbottom` (article one-column default) versus `\flushbottom`.
    pub raggedbottom: bool,
}

impl Stylesheet {
    /// `\documentclass[<size>pt]{article}` with geometry `margin=1in`.
    /// `size` is 10, 11 or 12; anything else falls back to 10 (LaTeX's
    /// behaviour for unknown options is to ignore them).
    pub fn article(size: u32, family: Family) -> Stylesheet {
        let (body, bs, script, ss, large, large_bs, big, big_bs) = match size {
            12 => (12.0, 14.5, 8.0, 6.0, 14.4, 18.0, 17.28, 22.0),
            11 => (10.95, 13.6, 8.0, 6.0, 12.0, 14.0, 14.4, 18.0),
            _ => (10.0, 12.0, 7.0, 5.0, 12.0, 14.0, 14.4, 18.0),
        };
        let (above, above_short, below_short) = match size {
            12 => (
                Skip::new(12.0, 3.0, 7.0),
                Skip::new(0.0, 3.0, 0.0),
                Skip::new(6.5, 3.5, 3.0),
            ),
            11 => (
                Skip::new(11.0, 3.0, 6.0),
                Skip::new(0.0, 3.0, 0.0),
                Skip::new(6.5, 3.5, 3.0),
            ),
            _ => (
                Skip::new(10.0, 2.0, 5.0),
                Skip::new(0.0, 3.0, 0.0),
                Skip::new(6.0, 3.0, 3.0),
            ),
        };
        let topskip = match size {
            12 => 12.0,
            11 => 11.0,
            _ => 10.0,
        };
        Stylesheet {
            family,
            page_width_pt: 612.0,
            page_height_pt: 792.0,
            margin_left_pt: 72.0,
            margin_top_pt: 72.0,
            text_width_pt: 612.0 - 144.0,
            text_height_pt: 792.0 - 144.0,
            body_size_pt: body,
            baselineskip_pt: bs,
            lineskip_pt: 1.0,
            lineskiplimit_pt: 0.0,
            topskip_pt: topskip,
            maxdepth_pt: topskip / 2.0,
            parindent_pt: 0.0,
            parskip: Skip::new(0.0, 1.0, 0.0),
            abovedisplayskip: above,
            abovedisplayshortskip: above_short,
            belowdisplayskip: above,
            belowdisplayshortskip: below_short,
            section: SectionStyle {
                before_ex: (3.5, 1.0, 0.2),
                after_ex: (2.3, 0.2),
                size_pt: big,
                baselineskip_pt: big_bs,
                bold: true,
            },
            subsection: SectionStyle {
                before_ex: (3.25, 1.0, 0.2),
                after_ex: (1.5, 0.2),
                size_pt: large,
                baselineskip_pt: large_bs,
                bold: true,
            },
            script_size_pt: script,
            scriptscript_size_pt: ss,
            tolerance: 200,
            pretolerance: 100,
            linepenalty: 10,
            adjdemerits: 10000,
            clubpenalty: 150,
            widowpenalty: 150,
            secpenalty: -300,
            raggedbottom: true,
        }
    }

    /// Derives the stylesheet from the compiler's parse result: class option
    /// size and font-selecting packages. `parindent` follows
    /// `\setlength{\parindent}{...}` only when the compiler exposes it (it does
    /// not yet), so the caller passes it explicitly.
    pub fn from_document(class_options: &str, packages: &[String], parindent_pt: f64) -> Stylesheet {
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
        let mut s = Stylesheet::article(size, family);
        s.parindent_pt = parindent_pt;
        s
    }

    pub fn heading(&self, level: u8) -> SectionStyle {
        if level <= 1 {
            self.section
        } else {
            self.subsection
        }
    }
}
