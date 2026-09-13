//! Font lookup by family name or file name.
//!
//! [`FontLocator`] is the seam: `crates/font-resources` (pinned, hashed
//! manifests) can implement it for reproducible builds; [`DirectoryLocator`]
//! is the development implementation over an explicit directory list (macOS
//! system font directories, `apps/mac/Fonts`, TeX Live's OpenType tree). It
//! reads only the table directory plus `name` and `OS/2` of each file (seek,
//! no full read) and never consults platform font services, so results are
//! a pure function of the directory contents.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::fontspec::FaceRequest;

/// A resolved face: where it is and how it names itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedFace {
    pub path: PathBuf,
    pub face_index: u32,
    /// Typographic family (name id 16) or family (id 1).
    pub family: String,
    /// Typographic subfamily (id 17) or subfamily (id 2).
    pub subfamily: String,
    /// Legacy family (id 1), e.g. "Helvetica Neue Condensed" in a collection.
    pub legacy_family: String,
    pub full_name: String,
    pub postscript_name: String,
    /// OS/2 usWeightClass (400 when absent).
    pub weight: u16,
    /// OS/2 usWidthClass (5 when absent).
    pub width: u16,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocateError {
    NotFound {
        request: String,
        searched: Vec<PathBuf>,
    },
    Io(String),
}

impl std::fmt::Display for LocateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocateError::NotFound { request, searched } => {
                write!(
                    f,
                    "font {request} not found in {} directories",
                    searched.len()
                )
            }
            LocateError::Io(e) => write!(f, "{e}"),
        }
    }
}

/// Resolves fontspec face requests to font files.
pub trait FontLocator {
    fn locate(&self, request: &FaceRequest) -> Result<LocatedFace, LocateError>;
}

/// Development locator over explicit directories.
pub struct DirectoryLocator {
    dirs: Vec<PathBuf>,
    max_depth: usize,
    index: OnceLock<Vec<LocatedFace>>,
}

/// macOS font directories (fonts there are licensed for use on the device;
/// nothing from them is committed).
pub const MACOS_FONT_DIRS: &[&str] = &[
    "/System/Library/Fonts",
    "/System/Library/Fonts/Supplemental",
    "/Library/Fonts",
];

impl DirectoryLocator {
    /// `dirs` are scanned recursively up to `max_depth` levels (0 = the
    /// directory itself), in order; earlier directories win ties.
    pub fn new(dirs: Vec<PathBuf>, max_depth: usize) -> DirectoryLocator {
        DirectoryLocator {
            dirs,
            max_depth,
            index: OnceLock::new(),
        }
    }

    pub fn dirs(&self) -> &[PathBuf] {
        &self.dirs
    }

    pub fn faces(&self) -> &[LocatedFace] {
        self.index.get_or_init(|| {
            let mut out = Vec::new();
            for d in &self.dirs {
                scan(d, self.max_depth, &mut out);
            }
            out
        })
    }

    fn not_found(&self, request: String) -> LocateError {
        LocateError::NotFound {
            request,
            searched: self.dirs.clone(),
        }
    }
}

fn scan(dir: &Path, depth: usize, out: &mut Vec<LocatedFace>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            if depth > 0 {
                scan(&p, depth - 1, out);
            }
            continue;
        }
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if matches!(ext.as_str(), "otf" | "ttf" | "ttc" | "otc")
            && let Ok(faces) = read_faces(&p)
        {
            out.extend(faces);
        }
    }
}

pub(crate) fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

impl FontLocator for DirectoryLocator {
    fn locate(&self, request: &FaceRequest) -> Result<LocatedFace, LocateError> {
        match request {
            FaceRequest::File { file, path } => {
                if let Some(dir) = path {
                    let p = Path::new(dir).join(file);
                    return read_faces(&p)
                        .ok()
                        .and_then(|v| v.into_iter().next())
                        .ok_or_else(|| self.not_found(p.display().to_string()));
                }
                let faces = self.faces();
                let by_name = |ci: bool| {
                    faces.iter().find(|f| {
                        let n = f.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if ci {
                            n.eq_ignore_ascii_case(file)
                        } else {
                            n == file
                        }
                    })
                };
                by_name(false)
                    .or_else(|| by_name(true))
                    .or_else(|| {
                        // `lmroman10-regular` without extension (NFSS .fd style).
                        faces.iter().find(|f| {
                            f.path.file_stem().and_then(|n| n.to_str()) == Some(file.as_str())
                        })
                    })
                    .cloned()
                    .ok_or_else(|| self.not_found(file.clone()))
            }
            FaceRequest::Name { name, bold, italic } => {
                select_by_name(self.faces(), name, *bold, *italic).ok_or_else(|| {
                    self.not_found(format!(
                        "{name}{}",
                        match (bold, italic) {
                            (false, false) => "",
                            (true, false) => "/B",
                            (false, true) => "/I",
                            (true, true) => "/BI",
                        }
                    ))
                })
            }
        }
    }
}

