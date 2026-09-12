//! FT-019 rev 4 (c): the Latin Modern route through the font engine, against
//! the only Computer-Modern-family oracle recorded on any branch.
//!
//! Oracle: `tools/native-validation/reports/oracle-20260912T050958Z.json` on
//! `origin/agent/mac-validation/native-verification`, result `wrap-sample` /
//! `A-default` (`\documentclass{article}` `\pagestyle{empty}`: Computer
//! Modern `cmr10`/`cmbx10`/`cmti10`, OT1, 10pt, Letter, article default
//! margins, justified, hyphenated, `\section` numbered), pdfTeX 1.40.29,
//! word boxes via PDFKit (`x` = glyph-box left, `bottom` = glyph-box bottom,
//! bp). The JSON carries the 91 oracle words the greedy compiler aligned;
//! the unaligned ones (`1`, `A`, `naïve`, `café`, `TeX`, `FlashTeX`, `oracle`,
//! `em`, `dash`) have no recorded box. **No Latin Modern oracle word boxes
//! exist on any branch** (`tests/visual-corpus/evidence/*` on the
//! visual-oracle branch is raster-only), and pdflatex is not installed on
//! this machine, so the `lmodern` 12pt variant awaits regeneration.
//!
//! Ours: the Latin Modern OpenType faces (`lmroman10-{regular,bold,italic}`,
//! `lmroman12-bold`) read by `flashtex-font-engine` on main through a local
//! copy of the `FaceMetrics` adapter published on
//! `origin/agent/mac-font-engine/tex-fonts` (f418238,
//! `adapters/paragraph.rs`; not yet on main, so not stacked on). Advances,
//! GPOS kerns and the f-ligatures come from the OTF; the interword glue is
//! TeX's `\fontdimen2..4,7` of the corresponding TFMs (the OTF has no glue
//! parameters). Latin Modern is designed metric-compatible with Computer
//! Modern, so this measures (1) the breaker on a CM-family justified
//! hyphenated paragraph and (2) how far the OTF's rounded advances and its
//! own GPOS kerning drift from the `cmr10` TFM the oracle used. The test
//! skips (with a message) when the fonts are not on the machine.

use std::path::{Path, PathBuf};

use flashtex_document_style::{BaseSize, ClassOptions, Paper};
use flashtex_font_engine::{Face, GlyphId, TrueTypeFace, load_from_path};
use flashtex_paragraph_layout::style::ArticleLayout;
use flashtex_paragraph_layout::*;

