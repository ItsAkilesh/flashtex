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
use std::path::{Path, PathBuf};
use std::rc::Rc;

use flashtex_font_engine::core14::{Core14, Core14Face};
use flashtex_font_engine::math::MathTable;
use flashtex_font_engine::resolve::FontSearch;
use flashtex_font_engine::truetype::{Outlines, TrueTypeFace};
use flashtex_font_engine::{sha256, Face};

use flashtex_font_resources::required_tfm::{Manifest as RequiredManifest, MetricAsset, RequiredMetrics};
use flashtex_project_files::ProjectRoot;

use crate::cff::{self, Cff};
use crate::tfm::Tfm;

/// The 12 pt metric set pdfLaTeX+`lmodern` lays the reference documents
/// out with, pinned to the official Latin Modern 2.004 release
/// (font-resources `fixtures/lm-required-metrics-provenance.json`):
/// `ec-lmr12` for text and `rm-lmr12/8/6` for the math roman family, plus
/// the GUST font licence next to them. They are read through
/// font-resources' rooted, digest-bound `RequiredMetrics::load` from the
/// TeX Live `texmf-dist` tree; a missing or mismatched file is a blocking
/// diagnostic, never a silent switch to OpenType metrics.
pub const REQUIRED_TFMS: [(&str, &str); 4] = [
    ("ec-lmr12.tfm", "299021120f0a29ef61278a2363903bd8defbb8faaade458eb79067342aecb56f"),
    ("rm-lmr12.tfm", "9d4e3d8e39a41b93d91f79c1c47d2297efb7b1af220b94860693c08361f227aa"),
    ("rm-lmr8.tfm", "80bcbfd844d2310ac1d3bead45aee25e91b1a4a0a60ff1771959b9a1e90ec1a2"),
    ("rm-lmr6.tfm", "eb0bfdf8db3ae1409639fac9c88f84923872500d882d9ff8dc37aff445c723fe"),
];
pub const REQUIRED_TFM_DIR: &str = "fonts/tfm/public/lm";
pub const REQUIRED_LICENSE: (&str, &str) = (
    "doc/fonts/lm/GUST-FONT-LICENSE.TXT",
    "49ea6cb9257bbee0a3979c48a774cd221550ac1c20c95549efe45fc99cc18050",
);

/// The two directory layouts the required set is accepted in, both read
/// through the rooted, digest-bound loader:
///
/// * **texmf** — a TeX Live style tree rooted at `<root>`:
///   `<root>/fonts/tfm/public/lm/{ec-lmr12,rm-lmr12,rm-lmr8,rm-lmr6}.tfm`
///   and `<root>/doc/fonts/lm/GUST-FONT-LICENSE.TXT` (MacTeX:
///   `/usr/local/texlive/2026/texmf-dist`; an app bundle can ship
///   `Contents/Resources/texmf/...` and point `FLASHTEX_FONT_DIRS` /
///   `FLASHTEX_TFM_DIRS` at `<root>/fonts/{opentype,tfm}/public/lm`, or
///   rely on the `../Resources/texmf` default below).
/// * **flat** — every TFM directory itself as the root, the four TFMs and
///   `GUST-FONT-LICENSE.TXT` directly inside it (a bundle's
///   `Contents/Resources/Fonts` next to the OTFs, or `FLASHTEX_TFM_DIRS`).
fn required_manifest(flat: bool) -> RequiredManifest {
    let (prefix, license) = if flat {
        (String::new(), "GUST-FONT-LICENSE.TXT".to_string())
    } else {
        (format!("{REQUIRED_TFM_DIR}/"), REQUIRED_LICENSE.0.to_string())
    };
    RequiredManifest {
        schema_version: 1,
        metrics: REQUIRED_TFMS
            .iter()
            .map(|(file, sha)| MetricAsset {
                path: format!("{prefix}{file}"),
                sha256: (*sha).to_string(),
                license_path: license.clone(),
                license_sha256: REQUIRED_LICENSE.1.to_string(),
            })
            .collect(),
    }
}

/// Why a TFM is not attached to a face.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TfmStatus {
    /// Loaded (digest-bound for the required set, parsed for the others).
    Loaded,
    /// A required 12 pt asset could not be loaded: blocking.
    RequiredUnavailable(String),
    /// A non-required TFM was not found or did not parse: the face uses
    /// its OpenType metrics and the typesetter warns.
    Missing(String),
}
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

