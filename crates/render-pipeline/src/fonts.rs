//! Font set: bounded resolution of the faces the pipeline may use, with
//! content-addressed identity for the display list.
//!
//! Default document face is Latin Modern (LaTeX's default, GUST Font
//! License, CFF OpenType) with the size-to-optical-design mapping of
//! `t1lmr.fd`; Times (Adobe Core 14 metrics through font-engine) is used only
//! when the document selects it (`\usepackage{times}` / `mathptmx`).
//! Resolution is bounded: an explicit directory list is probed for explicit
//! file names; nothing is scanned or substituted silently.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use flashtex_font_engine::core14::{Core14, Core14Face};
use flashtex_font_engine::{sha256, Face, GlyphId};

use crate::otf::{Bounds, OtfFace};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Family {
    LatinModern,
    Times,
}

/// Which typographic role a face plays; selects the design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    /// Text face (roman/bold/italic per the style).
    Text { bold: bool, italic: bool },
    /// Math symbols and operators (LM Math, or Core 14 Symbol for Times).
    MathSymbols,
}

/// Default search directories, probed in order. Only explicit file names are
/// opened. `FLASHTEX_FONT_DIRS` (colon separated) is prepended when set.
pub const DEFAULT_FONT_DIRS: [&str; 2] = [
    "/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math",
];

pub enum FaceKind {
    Otf(OtfFace),
    Core14(Core14Face),
}

/// A loaded face plus the identity fields the display list publishes.
pub struct LoadedFace {
    /// Stable human-readable id, e.g. `lmroman12-regular` or `Times-Roman`.
    pub font_id: String,
    pub kind: FaceKind,
    pub sha256_hex: String,
    pub byte_length: u64,
    pub units_per_em: u32,
    pub glyph_count: u32,
    pub postscript_name: String,
    /// `opentype-cff` (Latin Modern) or `core14-afm` (metrics only).
    pub format: &'static str,
}

impl LoadedFace {
    pub fn face(&self) -> &dyn Face {
        match &self.kind {
            FaceKind::Otf(f) => f,
            FaceKind::Core14(f) => f,
        }
    }

    pub fn program(&self) -> Option<&[u8]> {
        match &self.kind {
            FaceKind::Otf(f) => Some(f.program()),
            FaceKind::Core14(_) => None,
        }
    }

    /// Font-unit value in points at `size_pt`.
    pub fn pt(&self, units: i64, size_pt: f64) -> f64 {
        units as f64 * size_pt / f64::from(self.units_per_em)
    }

    /// Glyph extents in font units. CFF faces use real outlines; Core 14
    /// faces (no outlines available) use class-based AFM approximations,
    /// which is stated in README.
    pub fn bounds(&self, gid: GlyphId, ch: Option<char>) -> Bounds {
        match &self.kind {
            FaceKind::Otf(f) => f.bounds(gid).unwrap_or_default(),
            FaceKind::Core14(f) => {
                let h = f.which().header();
                let adv = i32::from(f.advance(gid).unwrap_or(0));
                let (y_min, y_max) = match ch {
                    Some(c) if c.is_ascii_digit() || c.is_uppercase() => (0, i32::from(h.cap_height.max(662))),
                    Some(c) if "bdfhklt".contains(c) => (0, i32::from(h.ascender)),
                    Some(c) if "gjpqy".contains(c) => (i32::from(h.descender), i32::from(h.x_height)),
                    Some(c) if "()[]{}/|".contains(c) => (i32::from(h.descender), i32::from(h.ascender)),
                    Some(c) if ",;".contains(c) => (-140, i32::from(h.x_height) / 3),
                    Some(c) if c.is_lowercase() => (0, i32::from(h.x_height)),
                    Some(c) if "+=<>-".contains(c) => (100, 500),
                    Some(c) if c == '.' => (0, 100),
                    _ => (i32::from(h.descender), i32::from(h.ascender)),
                };
                Bounds {
                    x_min: 0,
                    y_min,
                    x_max: adv,
                    y_max,
                    empty: false,
                }
            }
        }
    }
}

pub struct FontSet {
    dirs: Vec<PathBuf>,
    faces: RefCell<Vec<Rc<LoadedFace>>>,
    by_id: RefCell<BTreeMap<String, usize>>,
    /// File names that failed to load, with the reason (reported once).
    failures: RefCell<BTreeMap<String, String>>,
}

