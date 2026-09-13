//! hyperref: package options, `\hypersetup`, and the link, anchor and
//! bookmark records a PDF writer needs.
//!
//! Sources (TeX Live 2026): `hyperref.sty` 2026-01-29 v7.01p, `hpdftex.def`
//! and `nameref.sty` 2026-01-29 v2.58. Line numbers below refer to those
//! files.
//!
//! The compiler records what hyperref would write; it never draws anything:
//!
//! - [`Link`]: the source span of a link's visible text, what it points at
//!   and hyperref's colour class (`link`, `url`, `cite`, `file`).
//! - [`Anchor`]: a named destination (`section.1`, `equation.2`, `Item.3`,
//!   `cite.key`, ...), at the source span where hyperref's
//!   `\hyper@anchorstart` sits.
//! - [`Bookmark`]: an outline entry (`hpdftex.def` `\Hy@writebookmark`).
//!
//! Positions (link rectangles, destination coordinates) depend on layout and
//! belong to the consumer. The `Doc-Start` and `page.<n>` destinations exist
//! in every hyperref document and are not listed.

use crate::Span;

/// hyperref's link colour classes (`\@linkcolor`, `\@urlcolor`,
/// `\@citecolor`, `\@filecolor` and their border colours).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkKind {
    /// Internal links: `\ref`, `\autoref`, `\hyperlink`, `\hyperref[..]`.
    Link,
    /// `\url`, `\href{<uri>}`.
    Url,
    /// `\cite`.
    Cite,
    /// `\href{file:...}` / `run:` links (`\hyper@linkfile`).
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkTarget {
    /// A named destination (`\hyperlink{name}`, `\href{#name}`, `cite.key`).
    Destination(String),
    /// The destination recorded by `\label{key}`; resolved against the
    /// label table (an undefined label has no link, like hyperref's `??`).
    /// `\pageref` and `\autopageref` also link to the label's destination,
    /// not to `page.<n>` (pdfTeX 1.40.29 + hyperref 7.01p output).
    Label(String),
    /// An external URI (`/URI` action).
    Uri(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The link's visible text: every layout item whose span lies inside
    /// this span belongs to the link.
    pub span: Span,
    pub kind: LinkKind,
    pub target: LinkTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bookmark {
    /// hyperref level: part -1, section 1, subsection 2, subsubsection 3
    /// (`\toclevel@<name>`); `\pdfbookmark[<level>]` gives it explicitly.
    pub level: i32,
    /// The PDF string title (`\pdfstringdef`), numbered when
    /// `bookmarksnumbered` is set (`\Hy@numberline#1{#1 }`).
    pub title: String,
    pub destination: String,
    pub span: Span,
}

/// A colour as xcolor resolves it: the model decides the PDF operator
/// (`magenta` is CMYK, `red` RGB, `gray` a gray level).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Gray(f64),
    Rgb([f64; 3]),
    Cmyk([f64; 4]),
}

impl Color {
    /// A colour name as hyperref resolves it. Without xcolor, hyperref loads
    /// `color.sty`, whose `pdftex.def` defines only black, white, red,
    /// green, blue, cyan, magenta and yellow (any other name is LaTeX's
    /// "Undefined color" error). With xcolor, its base colours
    /// (`xcolor.sty` `\definecolorset`: rgb red/green/blue/brown/lime/
    /// orange/pink/purple/teal/violet/olive, cmyk cyan/magenta/yellow, gray
    /// black/darkgray/gray/lightgray/white) and `c!pct!c` mixes.
    pub fn parse(spec: &str, xcolor: bool) -> Option<Color> {
        let spec = spec.trim();
        if !xcolor {
            return match spec {
                "black" | "white" | "red" | "green" | "blue" | "cyan" | "magenta" | "yellow" => {
                    named_color(spec)
                }
                _ => None,
            };
        }
        let mut parts = spec.split('!');
        let mut color = named_color(parts.next()?.trim())?;
        loop {
            let Some(percent) = parts.next() else {
                return Some(color);
            };
            let percent: f64 = percent.trim().parse().ok()?;
            if !(0.0..=100.0).contains(&percent) {
                return None;
            }
            let other = match parts.next() {
                Some(name) => named_color(name.trim())?,
                None => Color::Gray(1.0),
            };
            color = color.mix(percent / 100.0, other);
        }
    }

    /// `p * self + (1 - p) * other`, in `self`'s model (xcolor converts
    /// the second colour to the first colour's model).
    fn mix(self, p: f64, other: Color) -> Color {
        let blend = |a: f64, b: f64| round5(p * a + (1.0 - p) * b);
        match self {
            Color::Gray(a) => Color::Gray(blend(a, other.to_gray())),
            Color::Rgb(a) => {
                let b = other.to_rgb();
                Color::Rgb([blend(a[0], b[0]), blend(a[1], b[1]), blend(a[2], b[2])])
            }
            Color::Cmyk(a) => {
                let b = other.to_cmyk();
                Color::Cmyk([
                    blend(a[0], b[0]),
                    blend(a[1], b[1]),
                    blend(a[2], b[2]),
                    blend(a[3], b[3]),
                ])
            }
        }
    }

    pub fn to_rgb(self) -> [f64; 3] {
        match self {
            Color::Gray(g) => [g, g, g],
            Color::Rgb(rgb) => rgb,
            Color::Cmyk([c, m, y, k]) => [
                1.0 - (c + k).min(1.0),
                1.0 - (m + k).min(1.0),
                1.0 - (y + k).min(1.0),
            ],
        }
    }

    fn to_gray(self) -> f64 {
        match self {
            Color::Gray(g) => g,
            other => {
                let [r, g, b] = other.to_rgb();
                round5(0.3 * r + 0.59 * g + 0.11 * b)
            }
        }
    }

    fn to_cmyk(self) -> [f64; 4] {
        match self {
            Color::Cmyk(cmyk) => cmyk,
            Color::Gray(g) => [0.0, 0.0, 0.0, 1.0 - g],
            Color::Rgb([r, g, b]) => {
                let (c, m, y) = (1.0 - r, 1.0 - g, 1.0 - b);
                let k = c.min(m).min(y);
                [c - k, m - k, y - k, k]
            }
        }
    }

    /// The PDF fill operator xcolor emits (`0 0 1 rg`, `0 1 0 0 k`, `0 g`).
    pub fn fill_operator(self) -> String {
        let join = |values: &[f64]| {
            values
                .iter()
                .map(|v| pdf_number(*v))
                .collect::<Vec<_>>()
                .join(" ")
        };
        match self {
            Color::Gray(g) => format!("{} g", pdf_number(g)),
            Color::Rgb(rgb) => format!("{} rg", join(&rgb)),
            Color::Cmyk(cmyk) => format!("{} k", join(&cmyk)),
        }
    }

    /// hycolor's border colour form: three RGB components (`1 0 0`).
    pub fn border_components(self) -> String {
        self.to_rgb()
            .iter()
            .map(|v| pdf_number(*v))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn named_color(name: &str) -> Option<Color> {
    Some(match name {
        "red" => Color::Rgb([1.0, 0.0, 0.0]),
        "green" => Color::Rgb([0.0, 1.0, 0.0]),
        "blue" => Color::Rgb([0.0, 0.0, 1.0]),
        "brown" => Color::Rgb([0.75, 0.5, 0.25]),
        "lime" => Color::Rgb([0.75, 1.0, 0.0]),
        "orange" => Color::Rgb([1.0, 0.5, 0.0]),
        "pink" => Color::Rgb([1.0, 0.75, 0.75]),
        "purple" => Color::Rgb([0.75, 0.0, 0.25]),
        "teal" => Color::Rgb([0.0, 0.5, 0.5]),
        "violet" => Color::Rgb([0.5, 0.0, 0.5]),
        "olive" => Color::Rgb([0.5, 0.5, 0.0]),
        "cyan" => Color::Cmyk([1.0, 0.0, 0.0, 0.0]),
        "magenta" => Color::Cmyk([0.0, 1.0, 0.0, 0.0]),
        "yellow" => Color::Cmyk([0.0, 0.0, 1.0, 0.0]),
        "black" => Color::Gray(0.0),
        "darkgray" => Color::Gray(0.25),
        "gray" => Color::Gray(0.5),
        "lightgray" => Color::Gray(0.75),
        "white" => Color::Gray(1.0),
        _ => return None,
    })
}

fn round5(v: f64) -> f64 {
    (v * 100_000.0).round() / 100_000.0
}

/// Shortest decimal for a colour component (`1`, `0.6`, `0.5`).
pub fn pdf_number(v: f64) -> String {
    let v = round5(v);
    if v == v.trunc() {
        return format!("{}", v as i64);
    }
    let text = format!("{v:.5}");
    text.trim_end_matches('0').to_string()
}

/// A border colour option: hycolor accepts either three numbers
/// (`linkbordercolor={0 0 1}`, kept verbatim as hyperref writes them) or a
/// colour name.
fn border_color(value: &str, xcolor: bool) -> Option<String> {
    let numbers: Vec<&str> = value.split_whitespace().collect();
    if numbers.len() == 3 && numbers.iter().all(|n| n.parse::<f64>().is_ok()) {
        return Some(numbers.join(" "));
    }
    Color::parse(value, xcolor).map(Color::border_components)
}

/// Package options and `\hypersetup` keys, with hyperref's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// `colorlinks`: colour the link text; at `\begin{document}` hyperref
    /// then sets `\@pdfborder` to `0 0 0` (hyperref.sty 4536-4541).
    pub colorlinks: bool,
    /// `hidelinks` (hyperref.sty 3213-3221): no colour, `0 0 0` border.
    pub hidelinks: bool,
    /// `draft`: no links, anchors or bookmarks at all.
    pub draft: bool,
    /// `pdfborder`, whitespace-normalised (`\def\@pdfborder{0 0 1}`).
    pub pdfborder: String,
    pub link_color: Color,
    pub cite_color: Color,
    pub url_color: Color,
    pub file_color: Color,
    /// `\@linkbordercolor{1 0 0}` etc. (hyperref.sty 3979-3984).
    pub link_border_color: String,
    pub cite_border_color: String,
    pub url_border_color: String,
    pub file_border_color: String,
    pub bookmarks: bool,
    pub bookmarks_numbered: bool,
    pub bookmarks_open: bool,
    /// `bookmarksdepth`, defaulting to `tocdepth` (article: 3).
    pub bookmarks_depth: i32,
    pub pdftitle: Option<String>,
    pub pdfauthor: Option<String>,
    pub pdfsubject: Option<String>,
    pub pdfkeywords: Option<String>,
    pub pdfcreator: Option<String>,
    pub pdfproducer: Option<String>,
    pub pdfpagemode: Option<String>,
    pub pdfstartview: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            colorlinks: false,
            hidelinks: false,
            draft: false,
            pdfborder: "0 0 1".into(),
            link_color: Color::Rgb([1.0, 0.0, 0.0]),
            cite_color: Color::Rgb([0.0, 1.0, 0.0]),
            url_color: Color::Cmyk([0.0, 1.0, 0.0, 0.0]),
            file_color: Color::Cmyk([1.0, 0.0, 0.0, 0.0]),
            link_border_color: "1 0 0".into(),
            cite_border_color: "0 1 0".into(),
            url_border_color: "0 1 1".into(),
            file_border_color: "0 .5 .5".into(),
            bookmarks: true,
            bookmarks_numbered: false,
            bookmarks_open: false,
            bookmarks_depth: 3,
            pdftitle: None,
            pdfauthor: None,
            pdfsubject: None,
            pdfkeywords: None,
            pdfcreator: None,
            pdfproducer: None,
            pdfpagemode: None,
            pdfstartview: None,
        }
    }
}