/// What font discovery reads from the process: the three override
/// variables and where the executable lives. [`Discovery::from_process`]
/// samples the real process; tests build one by hand so the bundle-relative
/// rules are checked without a host TeX installation and without touching
/// the environment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Discovery {
    /// `FLASHTEX_FONT_DIRS` (colon separated).
    pub font_dirs: Option<String>,
    /// `FLASHTEX_LM_DIR` (the pdf sibling's variable).
    pub lm_dir: Option<String>,
    /// `FLASHTEX_TFM_DIRS` (colon separated).
    pub tfm_dirs: Option<String>,
    /// The directory holding the executable (`Contents/MacOS` in an app
    /// bundle); `None` when the process cannot tell.
    pub exe_dir: Option<PathBuf>,
}

impl Discovery {
    pub fn from_process() -> Discovery {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Discovery {
            font_dirs: var("FLASHTEX_FONT_DIRS"),
            lm_dir: var("FLASHTEX_LM_DIR"),
            tfm_dirs: var("FLASHTEX_TFM_DIRS"),
            exe_dir: std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)),
        }
    }

    fn split(v: &Option<String>) -> Vec<PathBuf> {
        v.iter().flat_map(|v| v.split(':')).filter(|s| !s.is_empty()).map(PathBuf::from).collect()
    }

    /// The texmf trees an app bundle or a sibling directory can ship,
    /// relative to the executable: `<exe>/../Resources/texmf` (the bundle's
    /// `Contents/Resources/texmf`, sealed with the app) then `<exe>/texmf`.
    /// Under each: `fonts/opentype/public/{lm,lm-math}`,
    /// `fonts/tfm/public/lm` and `doc/fonts/lm/GUST-FONT-LICENSE.TXT`.
    pub fn bundle_texmf_roots(&self) -> Vec<PathBuf> {
        self.exe_dir.iter().flat_map(|d| [d.join("../Resources/texmf"), d.join("texmf")]).collect()
    }

    /// Font directories, in order: `FLASHTEX_FONT_DIRS`, `FLASHTEX_LM_DIR`,
    /// the bundled texmf trees' OpenType directories, a flat `Fonts`
    /// directory next to the executable or in the bundle's `Resources`,
    /// then [`DEFAULT_FONT_DIRS`] (host TeX). Nothing is scanned outside
    /// this list; explicit overrides always come first.
    pub fn font_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Discovery::split(&self.font_dirs);
        dirs.extend(self.lm_dir.iter().map(PathBuf::from));
        for root in self.bundle_texmf_roots() {
            dirs.push(root.join("fonts/opentype/public/lm"));
            dirs.push(root.join("fonts/opentype/public/lm-math"));
        }
        if let Some(dir) = &self.exe_dir {
            dirs.push(dir.join("Fonts"));
            dirs.push(dir.join("../Resources/Fonts"));
        }
        dirs.extend(DEFAULT_FONT_DIRS.iter().map(PathBuf::from));
        dirs
    }

    /// TFM directories, in order: `FLASHTEX_TFM_DIRS`, the bundled texmf
    /// trees' `fonts/tfm/public/lm`, then for every font directory its
    /// `/opentype/` → `/tfm/` sibling (the TeX Live layout) and the
    /// directory itself (the flat layout). Duplicates are dropped, first
    /// occurrence wins, so an explicit override keeps precedence over the
    /// same path discovered later.
    pub fn tfm_dirs_for(&self, font_dirs: &[PathBuf]) -> Vec<PathBuf> {
        let mut dirs = Discovery::split(&self.tfm_dirs);
        let mut push = |p: PathBuf| {
            if !dirs.contains(&p) {
                dirs.push(p);
            }
        };
        for root in self.bundle_texmf_roots() {
            push(root.join(REQUIRED_TFM_DIR));
        }
        for d in font_dirs {
            push(PathBuf::from(d.to_string_lossy().replace("/opentype/", "/tfm/")));
            push(d.clone());
        }
        dirs
    }
}

/// [`Discovery::font_dirs`] for the running process.
pub fn default_font_dirs() -> Vec<PathBuf> {
    Discovery::from_process().font_dirs()
}

