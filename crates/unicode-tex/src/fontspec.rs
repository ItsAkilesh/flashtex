//! fontspec: font-selection commands, their options, and what they resolve
//! to (which face files, which OpenType features, which interword glue
//! adjustments).
//!
//! Semantics follow fontspec v2.9 as shipped in TeX Live 2026
//! (`fontspec-xetex.sty`, `fontspec-luatex.sty`, `fontspec.cfg`) and were
//! checked against XeLaTeX/LuaLaTeX output in `fixtures/expected`.

use std::collections::BTreeSet;

use crate::keyval::{self, ArgScanner, KeyVal};
use crate::otl::Tag;

/// Which NFSS family a command defines.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FamilyRole {
    /// `\setmainfont` (`\rmfamily`).
    Main,
    /// `\setsansfont` (`\sffamily`).
    Sans,
    /// `\setmonofont` (`\ttfamily`).
    Mono,
    /// `\newfontfamily\cs`, `\newfontface\cs`, `\fontspec`, `\setmathrm`, ...
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontCommandKind {
    SetMainFont,
    SetSansFont,
    SetMonoFont,
    /// `\newfontfamily\cs` (also `\setfontfamily`, `\renewfontfamily`, `\providefontfamily`).
    NewFontFamily(String),
    /// `\newfontface\cs` (single face, no bold/italic).
    NewFontFace(String),
    /// `\fontspec{...}` (selects immediately).
    FontSpec,
    SetMathRm,
    SetMathSf,
    SetMathTt,
    SetBoldMathRm,
    /// `\defaultfontfeatures[targets]{options}`; empty targets = all fonts.
    DefaultFontFeatures(Vec<String>),
    /// `\addfontfeatures{options}` (applies to the current font).
    AddFontFeatures,
    /// `\setmathfont` (unicode-math), parsed with the same option model.
    SetMathFont,
}

impl FontCommandKind {
    pub fn role(&self) -> FamilyRole {
        match self {
            FontCommandKind::SetMainFont => FamilyRole::Main,
            FontCommandKind::SetSansFont => FamilyRole::Sans,
            FontCommandKind::SetMonoFont => FamilyRole::Mono,
            _ => FamilyRole::Other,
        }
    }
}

/// One parsed font command.
#[derive(Debug, Clone, PartialEq)]
pub struct FontCommand {
    pub kind: FontCommandKind,
    /// Font name or file name as written (empty for feature-only commands).
    pub name: String,
    /// All options (`[...]` before and after the name are concatenated in
    /// source order, as fontspec does).
    pub options: FontOptions,
    pub byte_offset: usize,
}

/// `Scale=` value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scale {
    Factor(f64),
    /// Scale so the x-height matches the current font's.
    MatchLowercase,
    /// Scale so the cap height matches the current font's.
    MatchUppercase,
}

impl Scale {
    /// The multiplier to apply to the requested size. `current` and `new`
    /// are the relevant heights (x-height or cap height) of the current font
    /// and of the new font, both at the same size (any unit).
    pub fn factor(self, current: f64, new: f64) -> f64 {
        match self {
            Scale::Factor(f) => f,
            Scale::MatchLowercase | Scale::MatchUppercase => {
                if new > 0.0 {
                    current / new
                } else {
                    1.0
                }
            }
        }
    }
}

/// `WordSpace=` value: `{a}` scales space, stretch and shrink by `a`;
/// `{a,b,c}` scales them separately. The extra space (`\fontdimen7`) is not
/// touched (measured: WordSpace=1.5 leaves 0.83333pt for Termes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WordSpace {
    pub space: f64,
    pub stretch: f64,
    pub shrink: f64,
}

/// `PunctuationSpace=` value, setting `\fontdimen7` (extra space).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PunctuationSpace {
    /// `WordSpace`: extra space = 0 × space (fontspec.cfg for `\ttfamily`).
    WordSpace,
    /// `TwiceWordSpace`: extra space = 1 × space.
    TwiceWordSpace,
    /// A number: extra space scaled by it.
    Scale(f64),
}

/// `Renderer=` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererOption {
    HarfBuzz,
    OpenType,
    Aat,
    Graphite,
    Node,
    Base,
}

