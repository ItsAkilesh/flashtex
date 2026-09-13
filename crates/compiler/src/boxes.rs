//! LaTeX box commands as structured inline nodes (`latex.ltx` ltboxes):
//! `\mbox`, `\makebox`, `\fbox`, `\framebox`, `\parbox`, `minipage`,
//! `\raisebox`, `\phantom`/`\hphantom`/`\vphantom`, `\smash`,
//! `\llap`/`\rlap`, `\strut`, saved boxes (`\newsavebox`, `\sbox`,
//! `\savebox`, `lrbox`, `\usebox`) and `\settowidth`/`\settoheight`/
//! `\settodepth` with `\newlength`.
//!
//! The parser records what the source asks for; it never measures. Box
//! geometry needs the content's font metrics, so a typesetter sets each node
//! with TeX's box arithmetic (`crates/tex-boxes`, whose `latex` module
//! transcribes these macros). Dimensions keep the source's decimal digits
//! and unit ([`BoxDimen`]) so a consumer can convert them to scaled points
//! exactly as `scan_dimen` does; `pt_approx` is only for this crate's own
//! Core 14 layout, which places box content inline.

use crate::parser::{Inline, ParagraphStyle};
use crate::Span;

/// A `<dimen>` argument: `[-]<integer>.<fraction><unit>`.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxDimen {
    pub negative: bool,
    pub integer: i32,
    /// Decimal fraction digits, most significant first.
    pub frac: Vec<u8>,
    pub unit: BoxUnit,
}

/// The unit (or internal dimension) a [`BoxDimen`] factor multiplies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxUnit {
    /// `pt`, `bp`, `in`, `cm`, `mm`, `pc`, `dd`, `cc` or `sp`.
    Physical(String),
    /// `em` of the current font.
    Em,
    /// `ex` of the current font.
    Ex,
    /// `\width`, `\height`, `\depth`, `\totalheight` of the content
    /// (`\@begin@tempboxa`), valid in `\makebox`/`\framebox`/`\savebox`
    /// widths and `\raisebox` heights.
    Width,
    Height,
    Depth,
    TotalHeight,
    /// An internal length: `\textwidth`, `\linewidth`, `\columnwidth`,
    /// `\hsize`, `\fboxsep`, `\fboxrule`, `\parindent`, `\baselineskip`,
    /// or a length declared with `\newlength`.
    Length(String),
}

impl BoxDimen {
    /// Parses `-1.5cm`, `.5\textwidth`, `2\width`, `\w`. A missing factor
    /// before an internal length is 1 (`\textwidth`). `None` for anything
    /// else (glue with `plus`/`minus`, expressions).
    pub fn parse(text: &str) -> Option<BoxDimen> {
        let mut rest = text.trim();
        let mut negative = false;
        loop {
            if let Some(r) = rest.strip_prefix('-') {
                negative = !negative;
                rest = r.trim_start();
            } else if let Some(r) = rest.strip_prefix('+') {
                rest = r.trim_start();
            } else {
                break;
            }
        }
        let digits_end = rest
            .char_indices()
            .find(|(_, c)| !(c.is_ascii_digit() || *c == '.' || *c == ','))
            .map_or(rest.len(), |(i, _)| i);
        let number = &rest[..digits_end];
        let unit_text = rest[digits_end..].trim();
        let (integer, frac) = if number.is_empty() {
            (1, Vec::new())
        } else {
            let (int_part, frac_part) = match number.find(['.', ',']) {
                Some(at) => (&number[..at], &number[at + 1..]),
                None => (number, ""),
            };
            if frac_part.contains(['.', ',']) {
                return None;
            }
            let integer = if int_part.is_empty() {
                0
            } else {
                int_part.parse::<i32>().ok()?
            };
            let frac = frac_part.bytes().map(|b| b - b'0').collect();
            (integer, frac)
        };
        let unit = if let Some(name) = unit_text.strip_prefix('\\') {
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphabetic() || c == '@') {
                return None;
            }
            match name {
                "width" => BoxUnit::Width,
                "height" => BoxUnit::Height,
                "depth" => BoxUnit::Depth,
                "totalheight" => BoxUnit::TotalHeight,
                other => BoxUnit::Length(other.to_string()),
            }
        } else {
            if number.is_empty() {
                return None;
            }
            match unit_text {
                "em" => BoxUnit::Em,
                "ex" => BoxUnit::Ex,
                "pt" | "bp" | "in" | "cm" | "mm" | "pc" | "dd" | "cc" | "sp" => {
                    BoxUnit::Physical(unit_text.to_string())
                }
                _ => return None,
            }
        };
        Some(BoxDimen {
            negative,
            integer,
            frac,
            unit,
        })
    }

    /// The factor as a float (`-1.5` for `-1.5cm`).
    pub fn factor(&self) -> f64 {
        let mut value = f64::from(self.integer);
        let mut scale = 0.1;
        for &digit in &self.frac {
            value += f64::from(digit) * scale;
            scale /= 10.0;
        }
        if self.negative {
            -value
        } else {
            value
        }
    }

    /// An approximate value in points for the Core 14 layout: `em`/`ex`
    /// against `body_pt`, `\width`-style content dimensions and unknown
    /// lengths as 0, `\textwidth`-style lengths against `text_width_pt`.
    pub fn pt_approx(&self, body_pt: f64, text_width_pt: f64) -> f64 {
        let unit = match &self.unit {
            BoxUnit::Physical(unit) => match unit.as_str() {
                "pt" => 1.0,
                "bp" => 72.27 / 72.0,
                "in" => 72.27,
                "cm" => 72.27 / 2.54,
                "mm" => 7.227 / 2.54,
                "pc" => 12.0,
                "dd" => 1238.0 / 1157.0,
                "cc" => 14856.0 / 1157.0,
                _ => 1.0 / 65536.0,
            },
            BoxUnit::Em => body_pt,
            BoxUnit::Ex => body_pt * 0.43,
            BoxUnit::Length(name) => match name.as_str() {
                "textwidth" | "linewidth" | "columnwidth" | "hsize" => text_width_pt,
                "fboxsep" => 3.0,
                "fboxrule" => 0.4,
                _ => 0.0,
            },
            BoxUnit::Width | BoxUnit::Height | BoxUnit::Depth | BoxUnit::TotalHeight => 0.0,
        };
        self.factor() * unit
    }

    /// Whether the dimension reads `\width`/`\height`/`\depth`/`\totalheight`.
    pub fn uses_content_dimensions(&self) -> bool {
        matches!(
            self.unit,
            BoxUnit::Width | BoxUnit::Height | BoxUnit::Depth | BoxUnit::TotalHeight
        )
    }
}

