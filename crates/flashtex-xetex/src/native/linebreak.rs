//! ICU's line breaker for `\XeTeXlinebreaklocale`: XeTeX_ext.c's
//! `linebreakstart` and `linebreaknext`, which `do_locale_linebreaks`
//! (xetex.web) calls to cut a native word into the pieces between which it
//! puts `\XeTeXlinebreakpenalty` and `\XeTeXlinebreakskip`.
//!
//! XeTeX opens `ubrk_open(UBRK_LINE, locale)` of TeX Live 2026's ICU 78.2
//! (`xetex --version`: "Compiled with ICU version 78.2") and returns
//! `ubrk_next` until `UBRK_DONE` (-1). Which rules ICU uses depends on the
//! locale, through its `brkitr` resource tree (ICU 78.2's
//! `icu4c/source/data/brkitr`, `BreakIterator::makeInstance` and
//! `buildInstance`, `ures_openNoDefault`: no fallback to the process's
//! default locale):
//!
//! | locale's language | `line` | `@lb=strict` | `@lb=normal` | `@lb=loose` |
//! |---|---|---|---|---|
//! | `zh` (`zh.txt`, `zh_Hant.txt`) | `line_cj` | `line_cj` | `line_normal_cj` | `line_loose_cj` |
//! | `ja`, `ko` | `line_normal` | `line_cj` | `line_normal_cj` | `line_loose_cj` |
//! | anything else (`root.txt`) | `line` | `line` | `line_normal` | `line_loose` |
//!
//! The rules themselves are UAX #14 of Unicode 17 with ICU's tailorings, and
//! the Southeast Asian scripts (class SA: Thai, Lao, Khmer, Myanmar) are cut
//! by ICU's dictionaries. Here they come from ICU4X's `icu_segmenter` 2.1
//! (Unicode-3.0 licence), whose line data is exported from ICU 78
//! (`icuexport` release-78.1rc) and whose rules follow ICU4C's `line*.txt`:
//! `line` is its `Strict`, `line_normal` its `Normal`, `line_loose` its
//! `Loose`; a `_cj` file differs from its plain one in treating U+201C as
//! OP and U+201D as CL (not QU), which is done here by giving the segmenter
//! `(` and `}` in their place (both OP/CL, not East Asian, like the quotes);
//! the `_normal_cj` and `_loose_cj` files' other CJK breaks are ICU4X's
//! `ja`/`zh` content locale. `lw=phrase` (ICU's phrase breaking for
//! Japanese and Korean) is not ported: it is treated as absent.
//! Graphite's breaking (`\XeTeXlinebreaklocale "G"` with a Graphite font)
//! is not reached: this port has no Graphite engine.

use icu_locale_core::{langid, LanguageIdentifier};
use icu_segmenter::options::{LineBreakOptions, LineBreakStrictness};
use icu_segmenter::LineSegmenter;

/// ICU's line rules (`brkitr/rules/line*.txt`), chosen by the locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rules {
    Line,
    LineCj,
    LineNormal,
    LineNormalCj,
    LineLoose,
    LineLooseCj,
}

/// The rules `ubrk_open(UBRK_LINE, locale)` uses for `locale`, the bytes of
/// the `\XeTeXlinebreaklocale` string (`gettexstring`, UTF-8).
pub fn rules_for_locale(locale: &[u8]) -> Rules {
    let locale = String::from_utf8_lossy(locale);
    // ICU's locale ID: the language is the first subtag, before `_`, `-`,
    // `.` or `@`; keywords follow `@`, separated by `;`.
    let (id, keywords) = match locale.find('@') {
        Some(i) => (&locale[..i], &locale[i + 1..]),
        None => (&locale[..], ""),
    };
    let lang = id
        .split(['_', '-', '.'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let lb = keywords.split(';').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        k.trim().eq_ignore_ascii_case("lb").then(|| v.trim())
    });
    let lb = match lb {
        Some(v @ ("strict" | "normal" | "loose")) => Some(v),
        _ => None,
    };
    match (lang.as_str(), lb) {
        ("zh", None | Some("strict")) => Rules::LineCj,
        ("ja" | "ko", None) => Rules::LineNormal,
        ("ja" | "ko", Some("strict")) => Rules::LineCj,
        ("zh" | "ja" | "ko", Some("normal")) => Rules::LineNormalCj,
        ("zh" | "ja" | "ko", _) => Rules::LineLooseCj,
        (_, Some("normal")) => Rules::LineNormal,
        (_, Some("loose")) => Rules::LineLoose,
        _ => Rules::Line,
    }
}

