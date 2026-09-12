//! `MathFontMetrics` with TeX's own metrics: the Appendix G parameters and
//! glyph boxes of the `lmmi`/`lmsy`/`lmex` TFMs that pdfLaTeX+`lmodern`
//! lays math out with, and `rm-lmr*` for the roman family (digits,
//! parentheses, operators). Latin Modern's math TFMs are metric-identical
//! to Computer Modern's (verified byte for byte on `lmmi12`, `lmsy10`,
//! `lmex10`, `lmmi8`, `lmsy8` against `cmmi12`, `cmsy10`, `cmex10`,
//! `cmmi8`, `cmsy8`), so families 1–3 come from math-layout's embedded
//! `CmMathMetrics` (`latex_12pt`/`latex_10pt`, plus LaTeX's 11pt sizes) and
//! only family 0, where `rm-lmr` differs from `cmr` in heights by up to
//! 0.015 em, is read from the installed TFM.
//!
//! Painting still uses the Latin Modern Math OpenType program: every glyph
//! the layout places is a (TFM font, code) pair that [`TexMathMetrics::otf_gid`]
//! maps to an original glyph id of that face (base glyph by character,
//! `lmex` size chains by index into `MathVariants`). Extensible assemblies
//! are not mapped; a glyph without a mapping is reported, never drawn as
//! `.notdef`.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use flashtex_math_layout::cm::{self, CmMathMetrics, Family};
use flashtex_math_layout::cm_tfm;
use flashtex_math_layout::metrics::Extensible;
use flashtex_math_layout::{FontId as MathFontId, Glyph, MathFontMetrics, MathParams, SizeClass};

use crate::fonts::LoadedFace;
use crate::mathfont::{MathFonts, MathSizes};
use crate::tfm::Tfm;

pub struct TexMathMetrics {
    cm: CmMathMetrics,
    sizes: MathSizes,
    /// `rm-lmr*` at text/script/scriptscript, when installed.
    roman: [Option<Rc<Tfm>>; 3],
    /// The OpenType face drawn (Latin Modern Math) and its variant table.
    otf: Rc<MathFonts>,
    unmapped: RefCell<Vec<(String, u8, char)>>,
}

impl TexMathMetrics {
    /// `base` is the document's body size (10/11/12). `otf` supplies the
    /// glyph program; `tfm_dirs` are probed for `rm-lmr<d>.tfm`.
    pub fn new(base: u32, otf: Rc<MathFonts>, tfm_dirs: &[PathBuf]) -> TexMathMetrics {
        let (cm, roman_names) = match base {
            10 => (CmMathMetrics::latex_10pt(), ["rm-lmr10", "rm-lmr7", "rm-lmr5"]),
            11 => (
                // size11.clo: \DeclareMathSizes{\@xipt}{\@xipt}{8}{6} with the
                // 10pt designs scaled to 10.95pt for text.
                CmMathMetrics {
                    sizes: [10.95, 8.0, 6.0],
                    extension: cm::ExtensionSizing::Fixed,
                    families: [
                        [&cm_tfm::CMR10, &cm_tfm::CMR8, &cm_tfm::CMR6],
                        [&cm_tfm::CMMI10, &cm_tfm::CMMI8, &cm_tfm::CMMI6],
                        [&cm_tfm::CMSY10, &cm_tfm::CMSY8, &cm_tfm::CMSY6],
                    ],
                },
                ["rm-lmr10", "rm-lmr8", "rm-lmr6"],
            ),
            _ => (CmMathMetrics::latex_12pt(), ["rm-lmr12", "rm-lmr8", "rm-lmr6"]),
        };
        let sizes = MathSizes {
            text: cm.sizes[0],
            script: cm.sizes[1],
            script_script: cm.sizes[2],
        };
        let load = |name: &str| -> Option<Rc<Tfm>> {
            let file = format!("{name}.tfm");
            let p = tfm_dirs.iter().map(|d| d.join(&file)).find(|p| p.is_file())?;
            Tfm::load(&p).ok().map(Rc::new)
        };
        let roman = [load(roman_names[0]), load(roman_names[1]), load(roman_names[2])];
        TexMathMetrics {
            cm,
            sizes,
            roman,
            otf,
            unmapped: RefCell::new(Vec::new()),
        }
    }

    pub fn sizes(&self) -> MathSizes {
        self.sizes
    }

    /// Whether the `rm-lmr` TFMs were found (the CM families are embedded).
    pub fn roman_available(&self) -> bool {
        self.roman.iter().all(Option::is_some)
    }

    pub fn face(&self) -> &Rc<LoadedFace> {
        self.otf.face()
    }

    pub fn otf_fonts(&self) -> &Rc<MathFonts> {
        &self.otf
    }