/// Name matching: an exact full-name or PostScript-name match selects that
/// face (then `bold`/`italic` pick a sibling in its family); otherwise the
/// family (typographic, then legacy) is matched and the face whose weight,
/// width and slope are closest to the request is chosen (regular = 400,
/// bold = 700, normal width, subfamily spelled canonically preferred).
pub fn select_by_name(
    faces: &[LocatedFace],
    name: &str,
    bold: bool,
    italic: bool,
) -> Option<LocatedFace> {
    let key = normalize(name);
    let exact = faces
        .iter()
        .find(|f| normalize(&f.full_name) == key || normalize(&f.postscript_name) == key);
    let family_key = match exact {
        Some(f) if !bold && !italic => return Some(f.clone()),
        Some(f) => normalize(&f.family),
        None => key.clone(),
    };
    let mut candidates: Vec<&LocatedFace> = faces
        .iter()
        .filter(|f| normalize(&f.family) == family_key)
        .collect();
    if candidates.is_empty() {
        candidates = faces
            .iter()
            .filter(|f| normalize(&f.legacy_family) == family_key)
            .collect();
    }
    let want_weight: i32 = if bold { 700 } else { 400 };
    let canonical = match (bold, italic) {
        (false, false) => ["regular", "roman", "book", "normal"].as_slice(),
        (true, false) => ["bold"].as_slice(),
        (false, true) => ["italic", "oblique"].as_slice(),
        (true, true) => ["bolditalic", "boldoblique"].as_slice(),
    };
    candidates
        .into_iter()
        .min_by_key(|f| {
            let slope = if f.italic == italic { 0 } else { 100_000 };
            let width = (i32::from(f.width) - 5).abs() * 10_000;
            let weight = (i32::from(f.weight) - want_weight).abs() * 10;
            let canon = if canonical.contains(&normalize(&f.subfamily).as_str()) {
                0
            } else {
                5
            };
            slope + width + weight + canon
        })
        .cloned()
}

/// Reads face descriptions from a font file (all faces of a collection).
pub fn read_faces(path: &Path) -> Result<Vec<LocatedFace>, LocateError> {
    let io = |e: std::io::Error| LocateError::Io(format!("{}: {e}", path.display()));
    let mut f = File::open(path).map_err(io)?;
    let mut head = [0u8; 12];
    f.read_exact(&mut head).map_err(io)?;
    let offsets: Vec<u32> = if &head[..4] == b"ttcf" {
        let n = be32(&head[8..12]).min(256);
        let mut buf = vec![0u8; 4 * n as usize];
        f.read_exact(&mut buf).map_err(io)?;
        buf.chunks(4).map(be32).collect()
    } else {
        vec![0]
    };
    let mut out = Vec::new();
    for (i, off) in offsets.iter().enumerate() {
        if let Some(face) = read_face(&mut f, *off, path, i as u32) {
            out.push(face);
        }
    }
    Ok(out)
}

fn be16(b: &[u8]) -> u16 {
    u16::from_be_bytes([b[0], b[1]])
}
fn be32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

fn read_at(f: &mut File, off: u64, len: usize) -> Option<Vec<u8>> {
    if len > 16 << 20 {
        return None;
    }
    f.seek(SeekFrom::Start(off)).ok()?;
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf).ok()?;
    Some(buf)
}