/// hyperref keys accepted without an effect on the recorded output (they
/// change backends, index/footnote/backref behaviour this compiler does not
/// produce, viewer preferences, or are already the default behaviour).
const INERT_KEYS: &[&str] = &[
    "final",
    "unicode",
    "pdfencoding",
    "psdextra",
    "breaklinks",
    "linktoc",
    "linktocpage",
    "hyperindex",
    "hyperfootnotes",
    "naturalnames",
    "plainpages",
    "pdfpagelabels",
    "pageanchor",
    "implicit",
    "hypertexnames",
    "pdfusetitle",
    "pdfdisplaydoctitle",
    "pdfnewwindow",
    "pdftoolbar",
    "pdfmenubar",
    "pdffitwindow",
    "pdfcenterwindow",
    "pdfhighlight",
    "pdfpagelayout",
    "pdflang",
    "pdfduplex",
    "pdfprintscaling",
    "pdfview",
    "pdfremotestartview",
    "pdflinkmargin",
    "pdfa",
    "pdftex",
    "anchorcolor",
    "menucolor",
    "runcolor",
    "menubordercolor",
    "runbordercolor",
    "allcolors",
    "allbordercolors",
    "frenchlinks",
    "verbose",
    "debug",
    "setpagesize",
    "raiselinks",
    "nesting",
    "bookmarksopenlevel",
    "bookmarkstype",
    "pdfcreationdate",
    "pdfmoddate",
    "pdftrapped",
    "pdfinfo",
    "baseurl",
    "citebordercolor",
    "filebordercolor",
    "linkbordercolor",
    "urlbordercolor",
];