    /// Glyphs the layout placed that have no OpenType counterpart
    /// (`(tfm font, code, char)`), drained for diagnostics.
    pub fn take_unmapped(&self) -> Vec<(String, u8, char)> {
        std::mem::take(&mut *self.unmapped.borrow_mut())
    }

    fn size_index(size: SizeClass) -> usize {
        match size {
            SizeClass::Text => 0,
            SizeClass::Script => 1,
            SizeClass::ScriptScript => 2,
        }
    }

    /// The roman-family glyph from `rm-lmr` when available, else `cmr`.
    fn roman_glyph(&self, code: u8, ch: char, size: SizeClass) -> Option<Glyph> {
        let font_id = self.cm.text_glyph('0', size)?.font_id;
        let Some(tfm) = &self.roman[Self::size_index(size)] else {
            return self.cm.glyph(ch, size);
        };
        let m = tfm.metrics(code)?;
        let at = self.cm.sizes[Self::size_index(size)];
        Some(Glyph {
            font_id,
            gid: u16::from(code),
            ch,
            size: at,
            width: Tfm::pt(m.width, at),
            height: Tfm::pt(m.height, at),
            depth: Tfm::pt(m.depth, at),
            italic: Tfm::pt(m.italic, at),
            skew: 0.0,
        })
    }

    /// The Latin Modern Math glyph id for a placed TFM glyph.
    pub fn otf_gid(&self, font: MathFontId, code: u8, ch: char) -> Option<u16> {
        let name = self.cm.font_name(font);
        let face = self.otf.face();
        let base = |c: char| face.face().glyph_id(MathFonts::math_char(c)).or_else(|| face.face().glyph_id(c)).map(|g| g.0);
        let result = if name.starts_with("cmex") {
            // Size chain in lmex: steps from the character's first cmex code
            // to `code` select the same-index vertical variant in MATH.
            let start = cm::delimiter_slot(ch)
                .map(|(_, large)| large)
                .or_else(|| cm::symbol_slot(ch).filter(|(f, _)| *f == Family::Extension).map(|(_, c)| c))
                .or(if ch == '\u{221A}' { Some(0x70) } else { None });
            match (start, base(ch)) {
                (Some(start), Some(base_gid)) => {
                    let font = &cm_tfm::CMEX10;
                    let mut cur = font.char(start);
                    let mut k = 0usize;
                    let mut found = None;
                    while let Some(c) = cur {
                        if c.code == code {
                            found = Some(k);
                            break;
                        }
                        cur = font.next_larger(c);
                        k += 1;
                        if k > 8 {
                            break;
                        }
                    }
                    match found {
                        // The text-size glyph of a cmex-based symbol (`\sum`)
                        // is the base glyph; delimiters/radicals start their
                        // chain one step above the cmr/cmsy base glyph.
                        Some(0) if cm::symbol_slot(ch).is_some_and(|(f, _)| f == Family::Extension) => Some(base_gid),
                        Some(k) => {
                            let idx = if cm::symbol_slot(ch).is_some_and(|(f, _)| f == Family::Extension) { k } else { k + 1 };
                            self.otf.variant_gid(base_gid, idx)
                        }
                        None => None,
                    }
                }
                _ => None,
            }
        } else {
            base(ch)
        };
        if result.is_none() {
            self.unmapped.borrow_mut().push((name, code, ch));
        }
        result
    }
}

impl MathFontMetrics for TexMathMetrics {
    fn params(&self, size: SizeClass) -> MathParams {
        self.cm.params(size)
    }

    fn font_name(&self, font: MathFontId) -> String {
        self.cm.font_name(font)
    }

    fn glyph(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        match cm::symbol_slot(ch) {
            Some((Family::Roman, code)) => self.roman_glyph(code, ch, size),
            _ => self.cm.glyph(ch, size),
        }
    }

    fn large_operator(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        self.cm.large_operator(ch, size)
    }

    fn delimiter_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph> {
        let mut v = self.cm.delimiter_sizes(ch, size);
        // The smallest delimiter is the roman/symbol text glyph.
        if let (Some(first), Some(((Family::Roman, code), _))) = (v.first_mut(), cm::delimiter_slot(ch)) {
            if let Some(g) = self.roman_glyph(code, ch, size) {
                *first = g;
            }
        }
        v
    }

    fn radical_sizes(&self, size: SizeClass) -> Vec<Glyph> {
        self.cm.radical_sizes(size)
    }

    fn accent_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph> {
        self.cm.accent_sizes(ch, size)
    }

    fn delimiter_extensible(&self, ch: char, size: SizeClass) -> Option<Extensible> {
        self.cm.delimiter_extensible(ch, size)
    }

    fn radical_extensible(&self, size: SizeClass) -> Option<Extensible> {
        self.cm.radical_extensible(size)
    }

    fn text_glyph(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        if ch.is_ascii() {
            self.roman_glyph(ch as u8, ch, size)
        } else {
            self.cm.text_glyph(ch, size)
        }
    }
}
