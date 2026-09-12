//! Font set: bounded resolution of the faces the pipeline may use, with
//! content-addressed identity for the display list.
//!
//! Default document face is Latin Modern (LaTeX's default look: Computer
//! Modern outlines, GUST Font License, OpenType CFF) with the
//! size-to-optical-design mapping of `t1lmr.fd`; Times (Adobe Core 14
//! metrics through font-engine) is used only when the document selects it
//! (`\usepackage{times}` / `mathptmx`). Resolution is bounded: an explicit
//! directory list is probed for explicit file names (font-engine's
//! `FontSearch`); nothing is scanned or substituted silently — a missing
//! Latin Modern file is a diagnostic and a `.notdef`-free failure, not a
//! silent Times.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

use flashtex_font_engine::core14::{Core14, Core14Face};
use flashtex_font_engine::math::MathTable;
use flashtex_font_engine::resolve::FontSearch;
use flashtex_font_engine::truetype::{Outlines, TrueTypeFace};
use flashtex_font_engine::{sha256, Face};

use crate::cff::{self, Cff};
use crate::tfm::Tfm;
use crate::ids::GlyphId;

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
    /// Math letters, symbols and operators: Latin Modern Math (`MATH`
    /// table) for both families, because `\usepackage{times}` leaves math
    /// in Computer Modern.
    Math,
}

/// Default search directories, probed in order for explicit file names.
/// `FLASHTEX_FONT_DIRS` (colon separated) is prepended when set.
pub const DEFAULT_FONT_DIRS: [&str; 12] = [
    // MacTeX / BasicTeX (TeX Live 2026, 2025).
    "/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm-math",
    "/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math",
    "/usr/local/texlive/2025/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2025/texmf-dist/fonts/opentype/public/lm-math",
    "/usr/local/texlive/2025basic/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2025basic/texmf-dist/fonts/opentype/public/lm-math",
    // Debian/Ubuntu `fonts-lmodern` and TeX Live packages.
    "/usr/share/texmf/fonts/opentype/public/lm",
    "/usr/share/texmf/fonts/opentype/public/lm-math",
    "/usr/share/texlive/texmf-dist/fonts/opentype/public/lm",
    "/usr/share/texlive/texmf-dist/fonts/opentype/public/lm-math",
];

/// Directories probed by default, in order: `FLASHTEX_FONT_DIRS` (colon
/// separated), `FLASHTEX_LM_DIR` (the pdf sibling's variable), a `Fonts`
/// directory next to the executable or in the enclosing app bundle's
/// `Resources`, then [`DEFAULT_FONT_DIRS`]. Nothing is scanned outside this
/// list.
pub fn default_font_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(v) = std::env::var("FLASHTEX_FONT_DIRS") {
        dirs.extend(v.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    }
    if let Ok(v) = std::env::var("FLASHTEX_LM_DIR") {
        if !v.is_empty() {
            dirs.push(PathBuf::from(v));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.join("Fonts"));
            dirs.push(dir.join("../Resources/Fonts"));
        }
    }
    dirs.extend(DEFAULT_FONT_DIRS.iter().map(PathBuf::from));
    dirs
}

/// Where the `.tfm` metrics of the text faces are looked for:
/// `FLASHTEX_TFM_DIRS` (colon separated), then every font directory with
/// `/opentype/` replaced by `/tfm/` (the TeX Live layout:
/// `fonts/opentype/public/lm` ↔ `fonts/tfm/public/lm`), then the font
/// directories themselves.
pub fn default_tfm_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(v) = std::env::var("FLASHTEX_TFM_DIRS") {
        dirs.extend(v.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
    }
    for d in default_font_dirs() {
        let text = d.to_string_lossy().replace("/opentype/", "/tfm/");
        let p = PathBuf::from(text);
        if !dirs.contains(&p) {
            dirs.push(p);
        }
        if !dirs.contains(&d) {
            dirs.push(d);
        }
    }
    dirs
}