/// (word, oracle x, oracle bottom, oracle starts line) — the 91 aligned words.
const ORACLE: &[(&str, f64, f64, bool)] = &[
    ("Wrapping", 157.977, 137.548, false),
    ("and", 232.569, 137.548, false),
    ("accents", 263.727, 137.548, false),
    ("reader", 171.712, 158.519, false),
    ("at", 202.783, 158.519, false),
    ("the", 215.524, 158.519, false),
    ("expects", 254.023, 158.519, false),
    ("the", 290.076, 158.519, false),
    ("layout", 307.799, 158.519, false),
    ("to", 338.534, 158.519, false),
    ("follow", 351.285, 158.519, false),
    ("the", 380.639, 158.519, false),
    ("source.", 398.362, 158.519, false),
    ("The", 434.469, 158.519, false),
    ("bold", 455.522, 158.519, false),
    ("phrase", 133.768, 170.474, true),
    ("and", 169.873, 170.474, false),
    ("the", 189.253, 170.474, false),
    ("emphasised", 206.407, 170.474, false),
    ("phrase", 258.349, 170.474, false),
    ("keep", 290.039, 170.474, false),
    ("their", 312.736, 170.474, false),
    ("words", 336.561, 170.474, false),
    ("in", 365.142, 170.474, false),
    ("order.", 376.773, 170.474, false),
    ("This", 148.712, 182.429, true),
    ("paragraph", 171.058, 182.429, false),
    ("is", 218.323, 182.429, false),
    ("deliberately", 227.939, 182.429, false),
    ("long", 281.808, 182.429, false),
    ("so", 302.993, 182.429, false),
    ("that", 314.832, 182.429, false),
    ("a", 336.017, 182.429, false),
    ("real", 343.927, 182.429, false),
    ("and", 414.704, 182.429, false),
    ("the", 433.685, 182.429, false),
    ("compiler", 154.875, 194.384, false),
    ("both", 194.825, 194.384, false),
    ("have", 217.869, 194.384, false),
    ("to", 240.353, 194.384, false),
    ("break", 252.058, 194.384, false),
    ("it", 279.002, 194.384, false),
    ("into", 288.483, 194.384, false),
    ("several", 308.202, 194.384, false),
    ("lines,", 340.466, 194.384, false),
    ("which", 365.590, 194.384, false),
    ("lets", 393.621, 194.384, false),
    ("the", 411.459, 194.384, false),
    ("comparison", 428.135, 194.384, false),
    ("tool", 133.768, 206.339, true),
    ("report", 154.667, 206.339, false),
    ("where", 185.593, 206.339, false),
    ("each", 215.096, 206.339, false),
    ("line", 238.213, 206.339, false),
    ("starts", 257.736, 206.339, false),
    ("and", 286.242, 206.339, false),
    ("every", 343.719, 206.339, false),
    ("word", 370.738, 206.339, false),
    ("drifts", 396.098, 206.339, false),
    ("from", 423.167, 206.339, false),
    ("the", 447.422, 206.339, false),
    ("position.", 154.357, 218.294, false),
    ("It", 198.171, 218.294, false),
    ("keeps", 209.618, 218.294, false),
    ("going", 236.902, 218.294, false),
    ("with", 264.134, 218.294, false),
    ("ordinary", 287.481, 218.294, false),
    ("prose,", 328.320, 218.294, false),
    ("a", 358.007, 218.294, false),
    ("few", 366.974, 218.294, false),
    ("longer", 385.616, 218.294, false),
    ("words", 416.196, 218.294, false),
    ("such", 445.435, 218.294, false),
    ("as", 468.568, 218.294, false),
    ("verification", 133.768, 230.250, true),
    ("and", 185.275, 230.250, false),
    ("reproducibility,", 204.644, 230.250, false),
    ("and", 273.887, 230.250, false),
    ("finally", 293.256, 230.250, false),
    ("ends", 323.419, 230.250, false),
    ("here.", 346.164, 230.250, false),
    ("Short", 148.712, 242.205, true),
    ("last", 175.858, 242.205, false),
    ("paragraph", 194.738, 242.205, false),
    ("with", 242.391, 242.205, false),
    ("fine", 265.081, 242.205, false),
    ("coffee", 283.897, 242.205, false),
    ("and", 311.299, 242.205, false),
    ("an", 330.668, 242.205, false),
    ("—", 383.857, 242.205, false),
    ("done.", 397.137, 242.205, false),
];

/// Where the oracle's nine lines start (by first word), from the recorded
/// `oracle_line_starts` count and the line-start flags above plus the
/// unaligned first words.
const ORACLE_LINE_FIRST_WORDS: [&str; 9] = [
    "1",
    "A",
    "phrase",
    "This",
    "FlashTeX",
    "tool",
    "oracle",
    "verification",
    "Short",
];

// ---------------------------------------------------------------------------
// Font engine adapter (local copy of f418238 `adapters::paragraph::FaceMetrics`
// plus TeX TFM glue parameters, which the OTF does not carry).
// ---------------------------------------------------------------------------