/// Which box command a [`TextBox`] is.
#[derive(Debug, Clone, PartialEq)]
pub enum TextBoxKind {
    /// `\mbox{..}` (`width: None`), `\makebox[w][pos]{..}`; with `frame`,
    /// `\fbox{..}` / `\framebox[w][pos]{..}`. Saved boxes (`\sbox`,
    /// `\savebox`, `lrbox`) are this kind too, emitted again by `\usebox`.
    Make {
        width: Option<BoxDimen>,
        pos: Option<char>,
        frame: bool,
    },
    /// `\raisebox{lift}[height][depth]{..}`.
    Raise {
        lift: BoxDimen,
        height: Option<BoxDimen>,
        depth: Option<BoxDimen>,
    },
    /// `\phantom` (both), `\hphantom` (`vertical: false`), `\vphantom`
    /// (`horizontal: false`) in text mode.
    Phantom { horizontal: bool, vertical: bool },
    /// `\smash{..}` in text mode.
    Smash,
    /// `\llap{..}` (`left: true`) and `\rlap{..}`.
    Lap { left: bool },
    /// `\parbox[pos][height][inner]{width}{..}` and
    /// `\begin{minipage}[pos][height][inner]{width} .. \end{minipage}`; the
    /// content is in [`TextBox::paragraphs`].
    Par {
        pos: Option<char>,
        height: Option<BoxDimen>,
        inner: Option<char>,
        width: BoxDimen,
        minipage: bool,
    },
    /// `\strut`: no content.
    Strut,
}

/// One paragraph of a `\parbox`/`minipage`, with the alignment declaration
/// (`\centering`, `\raggedright`, `\raggedleft`, or an alignment
/// environment) in force when it ended.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxParagraph {
    pub style: Option<ParagraphStyle>,
    pub content: Vec<Inline>,
}

/// A box command in running text.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBox {
    pub kind: TextBoxKind,
    /// Horizontal content (every kind except `Par` and `Strut`).
    pub content: Vec<Inline>,
    /// Vertical content (`Par` only).
    pub paragraphs: Vec<BoxParagraph>,
    /// The command through its last argument (`\usebox{..}` for a reused
    /// saved box, `\begin` through `\end` for `minipage`).
    pub span: Span,
    /// See `Inline::Text::space_before`.
    pub space_before: bool,
}

/// Which dimension of the measured content `\settowidth` & co. assign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasuredDimension {
    Width,
    Height,
    Depth,
}

/// A length assignment in the body: `\settowidth{\name}{..}` and friends
/// (`Measure`), or `\setlength{\name}{<dimen>}` of a `\newlength` (`Dimen`).
#[derive(Debug, Clone, PartialEq)]
pub enum LengthValue {
    Measure {
        which: MeasuredDimension,
        content: Vec<Inline>,
    },
    Dimen(BoxDimen),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LengthAssignment {
    /// The length's control-sequence name without the backslash.
    pub name: String,
    pub value: LengthValue,
    pub span: Span,
}

impl TextBox {
    /// Every inline list the box holds, for walks over nested content.
    pub fn inline_lists(&self) -> Vec<&[Inline]> {
        let mut lists: Vec<&[Inline]> = vec![&self.content];
        lists.extend(self.paragraphs.iter().map(|p| p.content.as_slice()));
        lists
    }

    /// Mutable counterpart of [`TextBox::inline_lists`].
    pub fn inline_lists_mut(&mut self) -> Vec<&mut Vec<Inline>> {
        let mut lists: Vec<&mut Vec<Inline>> = vec![&mut self.content];
        lists.extend(self.paragraphs.iter_mut().map(|p| &mut p.content));
        lists
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_physical_and_internal_dimensions() {
        let d = BoxDimen::parse("-1.25cm").unwrap();
        assert!(d.negative);
        assert_eq!((d.integer, d.frac.as_slice()), (1, &[2u8, 5][..]));
        assert_eq!(d.unit, BoxUnit::Physical("cm".into()));
        let d = BoxDimen::parse(".5\\textwidth").unwrap();
        assert_eq!((d.integer, d.frac.as_slice()), (0, &[5u8][..]));
        assert_eq!(d.unit, BoxUnit::Length("textwidth".into()));
        let d = BoxDimen::parse("\\width").unwrap();
        assert_eq!((d.integer, d.unit), (1, BoxUnit::Width));
        assert_eq!(BoxDimen::parse("2\\totalheight").unwrap().unit, BoxUnit::TotalHeight);
        assert!(BoxDimen::parse("3").is_none());
        assert!(BoxDimen::parse("1pt plus 1fil").is_none());
        assert!(BoxDimen::parse("cm").is_none());
    }
}