/// The `ec-lm*` TFM that `t1lm*.fd` pairs with a Latin Modern text file:
/// `lmroman12-regular` → `ec-lmr12`, `-bold` → `ec-lmbx12`, `-italic` →
/// `ec-lmri12`, `-bolditalic` → `ec-lmbxi10`. `None` for the math face and
/// for names this table does not know.
pub fn latin_modern_tfm(otf_stem: &str) -> Option<String> {
    let rest = otf_stem.strip_prefix("lmroman")?;
    let (digits, style) = rest.split_once('-')?;
    let d: u32 = digits.parse().ok()?;
    let series = match style {
        "regular" => "r",
        "bold" => "bx",
        "italic" => "ri",
        "bolditalic" => "bxi",
        _ => return None,
    };
    Some(format!("ec-lm{series}{d}.tfm"))
}

/// Glyph extents in font units: `[x_min, y_min, x_max, y_max]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bounds {
    pub x_min: i32,
    pub y_min: i32,
    pub x_max: i32,
    pub y_max: i32,
    /// No marking contours (space and friends).
    pub empty: bool,
}

pub enum FaceKind {
    /// An OpenType program parsed by font-engine plus this crate's CFF
    /// charstring reader for glyph bounds.
    Otf { face: TrueTypeFace, cff: Cff },
    Core14(Core14Face),
}

/// A loaded face plus the identity fields the display list publishes.
pub struct LoadedFace {
    /// Content-addressed id used on the wire: the SHA-256 hex of the program
    /// (font-engine's `FontId::content_sha256`).
    pub font_id: String,
    /// Stable human-readable name (file stem or Core 14 name), for
    /// diagnostics and tests only.
    pub name: String,
    pub kind: FaceKind,
    pub sha256: [u8; 32],
    pub byte_length: u64,
    pub units_per_em: u32,
    pub glyph_count: u32,
    pub postscript_name: String,
    /// `opentype-cff` (Latin Modern), `static-truetype` (glyf) or
    /// `core14-afm` (metrics only, no program).
    pub format: &'static str,
    pub path: Option<PathBuf>,
    /// The TeX metrics pdfTeX lays this face out with (`ec-lm*.tfm`), when
    /// found; shaping then takes widths/kerns/ligatures/heights from here.
    pub tfm: Option<Rc<Tfm>>,
    /// Why no TFM is attached (reported once by the typesetter).
    pub tfm_missing: Option<String>,
    bounds_cache: RefCell<BTreeMap<u16, Bounds>>,
}

impl LoadedFace {
    pub fn face(&self) -> &dyn Face {
        match &self.kind {
            FaceKind::Otf { face, .. } => face,
            FaceKind::Core14(f) => f,
        }
    }

    pub fn otf(&self) -> Option<&TrueTypeFace> {
        match &self.kind {
            FaceKind::Otf { face, .. } => Some(face),
            FaceKind::Core14(_) => None,
        }
    }

    pub fn program(&self) -> Option<&[u8]> {
        self.otf().map(TrueTypeFace::program)
    }

    pub fn math(&self) -> Option<&MathTable> {
        self.otf().and_then(TrueTypeFace::math)
    }

    /// Font-unit value in points at `size_pt`.
    pub fn pt(&self, units: i64, size_pt: f64) -> f64 {
        units as f64 * size_pt / f64::from(self.units_per_em)
    }

    /// paragraph-layout's opaque 32-byte identity: the content hash itself.
    pub fn layout_id(&self) -> flashtex_paragraph_layout::FontId {
        flashtex_paragraph_layout::FontId(self.sha256)
    }

    /// Glyph extents in font units. CFF faces use the real charstring
    /// bounds; Core 14 faces (no outlines available) use class-based
    /// approximations from the AFM header, stated in README.
    pub fn bounds(&self, gid: GlyphId, ch: Option<char>) -> Bounds {
        if let Some(b) = self.bounds_cache.borrow().get(&gid.0) {
            return *b;
        }
        let b = match &self.kind {
            FaceKind::Otf { face, cff } => {
                let bb = face
                    .cff_table()
                    .and_then(|t| cff.glyph_bbox(t, gid.0).ok())
                    .and_then(|(bb, _)| cff::round_bbox(bb));
                match bb {
                    Some([x0, y0, x1, y1]) => Bounds {
                        x_min: x0,
                        y_min: y0,
                        x_max: x1,
                        y_max: y1,
                        empty: false,
                    },
                    None => Bounds {
                        empty: true,
                        ..Bounds::default()
                    },
                }
            }
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
                    Some(' ') => (0, 0),
                    _ => (i32::from(h.descender), i32::from(h.ascender)),
                };
                Bounds {
                    x_min: 0,
                    y_min,
                    x_max: adv,
                    y_max,
                    empty: ch == Some(' '),
                }
            }
        };
        self.bounds_cache.borrow_mut().insert(gid.0, b);
        b
    }
}