impl Options {
    /// Applies a comma-separated `key=value` list (package options or a
    /// `\hypersetup` argument). Returns the keys it did not recognise or
    /// whose value it could not read, for a diagnostic.
    /// `xcolor`: whether xcolor is loaded (colour names and mixes it adds).
    pub fn apply(&mut self, list: &str, xcolor: bool) -> Vec<String> {
        let mut rejected = Vec::new();
        for (key, value) in split_keyvals(list) {
            if !self.apply_one(&key, value.as_deref(), xcolor) {
                rejected.push(match value {
                    Some(value) => format!("{key}={value}"),
                    None => key,
                });
            }
        }
        rejected
    }

    fn apply_one(&mut self, key: &str, value: Option<&str>, xcolor: bool) -> bool {
        let boolean = |value: Option<&str>| match value.map(str::trim) {
            None | Some("true") => Some(true),
            Some("false") => Some(false),
            _ => None,
        };
        let set_bool = |slot: &mut bool| match boolean(value) {
            Some(v) => {
                *slot = v;
                true
            }
            None => false,
        };
        let text = || value.map(|v| v.trim().to_string());
        match key {
            "colorlinks" => {
                let ok = set_bool(&mut self.colorlinks);
                if ok && self.colorlinks {
                    self.hidelinks = false;
                }
                ok
            }
            "hidelinks" if value.is_none() => {
                self.hidelinks = true;
                self.colorlinks = false;
                true
            }
            "draft" => set_bool(&mut self.draft),
            "final" if value.is_none() || boolean(value) == Some(true) => {
                self.draft = false;
                true
            }
            "bookmarks" => set_bool(&mut self.bookmarks),
            "bookmarksnumbered" => set_bool(&mut self.bookmarks_numbered),
            "bookmarksopen" => set_bool(&mut self.bookmarks_open),
            "bookmarksdepth" => match value.map(str::trim) {
                Some(v) => match v.parse::<i32>() {
                    Ok(depth) => {
                        self.bookmarks_depth = depth;
                        true
                    }
                    Err(_) => false,
                },
                None => false,
            },
            "pdfborder" => match value {
                Some(v) => {
                    let numbers: Vec<&str> = v.split_whitespace().collect();
                    if numbers.len() >= 3 && numbers.iter().all(|n| n.parse::<f64>().is_ok()) {
                        self.pdfborder = numbers.join(" ");
                        true
                    } else {
                        false
                    }
                }
                None => false,
            },
            "linkcolor" | "citecolor" | "urlcolor" | "filecolor" | "allcolors" => {
                let Some(color) = value.and_then(|v| Color::parse(v, xcolor)) else {
                    return false;
                };
                match key {
                    "linkcolor" => self.link_color = color,
                    "citecolor" => self.cite_color = color,
                    "urlcolor" => self.url_color = color,
                    "filecolor" => self.file_color = color,
                    _ => {
                        self.link_color = color;
                        self.cite_color = color;
                        self.url_color = color;
                        self.file_color = color;
                    }
                }
                true
            }
            "linkbordercolor" | "citebordercolor" | "urlbordercolor" | "filebordercolor"
            | "allbordercolors" => {
                let Some(color) = value.and_then(|v| border_color(v, xcolor)) else {
                    return false;
                };
                match key {
                    "linkbordercolor" => self.link_border_color = color,
                    "citebordercolor" => self.cite_border_color = color,
                    "urlbordercolor" => self.url_border_color = color,
                    "filebordercolor" => self.file_border_color = color,
                    _ => {
                        self.link_border_color = color.clone();
                        self.cite_border_color = color.clone();
                        self.url_border_color = color.clone();
                        self.file_border_color = color;
                    }
                }
                true
            }
            "pdftitle" => {
                self.pdftitle = text();
                value.is_some()
            }
            "pdfauthor" => {
                self.pdfauthor = text();
                value.is_some()
            }
            "pdfsubject" => {
                self.pdfsubject = text();
                value.is_some()
            }
            "pdfkeywords" => {
                self.pdfkeywords = text();
                value.is_some()
            }
            "pdfcreator" => {
                self.pdfcreator = text();
                value.is_some()
            }
            "pdfproducer" => {
                self.pdfproducer = text();
                value.is_some()
            }
            "pdfpagemode" => {
                self.pdfpagemode = text();
                value.is_some()
            }
            "pdfstartview" => {
                self.pdfstartview = text();
                value.is_some()
            }
            // Driver names select pdfTeX, which is the only backend here.
            "pdftex" | "hpdftex" => true,
            _ => INERT_KEYS.contains(&key),
        }
    }