/// A feature switch in fontspec's RawFeature syntax (`+smcp`, `-liga`, `ss01=2`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureSwitch {
    pub tag: Tag,
    pub on: bool,
}

/// Every fontspec option this crate models. Options it does not model are
/// kept verbatim in `unknown` so callers can diagnose them; nothing is
/// silently dropped.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FontOptions {
    pub path: Option<String>,
    pub extension: Option<String>,
    pub upright_font: Option<String>,
    pub bold_font: Option<String>,
    pub italic_font: Option<String>,
    pub bold_italic_font: Option<String>,
    pub slanted_font: Option<String>,
    pub small_caps_font: Option<String>,
    pub upright_features: Option<String>,
    pub bold_features: Option<String>,
    pub italic_features: Option<String>,
    pub bold_italic_features: Option<String>,
    pub scale: Option<Scale>,
    /// Feature switches in application order (later wins).
    pub features: Vec<FeatureSwitch>,
    /// `Ligatures=TeX` / `Mapping=tex-text` (Some(true)), explicitly reset (Some(false)).
    pub tex_ligatures: Option<bool>,
    pub color: Option<String>,
    pub opacity: Option<f64>,
    /// `LetterSpace=` in percent of the font size.
    pub letter_space: Option<f64>,
    pub word_space: Option<WordSpace>,
    pub punctuation_space: Option<PunctuationSpace>,
    pub hyphen_char: Option<String>,
    pub renderer: Option<RendererOption>,
    pub script: Option<Tag>,
    pub language: Option<Tag>,
    pub size_features: Option<String>,
    pub nfss_family: Option<String>,
    /// unicode-math keys seen on `\setmathfont` (`math-style`, `bold-style`, `range`, `version`, ...).
    pub math_keys: Vec<KeyVal>,
    pub unknown: Vec<KeyVal>,
}

fn tag(s: &str) -> Tag {
    Tag::from_str(s)
}

fn on(features: &mut Vec<FeatureSwitch>, t: &str) {
    features.push(FeatureSwitch {
        tag: tag(t),
        on: true,
    });
}

fn off(features: &mut Vec<FeatureSwitch>, t: &str) {
    features.push(FeatureSwitch {
        tag: tag(t),
        on: false,
    });
}

impl FontOptions {
    /// Parses a fontspec option list (the contents of `[...]`).
    pub fn parse(s: &str) -> FontOptions {
        let mut o = FontOptions::default();
        o.apply(s);
        o
    }

    /// Applies another option list on top of this one (later options win).
    pub fn apply(&mut self, s: &str) {
        for kv in keyval::parse(s) {
            self.apply_one(kv);
        }
    }

    /// Merges `other` over `self` (used for defaults ← command options).
    pub fn merge(&mut self, other: &FontOptions) {
        macro_rules! take {
            ($($f:ident),*) => { $( if other.$f.is_some() { self.$f = other.$f.clone(); } )* };
        }
        take!(
            path,
            extension,
            upright_font,
            bold_font,
            italic_font,
            bold_italic_font,
            slanted_font,
            small_caps_font,
            upright_features,
            bold_features,
            italic_features,
            bold_italic_features,
            scale,
            tex_ligatures,
            color,
            opacity,
            letter_space,
            word_space,
            punctuation_space,
            hyphen_char,
            renderer,
            script,
            language,
            size_features,
            nfss_family
        );
        self.features.extend(other.features.iter().cloned());
        self.math_keys.extend(other.math_keys.iter().cloned());
        self.unknown.extend(other.unknown.iter().cloned());
    }