/// Latin Modern 2.004 OT1 TFM metrics (`tests/fixtures/lm_tfm.rs`, generated
/// by `tools/gen_tfm_fixture.py` from `rm-lmr10`/`rm-lmbx10`/`rm-lmri10`/
/// `rm-lmbx12.tfm`): `\fontdimen1..7`, widths, KRN pairs and f-ligatures in
/// 1/1000 em. These are the metrics pdflatex's `lmodern` OT1 route uses; the
/// `A-default` oracle used Knuth's `cmr10` family, whose widths Latin Modern
/// reproduces (its kerning program differs in places, measured below).
#[path = "fixtures/lm_tfm.rs"]
mod lm_tfm;

/// TeX `\fontdimen2,3,4,7` in 1/1000 em, taken from the TFM fixture.
#[derive(Clone, Copy)]
struct TexGlue {
    space: f64,
    stretch: f64,
    shrink: f64,
    extra: f64,
}

impl TexGlue {
    const fn from_params(p: &[f64; 7]) -> TexGlue {
        TexGlue {
            space: p[1],
            stretch: p[2],
            shrink: p[3],
            extra: p[6],
        }
    }
}

const CMR10: TexGlue = TexGlue::from_params(&lm_tfm::RM_LMR10_PARAMS);
const CMBX10: TexGlue = TexGlue::from_params(&lm_tfm::RM_LMBX10_PARAMS);
const CMTI10: TexGlue = TexGlue::from_params(&lm_tfm::RM_LMRI10_PARAMS);
const CMBX12: TexGlue = TexGlue::from_params(&lm_tfm::RM_LMBX12_PARAMS);

struct LmFace {
    face: TrueTypeFace,
    glue: TexGlue,
    /// TFM (char, width, height, depth) table: TeX's per-character boxes,
    /// which the OTF (CFF outlines unparsed) cannot supply.
    tfm: &'static [(char, f64, f64, f64)],
    #[allow(dead_code)]
    label: &'static str,
}

impl LmFace {
    fn tfm_box(&self, ch: char) -> Option<(f64, f64)> {
        self.tfm.iter().find(|c| c.0 == ch).map(|c| (c.2, c.3))
    }
    /// Fallback for characters outside the OT1 table (accented letters are
    /// `\accent` constructions in OT1): the tallest/deepest table entry.
    fn tfm_max(&self) -> (f64, f64) {
        self.tfm.iter().fold((0.0, 0.0), |(h, d), c| (h.max(c.2), d.max(c.3)))
    }
}

impl LmFace {
    fn gid(&self, ch: char) -> GlyphId {
        self.face.glyph_id(ch).unwrap_or(GlyphId::NOTDEF)
    }
}

impl FontMetricsSource for LmFace {
    fn font_id(&self) -> FontId {
        FontId(self.face.id().content_sha256)
    }
    fn units_per_em(&self) -> f64 {
        f64::from(self.face.units_per_em())
    }
    fn advance(&self, ch: char) -> f64 {
        f64::from(self.face.advance(self.gid(ch)).unwrap_or(0))
    }
    fn kern(&self, left: char, right: char) -> f64 {
        match (self.face.glyph_id(left), self.face.glyph_id(right)) {
            (Some(l), Some(r)) => f64::from(self.face.kerning(l, r).0),
            _ => 0.0,
        }
    }
    fn glyph_id(&self, ch: char) -> u32 {
        u32::from(self.gid(ch).0)
    }
    fn ascender(&self) -> f64 {
        f64::from(self.face.vertical_metrics().ascender)
    }
    fn descender(&self) -> f64 {
        f64::from(self.face.vertical_metrics().descender)
    }
    fn glyph_height(&self, ch: char) -> f64 {
        self.tfm_box(ch).unwrap_or_else(|| self.tfm_max()).0
    }
    fn glyph_depth(&self, ch: char) -> f64 {
        self.tfm_box(ch).unwrap_or_else(|| self.tfm_max()).1
    }
    fn line_gap(&self) -> f64 {
        f64::from(self.face.vertical_metrics().line_gap)
    }
    fn space(&self) -> f64 {
        self.glue.space
    }
    fn space_stretch(&self) -> f64 {
        self.glue.stretch
    }
    fn space_shrink(&self) -> f64 {
        self.glue.shrink
    }
    fn extra_space(&self) -> f64 {
        self.glue.extra
    }
    fn ligature(&self, left: char, right: char) -> Option<Ligature> {
        let (l, r) = (self.face.glyph_id(left)?, self.face.glyph_id(right)?);
        let lig = self.face.ligature(&[l, r])?;
        ['\u{FB00}', '\u{FB01}', '\u{FB02}', '\u{FB03}', '\u{FB04}']
            .into_iter()
            .find(|c| self.face.glyph_id(*c) == Some(lig))
            .map(|c| Ligature { result: c })
    }
}