pub struct FontSet {
    search: FontSearch,
    faces: RefCell<Vec<Rc<LoadedFace>>>,
    by_name: RefCell<BTreeMap<String, usize>>,
    /// File names that failed to load, with the reason (reported once).
    failures: RefCell<BTreeMap<String, String>>,
    tfm_dirs: Vec<PathBuf>,
}

pub struct Resolved {
    pub face: Rc<LoadedFace>,
    /// Set when the requested Latin Modern file was unavailable. The face
    /// returned is then Times, and the caller must publish this reason as
    /// an error diagnostic: the output is not the requested document.
    pub substituted: Option<String>,
}

impl FontSet {
    /// Bounded search list: `FLASHTEX_FONT_DIRS` entries first, then any
    /// explicit extra directories, then [`default_font_dirs`].
    pub fn with_default_dirs(extra: &[PathBuf]) -> FontSet {
        let mut dirs: Vec<PathBuf> = Vec::new();
        if let Ok(v) = std::env::var("FLASHTEX_FONT_DIRS") {
            dirs.extend(v.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
        }
        dirs.extend(extra.iter().cloned());
        dirs.extend(default_font_dirs());
        FontSet::new(dirs)
    }

    /// Whether the Latin Modern text and math faces the tests and the
    /// default document need are reachable through this set's directories.
    pub fn latin_modern_available(&self) -> bool {
        let has = |file: &str| self.dirs().iter().any(|d| d.join(file).is_file());
        has("lmroman12-regular.otf") && has("lmroman10-regular.otf") && has("latinmodern-math.otf")
    }

    pub fn new(dirs: Vec<PathBuf>) -> FontSet {
        let mut search = FontSearch::new();
        let mut tfm_dirs: Vec<PathBuf> = Vec::new();
        if let Ok(v) = std::env::var("FLASHTEX_TFM_DIRS") {
            tfm_dirs.extend(v.split(':').filter(|s| !s.is_empty()).map(PathBuf::from));
        }
        for d in dirs {
            let sibling = PathBuf::from(d.to_string_lossy().replace("/opentype/", "/tfm/"));
            for p in [sibling, d.clone()] {
                if !tfm_dirs.contains(&p) {
                    tfm_dirs.push(p);
                }
            }
            search = search.with_dir(d);
        }
        FontSet {
            search,
            faces: RefCell::new(Vec::new()),
            by_name: RefCell::new(BTreeMap::new()),
            failures: RefCell::new(BTreeMap::new()),
            tfm_dirs,
        }
    }

    pub fn dirs(&self) -> &[PathBuf] {
        self.search.dirs()
    }

    /// Where `.tfm` files are looked for (see [`default_tfm_dirs`]).
    pub fn tfm_dirs(&self) -> &[PathBuf] {
        &self.tfm_dirs
    }

    /// Every face loaded so far, in load order.
    pub fn loaded(&self) -> Vec<Rc<LoadedFace>> {
        self.faces.borrow().clone()
    }

    pub fn failures(&self) -> Vec<(String, String)> {
        self.failures.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn by_name(&self, name: &str) -> Option<Rc<LoadedFace>> {
        let idx = *self.by_name.borrow().get(name)?;
        self.faces.borrow().get(idx).cloned()
    }

    pub fn by_font_id(&self, font_id: &str) -> Option<Rc<LoadedFace>> {
        self.faces.borrow().iter().find(|f| f.font_id == font_id).cloned()
    }

    /// Latin Modern optical-size file for a role, per `t1lmr.fd` (the
    /// design-size boundaries LaTeX uses for `ec-lmr*`).
    pub fn latin_modern_file(role: Role, size_pt: f64) -> String {
        let s = size_pt;
        match role {
            Role::Math => "latinmodern-math.otf".to_string(),
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
        }
    }

    fn core14_for(role: Role) -> Core14 {
        match role {
            Role::Math => Core14::Symbol,
            Role::Text { bold: false, italic: false } => Core14::TimesRoman,
            Role::Text { bold: true, italic: false } => Core14::TimesBold,
            Role::Text { bold: false, italic: true } => Core14::TimesItalic,
            Role::Text { bold: true, italic: true } => Core14::TimesBoldItalic,
        }
    }

    /// Resolves (and loads once) the face for `family`/`role` at `size_pt`.
    pub fn resolve(&self, family: Family, role: Role, size_pt: f64) -> Resolved {
        match (family, role) {
            (Family::Times, Role::Text { .. }) => Resolved {
                face: self.core14(Self::core14_for(role)),
                substituted: None,
            },
            (_, _) => {
                let file = Self::latin_modern_file(role, size_pt);
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
        let name = f.postscript_name().to_string();
        if let Some(existing) = self.by_name(&name) {
            return existing;
        }
        let sha = f.id().content_sha256;
        let loaded = LoadedFace {
            font_id: sha256::hex(&sha),
            name: name.clone(),
            sha256: sha,
            byte_length: 0,
            units_per_em: u32::from(f.units_per_em()),
            glyph_count: u32::from(f.num_glyphs()),
            postscript_name: f.postscript_name().to_string(),
            format: "core14-afm",
            path: None,
            kind: FaceKind::Core14(f),
            tfm: None,
            tfm_missing: None,
            bounds_cache: RefCell::new(BTreeMap::new()),
        };
        self.insert(name, loaded)
    }

    /// Loads an explicit file name from the bounded search list.
    pub fn otf(&self, file: &str) -> Result<Rc<LoadedFace>, String> {
        let name = file.trim_end_matches(".otf").trim_end_matches(".ttf").to_string();
        if let Some(existing) = self.by_name(&name) {
            return Ok(existing);
        }
        if let Some(reason) = self.failures.borrow().get(file) {
            return Err(reason.clone());
        }
        let fail = |reason: String| -> String {
            self.failures.borrow_mut().insert(file.to_string(), reason.clone());
            reason
        };
        let Some(path) = self.search.find(file) else {
            let n = self.search.dirs().len();
            return Err(fail(format!(
                "not found in {n} search director{}: {}",
                if n == 1 { "y" } else { "ies" },
                self.search.dirs().iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(", ")
            )));
        };
        let face = match self.search.load(file, 0) {
            Ok(f) => f,
            Err(e) => return Err(fail(e.to_string())),
        };
        let (format, cff) = match face.outlines() {
            Outlines::Cff => {
                let table = face.cff_table().ok_or_else(|| fail("OTTO face without CFF table".into()))?;
                let cff = Cff::parse(table).map_err(|e| fail(format!("CFF: {e}")))?;
                if cff.num_glyphs() != usize::from(face.num_glyphs()) {
                    return Err(fail(format!(
                        "CFF has {} charstrings but maxp says {}",
                        cff.num_glyphs(),
                        face.num_glyphs()
                    )));
                }
                ("opentype-cff", cff)
            }
            Outlines::Glyf => {
                return Err(fail("glyf outlines are not used by this pipeline (Latin Modern is CFF)".into()));
            }
        };
        let sha = face.id().content_sha256;
        let (tfm, tfm_missing) = match latin_modern_tfm(&name) {
            Some(tfm_file) => match self.tfm_dirs.iter().map(|d| d.join(&tfm_file)).find(|p| p.is_file()) {
                Some(p) => match Tfm::load(&p) {
                    Ok(t) if t.has_boundary() => (None, Some(format!("{tfm_file} declares a boundary character program, which this reader does not run"))),
                    Ok(t) => (Some(Rc::new(t)), None),
                    Err(e) => (None, Some(e)),
                },
                None => (
                    None,
                    Some(format!(
                        "{tfm_file} not found in {}",
                        self.tfm_dirs.iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(", ")
                    )),
                ),
            },
            None => (None, None),
        };
        let loaded = LoadedFace {
            font_id: sha256::hex(&sha),
            name: name.clone(),
            sha256: sha,
            byte_length: face.program().len() as u64,
            units_per_em: u32::from(face.units_per_em()),
            glyph_count: u32::from(face.num_glyphs()),
            postscript_name: face.postscript_name().to_string(),
            format,
            path: Some(path),
            kind: FaceKind::Otf { face, cff },
            tfm,
            tfm_missing,
            bounds_cache: RefCell::new(BTreeMap::new()),
        };
        Ok(self.insert(name, loaded))
    }

    fn insert(&self, name: String, loaded: LoadedFace) -> Rc<LoadedFace> {
        let rc = Rc::new(loaded);
        let mut faces = self.faces.borrow_mut();
        self.by_name.borrow_mut().insert(name, faces.len());
        faces.push(rc.clone());
        rc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn lm_available() -> bool {
        FontSet::with_default_dirs(&[]).latin_modern_available()
    }

    #[test]
    fn optical_sizes_follow_t1lmr_fd() {
        assert_eq!(FontSet::latin_modern_file(Role::Text { bold: false, italic: false }, 12.0), "lmroman12-regular.otf");
        assert_eq!(FontSet::latin_modern_file(Role::Text { bold: false, italic: false }, 10.0), "lmroman10-regular.otf");
        assert_eq!(FontSet::latin_modern_file(Role::Text { bold: false, italic: false }, 17.28), "lmroman17-regular.otf");
        assert_eq!(FontSet::latin_modern_file(Role::Text { bold: true, italic: false }, 14.4), "lmroman12-bold.otf");
        assert_eq!(FontSet::latin_modern_file(Role::Text { bold: false, italic: false }, 8.0), "lmroman8-regular.otf");
    }

    #[test]
    fn latin_modern_glyph_bounds_match_the_design() {
        if !lm_available() {
            eprintln!("skipping: Latin Modern not installed");
            return;
        }
        let set = FontSet::with_default_dirs(&[]);
        let r = set.resolve(Family::LatinModern, Role::Text { bold: false, italic: false }, 10.0);
        assert!(r.substituted.is_none());
        let f = r.face;
        assert_eq!(f.format, "opentype-cff");
        assert_eq!(f.units_per_em, 1000);
        let x = f.face().glyph_id('x').unwrap();
        let b = f.bounds(x, Some('x'));
        // Computer Modern x-height is 430.55 units (lmr10 TFM fontdimen 5).
        assert!((b.y_max - 431).abs() <= 2, "x y_max {}", b.y_max);
        assert_eq!(b.y_min, 0);
        let h = f.face().glyph_id('H').unwrap();
        let hb = f.bounds(h, Some('H'));
        // Cap height 683.33 units in lmr10.
        assert!((hb.y_max - 683).abs() <= 2, "H y_max {}", hb.y_max);
        let p = f.face().glyph_id('p').unwrap();
        let pb = f.bounds(p, Some('p'));
        // Descender depth 194.44 units.
        assert!((pb.y_min + 194).abs() <= 2, "p y_min {}", pb.y_min);
        let sp = f.face().glyph_id(' ').unwrap();
        assert!(f.bounds(sp, Some(' ')).empty);
        // Ids are content hashes, distinct per file.
        let b12 = set.resolve(Family::LatinModern, Role::Text { bold: false, italic: false }, 12.0).face;
        assert_ne!(f.font_id, b12.font_id);
        assert_eq!(f.font_id.len(), 64);
    }

    #[test]
    fn missing_latin_modern_is_reported_not_silent() {
        let set = FontSet::new(vec![PathBuf::from("/nonexistent/flashtex-fonts")]);
        let r = set.resolve(Family::LatinModern, Role::Text { bold: false, italic: false }, 10.0);
        assert!(r.substituted.is_some());
        assert_eq!(r.face.format, "core14-afm");
        assert_eq!(set.failures().len(), 1);
    }
}