    fn apply_one(&mut self, kv: KeyVal) {
        let v = kv.value.clone().unwrap_or_default();
        let values: Vec<String> = keyval::split_top_level(&v, ',')
            .into_iter()
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect();
        let f = &mut self.features;
        match kv.key.as_str() {
            "Path" => self.path = Some(v),
            "Extension" => self.extension = Some(v),
            "UprightFont" => self.upright_font = Some(v),
            "BoldFont" => self.bold_font = Some(v),
            "ItalicFont" => self.italic_font = Some(v),
            "BoldItalicFont" => self.bold_italic_font = Some(v),
            "SlantedFont" => self.slanted_font = Some(v),
            "SmallCapsFont" => self.small_caps_font = Some(v),
            "UprightFeatures" => self.upright_features = Some(v),
            "BoldFeatures" => self.bold_features = Some(v),
            "ItalicFeatures" => self.italic_features = Some(v),
            "BoldItalicFeatures" => self.bold_italic_features = Some(v),
            "SizeFeatures" => self.size_features = Some(v),
            "NFSSFamily" => self.nfss_family = Some(v),
            "Scale" => {
                self.scale = match v.as_str() {
                    "MatchLowercase" => Some(Scale::MatchLowercase),
                    "MatchUppercase" => Some(Scale::MatchUppercase),
                    x => match x.parse::<f64>() {
                        Ok(n) => Some(Scale::Factor(n)),
                        Err(_) => {
                            self.unknown.push(kv);
                            return;
                        }
                    },
                }
            }
            "Color" | "Colour" => self.color = Some(v),
            "Opacity" => self.opacity = v.parse().ok(),
            "LetterSpace" => self.letter_space = v.parse().ok(),
            "WordSpace" => {
                let nums: Vec<f64> = values.iter().filter_map(|x| x.parse().ok()).collect();
                self.word_space = match nums.as_slice() {
                    [a] => Some(WordSpace {
                        space: *a,
                        stretch: *a,
                        shrink: *a,
                    }),
                    [a, b, c] => Some(WordSpace {
                        space: *a,
                        stretch: *b,
                        shrink: *c,
                    }),
                    _ => {
                        self.unknown.push(kv);
                        return;
                    }
                }
            }
            "PunctuationSpace" => {
                self.punctuation_space = match v.as_str() {
                    "WordSpace" => Some(PunctuationSpace::WordSpace),
                    "TwiceWordSpace" => Some(PunctuationSpace::TwiceWordSpace),
                    x => x.parse().ok().map(PunctuationSpace::Scale),
                }
            }
            "HyphenChar" => self.hyphen_char = Some(v),
            "Renderer" => {
                self.renderer = match v.as_str() {
                    "HarfBuzz" | "Harfbuzz" => Some(RendererOption::HarfBuzz),
                    "OpenType" | "ICU" => Some(RendererOption::OpenType),
                    "AAT" => Some(RendererOption::Aat),
                    "Graphite" => Some(RendererOption::Graphite),
                    "Node" => Some(RendererOption::Node),
                    "Base" => Some(RendererOption::Base),
                    _ => None,
                }
            }
            "Script" => self.script = script_tag(&v),
            "Language" => self.language = language_tag(&v),
            "Mapping" => {
                if v == "tex-text" {
                    self.tex_ligatures = Some(true);
                } else {
                    self.unknown.push(kv);
                }
            }
            "RawFeature" => {
                for item in v.split([';', ',']) {
                    let item = item.trim();
                    if item.is_empty() {
                        continue;
                    }
                    if let Some(t) = item.strip_prefix('+') {
                        on(f, t.split('=').next().unwrap());
                    } else if let Some(t) = item.strip_prefix('-') {
                        off(f, t);
                    } else if let Some((t, n)) = item.split_once('=') {
                        if n.trim() == "0" {
                            off(f, t.trim())
                        } else {
                            on(f, t.trim())
                        }
                    } else if item.len() == 4 {
                        on(f, item);
                    }
                }
            }
            "Ligatures" => {
                for val in &values {
                    match val.as_str() {
                        "Required" => on(f, "rlig"),
                        "NoRequired" | "RequiredOff" => off(f, "rlig"),
                        "Common" => on(f, "liga"),
                        "NoCommon" | "CommonOff" => off(f, "liga"),
                        "Contextual" => on(f, "clig"),
                        "NoContextual" | "ContextualOff" => off(f, "clig"),
                        "Rare" | "Discretionary" => on(f, "dlig"),
                        "NoRare" | "NoDiscretionary" | "RareOff" | "DiscretionaryOff" => {
                            off(f, "dlig")
                        }
                        "Historic" => on(f, "hlig"),
                        "NoHistoric" | "HistoricOff" => off(f, "hlig"),
                        "TeX" => self.tex_ligatures = Some(true),
                        "TeXOff" | "NoTeX" => self.tex_ligatures = Some(false),
                        "ResetAll" => {
                            for t in ["rlig", "liga", "clig", "dlig", "hlig"] {
                                f.retain(|s| s.tag != tag(t));
                            }
                            self.tex_ligatures = None;
                        }
                        _ => self.unknown.push(KeyVal {
                            key: "Ligatures".into(),
                            value: Some(val.clone()),
                        }),
                    }
                }
            }
            "Numbers" => {
                for val in &values {
                    match val.as_str() {
                        "Uppercase" | "Lining" => on(f, "lnum"),
                        "Lowercase" | "OldStyle" => on(f, "onum"),
                        "Proportional" => on(f, "pnum"),
                        "Monospaced" | "Tabular" => on(f, "tnum"),
                        "SlashedZero" => on(f, "zero"),
                        "NoSlashedZero" => off(f, "zero"),
                        "Arabic" => on(f, "anum"),
                        "ResetAll" => f.retain(|s| {
                            !["lnum", "onum", "pnum", "tnum", "zero", "anum"]
                                .contains(&s.tag.as_str())
                        }),
                        _ => self.unknown.push(KeyVal {
                            key: "Numbers".into(),
                            value: Some(val.clone()),
                        }),
                    }
                }
            }
            "Letters" => {
                for val in &values {
                    match val.as_str() {
                        "SmallCaps" => on(f, "smcp"),
                        "PetiteCaps" => on(f, "pcap"),
                        "UppercaseSmallCaps" => on(f, "c2sc"),
                        "UppercasePetiteCaps" => on(f, "c2pc"),
                        "Unicase" => on(f, "unic"),
                        "ResetAll" => f.retain(|s| {
                            !["smcp", "pcap", "c2sc", "c2pc", "unic"].contains(&s.tag.as_str())
                        }),
                        _ => self.unknown.push(KeyVal {
                            key: "Letters".into(),
                            value: Some(val.clone()),
                        }),
                    }
                }
            }
            "Kerning" => match v.as_str() {
                "On" => on(f, "kern"),
                "Off" => off(f, "kern"),
                "Uppercase" => on(f, "cpsp"),
                _ => self.unknown.push(kv),
            },
            "Fractions" => match v.as_str() {
                "On" => on(f, "frac"),
                "Off" => off(f, "frac"),
                "Alternate" => on(f, "afrc"),
                _ => self.unknown.push(kv),
            },
            "VerticalPosition" => {
                for val in &values {
                    let t = match val.as_str() {
                        "Superior" => "sups",
                        "Inferior" => "subs",
                        "Numerator" => "numr",
                        "Denominator" => "dnom",
                        "ScientificInferior" => "sinf",
                        "Ordinal" => "ordn",
                        _ => {
                            self.unknown.push(KeyVal {
                                key: "VerticalPosition".into(),
                                value: Some(val.clone()),
                            });
                            continue;
                        }
                    };
                    on(f, t);
                }
            }
            "Contextuals" => {
                for val in &values {
                    match val.as_str() {
                        "Swash" => on(f, "cswh"),
                        "Alternate" => on(f, "calt"),
                        "NoAlternate" | "AlternateOff" => off(f, "calt"),
                        "WordInitial" => on(f, "init"),
                        "WordFinal" => on(f, "fina"),
                        "LineFinal" => on(f, "falt"),
                        "Inner" => on(f, "medi"),
                        _ => self.unknown.push(KeyVal {
                            key: "Contextuals".into(),
                            value: Some(val.clone()),
                        }),
                    }
                }
            }
            "Style" => {
                for val in &values {
                    match val.as_str() {
                        "Alternate" => on(f, "salt"),
                        "Italic" => on(f, "ital"),
                        "Ruby" => on(f, "ruby"),
                        "Swash" => on(f, "swsh"),
                        "Historic" => on(f, "hist"),
                        "TitlingCaps" => on(f, "titl"),
                        "HorizontalKana" => on(f, "hkna"),
                        "VerticalKana" => on(f, "vkna"),
                        _ => self.unknown.push(KeyVal {
                            key: "Style".into(),
                            value: Some(val.clone()),
                        }),
                    }
                }
            }
            "StylisticSet" => {
                for val in &values {
                    match val.parse::<u8>() {
                        Ok(n) if (1..=20).contains(&n) => on(f, &format!("ss{n:02}")),
                        _ => self.unknown.push(KeyVal {
                            key: "StylisticSet".into(),
                            value: Some(val.clone()),
                        }),
                    }
                }
            }
            "CharacterVariant" => {
                for val in &values {
                    let n = val.split(':').next().unwrap_or("");
                    match n.parse::<u8>() {
                        Ok(n) if n <= 99 => on(f, &format!("cv{n:02}")),
                        _ => self.unknown.push(KeyVal {
                            key: "CharacterVariant".into(),
                            value: Some(val.clone()),
                        }),
                    }
                }
            }
            k if k.contains('-')
                || matches!(
                    k,
                    "range" | "version" | "mathrm" | "mathit" | "mathbf" | "mathsf" | "mathtt"
                ) =>
            {
                self.math_keys.push(kv)
            }
            _ => self.unknown.push(kv),
        }
    }
}