fn read_face(f: &mut File, dir: u32, path: &Path, index: u32) -> Option<LocatedFace> {
    let hdr = read_at(f, u64::from(dir), 12)?;
    let n = usize::from(be16(&hdr[4..6]));
    let recs = read_at(f, u64::from(dir) + 12, 16 * n)?;
    let mut name = None;
    let mut os2 = None;
    for r in recs.chunks(16) {
        let (off, len) = (be32(&r[8..12]), be32(&r[12..16]) as usize);
        match &r[..4] {
            b"name" => name = read_at(f, u64::from(off), len),
            b"OS/2" => os2 = read_at(f, u64::from(off), len),
            _ => {}
        }
    }
    let names = parse_name_table(name.as_deref()?);
    let get = |id: u16| names.iter().find(|(i, _)| *i == id).map(|(_, s)| s.clone());
    let legacy_family = get(1).unwrap_or_default();
    let family = get(16).unwrap_or_else(|| legacy_family.clone());
    let legacy_sub = get(2).unwrap_or_default();
    let subfamily = get(17).unwrap_or_else(|| legacy_sub.clone());
    let (weight, width, italic) = match os2.as_deref() {
        Some(o) if o.len() >= 64 => (
            be16(&o[4..6]),
            be16(&o[6..8]),
            be16(&o[62..64]) & 0x0201 != 0,
        ),
        _ => (
            if normalize(&subfamily).contains("bold") {
                700
            } else {
                400
            },
            5,
            normalize(&subfamily).contains("italic"),
        ),
    };
    Some(LocatedFace {
        path: path.to_path_buf(),
        face_index: index,
        full_name: get(4).unwrap_or_else(|| format!("{family} {subfamily}")),
        postscript_name: get(6).unwrap_or_default(),
        family,
        subfamily,
        legacy_family,
        weight,
        width,
        italic,
    })
}

/// `(name id, string)` for ids 1, 2, 4, 6, 16, 17, preferring Windows
/// Unicode English (3/1/0x409), then any Unicode record, then Mac Roman.
pub fn parse_name_table(t: &[u8]) -> Vec<(u16, String)> {
    let mut best: Vec<(u16, u8, String)> = Vec::new();
    if t.len() < 6 {
        return Vec::new();
    }
    let count = usize::from(be16(&t[2..4]));
    let storage = usize::from(be16(&t[4..6]));
    for i in 0..count {
        let r = 6 + 12 * i;
        if r + 12 > t.len() {
            break;
        }
        let (platform, encoding, lang, id) = (
            be16(&t[r..]),
            be16(&t[r + 2..]),
            be16(&t[r + 4..]),
            be16(&t[r + 6..]),
        );
        if !matches!(id, 1 | 2 | 4 | 6 | 16 | 17) {
            continue;
        }
        let (len, off) = (
            usize::from(be16(&t[r + 8..])),
            usize::from(be16(&t[r + 10..])),
        );
        let Some(raw) = t.get(storage + off..storage + off + len) else {
            continue;
        };
        let (rank, s) = match (platform, encoding) {
            (3, 1) | (3, 10) | (0, _) => {
                let units: Vec<u16> = raw.chunks(2).filter(|c| c.len() == 2).map(be16).collect();
                (
                    if platform == 3 && lang == 0x409 { 0 } else { 1 },
                    String::from_utf16_lossy(&units),
                )
            }
            (1, 0) => (2, raw.iter().map(|&b| b as char).collect()),
            _ => continue,
        };
        match best.iter_mut().find(|(i, _, _)| *i == id) {
            Some(e) if e.1 > rank => *e = (id, rank, s),
            Some(_) => {}
            None => best.push((id, rank, s)),
        }
    }
    best.into_iter().map(|(i, _, s)| (i, s)).collect()
}