    /// The `/Border` array hyperref writes (`\Hy@setpdfborder`).
    pub fn border(&self) -> &str {
        if self.colorlinks || self.hidelinks {
            "0 0 0"
        } else {
            &self.pdfborder
        }
    }

    /// The `/C` array for a link class. pdfTeX writes it even when the
    /// border is invisible.
    pub fn border_color(&self, kind: LinkKind) -> &str {
        match kind {
            LinkKind::Link => &self.link_border_color,
            LinkKind::Url => &self.url_border_color,
            LinkKind::Cite => &self.cite_border_color,
            LinkKind::File => &self.file_border_color,
        }
    }

    /// The text colour under `colorlinks` (`\Hy@colorlink`); `None` keeps
    /// the surrounding colour.
    pub fn text_color(&self, kind: LinkKind) -> Option<Color> {
        if !self.colorlinks {
            return None;
        }
        Some(match kind {
            LinkKind::Link => self.link_color,
            LinkKind::Url => self.url_color,
            LinkKind::Cite => self.cite_color,
            LinkKind::File => self.file_color,
        })
    }
}

/// Splits `a, b=c, d={e, f}` at top-level commas; values lose one level of
/// braces, as keyval does.
pub fn split_keyvals(list: &str) -> Vec<(String, Option<String>)> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();
    for ch in list.chars() {
        match ch {
            '{' => {
                depth += 1;
                current.push(ch);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => items.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    items.push(current);
    items
        .into_iter()
        .filter_map(|item| {
            let item = item.trim();
            if item.is_empty() {
                return None;
            }
            Some(match item.split_once('=') {
                Some((key, value)) => (key.trim().to_string(), Some(strip_braces(value.trim()))),
                None => (item.to_string(), None),
            })
        })
        .collect()
}

fn strip_braces(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'{' && bytes[bytes.len() - 1] == b'}' {
        let inner = &value[1..value.len() - 1];
        let mut depth = 0i32;
        let balanced = inner.chars().all(|c| {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            depth >= 0
        }) && depth == 0;
        if balanced {
            return inner.trim().to_string();
        }
    }
    value.to_string()
}

/// hyperref's English autoref names (hyperref.sty 8302-8318
/// `\providecommand*`; also the `english` language block at 2841-2856).
/// `AMS`, `Hfootnote` and `Item` follow another name, so a redefinition of
/// the target changes them too.
pub const AUTOREF_NAMES: &[(&str, AutorefName)] = &[
    ("AMSautorefname", AutorefName::Alias("equationautorefname")),
    ("FancyVerbLineautorefname", AutorefName::Text("line")),
    (
        "Hfootnoteautorefname",
        AutorefName::Alias("footnoteautorefname"),
    ),
    ("Itemautorefname", AutorefName::Alias("itemautorefname")),
    ("appendixautorefname", AutorefName::Text("Appendix")),
    ("chapterautorefname", AutorefName::Text("chapter")),
    ("equationautorefname", AutorefName::Text("Equation")),
    ("figureautorefname", AutorefName::Text("Figure")),
    ("footnoteautorefname", AutorefName::Text("footnote")),
    ("itemautorefname", AutorefName::Text("item")),
    ("pageautorefname", AutorefName::Text("page")),
    ("paragraphautorefname", AutorefName::Text("paragraph")),
    ("partautorefname", AutorefName::Text("Part")),
    ("sectionautorefname", AutorefName::Text("section")),
    ("subparagraphautorefname", AutorefName::Text("subparagraph")),
    ("subsectionautorefname", AutorefName::Text("subsection")),
    (
        "subsubsectionautorefname",
        AutorefName::Text("subsubsection"),
    ),
    ("tableautorefname", AutorefName::Text("Table")),
    ("theoremautorefname", AutorefName::Text("Theorem")),
];

/// article.cls / latex.ltx `\<counter>name` fallbacks `\HyRef@testreftype`
/// also consults.
const KERNEL_NAMES: &[(&str, &str)] = &[
    ("figurename", "Figure"),
    ("tablename", "Table"),
    ("partname", "Part"),
    ("appendixname", "Appendix"),
    ("pagename", "Page"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutorefName {
    Text(&'static str),
    Alias(&'static str),
}

/// The default expansion of a name macro (`\sectionautorefname`,
/// `\figurename`, ...), following aliases through `lookup` so a user
/// redefinition of the alias target is honoured.
pub fn default_name(macro_name: &str, lookup: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    if let Some((_, name)) = AUTOREF_NAMES.iter().find(|(n, _)| *n == macro_name) {
        return match name {
            AutorefName::Text(text) => Some((*text).to_string()),
            AutorefName::Alias(target) => lookup(target).or_else(|| default_name(target, lookup)),
        };
    }
    KERNEL_NAMES
        .iter()
        .find(|(n, _)| *n == macro_name)
        .map(|(_, text)| (*text).to_string())
}

/// `\HyRef@testreftype` (hyperref.sty 8245-8281): the autoref prefix for a
/// destination name. The type is the part before the first `.`; hyperref
/// tries `\<type>autorefname`, `\<type>name`, then the same two with a
/// trailing `*` stripped (`section*` uses `\sectionautorefname`). `None`
/// is hyperref's "No autoref name" warning: the number alone is typeset.
/// `lookup` returns a user (re)definition of a macro, if any.
pub fn autoref_prefix(
    destination: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    let kind = destination.split('.').next().unwrap_or(destination);
    let find = |name: String| lookup(&name).or_else(|| default_name(&name, lookup));
    find(format!("{kind}autorefname"))
        .or_else(|| find(format!("{kind}name")))
        .or_else(|| {
            let base = kind.strip_suffix('*')?;
            find(format!("{base}autorefname")).or_else(|| find(format!("{base}name")))
        })
}

/// A PDF text string as hyperref writes it with `unicode=true` (the default
/// since v7): UTF-16BE with a byte-order mark.
pub fn pdf_text_string(text: &str) -> Vec<u8> {
    let mut out = vec![0xFE, 0xFF];
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_be_bytes());
    }
    out
}

/// Everything the document's hyperref usage recorded.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Hyperref {
    /// `\usepackage{hyperref}` was seen. Links, anchors and bookmarks are
    /// recorded only then (`\url`/`\href` still typeset their text without).
    pub loaded: bool,
    pub options: Options,
    pub links: Vec<Link>,
    pub anchors: Vec<Anchor>,
    pub bookmarks: Vec<Bookmark>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn keyvals_split_at_top_level_commas_and_strip_one_brace_level() {
        assert_eq!(
            split_keyvals("colorlinks, pdftitle={A, B}, pdfborder={0 0 1},"),
            vec![
                ("colorlinks".to_string(), None),
                ("pdftitle".to_string(), Some("A, B".to_string())),
                ("pdfborder".to_string(), Some("0 0 1".to_string())),
            ]
        );
    }

    #[test]
    fn options_follow_hyperref_defaults_and_overrides() {
        let mut options = Options::default();
        assert_eq!(options.border(), "0 0 1");
        assert_eq!(options.border_color(LinkKind::Url), "0 1 1");
        assert_eq!(options.text_color(LinkKind::Link), None);
        assert!(options
            .apply("colorlinks,linkcolor=blue,pdftitle={T, U}", false)
            .is_empty());
        assert_eq!(options.border(), "0 0 0");
        assert_eq!(
            options.text_color(LinkKind::Link).unwrap().fill_operator(),
            "0 0 1 rg"
        );
        assert_eq!(
            options.text_color(LinkKind::Url).unwrap().fill_operator(),
            "0 1 0 0 k"
        );
        assert_eq!(options.pdftitle.as_deref(), Some("T, U"));
        assert_eq!(
            options.apply("nonsense=1, bookmarksdepth=x, urlcolor=teal", false),
            vec![
                "nonsense=1".to_string(),
                "bookmarksdepth=x".to_string(),
                "urlcolor=teal".to_string()
            ]
        );
        let mut hidden = Options::default();
        hidden.apply("hidelinks", false);
        assert_eq!(hidden.border(), "0 0 0");
        assert_eq!(hidden.text_color(LinkKind::Link), None);
    }

    #[test]
    fn xcolor_mixes_in_the_first_colours_model() {
        assert_eq!(
            Color::parse("red!60!black", true).unwrap().fill_operator(),
            "0.6 0 0 rg"
        );
        assert_eq!(
            Color::parse("blue!50", true).unwrap().fill_operator(),
            "0.5 0.5 1 rg"
        );
        assert_eq!(Color::parse("gray", true).unwrap().fill_operator(), "0.5 g");
        assert!(
            Color::parse("gray", false).is_none(),
            "color.sty has no gray"
        );
        assert!(Color::parse("red!50", false).is_none());
        assert!(Color::parse("chartreuse", true).is_none());
        assert_eq!(border_color("0 0 1", false).as_deref(), Some("0 0 1"));
        assert_eq!(border_color("red", false).as_deref(), Some("1 0 0"));
    }

    #[test]
    fn autoref_prefix_follows_testreftype() {
        assert_eq!(
            autoref_prefix("section.1", &none).as_deref(),
            Some("section")
        );
        assert_eq!(
            autoref_prefix("section*.3", &none).as_deref(),
            Some("section")
        );
        assert_eq!(autoref_prefix("Item.2", &none).as_deref(), Some("item"));
        assert_eq!(
            autoref_prefix("equation.4", &none).as_deref(),
            Some("Equation")
        );
        assert_eq!(autoref_prefix("thm.1", &none), None);
        let renamed = |name: &str| (name == "itemautorefname").then(|| "Step".to_string());
        assert_eq!(autoref_prefix("Item.2", &renamed).as_deref(), Some("Step"));
    }

    #[test]
    fn pdf_text_strings_are_utf16_with_a_byte_order_mark() {
        assert_eq!(pdf_text_string("Hi"), vec![0xFE, 0xFF, 0, b'H', 0, b'i']);
    }
}