/// fontspec `Script=` names → OpenType script tags (subset).
pub fn script_tag(name: &str) -> Option<Tag> {
    let t = match name {
        "Latin" => "latn",
        "Greek" => "grek",
        "Cyrillic" => "cyrl",
        "Hebrew" => "hebr",
        "Arabic" => "arab",
        "Armenian" => "armn",
        "Georgian" => "geor",
        "Default" | "DFLT" => "DFLT",
        s if s.len() == 4 => s,
        _ => return None,
    };
    Some(tag(t))
}

/// fontspec `Language=` names → OpenType language system tags (subset).
pub fn language_tag(name: &str) -> Option<Tag> {
    let t = match name {
        "Default" => return None,
        "Turkish" => "TRK ",
        "Polish" => "PLK ",
        "Romanian" => "ROM ",
        "Dutch" => "NLD ",
        "German" => "DEU ",
        "French" => "FRA ",
        "Catalan" => "CAT ",
        "Moldavian" => "MOL ",
        "Azeri" => "AZE ",
        "Crimean Tatar" => "CRT ",
        "Serbian" => "SRB ",
        "Macedonian" => "MKD ",
        "Bulgarian" => "BGR ",
        "Russian" => "RUS ",
        s if s.len() <= 4 => return Some(Tag::from_str(&format!("{s:<4}"))),
        _ => return None,
    };
    Some(tag(t))
}