impl FontSet {
    /// Bounded search list: `FLASHTEX_FONT_DIRS` entries first, then the
    /// TeX Live defaults, then any explicit extra directories.
    pub fn with_default_dirs(extra: &[PathBuf]) -> FontSet {
        let mut dirs: Vec<PathBuf> = Vec::new();
        if let Ok(v) = std::env::var("FLASHTEX_FONT_DIRS") {
            dirs.extend(v.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
        }
        dirs.extend(extra.iter().cloned());
        dirs.extend(DEFAULT_FONT_DIRS.iter().map(PathBuf::from));
        FontSet::new(dirs)
    }

    pub fn new(dirs: Vec<PathBuf>) -> FontSet {
        FontSet {
            dirs,
            faces: RefCell::new(Vec::new()),
            by_id: RefCell::new(BTreeMap::new()),
            failures: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn dirs(&self) -> &[PathBuf] {
        &self.dirs
    }

    /// Every face loaded so far, in load order (the display-list resource
    /// table is emitted from this, restricted to faces actually used).
    pub fn loaded(&self) -> Vec<Rc<LoadedFace>> {
        self.faces.borrow().clone()
    }

    pub fn failures(&self) -> Vec<(String, String)> {
        self.failures
            .borrow()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn by_id(&self, id: &str) -> Option<Rc<LoadedFace>> {
        let idx = *self.by_id.borrow().get(id)?;
        self.faces.borrow().get(idx).cloned()
    }

    /// Latin Modern optical-size file for a text role, per `t1lmr.fd`.
    pub fn latin_modern_file(role: Role, size_pt: f64) -> Option<String> {
        let s = size_pt;
        Some(match role {
            Role::MathSymbols => "latinmodern-math.otf".to_string(),
            Role::Text { bold: false, italic: false } => {
                let d = if s < 5.5 {
                    5
                } else if s < 6.5 {
                    6
                } else if s < 7.5 {
                    7
                } else if s < 8.5 {
                    8
                } else if s < 9.5 {
                    9
                } else if s < 11.0 {
                    10
                } else if s < 15.0 {
                    12
                } else {
                    17
                };
                format!("lmroman{d}-regular.otf")
            }
            Role::Text { bold: true, italic: false } => {
                let d = if s < 5.5 {
                    5
                } else if s < 6.5 {
                    6
                } else if s < 7.5 {
                    7
                } else if s < 8.5 {
                    8
                } else if s < 9.5 {
                    9
                } else if s < 11.0 {
                    10
                } else {
                    12
                };
                format!("lmroman{d}-bold.otf")
            }
            Role::Text { bold: false, italic: true } => {
                let d = if s < 7.5 {
                    7
                } else if s < 8.5 {
                    8
                } else if s < 9.5 {
                    9
                } else if s < 11.0 {
                    10
                } else {
                    12
                };
                format!("lmroman{d}-italic.otf")
            }
            Role::Text { bold: true, italic: true } => "lmroman10-bolditalic.otf".to_string(),
        })
    }

    fn core14_for(role: Role) -> Core14 {
        match role {
            Role::MathSymbols => Core14::Symbol,
            Role::Text { bold: false, italic: false } => Core14::TimesRoman,
            Role::Text { bold: true, italic: false } => Core14::TimesBold,
            Role::Text { bold: false, italic: true } => Core14::TimesItalic,
            Role::Text { bold: true, italic: true } => Core14::TimesBoldItalic,
        }
    }

    /// Resolves (and loads once) the face for `family`/`role` at `size_pt`.
    /// Latin Modern failures fall back to Times *with an explicit error
    /// string returned alongside*, so callers report the substitution.
    pub fn resolve(&self, family: Family, role: Role, size_pt: f64) -> Resolved {
        match family {
            Family::Times => Resolved {
                face: self.core14(Self::core14_for(role)),
                substituted: None,
            },
            Family::LatinModern => {
                let file = Self::latin_modern_file(role, size_pt).expect("role has a file");
                match self.otf(&file) {
                    Ok(f) => Resolved {
                        face: f,
                        substituted: None,
                    },
                    Err(reason) => Resolved {
                        face: self.core14(Self::core14_for(role)),
                        substituted: Some(format!("{file}: {reason}")),
                    },
                }
            }
        }
    }

    fn core14(&self, which: Core14) -> Rc<LoadedFace> {
        let f = Core14Face::new(which);
        let id = f.postscript_name().to_string();
        if let Some(existing) = self.by_id(&id) {
            return existing;
        }
        let loaded = LoadedFace {
            font_id: id.clone(),
            sha256_hex: sha256::hex(&f.id().content_sha256),
            byte_length: 0,
            units_per_em: u32::from(f.units_per_em()),
            glyph_count: u32::from(f.num_glyphs()),
            postscript_name: f.postscript_name().to_string(),
            format: "core14-afm",
            kind: FaceKind::Core14(f),
        };
        self.insert(id, loaded)
    }

    fn otf(&self, file: &str) -> Result<Rc<LoadedFace>, String> {
        let id = file.trim_end_matches(".otf").to_string();
        if let Some(existing) = self.by_id(&id) {
            return Ok(existing);
        }
        if let Some(reason) = self.failures.borrow().get(file) {
            return Err(reason.clone());
        }
        let path = self
            .dirs
            .iter()
            .map(|d| d.join(file))
            .find(|p| p.is_file())
            .ok_or_else(|| format!("not found in {} search director{}", self.dirs.len(), if self.dirs.len() == 1 { "y" } else { "ies" }));
        let path = match path {
            Ok(p) => p,
            Err(reason) => {
                self.failures.borrow_mut().insert(file.to_string(), reason.clone());
                return Err(reason);
            }
        };
        match OtfFace::load(&path) {
            Ok(f) => {
                let loaded = LoadedFace {
                    font_id: id.clone(),
                    sha256_hex: sha256::hex(&f.id().content_sha256),
                    byte_length: f.program().len() as u64,
                    units_per_em: u32::from(f.units_per_em()),
                    glyph_count: u32::from(f.num_glyphs()),
                    postscript_name: f.postscript_name().to_string(),
                    format: "opentype-cff",
                    kind: FaceKind::Otf(f),
                };
                Ok(self.insert(id, loaded))
            }
            Err(e) => {
                let reason = e.to_string();
                self.failures.borrow_mut().insert(file.to_string(), reason.clone());
                Err(reason)
            }
        }
    }

    fn insert(&self, id: String, loaded: LoadedFace) -> Rc<LoadedFace> {
        let rc = Rc::new(loaded);
        let mut faces = self.faces.borrow_mut();
        self.by_id.borrow_mut().insert(id, faces.len());
        faces.push(rc.clone());
        rc
    }
}

pub struct Resolved {
    pub face: Rc<LoadedFace>,
    /// Set when the requested face was unavailable and Times stood in.
    pub substituted: Option<String>,
}