/// The boundaries `ubrk_next` returns for `text` (UTF-16) after
/// `ubrk_setText`, in order: every line-break opportunity after the start,
/// the end of the text last (none for an empty text).
pub fn boundaries(rules: Rules, text: &[u16]) -> Vec<i32> {
    let (strictness, cj) = match rules {
        Rules::Line => (LineBreakStrictness::Strict, false),
        Rules::LineCj => (LineBreakStrictness::Strict, true),
        Rules::LineNormal => (LineBreakStrictness::Normal, false),
        Rules::LineNormalCj => (LineBreakStrictness::Normal, true),
        Rules::LineLoose => (LineBreakStrictness::Loose, false),
        Rules::LineLooseCj => (LineBreakStrictness::Loose, true),
    };
    let mut options = LineBreakOptions::default();
    options.strictness = Some(strictness);
    // ICU4X's `ja`/`zh` content locale: the CJK tailorings of
    // line_normal_cj.txt and line_loose_cj.txt (no effect in Strict).
    let ja: LanguageIdentifier = langid!("ja");
    options.content_locale = cj.then_some(&ja);
    let seg = LineSegmenter::new_dictionary(options);
    let text: std::borrow::Cow<[u16]> = if cj {
        text.iter()
            .map(|&c| match c {
                0x201C => u16::from(b'('),
                0x201D => u16::from(b'}'),
                c => c,
            })
            .collect::<Vec<u16>>()
            .into()
    } else {
        text.into()
    };
    seg.segment_utf16(&text)
        .filter(|&b| b > 0)
        .map(|b| b as i32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn locales_choose_icus_brkitr_rules() {
        assert_eq!(rules_for_locale(b""), Rules::Line);
        assert_eq!(rules_for_locale(b"en"), Rules::Line);
        assert_eq!(rules_for_locale(b"th"), Rules::Line);
        assert_eq!(rules_for_locale(b"kh"), Rules::Line);
        assert_eq!(rules_for_locale(b"zh"), Rules::LineCj);
        assert_eq!(rules_for_locale(b"zh_Hant_TW"), Rules::LineCj);
        assert_eq!(rules_for_locale(b"zh-CN"), Rules::LineCj);
        assert_eq!(rules_for_locale(b"ja"), Rules::LineNormal);
        assert_eq!(rules_for_locale(b"ko_KR"), Rules::LineNormal);
        assert_eq!(rules_for_locale(b"ja@lb=strict"), Rules::LineCj);
        assert_eq!(rules_for_locale(b"ja@lb=loose"), Rules::LineLooseCj);
        assert_eq!(rules_for_locale(b"zh@lb=normal"), Rules::LineNormalCj);
        assert_eq!(rules_for_locale(b"en@lb=normal"), Rules::LineNormal);
        assert_eq!(rules_for_locale(b"en@lb=bogus"), Rules::Line);
    }

    /// The fixture's text, with its `\uXXXX` escapes.
    fn unescape(t: &str) -> Vec<u16> {
        let mut out = vec![];
        let mut rest = t;
        while let Some(i) = rest.find("\\u") {
            out.extend(rest[..i].encode_utf16());
            out.push(u16::from_str_radix(&rest[i + 2..i + 6], 16).unwrap());
            rest = &rest[i + 6..];
        }
        out.extend(rest.encode_utf16());
        out
    }

    /// The pieces TeX Live 2026's xetex (ICU 78.2) cuts native words into
    /// under `\XeTeXlinebreaklocale`, one word per line of
    /// `linebreak_xetex.tsv`: root, `zh`, `zh_TW`, `ja`, `ko`, the `lb`
    /// keyword, Thai, Lao, Khmer and Myanmar (ICU's dictionaries), Tibetan,
    /// Hebrew, Arabic, Devanagari, emoji and format characters.
    #[test]
    fn boundaries_as_xetex_cuts_words() {
        let mut bad = vec![];
        let mut n = 0;
        for line in include_str!("linebreak_xetex.tsv").lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split('\t').collect();
            let want: Vec<i32> = f[2].split(',').map(|x| x.parse().unwrap()).collect();
            let got = boundaries(rules_for_locale(f[0].as_bytes()), &unescape(f[1]));
            n += 1;
            if got != want {
                bad.push(format!("{} {:?}: xetex {:?}, here {:?}", f[0], f[1], want, got));
            }
        }
        assert!(n >= 30, "{n} fixture lines");
        assert!(bad.is_empty(), "{} of {n} differ:\n{}", bad.len(), bad.join("\n"));
        assert_eq!(boundaries(Rules::Line, &[]), Vec::<i32>::new());
        assert_eq!(boundaries(Rules::LineCj, &u("ab")), vec![2]);
    }
}
