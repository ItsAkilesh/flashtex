//! TFM loading and NFSS font selection for the PoC (OT1 Computer Modern).
//!
//! Metrics come from `tex-text-encoding`'s exact TFM loader; `tex-boxes`
//! consumes them through its `CharMetrics` trait (seam: two crates, two
//! views of the same font — see `SEAMS.md`).

use std::path::PathBuf;
use std::process::Command;

use flashtex_class_geometry::Sp;
use flashtex_tex_boxes::node::CharMetrics;
use flashtex_tex_text_encoding::tfm::ScaledFont;

/// One `\font` instance: a TFM at a size.
#[derive(Debug, Clone)]
pub struct LoadedFont {
    pub tfm: String,
    pub size: i32,
    pub font: ScaledFont,
}

/// The font table; ids are indexes and are what `tex-boxes` nodes carry.
#[derive(Debug, Default)]
pub struct Fonts {
    pub fonts: Vec<LoadedFont>,
    search: Vec<PathBuf>,
}

impl Fonts {
    /// Search order: `$FLASHTEX_TFM_DIR`, the crate's committed `tfm/`
    /// copies, then `kpsewhich` (TeX Live lookup only, no TeX run).
    pub fn new() -> Fonts {
        let mut search = Vec::new();
        if let Ok(d) = std::env::var("FLASHTEX_TFM_DIR") {
            search.push(PathBuf::from(d));
        }
        search.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tfm"));
        Fonts { fonts: Vec::new(), search }
    }

    fn read_tfm(&self, name: &str) -> Result<Vec<u8>, String> {
        for d in &self.search {
            let p = d.join(format!("{name}.tfm"));
            if p.exists() {
                return std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()));
            }
        }
        let out = Command::new("kpsewhich")
            .arg(format!("{name}.tfm"))
            .output()
            .map_err(|e| format!("{name}.tfm not found (kpsewhich: {e})"))?;
        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if path.is_empty() {
            return Err(format!("{name}.tfm not found"));
        }
        std::fs::read(&path).map_err(|e| format!("{path}: {e}"))
    }

    /// `\font\x=<tfm> at <size>`; returns the font id.
    pub fn load(&mut self, tfm: &str, size: i32) -> Result<u32, String> {
        if let Some(i) = self.fonts.iter().position(|f| f.tfm == tfm && f.size == size) {
            return Ok(i as u32);
        }
        let bytes = self.read_tfm(tfm)?;
        let font = ScaledFont::from_bytes(tfm, &bytes, size).map_err(|e| e.to_string())?;
        self.fonts.push(LoadedFont { tfm: tfm.to_string(), size, font });
        Ok((self.fonts.len() - 1) as u32)
    }

    pub fn get(&self, id: u32) -> &ScaledFont {
        &self.fonts[id as usize].font
    }
}

impl CharMetrics for Fonts {
    fn char_dims(&self, font: u32, ch: u32) -> (i32, i32, i32) {
        let f = self.get(font);
        let c = ch as u8;
        (f.width(c), f.height(c), f.depth(c))
    }

    fn font_identifier(&self, font: u32) -> String {
        let f = &self.fonts[font as usize];
        format!("\\{}", f.tfm)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Series {
    Medium,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Upright,
    Italic,
}

fn pt(s: &str) -> i32 {
    Sp::parse(&format!("{s}pt")).expect("size").0 as i32
}

/// `ot1cmr.fd` (TeX Live 2026): the TFM NFSS loads for OT1/cmr/series/shape
/// at `size` sp. `None` for sizes the class files never select.
pub fn ot1_cmr_tfm(series: Series, shape: Shape, size: i32) -> Option<String> {
    const KEYS: [&str; 12] = ["5", "6", "7", "8", "9", "10", "10.95", "12", "14.4", "17.28", "20.74", "24.88"];
    let key = KEYS.iter().find(|k| pt(k) == size)?;
    let small = |base: &str| format!("{base}{key}");
    Some(match (series, shape) {
        (Series::Medium, Shape::Upright) => match *key {
            "5" | "6" | "7" | "8" | "9" | "10" | "12" => small("cmr"),
            "10.95" => "cmr10".into(),
            "14.4" => "cmr12".into(),
            _ => "cmr17".into(),
        },
        (Series::Medium, Shape::Italic) => match *key {
            "5" | "6" | "7" => "cmti7".into(),
            "8" | "9" => small("cmti"),
            "10" | "10.95" => "cmti10".into(),
            _ => "cmti12".into(),
        },
        (Series::Bold, Shape::Upright) => match *key {
            "5" | "6" | "7" | "8" | "9" => small("cmbx"),
            "10" | "10.95" => "cmbx10".into(),
            _ => "cmbx12".into(),
        },
        (Series::Bold, Shape::Italic) => "cmbxti10".into(),
    })
}