/// TeX Live font files for families documents usually request by name.
/// luaotfload's database finds these by name; XeTeX on macOS only finds
/// them by name when they are installed as system fonts, so documents
/// usually give the file name. `name` is compared case- and space-insensitively.
pub fn tex_font_file(name: &str, bold: bool, italic: bool) -> Option<String> {
    let n = normalize(name);
    let style = match (bold, italic) {
        (false, false) => "regular",
        (true, false) => "bold",
        (false, true) => "italic",
        (true, true) => "bolditalic",
    };
    let gyre = [
        "adventor", "bonum", "chorus", "cursor", "heros", "pagella", "schola", "termes",
    ];
    for g in gyre {
        if n == format!("texgyre{g}") {
            return Some(format!("texgyre{g}-{style}.otf"));
        }
        if n == format!("texgyre{g}math") {
            return Some(format!("texgyre{g}-math.otf"));
        }
    }
    Some(match n.as_str() {
        "latinmodernmath" => "latinmodern-math.otf".into(),
        "latinmodernroman" => format!("lmroman10-{style}.otf"),
        "latinmodernsans" => format!(
            "lmsans10-{}.otf",
            if italic {
                if bold { "boldoblique" } else { "oblique" }
            } else if bold {
                "bold"
            } else {
                "regular"
            }
        ),
        "latinmodernmono" => format!("lmmono10-{}.otf", if italic { "italic" } else { "regular" }),
        "stixtwomath" => "STIXTwoMath-Regular.otf".into(),
        "newcomputermodernmath" | "newcmmath" => "NewCMMath-Regular.otf".into(),
        "xitsmath" => "XITSMath-Regular.otf".into(),
        "libertinusmath" => "LibertinusMath-Regular.otf".into(),
        "asanamath" => "Asana-Math.otf".into(),
        "firamath" => "FiraMath-Regular.otf".into(),
        "garamondmath" => "Garamond-Math.otf".into(),
        _ => return None,
    })
}

/// The file NFSS `TU/lmr` (fontspec's default when no `\setmainfont` is
/// given) selects for a size: Latin Modern optical sizes from `tulmr.fd`.
pub fn latin_modern_default_file(size_pt: f64, bold: bool, italic: bool) -> String {
    let design = match size_pt {
        s if s < 5.5 => 5,
        s if s < 6.5 => 6,
        s if s < 7.5 => 7,
        s if s < 8.5 => 8,
        s if s < 9.5 => 9,
        s if s < 11.0 => 10,
        s if s < 15.0 => 12,
        _ => 17,
    };
    match (bold, italic) {
        (false, false) => format!("lmroman{design}-regular"),
        (true, false) => format!("lmroman{}-bold", design.min(12)),
        (false, true) => format!("lmroman{}-italic", design.clamp(7, 12)),
        (true, true) => "lmroman10-bolditalic".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(family: &str, sub: &str, weight: u16, width: u16, italic: bool) -> LocatedFace {
        LocatedFace {
            path: PathBuf::from(format!("/x/{family}-{sub}.ttf")),
            face_index: 0,
            family: family.into(),
            subfamily: sub.into(),
            legacy_family: family.into(),
            full_name: format!("{family} {sub}"),
            postscript_name: format!("{}-{}", family.replace(' ', ""), sub.replace(' ', "")),
            weight,
            width,
            italic,
        }
    }

    #[test]
    fn style_selection_prefers_normal_width() {
        let faces = vec![
            face("Helvetica Neue", "Condensed Bold", 700, 3, false),
            face("Helvetica Neue", "Medium", 500, 5, false),
            face("Helvetica Neue", "Regular", 400, 5, false),
            face("Helvetica Neue", "Bold", 700, 5, false),
            face("Helvetica Neue", "Italic", 400, 5, true),
        ];
        assert_eq!(
            select_by_name(&faces, "Helvetica Neue", false, false)
                .unwrap()
                .subfamily,
            "Regular"
        );
        assert_eq!(
            select_by_name(&faces, "helveticaneue", true, false)
                .unwrap()
                .subfamily,
            "Bold"
        );
        assert_eq!(
            select_by_name(&faces, "Helvetica Neue", true, true)
                .unwrap()
                .subfamily,
            "Italic"
        );
        assert_eq!(
            select_by_name(&faces, "Helvetica Neue Medium", false, false)
                .unwrap()
                .subfamily,
            "Medium"
        );
    }

    #[test]
    fn tex_names() {
        assert_eq!(
            tex_font_file("TeX Gyre Termes", true, false).as_deref(),
            Some("texgyretermes-bold.otf")
        );
        assert_eq!(
            tex_font_file("Latin Modern Math", false, false).as_deref(),
            Some("latinmodern-math.otf")
        );
        assert_eq!(
            latin_modern_default_file(12.0, false, false),
            "lmroman12-regular"
        );
        assert_eq!(
            latin_modern_default_file(10.0, true, true),
            "lmroman10-bolditalic"
        );
    }
}