/// Parses all fontspec commands (and `\setmathfont`) in `src`.
pub fn parse_font_commands(src: &str) -> Vec<FontCommand> {
    let code = keyval::blank_comments(src);
    let mut out = Vec::new();
    let bytes = code.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            i += 1;
            continue;
        }
        let start = i;
        let mut j = i + 1;
        while j < bytes.len() && bytes[j].is_ascii_alphabetic() {
            j += 1;
        }
        let name = &code[i + 1..j];
        let mut a = ArgScanner::new(&code, j);
        let kind = match name {
            "setmainfont" | "setromanfont" => Some(FontCommandKind::SetMainFont),
            "setsansfont" => Some(FontCommandKind::SetSansFont),
            "setmonofont" => Some(FontCommandKind::SetMonoFont),
            "newfontfamily" | "setfontfamily" | "renewfontfamily" | "providefontfamily" => a
                .mandatory()
                .map(|cs| FontCommandKind::NewFontFamily(cs.to_string())),
            "newfontface" | "setfontface" | "renewfontface" | "providefontface" => a
                .mandatory()
                .map(|cs| FontCommandKind::NewFontFace(cs.to_string())),
            "fontspec" => Some(FontCommandKind::FontSpec),
            "setmathrm" => Some(FontCommandKind::SetMathRm),
            "setmathsf" => Some(FontCommandKind::SetMathSf),
            "setmathtt" => Some(FontCommandKind::SetMathTt),
            "setboldmathrm" => Some(FontCommandKind::SetBoldMathRm),
            "setmathfont" => Some(FontCommandKind::SetMathFont),
            "defaultfontfeatures" => {
                let targets = a
                    .optional()
                    .map(|t| {
                        t.split(',')
                            .map(|x| x.trim().to_string())
                            .filter(|x| !x.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(opts) = a.mandatory() {
                    out.push(FontCommand {
                        kind: FontCommandKind::DefaultFontFeatures(targets),
                        name: String::new(),
                        options: FontOptions::parse(opts),
                        byte_offset: start,
                    });
                }
                i = a.pos.max(j);
                continue;
            }
            "addfontfeatures" | "addfontfeature" => {
                if let Some(opts) = a.mandatory() {
                    out.push(FontCommand {
                        kind: FontCommandKind::AddFontFeatures,
                        name: String::new(),
                        options: FontOptions::parse(opts),
                        byte_offset: start,
                    });
                }
                i = a.pos.max(j);
                continue;
            }
            _ => None,
        };
        let Some(kind) = kind else {
            i = j.max(i + 1);
            continue;
        };
        let mut options = FontOptions::default();
        if let Some(o) = a.optional() {
            options.apply(o);
        }
        let Some(font) = a.mandatory() else {
            i = j;
            continue;
        };
        if let Some(o) = a.optional() {
            options.apply(o);
        }
        out.push(FontCommand {
            kind,
            name: font.trim().to_string(),
            options,
            byte_offset: start,
        });
        i = a.pos;
    }
    out
}

/// The defaults `fontspec.cfg` (TeX Live 2026) installs:
/// `[\rmfamily,\sffamily]{Ligatures=TeX}` and
/// `[\ttfamily]{WordSpace={1,0,0}, HyphenChar=None, PunctuationSpace=WordSpace}`.
pub fn cfg_defaults(role: &FamilyRole) -> FontOptions {
    match role {
        FamilyRole::Main | FamilyRole::Sans => FontOptions::parse("Ligatures=TeX"),
        FamilyRole::Mono => {
            FontOptions::parse("WordSpace={1,0,0},HyphenChar=None,PunctuationSpace=WordSpace")
        }
        FamilyRole::Other => FontOptions::default(),
    }
}

/// The four NFSS shapes fontspec defines for a family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FaceSlot {
    Upright,
    Bold,
    Italic,
    BoldItalic,
}