fn lm_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(d) = std::env::var("FLASHTEX_LM_DIR") {
        candidates.push(PathBuf::from(d));
    }
    for tl in ["2026basic", "2026", "2025"] {
        candidates.push(PathBuf::from(format!(
            "/usr/local/texlive/{tl}/texmf-dist/fonts/opentype/public/lm"
        )));
    }
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps/mac/Fonts")
            .to_path_buf(),
    );
    candidates.into_iter().find(|d| {
        ["lmroman10-regular", "lmroman10-bold", "lmroman10-italic", "lmroman12-bold"]
            .iter()
            .all(|f| d.join(format!("{f}.otf")).is_file())
    })
}

struct Fonts {
    roman: LmFace,
    bold: LmFace,
    italic: LmFace,
    heading: LmFace,
}

fn load(
    dir: &Path,
    name: &str,
    glue: TexGlue,
    tfm: &'static [(char, f64, f64, f64)],
    label: &'static str,
) -> LmFace {
    let path = dir.join(format!("{name}.otf"));
    let face = load_from_path(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert_eq!(face.units_per_em(), 1000);
    LmFace {
        face,
        glue,
        tfm,
        label,
    }
}

fn fonts() -> Option<Fonts> {
    let Some(dir) = lm_dir() else {
        eprintln!(
            "SKIP: Latin Modern OTF faces not found (set FLASHTEX_LM_DIR to a directory holding lmroman10-regular/bold/italic and lmroman12-bold .otf)"
        );
        return None;
    };
    Some(Fonts {
        roman: load(
            &dir,
            "lmroman10-regular",
            CMR10,
            lm_tfm::RM_LMR10_CHARS,
            "cmr10",
        ),
        bold: load(
            &dir,
            "lmroman10-bold",
            CMBX10,
            lm_tfm::RM_LMBX10_CHARS,
            "cmbx10",
        ),
        italic: load(
            &dir,
            "lmroman10-italic",
            CMTI10,
            lm_tfm::RM_LMRI10_CHARS,
            "cmti10",
        ),
        heading: load(
            &dir,
            "lmroman12-bold",
            CMBX12,
            lm_tfm::RM_LMBX12_CHARS,
            "cmbx12",
        ),
    })
}

// ---------------------------------------------------------------------------
// The document
// ---------------------------------------------------------------------------

/// `\normalsize` and `\Large` in the 10pt article.
const BODY: f64 = 10.0;
const LARGE: f64 = 14.4;

struct Seg<'a>(&'a LmFace, f64, &'static str);

struct Para {
    items: Vec<Item>,
    text: String,
    heading: bool,
    indent: bool,
}

fn build(hyph: &dyn Hyphenator, segs: &[Seg<'_>], heading: bool, indent: bool) -> Para {
    let mut b = ParagraphBuilder::new(hyph);
    let mut text = String::new();
    for Seg(font, size, s) in segs {
        b.text(*font, *size, s, text.len());
        text.push_str(s);
    }
    Para {
        items: b.finish(Glue::fil()),
        text,
        heading,
        indent,
    }
}

fn paragraphs(f: &Fonts, hyph: &dyn Hyphenator) -> Vec<Para> {
    // `\section{Wrapping and accents}`: `\@seccntformat` sets the number
    // followed by `\quad` (1em of the `\Large` bold font: cmbx12's quad is
    // 1.125em = 16.2pt at 14.4pt) in an unbreakable box, then the title.
    let mut heading = ParagraphBuilder::new(hyph);
    heading.text(&f.heading, LARGE, "1", 0);
    heading.glue(Glue::fixed(lm_tfm::RM_LMBX12_PARAMS[5] / 1000.0 * LARGE));
    heading.penalty(INFINITE_PENALTY);
    heading.text(&f.heading, LARGE, "Wrapping and accents", 2);
    let heading = Para {
        items: heading.finish(Glue::fil()),
        text: "1 Wrapping and accents".to_string(),
        heading: true,
        indent: false,
    };
    vec![
        heading,
        // The paragraph after `\section` is not indented (`\@afterindentfalse`).
        build(
            hyph,
            &[
                Seg(
                    &f.roman,
                    BODY,
                    "A naïve reader at the café expects the layout to follow the source.\nThe ",
                ),
                Seg(&f.bold, BODY, "bold phrase"),
                Seg(&f.roman, BODY, " and the "),
                Seg(&f.italic, BODY, "emphasised phrase"),
                Seg(&f.roman, BODY, " keep their words in order."),
            ],
            false,
            false,
        ),
        build(
            hyph,
            &[Seg(
                &f.roman,
                BODY,
                "This paragraph is deliberately long so that a real TeX engine and the FlashTeX compiler both have to break it into several lines, which lets the comparison tool report where each line starts and how far every word drifts from the oracle position. It keeps going with ordinary prose, a few longer words such as verification and reproducibility, and finally ends here.",
            )],
            false,
            true,
        ),
        build(
            hyph,
            &[Seg(
                &f.roman,
                BODY,
                "Short last paragraph with fine coffee and an em dash \u{2014} done.",
            )],
            false,
            true,
        ),
    ]
}

struct Word {
    text: String,
    x_bp: f64,
    right_bp: f64,
    baseline_bp: f64,
    line_start: bool,
    /// Whether the run's line has finite glue set (not a ragged/last line).
    natural_glue: bool,
}

struct Ours {
    words: Vec<Word>,
    lines: Vec<Lines>,
}

fn ours(f: &Fonts) -> Ours {
    let article = ArticleLayout::new(
        ClassOptions {
            paper: Paper::Letter,
            size: BaseSize::Pt10,
        },
        None,
    );
    // article 10pt on Letter: \textwidth 345pt, \oddsidemargin 62pt (text
    // origin 134.27pt = 133.768bp), \parindent 15pt, \baselineskip 12pt,
    // \topskip 10pt.
    assert!((article.line.line_width - 345.0).abs() < 1e-9);
    assert!((article.page.margin_left - 134.27).abs() < 1e-9);
    assert!((article.line.parindent - 15.0).abs() < 1e-9);
    assert!((article.line.baselineskip - 12.0).abs() < 1e-9);
    assert!((article.page.topskip - 10.0).abs() < 1e-9);
    let hyph = LiangHyphenator::en_us_subset();
    let paras = paragraphs(f, &hyph);
    let mut blocks = Vec::new();
    let mut all_lines = Vec::new();
    let mut texts = Vec::new();
    // `ex` for the `\section` skips is the body font's x-height (cmr10:
    // 4.30554pt; the OTF declares the same 431/1000 em).
    let ex = f64::from(f.roman.face.vertical_metrics().x_height) * BODY / 1000.0;
    for p in &paras {
        let mut params = article.line.clone();
        params.parindent = if p.indent { article.line.parindent } else { 0.0 };
        if p.heading {
            params.mode = BreakMode::RaggedRight;
        }
        let lines = layout_paragraph(&p.items, &params);
        all_lines.push(lines.clone());
        texts.push(p.text.clone());
        blocks.push(if p.heading {
            flashtex_paragraph_layout::style::heading_block(1, ex, lines).unwrap()
        } else {
            ParagraphBlock::body(lines)
        });
    }
    let pages = layout_pages(&blocks, &article.page);
    assert_eq!(pages.pages.len(), 1);
    assert!(pages.overflow.is_empty());
    let mut words: Vec<Word> = Vec::new();
    let page = &pages.pages[0];
    let mut last_line: Option<f64> = None;
    let mut last_end: Option<(usize, usize)> = None;
    for r in &page.runs {
        let placed = page
            .lines
            .iter()
            .find(|l| l.baseline_y == r.baseline_y)
            .unwrap();
        let text = &texts[placed.paragraph];
        let l = &all_lines[placed.paragraph].lines[placed.line];
        let natural_glue = blocks[placed.paragraph].keep_with_next
            || placed.line + 1 == all_lines[placed.paragraph].lines.len();
        let line_start = last_line != Some(r.baseline_y);
        let piece = if r.is_hyphen {
            "-".to_string()
        } else {
            text[r.source.clone()].to_string()
        };
        let glued = !line_start
            && last_end == Some((placed.paragraph, r.source.start))
            && !text[..r.source.start].ends_with(char::is_whitespace);
        if (glued || r.is_hyphen) && !line_start {
            let w = words.last_mut().unwrap();
            w.text.push_str(&piece);
            w.right_bp = tex_pt_to_bp(r.x + r.width);
        } else {
            words.push(Word {
                text: piece,
                x_bp: tex_pt_to_bp(r.x),
                right_bp: tex_pt_to_bp(r.x + r.width),
                baseline_bp: tex_pt_to_bp(r.baseline_y),
                line_start,
                natural_glue,
            });
        }
        let _ = l;
        last_line = Some(r.baseline_y);
        last_end = Some((placed.paragraph, r.source.end));
    }
    Ours {
        words,
        lines: all_lines,
    }
}

/// Greedy in-order alignment of the 91 oracle words onto ours.
fn align(ours: &[Word]) -> Vec<usize> {
    let mut idx = Vec::with_capacity(ORACLE.len());
    let mut i = 0;
    for (w, ..) in ORACLE {
        while i < ours.len() && ours[i].text != *w {
            i += 1;
        }
        assert!(i < ours.len(), "oracle word {w:?} not found in ours after index {i}");
        idx.push(i);
        i += 1;
    }
    idx
}

#[test]
fn latin_modern_route_matches_the_computer_modern_oracle_line_starts() {
    let Some(f) = fonts() else { return };
    let o = ours(&f);
    let idx = align(&o.words);
    // Line starts: ours vs the oracle's nine.
    let our_starts: Vec<&str> = o
        .words
        .iter()
        .filter(|w| w.line_start)
        .map(|w| w.text.as_str())
        .collect();
    println!("line starts ours:   {our_starts:?}");
    println!("line starts oracle: {ORACLE_LINE_FIRST_WORDS:?}");
    let mut starts_ok = 0;
    let mut dx = Vec::new();
    let mut dx_line_start = Vec::new();
    let mut dy = Vec::new();
    let mut natural_gap = Vec::new();
    let mut set_gap = Vec::new();
    let mut largest: Vec<(f64, String, String)> = Vec::new();
    // The oracle's `bottom` is baseline + PDFKit's descent of the embedded
    // CM Type 1 font; take that constant from the first body word.
    let descent = ORACLE[3].2 - o.words[idx[3]].baseline_bp;
    println!("descent constant (oracle bottom - our baseline, body) {descent:.3} bp");
    let mut seen_bottom = f64::NAN;
    for (k, (w, ox, ob, ostart)) in ORACLE.iter().enumerate() {
        let ow = &o.words[idx[k]];
        assert_eq!(ow.text, *w);
        if *ob != seen_bottom {
            seen_bottom = *ob;
            println!(
                "  line of {w:?}: oracle bottom {ob:.3}, ours baseline {:.3} (+descent {:.3}) -> dy {:+.3} bp",
                ow.baseline_bp,
                ow.baseline_bp + descent,
                ow.baseline_bp + descent - ob
            );
        }
        if ow.line_start == *ostart {
            starts_ok += 1;
        }
        let d = ow.x_bp - ox;
        dx.push(d.abs());
        if *ostart {
            dx_line_start.push(d.abs());
        }
        dy.push((ow.baseline_bp + descent - ob).abs());
        // Gap to the next aligned word on the same line.
        if let Some((nw, nx, ..)) = ORACLE.get(k + 1)
            && !ORACLE[k + 1].3
            && idx[k + 1] == idx[k] + 1
        {
            let next = &o.words[idx[k + 1]];
            let ours_gap = next.x_bp - ow.right_bp;
            let oracle_gap = nx - ox - (ow.right_bp - ow.x_bp);
            let delta = oracle_gap - ours_gap;
            if ow.natural_glue {
                natural_gap.push(delta.abs());
            } else {
                set_gap.push(delta.abs());
            }
            largest.push((delta, w.to_string(), nw.to_string()));
        }
    }
    largest.sort_by(|a, b| b.0.abs().partial_cmp(&a.0.abs()).unwrap());
    let stat = |v: &[f64]| {
        let n = v.len().max(1) as f64;
        (
            v.len(),
            v.iter().sum::<f64>() / n,
            v.iter().cloned().fold(0.0, f64::max),
        )
    };
    let (n, m, mx) = stat(&dx);
    let (_, ms, mxs) = stat(&dx_line_start);
    let (_, my, mxy) = stat(&dy);
    let (ng, mg, mxg) = stat(&natural_gap);
    let (ns, mss, mxss) = stat(&set_gap);
    println!(
        "LM/CM A-default: aligned words {n}; line starts {starts_ok}/{n}; |dx| mean {m:.3} max {mx:.3} bp (line-start words mean {ms:.3} max {mxs:.3}); |dy| mean {my:.3} max {mxy:.3} bp"
    );
    println!(
        "gaps on natural-glue lines (heading, last lines: width difference only) n {ng} mean {mg:.4} max {mxg:.4} bp; on set lines n {ns} mean {mss:.4} max {mxss:.4} bp"
    );
    for (d, a, b) in largest.iter().take(8) {
        println!("  gap {d:+.4} bp after {a:?} before {b:?}");
    }
    for (i, l) in o.lines.iter().enumerate() {
        println!(
            "paragraph {i}: {} lines, pass {}, demerits {}, hyphenated {}",
            l.lines.len(),
            l.stats.pass,
            l.stats.total_demerits,
            l.stats.hyphenated_lines
        );
        for (line, bp) in l.lines.iter().zip(&l.breaks) {
            println!(
                "    line {}: natural {:.3} set {:.3} ratio {:+.4} badness {} {:?} demerits {} hyphenated {}",
                line.index,
                line.natural_width,
                line.set_width,
                line.ratio,
                line.badness,
                bp.fitness,
                bp.demerits,
                line.hyphenated
            );
        }
    }
    assert_eq!(our_starts, ORACLE_LINE_FIRST_WORDS);
    assert_eq!(starts_ok, ORACLE.len());
    assert!(mxs < 0.05, "line-start x max {mxs}");
    assert!(mxy < 0.05, "baseline max {mxy}");
    // Every word within 0.25 bp: the OTF's integer-unit advances and its own
    // GPOS kerning versus cmr10's TFM values accumulate along a line, and the
    // set glue absorbs the difference at the right margin.
    assert!(mx < 0.25, "|dx| max {mx}");
}