/// [`Discovery::tfm_dirs_for`] over [`default_font_dirs`] for the running
/// process.
pub fn default_tfm_dirs() -> Vec<PathBuf> {
    let d = Discovery::from_process();
    d.tfm_dirs_for(&d.font_dirs())
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
    /// Content-addressed id used on the wire: the SHA-256 hex of the RAW
    /// font file bytes (what `fonts[].sha256` of rendering-v2 and
    /// font-resources' digest checks mean). font-engine's own
    /// `FontId::content_sha256` hashes bytes ‖ face_index and is kept in
    /// `engine_id` for diagnostics only.
    pub font_id: String,
    /// font-engine's identity (SHA-256 of bytes ‖ big-endian face index).
    pub engine_id: String,
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
    pub tfm_status: TfmStatus,
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
    /// The required 12 pt set, loaded once on first use.
    required: RefCell<Option<Result<Rc<RequiredMetrics>, String>>>,
    /// Whether the required set was found in the flat layout.
    required_flat: RefCell<bool>,
    /// Non-required TFMs parsed so far, by file name.
    tfms: RefCell<BTreeMap<String, Result<Rc<Tfm>, String>>>,
    /// Shaped words, keyed by (face, text); shaping is size-independent and
    /// a keystroke changes one word, so this outlives requests. Bounded.
    shaper: crate::shape::Shaper,
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
        let d = Discovery::from_process();
        let mut dirs = Discovery::split(&d.font_dirs);
        dirs.extend(extra.iter().cloned());
        dirs.extend(d.font_dirs());
        let tfm_dirs = d.tfm_dirs_for(&dirs);
        FontSet::with_dirs(dirs, tfm_dirs)
    }

    /// Everything [`Discovery`] finds, and nothing else: the set the
    /// packaged helper runs with.
    pub fn from_discovery(d: &Discovery) -> FontSet {
        let dirs = d.font_dirs();
        let tfm_dirs = d.tfm_dirs_for(&dirs);
        FontSet::with_dirs(dirs, tfm_dirs)
    }

    /// Whether the Latin Modern text and math faces the tests and the
    /// default document need are reachable through this set's directories.
    pub fn latin_modern_available(&self) -> bool {
        let has = |file: &str| self.dirs().iter().any(|d| d.join(file).is_file());
        has("lmroman12-regular.otf") && has("lmroman10-regular.otf") && has("latinmodern-math.otf")
    }

    /// Explicit font directories; TFMs come from `FLASHTEX_TFM_DIRS` and
    /// the directories' TeX Live / flat siblings (no bundle probing).
    pub fn new(dirs: Vec<PathBuf>) -> FontSet {
        let d = Discovery { tfm_dirs: std::env::var("FLASHTEX_TFM_DIRS").ok(), ..Discovery::default() };
        let tfm_dirs = d.tfm_dirs_for(&dirs);
        FontSet::with_dirs(dirs, tfm_dirs)
    }

    /// Explicit font and TFM directories, both searched in the given order.
    pub fn with_dirs(dirs: Vec<PathBuf>, tfm_dirs: Vec<PathBuf>) -> FontSet {
        let mut search = FontSearch::new();
        for d in dirs {
            search = search.with_dir(d);
        }
        FontSet {
            search,
            faces: RefCell::new(Vec::new()),
            by_name: RefCell::new(BTreeMap::new()),
            failures: RefCell::new(BTreeMap::new()),
            tfm_dirs,
            required: RefCell::new(None),
            required_flat: RefCell::new(false),
            tfms: RefCell::new(BTreeMap::new()),
            shaper: crate::shape::Shaper::new(),
        }
    }

    /// The shaping cache shared by every request on this font set.
    pub fn shaper(&self) -> &crate::shape::Shaper {
        &self.shaper
    }

    /// The `texmf-dist` roots implied by the TFM directories
    /// (`<root>/fonts/tfm/public/lm`).
    fn texmf_roots(&self) -> Vec<PathBuf> {
        self.tfm_dirs
            .iter()
            .filter_map(|d| {
                let s = d.to_string_lossy();
                s.strip_suffix(&format!("/{REQUIRED_TFM_DIR}")).map(PathBuf::from)
            })
            .collect()
    }

    /// The required 12 pt metrics (see [`REQUIRED_TFMS`]), loaded through
    /// font-resources from the first `texmf-dist` root that satisfies the
    /// whole manifest. `Err` names what failed.
    pub fn required_metrics(&self) -> Result<Rc<RequiredMetrics>, String> {
        if let Some(r) = &*self.required.borrow() {
            return r.clone();
        }
        // texmf roots first (TeX Live / bundled tree), then every TFM
        // directory as a flat root.
        let candidates: Vec<(PathBuf, bool)> = self
            .texmf_roots()
            .into_iter()
            .map(|r| (r, false))
            .chain(self.tfm_dirs.iter().filter(|d| d.is_dir()).map(|d| (d.clone(), true)))
            .collect();
        let mut errors = Vec::new();
        let mut result = Err(String::new());
        for (root, flat) in &candidates {
            match ProjectRoot::open(root) {
                Ok(pr) => match RequiredMetrics::load(&pr, &required_manifest(*flat)) {
                    Ok(m) => {
                        result = Ok(Rc::new(m));
                        *self.required_flat.borrow_mut() = *flat;
                        break;
                    }
                    Err(e) => errors.push(format!("{}{}: {e:?}", root.display(), if *flat { " (flat)" } else { "" })),
                },
                Err(e) => errors.push(format!("{}: {e:?}", root.display())),
            }
        }
        if result.is_err() {
            result = Err(if candidates.is_empty() {
                format!(
                    "no TFM directory exists among ({})",
                    self.tfm_dirs.iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(", ")
                )
            } else {
                errors.join("; ")
            });
        }
        *self.required.borrow_mut() = Some(result.clone());
        result
    }

    /// A TFM by file name: the digest-bound required set when it holds
    /// the file, else the search directories through the shared parser.
    pub fn tfm(&self, file: &str) -> Result<Rc<Tfm>, TfmStatus> {
        if REQUIRED_TFMS.iter().any(|(f, _)| *f == file) {
            let set = self.required_metrics().map_err(TfmStatus::RequiredUnavailable)?;
            let key = if *self.required_flat.borrow() { file.to_string() } else { format!("{REQUIRED_TFM_DIR}/{file}") };
            let (_, t) = set.get(&key).map_err(|e| TfmStatus::RequiredUnavailable(format!("{e:?}")))?;
            return Ok(Rc::new(Tfm::from_shared(t.clone())));
        }
        if let Some(r) = self.tfms.borrow().get(file) {
            return r.clone().map_err(TfmStatus::Missing);
        }
        let r = match self.tfm_dirs.iter().map(|d| d.join(file)).find(|p| p.is_file()) {
            Some(p) => Tfm::load(&p).map(Rc::new),
            None => Err(format!(
                "{file} not found in {}",
                self.tfm_dirs.iter().map(|d| d.display().to_string()).collect::<Vec<_>>().join(", ")
            )),
        };
        self.tfms.borrow_mut().insert(file.to_string(), r.clone());
        r.map_err(TfmStatus::Missing)
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
        // Look the face up before constructing it: `Core14Face::new` hashes
        // the metrics (SHA-256) and `resolve` runs once per word.
        if let Some(existing) = self.by_name(which.header().font_name) {
            return existing;
        }
        let f = Core14Face::new(which);
        let name = f.postscript_name().to_string();
        if let Some(existing) = self.by_name(&name) {
            return existing;
        }
        let sha = f.id().content_sha256;
        let loaded = LoadedFace {
            font_id: sha256::hex(&sha),
            engine_id: sha256::hex(&sha),
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
            tfm_status: TfmStatus::Missing("Core 14 face: AFM metrics".into()),
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
        // The published digest is over the raw file bytes; the engine's id
        // (bytes ‖ face index) is a different value and is not the resource
        // digest rendering-core / font-resources verify.
        let sha = sha256::digest(face.program());
        let engine_id = sha256::hex(&face.id().content_sha256);
        let (tfm, tfm_missing, tfm_status) = match latin_modern_tfm(&name) {
            Some(tfm_file) => match self.tfm(&tfm_file) {
                Ok(t) => (Some(t), None, TfmStatus::Loaded),
                Err(TfmStatus::RequiredUnavailable(e)) => {
                    let msg = format!("required metric asset {tfm_file}: {e}");
                    (None, Some(msg.clone()), TfmStatus::RequiredUnavailable(msg))
                }
                Err(TfmStatus::Missing(e)) => (None, Some(e.clone()), TfmStatus::Missing(e)),
                Err(TfmStatus::Loaded) => unreachable!(),
            },
            None => (None, None, TfmStatus::Missing("no TFM pairs with this file".into())),
        };
        let loaded = LoadedFace {
            font_id: sha256::hex(&sha),
            engine_id,
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
            tfm_status,
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
