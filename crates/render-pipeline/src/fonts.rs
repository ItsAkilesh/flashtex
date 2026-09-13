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
    /// LaTeX's default `cmr` under `\usepackage[T1]{fontenc}` (no
    /// `lmodern`): the EC metrics `t1cmr.fd` loads (`ecrm1095`, `ecbx1200`,
    /// ...) laid out with the Latin Modern outlines, which draw the same
    /// Computer Modern designs. Math is unchanged (Latin Modern Math).
    ComputerModern,
}

/// Which typographic role a face plays; selects the design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Role {
    /// Text face (roman/bold/italic per the style).
    Text { bold: bool, italic: bool },
    /// Upright-medium slanted text (`\slshape`, running heads): Latin Modern
    /// `lmromanslant*` with `ec-lmro*` metrics.
    Slanted,
    /// Typewriter text (`\ttfamily`: verbatim, `\verb`, listings): Latin
    /// Modern Mono outlines laid out with the metrics [`MonoMetrics`]
    /// selects ([`FontSet::resolve_mono`]).
    Mono { bold: bool },
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
        // The OT1 `cmtt` metrics of typewriter text (`MonoMetrics::CmOt1`),
        // after every Latin Modern candidate and before the EC directories.
        for root in self.bundle_texmf_roots() {
            push(root.join(CM_TFM_DIR));
        }
        for d in font_dirs {
            let d = d.to_string_lossy();
            if let Some(at) = d.find("/fonts/opentype/public/lm") {
                push(PathBuf::from(format!("{}/{CM_TFM_DIR}", &d[..at])));
            }
        }
        // The EC metrics of T1 `cmr` documents (`Family::ComputerModern`):
        // the bundled trees' and each TeX Live tree's `fonts/tfm/jknappen/ec`,
        // after every Latin Modern candidate so their order is unchanged.
        for root in self.bundle_texmf_roots() {
            push(root.join(EC_TFM_DIR));
        }
        for d in font_dirs {
            let d = d.to_string_lossy();
            if let Some(at) = d.find("/fonts/opentype/public/lm") {
                push(PathBuf::from(format!("{}/{EC_TFM_DIR}", &d[..at])));
            }
        }
        dirs
    }
}

/// How diagnostics spell the executable's directory.
pub const EXE_DIR_LABEL: &str = "<executable-dir>";