impl FaceSlot {
    pub fn new(bold: bool, italic: bool) -> FaceSlot {
        match (bold, italic) {
            (false, false) => FaceSlot::Upright,
            (true, false) => FaceSlot::Bold,
            (false, true) => FaceSlot::Italic,
            (true, true) => FaceSlot::BoldItalic,
        }
    }
    pub fn bold(self) -> bool {
        matches!(self, FaceSlot::Bold | FaceSlot::BoldItalic)
    }
    pub fn italic(self) -> bool {
        matches!(self, FaceSlot::Italic | FaceSlot::BoldItalic)
    }
}

/// What to look up for one face.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FaceRequest {
    /// A font by name; `bold`/`italic` ask for that style of the family
    /// (XeTeX `Name/B`, luaotfload `name:Name/B`).
    Name {
        name: String,
        bold: bool,
        italic: bool,
    },
    /// A font file (`Extension=` given or the name ends in a font extension).
    File { file: String, path: Option<String> },
}

const FONT_EXTENSIONS: &[&str] = &[".otf", ".ttf", ".ttc", ".otc", ".OTF", ".TTF", ".TTC"];

/// Face requests for `slot` of a family defined by `name` + `options`,
/// following fontspec's rules:
///
/// * file mode when `Extension` is set or `name` ends with a font extension;
///   `UprightFont`/`BoldFont`/... replace `*` with `name` and append the
///   extension. In file mode a missing `BoldFont`/`ItalicFont` means NFSS
///   substitutes the upright face (fontspec does not guess file names).
/// * name mode: an explicit `BoldFont=` etc. is looked up by name; otherwise
///   the family's bold/italic style is requested.
///
/// Returns `None` when fontspec defines no face for the slot, in which case
/// NFSS substitutes (bold italic → bold, then upright; italic → upright).
pub fn face_request(name: &str, options: &FontOptions, slot: FaceSlot) -> Option<FaceRequest> {
    let ext = options.extension.clone().unwrap_or_default();
    let file_mode = !ext.is_empty() || FONT_EXTENSIONS.iter().any(|e| name.ends_with(e));
    let explicit = match slot {
        FaceSlot::Upright => options.upright_font.as_ref(),
        FaceSlot::Bold => options.bold_font.as_ref(),
        FaceSlot::Italic => options.italic_font.as_ref(),
        FaceSlot::BoldItalic => options.bold_italic_font.as_ref(),
    };
    let expand = |pattern: &str| pattern.replace('*', name);
    if file_mode {
        let base = match (slot, explicit) {
            (_, Some(p)) => expand(p),
            (FaceSlot::Upright, None) => name.to_string(),
            _ => return None,
        };
        Some(FaceRequest::File {
            file: format!("{base}{ext}"),
            path: options.path.clone(),
        })
    } else {
        match explicit {
            Some(p) => Some(FaceRequest::Name {
                name: expand(p),
                bold: false,
                italic: false,
            }),
            None => Some(FaceRequest::Name {
                name: name.to_string(),
                bold: slot.bold(),
                italic: slot.italic(),
            }),
        }
    }
}