/// A comma-separated directory list for diagnostics. Directories under
/// `exe_dir` (the bundle and sibling trees [`Discovery`] derives from the
/// executable) are written relative to [`EXE_DIR_LABEL`], so identical
/// documents produce identical output wherever the binary is installed.
/// Explicit and host directories are configuration and stay as given.
pub fn describe_dirs(dirs: &[PathBuf], exe_dir: Option<&Path>) -> String {
    dirs.iter()
        .map(|d| match exe_dir.and_then(|e| d.strip_prefix(e).ok()) {
            Some(rel) if rel.as_os_str().is_empty() => EXE_DIR_LABEL.to_string(),
            Some(rel) => format!("{EXE_DIR_LABEL}/{}", rel.display()),
            None => d.display().to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
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
    if let Some(rest) = otf_stem.strip_prefix("lmromanslant") {
        let d: u32 = rest.strip_suffix("-regular")?.parse().ok()?;
        return Some(format!("ec-lmro{d}.tfm"));
    }
    if otf_stem == "lmmonolt10-bold" {
        return Some("ec-lmtk10.tfm".to_string());
    }
    if let Some(rest) = otf_stem.strip_prefix("lmmono") {
        let d: u32 = rest.strip_suffix("-regular")?.parse().ok()?;
        return Some(format!("ec-lmtt{d}.tfm"));
    }
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

/// Where TeX Live keeps the EC metrics (`jknappen/ec`), relative to a
/// texmf root.
pub const EC_TFM_DIR: &str = "fonts/tfm/jknappen/ec";

/// Where TeX Live keeps the Computer Modern metrics (`cmtt10`, ...),
/// relative to a texmf root.
pub const CM_TFM_DIR: &str = "fonts/tfm/public/cm";

/// Which metrics LaTeX lays `\ttfamily` out with. `\ttdefault` is `cmtt`
/// unless a package changes it (`lmodern`: `lmtt`; `times`: `pcr`, which
/// [`FontSet::resolve_mono`] sets in Courier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MonoMetrics {
    /// OT1 `cmtt` (ot1cmtt.fd): `cmtt8` for 5-8pt, `cmtt9`, `cmtt10` for
    /// 10/10.95pt, `cmtt12` from 12pt; `bx` is `ssub` to medium.
    #[default]
    CmOt1,
    /// T1 `cmtt` (t1cmtt.fd): `ectt0800` for 5-8pt, else `ectt<size>` at
    /// the EC size nearest; `bx` substitutes medium.
    EcT1,
    /// `lmodern` (t1lmtt.fd, ot1lmtt.fd): `ec-lmtt8/9/10/12`, `b`/`bx` the
    /// dark `ec-lmtk10`.
    LatinModern,
}

/// t1lmtt.fd `m/n` design sizes: <-8.5> 8, <8.5-9.5> 9, <9.5-11> 10, <11-> 12.
fn mono_design(size_pt: f64) -> u32 {
    if size_pt < 8.5 {
        8
    } else if size_pt < 9.5 {
        9
    } else if size_pt < 11.0 {
        10
    } else {
        12
    }
}

/// The TFM LaTeX loads for typewriter text at `size_pt` (see [`MonoMetrics`]).
pub fn mono_tfm_file(metrics: MonoMetrics, bold: bool, size_pt: f64) -> String {
    match metrics {
        MonoMetrics::LatinModern if bold => "ec-lmtk10.tfm".to_string(),
        MonoMetrics::LatinModern => format!("ec-lmtt{}.tfm", mono_design(size_pt)),
        MonoMetrics::CmOt1 => {
            // `<5><6><7><8>cmtt8<9>cmtt9<10><10.95>cmtt10<12>...cmtt12`; an
            // undeclared size takes the nearest declared one.
            let d = if size_pt < 8.5 {
                8
            } else if size_pt < 9.5 {
                9
            } else if size_pt < 11.475 {
                10
            } else {
                12
            };
            format!("cmtt{d}.tfm")
        }
        MonoMetrics::EcT1 => {
            if size_pt < 8.5 {
                return "ectt0800.tfm".to_string();
            }
            let (_, suffix) = EC_SIZES
                .iter()
                .min_by(|a, b| (a.0 - size_pt).abs().total_cmp(&(b.0 - size_pt).abs()))
                .expect("EC sizes");
            format!("ectt{suffix}.tfm")
        }
    }
}

/// The sizes `t1cmr.fd` declares for every EC shape
/// (`<5><6><7><8><9><10><10.95><12><14.4><17.28><20.74><24.88><29.86><35.83>genb*ecrm`)
/// and the file-name suffix `genb*` builds from each.
const EC_SIZES: [(f64, &str); 14] = [
    (5.0, "0500"),
    (6.0, "0600"),
    (7.0, "0700"),
    (8.0, "0800"),
    (9.0, "0900"),
    (10.0, "1000"),
    (10.95, "1095"),
    (12.0, "1200"),
    (14.4, "1440"),
    (17.28, "1728"),
    (20.74, "2074"),
    (24.88, "2488"),
    (29.86, "2986"),
    (35.83, "3583"),
];

/// The EC metric file `t1cmr.fd` loads for a text role at `size_pt`:
/// `m/n` `ecrm`, `bx/n` `ecbx`, `m/it` `ecti`, `bx/it` `ecbi`, `m/sl`
/// `ecsl`, at the declared size nearest `size_pt` (an undeclared size is a
/// LaTeX size substitution to the nearest one). `None` for math.
pub fn ec_tfm_file(role: Role, size_pt: f64) -> Option<String> {
    let prefix = match role {
        Role::Math | Role::Mono { .. } => return None,
        Role::Text { bold: false, italic: false } => "ecrm",
        Role::Text { bold: true, italic: false } => "ecbx",
        Role::Text { bold: false, italic: true } => "ecti",
        Role::Text { bold: true, italic: true } => "ecbi",
        Role::Slanted => "ecsl",
    };
    let (_, suffix) = EC_SIZES
        .iter()
        .min_by(|a, b| (a.0 - size_pt).abs().total_cmp(&(b.0 - size_pt).abs()))?;
    Some(format!("{prefix}{suffix}.tfm"))
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
    pub font_id: Rc<str>,
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
    /// Shaping-cache identity: `font_id` for the default metrics, extended
    /// with the TFM for a face laid out with EC metrics (the same program
    /// then shapes differently).
    pub shape_key: Rc<str>,
    /// Set when EC metrics were requested but unavailable and the Latin
    /// Modern (`ec-lm*`) TFM was attached instead (reported once).
    pub metrics_fallback: Option<String>,
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
    /// The executable's directory when the search list was derived from
    /// it; diagnostics show directories under it relative to
    /// [`EXE_DIR_LABEL`] so output never depends on the install location.
    exe_dir: Option<PathBuf>,
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
        FontSet::with_dirs(dirs, tfm_dirs).with_exe_dir(d.exe_dir)
    }

    /// Everything [`Discovery`] finds, and nothing else: the set the
    /// packaged helper runs with.
    pub fn from_discovery(d: &Discovery) -> FontSet {
        let dirs = d.font_dirs();
        let tfm_dirs = d.tfm_dirs_for(&dirs);
        FontSet::with_dirs(dirs, tfm_dirs).with_exe_dir(d.exe_dir.clone())
    }

    /// Records the executable directory the search list was derived from,
    /// so diagnostics name those directories relative to it.
    pub fn with_exe_dir(mut self, exe_dir: Option<PathBuf>) -> FontSet {
        self.exe_dir = exe_dir;
        self
    }

    /// `dirs` for a diagnostic: see [`describe_dirs`].
    fn describe(&self, dirs: &[PathBuf]) -> String {
        describe_dirs(dirs, self.exe_dir.as_deref())
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
            exe_dir: None,
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
                    Err(e) => errors.push(format!(
                        "{}{}: {e:?}",
                        self.describe(std::slice::from_ref(root)),
                        if *flat { " (flat)" } else { "" }
                    )),
                },
                Err(e) => errors.push(format!("{}: {e:?}", self.describe(std::slice::from_ref(root)))),
            }
        }
        if result.is_err() {
            result = Err(if candidates.is_empty() {
                format!(
                    "no TFM directory exists among ({})",
                    self.describe(&self.tfm_dirs)
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
                self.describe(&self.tfm_dirs)
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
        self.faces.borrow().iter().find(|f| &*f.font_id == font_id).cloned()
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
            Role::Mono { bold: true } => "lmmonolt10-bold.otf".to_string(),
            Role::Mono { bold: false } => format!("lmmono{}-regular.otf", mono_design(s)),
            // t1lmr.fd `m/sl`: <-8.5> 8, <8.5-9.5> 9, <9.5-11> 10, <11-15> 12, <15-> 17.
            Role::Slanted => {
                let d = if s < 8.5 {
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
                format!("lmromanslant{d}-regular.otf")
            }
        }
    }

    fn core14_for(role: Role) -> Core14 {
        match role {
            Role::Math => Core14::Symbol,
            Role::Text { bold: false, italic: false } => Core14::TimesRoman,
            Role::Text { bold: true, italic: false } => Core14::TimesBold,
            Role::Text { bold: false, italic: true } => Core14::TimesItalic,
            Role::Text { bold: true, italic: true } => Core14::TimesBoldItalic,
            Role::Slanted => Core14::TimesItalic,
            Role::Mono { .. } => Core14::Courier,
        }
    }

    /// Resolves the typewriter face at `size_pt`: Latin Modern Mono outlines
    /// laid out with [`mono_tfm_file`] (the `ec-lm*` mono TFM when that file
    /// is missing, reported through `metrics_fallback`); Courier under
    /// `times` (`\ttdefault` pcr). OT1/T1 `cmtt` has no bold: `bx` is the
    /// medium face.
    pub fn resolve_mono(&self, family: Family, metrics: MonoMetrics, bold: bool, size_pt: f64) -> Resolved {
        if family == Family::Times {
            return Resolved {
                face: self.core14(Core14::Courier),
                substituted: None,
            };
        }
        let bold = bold && metrics == MonoMetrics::LatinModern;
        let role = Role::Mono { bold };
        let file = Self::latin_modern_file(role, size_pt);
        let tfm = mono_tfm_file(metrics, bold, size_pt);
        match self.otf_with_tfm(&file, Some(&tfm)) {
            Ok(face) => Resolved { face, substituted: None },
            Err(reason) => Resolved {
                face: self.core14(Core14::Courier),
                substituted: Some(format!("{file}: {reason}")),
            },
        }
    }

    /// Resolves (and loads once) the face for `family`/`role` at `size_pt`.
    pub fn resolve(&self, family: Family, role: Role, size_pt: f64) -> Resolved {
        match (family, role) {
            (Family::Times, Role::Text { .. } | Role::Slanted | Role::Mono { .. }) => Resolved {
                face: self.core14(Self::core14_for(role)),
                substituted: None,
            },
            (_, _) => {
                let file = Self::latin_modern_file(role, size_pt);
                let loaded = match (family, ec_tfm_file(role, size_pt)) {
                    (Family::ComputerModern, Some(ec)) => self.otf_with_tfm(&file, Some(&ec)),
                    _ => self.otf(&file),
                };
                match loaded {
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
            font_id: Rc::from(sha256::hex(&sha)),
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
            shape_key: Rc::from(sha256::hex(&sha)),
            metrics_fallback: None,
            bounds_cache: RefCell::new(BTreeMap::new()),
        };
        self.insert(name, loaded)
    }

    /// Loads an explicit file name from the bounded search list.
    pub fn otf(&self, file: &str) -> Result<Rc<LoadedFace>, String> {
        self.otf_with_tfm(file, None)
    }

    /// [`FontSet::otf`] laid out with the EC metric file `ec_tfm` instead of
    /// the `ec-lm*` TFM paired with the file. The face is a separate entry
    /// named `<stem>+<tfm stem>` (same program and wire `font_id`, its own
    /// `shape_key`). When `ec_tfm` is not found the `ec-lm*` TFM is attached
    /// and [`LoadedFace::metrics_fallback`] says so.
    pub fn otf_with_tfm(&self, file: &str, ec_tfm: Option<&str>) -> Result<Rc<LoadedFace>, String> {
        let stem = file.trim_end_matches(".otf").trim_end_matches(".ttf").to_string();
        let name = match ec_tfm {
            Some(t) => format!("{stem}+{}", t.trim_end_matches(".tfm")),
            None => stem.clone(),
        };
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
                "not found in {n} search director{}: {}; add a directory holding it with --font-dir or FLASHTEX_FONT_DIRS",
                if n == 1 { "y" } else { "ies" },
                self.describe(self.search.dirs())
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
        let mut metrics_fallback = None;
        let tfm_choice = match ec_tfm {
            Some(ec) => match self.tfm(ec) {
                Ok(_) => Some(ec.to_string()),
                Err(_) => {
                    let lm = latin_modern_tfm(&stem);
                    metrics_fallback = Some(format!(
                        "{ec} (T1 cmr metrics) not found; {} used, so line breaks can differ from pdfLaTeX",
                        lm.as_deref().unwrap_or("OpenType advances")
                    ));
                    lm
                }
            },
            None => latin_modern_tfm(&stem),
        };
        let (tfm, tfm_missing, tfm_status) = match tfm_choice {
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
            font_id: Rc::from(sha256::hex(&sha)),
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
            shape_key: match ec_tfm {
                None => Rc::from(sha256::hex(&sha)),
                Some(t) => Rc::from(format!("{}+{t}", sha256::hex(&sha))),
            },
            metrics_fallback,
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
    fn missing_font_diagnostics_do_not_depend_on_the_executable_location() {
        let message = |exe: &str| {
            let d = Discovery { exe_dir: Some(PathBuf::from(exe)), ..Discovery::default() };
            let set = FontSet::from_discovery(&d);
            let Err(otf) = set.otf("flashtex-no-such-font.otf") else { panic!("font unexpectedly found") };
            let tfm = match set.tfm("flashtex-no-such-metrics.tfm") {
                Err(TfmStatus::Missing(m)) => m,
                other => panic!("unexpected {:?}", other.map(|_| ())),
            };
            (otf, tfm)
        };
        let a = message("/opt/flashtex-a/bin");
        let b = message("/Users/someone/Applications/FlashTeX.app/Contents/MacOS");
        assert_eq!(a, b);
        for m in [&a.0, &a.1] {
            assert!(!m.contains("flashtex-a") && !m.contains("someone"), "{m}");
            assert!(m.contains("<executable-dir>/../Resources/texmf/fonts"), "{m}");
        }
        assert!(a.0.contains("--font-dir"), "{}", a.0);
        assert_eq!(
            describe_dirs(&[PathBuf::from("/x/bin"), PathBuf::from("/x/bin/Fonts"), PathBuf::from("/usr/share/fonts")], Some(Path::new("/x/bin"))),
            "<executable-dir>, <executable-dir>/Fonts, /usr/share/fonts"
        );
    }

    #[test]
    fn t1_cmr_sizes_select_the_ec_metric_files_of_t1cmr_fd() {
        let rm = Role::Text { bold: false, italic: false };
        assert_eq!(ec_tfm_file(rm, 10.95).as_deref(), Some("ecrm1095.tfm"));
        assert_eq!(ec_tfm_file(rm, 10.0).as_deref(), Some("ecrm1000.tfm"));
        assert_eq!(ec_tfm_file(rm, 9.0).as_deref(), Some("ecrm0900.tfm"));
        assert_eq!(ec_tfm_file(rm, 14.4).as_deref(), Some("ecrm1440.tfm"));
        assert_eq!(ec_tfm_file(Role::Text { bold: true, italic: false }, 12.0).as_deref(), Some("ecbx1200.tfm"));
        assert_eq!(ec_tfm_file(Role::Text { bold: true, italic: false }, 17.28).as_deref(), Some("ecbx1728.tfm"));
        assert_eq!(ec_tfm_file(Role::Text { bold: false, italic: true }, 10.95).as_deref(), Some("ecti1095.tfm"));
        assert_eq!(ec_tfm_file(Role::Text { bold: true, italic: true }, 10.95).as_deref(), Some("ecbi1095.tfm"));
        assert_eq!(ec_tfm_file(Role::Slanted, 10.95).as_deref(), Some("ecsl1095.tfm"));
        // An undeclared size substitutes the nearest declared one.
        assert_eq!(ec_tfm_file(rm, 10.5).as_deref(), Some("ecrm1095.tfm"));
        assert_eq!(ec_tfm_file(rm, 50.0).as_deref(), Some("ecrm3583.tfm"));
        assert_eq!(ec_tfm_file(Role::Math, 10.95), None);
    }

    #[test]
    fn computer_modern_shares_the_program_but_not_the_metrics_of_latin_modern() {
        let set = FontSet::with_default_dirs(&[]);
        if !set.latin_modern_available() {
            eprintln!("skipping: Latin Modern not installed");
            return;
        }
        let rm = Role::Text { bold: false, italic: false };
        let lm = set.resolve(Family::LatinModern, rm, 10.95).face;
        let cm = set.resolve(Family::ComputerModern, rm, 10.95).face;
        // Same OpenType program on the wire, separate shaping identity.
        assert_eq!(lm.font_id, cm.font_id);
        assert_ne!(lm.shape_key, cm.shape_key);
        assert_eq!(lm.name, "lmroman10-regular");
        let has_ec = set.tfm_dirs().iter().any(|d| d.join("ecrm1095.tfm").is_file());
        if has_ec {
            assert_eq!(cm.name, "lmroman10-regular+ecrm1095");
            assert!(cm.metrics_fallback.is_none());
            let (l, c) = (lm.tfm.as_ref().unwrap(), cm.tfm.as_ref().unwrap());
            assert_eq!(l.design_size_pt, 10.0);
            assert!((c.design_size_pt - 10.95).abs() < 1e-3);
            // Shaping goes through the face's own TFM, not a cached LM run.
            let shaper = crate::shape::Shaper::new();
            let (a, b) = (shaper.shape(&lm, "counterexample"), shaper.shape(&cm, "counterexample"));
            assert!(b.width_pt(10.95) < a.width_pt(10.95), "{} vs {}", b.width_pt(10.95), a.width_pt(10.95));
        } else {
            // No EC metrics: Latin Modern's TFM stands in, and that is said.
            assert!(cm.metrics_fallback.as_deref().is_some_and(|m| m.contains("ecrm1095.tfm")));
            assert!(cm.tfm.is_some());
        }
    }

    #[test]
    fn missing_ec_metrics_fall_back_to_latin_modern_and_say_so() {
        let fonts = FontSet::with_default_dirs(&[]);
        if !fonts.latin_modern_available() {
            eprintln!("skipping: Latin Modern not installed");
            return;
        }
        // Only the Latin Modern TFM directories: no jknappen/ec.
        let tfm_dirs: Vec<PathBuf> = fonts.tfm_dirs().iter().filter(|d| !d.ends_with(EC_TFM_DIR)).cloned().collect();
        let set = FontSet::with_dirs(fonts.dirs().to_vec(), tfm_dirs);
        let cm = set.resolve(Family::ComputerModern, Role::Text { bold: false, italic: false }, 10.95).face;
        let note = cm.metrics_fallback.as_deref().expect("fallback reported");
        assert!(note.contains("ecrm1095.tfm") && note.contains("ec-lmr10.tfm"), "{note}");
        assert_eq!(cm.tfm_status, TfmStatus::Loaded);
    }

    #[test]
    fn ec_metrics_are_discovered_next_to_each_tex_live_tree_after_latin_modern() {
        let d = Discovery::default();
        let fonts = vec![PathBuf::from("/tl/texmf-dist/fonts/opentype/public/lm"), PathBuf::from("/flat/Fonts")];
        let dirs = d.tfm_dirs_for(&fonts);
        let lm = dirs.iter().position(|p| p == Path::new("/tl/texmf-dist/fonts/tfm/public/lm")).unwrap();
        let ec = dirs.iter().position(|p| p == Path::new("/tl/texmf-dist/fonts/tfm/jknappen/ec")).unwrap();
        assert!(lm < ec);
        assert_eq!(dirs.last().unwrap(), Path::new("/tl/texmf-dist/fonts/tfm/jknappen/ec"));
        assert!(!dirs.iter().any(|p| p.starts_with("/flat") && p.ends_with(EC_TFM_DIR)));
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