/// NFSS substitution order when `face_request` returns `None`.
pub fn substitution_chain(slot: FaceSlot) -> &'static [FaceSlot] {
    match slot {
        FaceSlot::Upright => &[FaceSlot::Upright],
        FaceSlot::Bold => &[FaceSlot::Bold, FaceSlot::Upright],
        FaceSlot::Italic => &[FaceSlot::Italic, FaceSlot::Upright],
        FaceSlot::BoldItalic => &[FaceSlot::BoldItalic, FaceSlot::Bold, FaceSlot::Upright],
    }
}

/// The OpenType feature decisions for one face.
#[derive(Debug, Clone, PartialEq)]
pub struct FeaturePlan {
    pub on: BTreeSet<Tag>,
    pub off: BTreeSet<Tag>,
    pub tex_ligatures: bool,
    pub script: Tag,
    pub language: Option<Tag>,
    pub letter_space_percent: f64,
    pub word_space: Option<WordSpace>,
    pub punctuation_space: Option<PunctuationSpace>,
    pub scale: Option<Scale>,
}

impl FeaturePlan {
    /// Resolves `cfg defaults ← \defaultfontfeatures ← command options ← per-shape features`.
    /// fontspec passes `script=latn` unless `Script=` is given (both engines'
    /// `\fontname` shows `script=latn;language=dflt`).
    pub fn resolve(
        role: &FamilyRole,
        defaults: &[&FontOptions],
        options: &FontOptions,
        slot: FaceSlot,
    ) -> FeaturePlan {
        let mut merged = cfg_defaults(role);
        for d in defaults {
            merged.merge(d);
        }
        merged.merge(options);
        let per_shape = match slot {
            FaceSlot::Upright => merged.upright_features.clone(),
            FaceSlot::Bold => merged.bold_features.clone(),
            FaceSlot::Italic => merged.italic_features.clone(),
            FaceSlot::BoldItalic => merged.bold_italic_features.clone(),
        };
        if let Some(extra) = per_shape {
            merged.apply(&extra);
        }
        let mut on = BTreeSet::new();
        let mut off = BTreeSet::new();
        for s in &merged.features {
            if s.on {
                off.remove(&s.tag);
                on.insert(s.tag);
            } else {
                on.remove(&s.tag);
                off.insert(s.tag);
            }
        }
        FeaturePlan {
            on,
            off,
            tex_ligatures: merged.tex_ligatures.unwrap_or(false),
            script: merged.script.unwrap_or(tag("latn")),
            language: merged.language,
            letter_space_percent: merged.letter_space.unwrap_or(0.0),
            word_space: merged.word_space,
            punctuation_space: merged.punctuation_space,
            scale: merged.scale,
        }
    }

    /// Feature set the shaper applies: engine defaults plus `on`, minus `off`.
    pub fn active(&self, defaults: &[&str]) -> BTreeSet<Tag> {
        let mut set: BTreeSet<Tag> = defaults.iter().map(|t| tag(t)).collect();
        set.extend(self.on.iter().copied());
        for t in &self.off {
            set.remove(t);
        }
        set
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_setmainfont_with_both_option_positions() {
        let src = r"\setmainfont[Scale=MatchLowercase]{TeX Gyre Pagella}[Numbers={OldStyle,Proportional}, Letters=SmallCaps, RawFeature={+ss01;-liga}]";
        let c = &parse_font_commands(src)[0];
        assert_eq!(c.kind, FontCommandKind::SetMainFont);
        assert_eq!(c.name, "TeX Gyre Pagella");
        assert_eq!(c.options.scale, Some(Scale::MatchLowercase));
        let p = FeaturePlan::resolve(&c.kind.role(), &[], &c.options, FaceSlot::Upright);
        let on: Vec<_> = p.on.iter().map(|t| t.as_str().to_string()).collect();
        assert_eq!(on, ["onum", "pnum", "smcp", "ss01"]);
        assert!(p.off.contains(&tag("liga")));
        assert!(p.tex_ligatures, "Ligatures=TeX is a \\rmfamily default");
    }

    #[test]
    fn newfontfamily_and_file_faces() {
        let src = r"\newfontfamily\termes{texgyretermes}[Extension=.otf, UprightFont=*-regular, BoldFont=*-bold, WordSpace={2,0,0}, LetterSpace=5]";
        let c = &parse_font_commands(src)[0];
        assert_eq!(c.kind, FontCommandKind::NewFontFamily("\\termes".into()));
        assert_eq!(
            face_request(&c.name, &c.options, FaceSlot::Bold),
            Some(FaceRequest::File {
                file: "texgyretermes-bold.otf".into(),
                path: None
            })
        );
        assert_eq!(face_request(&c.name, &c.options, FaceSlot::Italic), None);
        let p = FeaturePlan::resolve(&c.kind.role(), &[], &c.options, FaceSlot::Upright);
        assert!(!p.tex_ligatures);
        assert_eq!(
            p.word_space,
            Some(WordSpace {
                space: 2.0,
                stretch: 0.0,
                shrink: 0.0
            })
        );
        assert_eq!(p.letter_space_percent, 5.0);
    }

    #[test]
    fn name_mode_requests_styles_and_mono_defaults() {
        let c = &parse_font_commands(r"\setmonofont{Menlo}")[0];
        assert_eq!(
            face_request(&c.name, &c.options, FaceSlot::BoldItalic),
            Some(FaceRequest::Name {
                name: "Menlo".into(),
                bold: true,
                italic: true
            })
        );
        let p = FeaturePlan::resolve(&FamilyRole::Mono, &[], &c.options, FaceSlot::Upright);
        assert_eq!(
            p.word_space,
            Some(WordSpace {
                space: 1.0,
                stretch: 0.0,
                shrink: 0.0
            })
        );
        assert_eq!(p.punctuation_space, Some(PunctuationSpace::WordSpace));
        assert!(!p.tex_ligatures);
    }

    #[test]
    fn defaultfontfeatures_and_unknown_kept() {
        let cmds = parse_font_commands(
            r"\defaultfontfeatures[\rmfamily]{Scale=1.1} \fontspec[Frobnicate=yes, Color=red]{Arial}",
        );
        assert_eq!(cmds.len(), 2);
        assert_eq!(
            cmds[0].kind,
            FontCommandKind::DefaultFontFeatures(vec!["\\rmfamily".into()])
        );
        assert_eq!(cmds[1].options.unknown[0].key, "Frobnicate");
        assert_eq!(cmds[1].options.color.as_deref(), Some("red"));
    }

    #[test]
    fn setmathfont_keeps_math_keys() {
        let c = &parse_font_commands(
            r"\setmathfont{STIX Two Math}[math-style=ISO, bold-style=upright]",
        )[0];
        assert_eq!(c.kind, FontCommandKind::SetMathFont);
        assert_eq!(c.options.math_keys.len(), 2);
        assert!(c.options.unknown.is_empty());
    }
}
