//! Engine detection: how a document signals that it expects XeLaTeX or
//! LuaLaTeX, what compile mode FlashTeX should choose, and precise
//! diagnostics for Lua code FlashTeX cannot execute.
//!
//! Detection is lexical (comments blanked, byte offsets preserved) and never
//! executes anything. It reports every signal with its byte offset and line
//! so an IDE can underline it.

use crate::engine::Engine;
use crate::keyval::{self, ArgScanner};

/// What a single piece of source says about the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    /// `% !TEX program = xelatex` / `TS-program` magic comment (explicit choice).
    MagicComment(Engine),
    /// A package that only works under XeTeX/LuaTeX (`fontspec`, `unicode-math`,
    /// `polyglossia`, `mathspec`, `xunicode`, `xltxtra`, `realscripts`).
    UnicodeEnginePackage,
    /// A package that requires XeTeX specifically (`xeCJK`, `xltxtra`, `xunicode`, `mathspec`, `xepersian`, `bidi`).
    XeTeXOnlyPackage,
    /// A package that requires LuaTeX specifically (`luacode`, `luatexja`, `luaotfload`,
    /// `lua-visual-debug`, `luamplib`, `lualatex-math`, `selnolig`, `lua-ul`, `luatex85`).
    LuaTeXOnlyPackage,
    /// fontspec font-selection command (`\setmainfont`, `\newfontfamily`, ...).
    FontspecCommand,
    /// `\setmathfont` (unicode-math).
    MathFontCommand,
    /// An engine conditional (`\ifxetex`, `\ifluatex`, `\iftutex`, `\ifPDFTeX`, `\sys_if_engine_...`).
    EngineConditional,
    /// `\RequireXeTeX`, `\RequireLuaTeX`, `\RequireTUTeX` (iftex).
    EngineRequirement(Option<Engine>),
    /// A XeTeX primitive (`\XeTeXinputencoding`, `\XeTeXlinebreaklocale`, ...).
    XeTeXPrimitive,
    /// Lua code (`\directlua`, `\luaexec`, `\latelua`, `luacode` environments, ...).
    LuaCode,
    /// `inputenc`/`fontenc` with 8-bit encodings: written for pdfLaTeX (still
    /// compatible with the Unicode engines, which ignore `inputenc`).
    PdfTeXPackage,
    /// pdfTeX-only primitives (`\pdfglyphtounicode`, `\pdfmapfile`, `\pdfgentounicode`).
    PdfTeXPrimitive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub kind: SignalKind,
    pub byte_offset: usize,
    pub line: usize,
    /// The matched source text (command or package name).
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub byte_offset: usize,
    pub line: usize,
    pub code: &'static str,
    pub message: String,
}

/// How FlashTeX should compile the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileMode {
    /// pdfLaTeX semantics: 8-bit TFM fonts, `inputenc` UTF-8 → LICR.
    Legacy8Bit,
    /// Unicode-font mode emulating `engine`: UTF-8 characters straight to
    /// OpenType glyphs, fontspec/unicode-math active, `\ifxetex`/`\ifluatex`
    /// answer as that engine would.
    UnicodeFonts { engine: Engine },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Detection {
    pub mode: CompileMode,
    /// True when the choice came from an explicit magic comment or a hard
    /// engine requirement rather than being inferred.
    pub explicit: bool,
    pub signals: Vec<Signal>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Detection {
    pub fn has(&self, kind: SignalKind) -> bool {
        self.signals.iter().any(|s| s.kind == kind)
    }

    /// Truth value FlashTeX should give an engine conditional in this mode.
    pub fn conditional_value(&self, name: &str) -> Option<bool> {
        let engine = match self.mode {
            CompileMode::Legacy8Bit => Engine::PdfTeX,
            CompileMode::UnicodeFonts { engine } => engine,
        };
        let n = name.trim_start_matches('\\');
        Some(match n {
            "ifxetex" | "ifXeTeX" => engine == Engine::XeTeX,
            "ifluatex" | "ifLuaTeX" => engine == Engine::LuaTeX,
            "iftutex" | "ifTUTeX" => engine.is_unicode(),
            "ifpdftex" | "ifPDFTeX" | "ifpdf" => engine == Engine::PdfTeX,
            _ => return None,
        })
    }
}

const UNICODE_PACKAGES: &[&str] = &[
    "fontspec",
    "unicode-math",
    "polyglossia",
    "realscripts",
    "fontsetup",
    "libertinus-otf",
];
const XETEX_PACKAGES: &[&str] = &[
    "xecjk",
    "xltxtra",
    "xunicode",
    "mathspec",
    "xepersian",
    "bidi",
    "xeindex",
    "xesearch",
    "xetexko",
    "ucharclasses",
];
const LUATEX_PACKAGES: &[&str] = &[
    "luacode",
    "luatexja",
    "luatexja-fontspec",
    "luaotfload",
    "lua-visual-debug",
    "luamplib",
    "lualatex-math",
    "selnolig",
    "lua-ul",
    "luatex85",
    "luacolor",
    "chickenize",
    "pyluatex",
    "lua-widow-control",
    "luaquotes",
];
const PDFTEX_PACKAGES: &[&str] = &["inputenc", "fontenc", "pdftexcmds", "microtype-pdftex"];
const FONTSPEC_COMMANDS: &[&str] = &[
    "setmainfont",
    "setsansfont",
    "setmonofont",
    "newfontfamily",
    "newfontface",
    "fontspec",
    "setromanfont",
    "defaultfontfeatures",
    "addfontfeatures",
    "addfontfeature",
    "setmathrm",
    "setmathsf",
    "setmathtt",
    "setboldmathrm",
    "newfontscript",
    "newfontlanguage",
    "setfontfamily",
    "renewfontfamily",
    "providefontfamily",
    "setfontface",
    "renewfontface",
];
const CONDITIONALS: &[&str] = &[
    "ifxetex", "ifXeTeX", "ifluatex", "ifLuaTeX", "iftutex", "ifTUTeX", "ifpdftex", "ifPDFTeX",
    "ifpdf",
];
const LUA_COMMANDS: &[&str] = &[
    "directlua",
    "luaexec",
    "latelua",
    "luadirect",
    "luastring",
    "luastringN",
    "luastringO",
    "luaescapestring",
    "luafunction",
    "luadef",
    "lateluafunction",
];
const LUA_ENVIRONMENTS: &[&str] = &["luacode", "luacode*", "luacodestar"];
const XETEX_PRIMITIVE_PREFIX: &str = "XeTeX";
const PDFTEX_PRIMITIVES: &[&str] = &[
    "pdfglyphtounicode",
    "pdfmapfile",
    "pdfmapline",
    "pdfgentounicode",
    "pdfinclusioncopyfonts",
];

/// Scans `src` (a whole `.tex` file) and decides the compile mode.
pub fn detect(src: &str) -> Detection {
    let mut signals = Vec::new();
    let mut diagnostics = Vec::new();
    let lines = LineIndex::new(src);

    // Magic comments live in comments, so scan the raw text first.
    for (off, line) in line_starts(src) {
        let t = line.trim_start();
        if !t.starts_with('%') {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if let Some(i) = lower.find("!tex") {
            let rest = &lower[i..];
            if rest.contains("program") {
                let value = rest.split('=').nth(1).map(str::trim).unwrap_or("");
                let engine = if value.starts_with("xelatex") || value.starts_with("xetex") {
                    Some(Engine::XeTeX)
                } else if value.starts_with("lualatex")
                    || value.starts_with("luatex")
                    || value.starts_with("luahblatex")
                {
                    Some(Engine::LuaTeX)
                } else if value.starts_with("pdflatex") || value.starts_with("pdftex") {
                    Some(Engine::PdfTeX)
                } else {
                    None
                };
                if let Some(e) = engine {
                    signals.push(Signal {
                        kind: SignalKind::MagicComment(e),
                        byte_offset: off,
                        line: lines.line(off),
                        text: t.trim_end().to_string(),
                    });
                }
            }
        }
    }

    let code = keyval::blank_comments(src);
    let bytes = code.as_bytes();
    let mut i = 0;
    let mut lua_env_open: Option<usize> = None;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            i += 1;
            continue;
        }
        let start = i;
        let mut j = i + 1;
        while j < bytes.len() && (bytes[j].is_ascii_alphabetic() || bytes[j] == b'@') {
            j += 1;
        }
        if j == i + 1 {
            i += 2;
            continue;
        }
        let name = &code[i + 1..j];
        let line = lines.line(start);
        let push = |kind: SignalKind, text: &str, signals: &mut Vec<Signal>| {
            signals.push(Signal {
                kind,
                byte_offset: start,
                line,
                text: text.to_string(),
            });
        };
        match name {
            "usepackage" | "RequirePackage" | "RequirePackageWithOptions" => {
                let mut a = ArgScanner::new(&code, j);
                let _ = a.optional();
                if let Some(arg) = a.mandatory() {
                    for pkg in arg.split(',').map(|p| p.trim()) {
                        let lower = pkg.to_ascii_lowercase();
                        if UNICODE_PACKAGES.contains(&lower.as_str()) {
                            push(SignalKind::UnicodeEnginePackage, pkg, &mut signals);
                        } else if XETEX_PACKAGES.contains(&lower.as_str()) {
                            push(SignalKind::XeTeXOnlyPackage, pkg, &mut signals);
                        } else if LUATEX_PACKAGES.contains(&lower.as_str()) {
                            push(SignalKind::LuaTeXOnlyPackage, pkg, &mut signals);
                        } else if PDFTEX_PACKAGES.contains(&lower.as_str()) {
                            push(SignalKind::PdfTeXPackage, pkg, &mut signals);
                        }
                    }
                    j = a.pos;
                }
            }
            "setmathfont" => push(SignalKind::MathFontCommand, name, &mut signals),
            n if FONTSPEC_COMMANDS.contains(&n) => {
                push(SignalKind::FontspecCommand, n, &mut signals)
            }
            n if CONDITIONALS.contains(&n) => push(SignalKind::EngineConditional, n, &mut signals),
            "RequireXeTeX" => push(
                SignalKind::EngineRequirement(Some(Engine::XeTeX)),
                name,
                &mut signals,
            ),
            "RequireLuaTeX" => push(
                SignalKind::EngineRequirement(Some(Engine::LuaTeX)),
                name,
                &mut signals,
            ),
            "RequireTUTeX" => push(SignalKind::EngineRequirement(None), name, &mut signals),
            "RequirePDFTeX" => push(
                SignalKind::EngineRequirement(Some(Engine::PdfTeX)),
                name,
                &mut signals,
            ),
            n if n.starts_with("sys_if_engine_") => {
                push(SignalKind::EngineConditional, n, &mut signals)
            }
            n if n.starts_with(XETEX_PRIMITIVE_PREFIX) => {
                push(SignalKind::XeTeXPrimitive, n, &mut signals)
            }
            n if PDFTEX_PRIMITIVES.contains(&n) => {
                push(SignalKind::PdfTeXPrimitive, n, &mut signals)
            }
            n if LUA_COMMANDS.contains(&n) => {
                push(SignalKind::LuaCode, n, &mut signals);
                let mut a = ArgScanner::new(&code, j);
                let snippet = a.mandatory().unwrap_or("");
                diagnostics.push(lua_diagnostic(start, line, n, snippet));
            }
            "begin" | "end" => {
                let mut a = ArgScanner::new(&code, j);
                if let Some(env) = a.mandatory() {
                    if LUA_ENVIRONMENTS.contains(&env) {
                        if name == "begin" {
                            push(SignalKind::LuaCode, env, &mut signals);
                            lua_env_open = Some(start);
                        } else if let Some(open) = lua_env_open.take() {
                            let body = &code[open..start];
                            let body = body.split_once('}').map(|x| x.1).unwrap_or(body);
                            diagnostics.push(lua_diagnostic(open, lines.line(open), env, body));
                        }
                    }
                    j = a.pos;
                }
            }
            _ => {}
        }
        i = j;
    }
    if let Some(open) = lua_env_open {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            byte_offset: open,
            line: lines.line(open),
            code: "lua-environment-unterminated",
            message: "luacode environment is never closed".into(),
        });
    }

    // Lua code inside an engine conditional is usually guarded with a
    // non-Lua alternative: downgrade those to warnings.
    if signals
        .iter()
        .any(|s| s.kind == SignalKind::EngineConditional)
    {
        for d in &mut diagnostics {
            if d.code == "lua-code-unsupported" && inside_conditional(&code, d.byte_offset) {
                d.severity = Severity::Warning;
                d.message.push_str(
                    " (inside an engine conditional; FlashTeX takes the non-LuaTeX branch)",
                );
            }
        }
    }

    let (mode, explicit) = decide(&signals, &mut diagnostics, &lines);
    Detection {
        mode,
        explicit,
        signals,
        diagnostics,
    }
}

fn decide(
    signals: &[Signal],
    diagnostics: &mut Vec<Diagnostic>,
    _lines: &LineIndex,
) -> (CompileMode, bool) {
    let magic = signals.iter().rev().find_map(|s| match s.kind {
        SignalKind::MagicComment(e) => Some(e),
        _ => None,
    });
    let required = signals.iter().find_map(|s| match s.kind {
        SignalKind::EngineRequirement(Some(e)) => Some(e),
        _ => None,
    });
    let any = |k: SignalKind| signals.iter().any(|s| s.kind == k);
    let xe_only = any(SignalKind::XeTeXOnlyPackage) || any(SignalKind::XeTeXPrimitive);
    let lua_only =
        any(SignalKind::LuaTeXOnlyPackage) || signals.iter().any(|s| s.kind == SignalKind::LuaCode);
    let unicode = any(SignalKind::UnicodeEnginePackage)
        || any(SignalKind::FontspecCommand)
        || any(SignalKind::MathFontCommand)
        || any(SignalKind::EngineRequirement(None));

    if xe_only && lua_only {
        let at = signals
            .iter()
            .find(|s| matches!(s.kind, SignalKind::LuaTeXOnlyPackage | SignalKind::LuaCode))
            .unwrap();
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            byte_offset: at.byte_offset,
            line: at.line,
            code: "conflicting-engine-signals",
            message: "document uses both XeTeX-only and LuaTeX-only features; emulating XeTeX"
                .into(),
        });
    }

    if let Some(e) = required.or(magic) {
        if e == Engine::PdfTeX && (unicode || xe_only || lua_only) {
            let at = signals
                .iter()
                .find(|s| {
                    !matches!(
                        s.kind,
                        SignalKind::MagicComment(_) | SignalKind::EngineRequirement(_)
                    )
                })
                .unwrap();
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                byte_offset: at.byte_offset,
                line: at.line,
                code: "pdftex-with-unicode-engine-package",
                message: format!(
                    "`{}` requires XeLaTeX or LuaLaTeX but the document selects pdfLaTeX",
                    at.text
                ),
            });
        }
        let mode = if e == Engine::PdfTeX {
            CompileMode::Legacy8Bit
        } else {
            CompileMode::UnicodeFonts { engine: e }
        };
        return (mode, true);
    }
    if xe_only {
        return (
            CompileMode::UnicodeFonts {
                engine: Engine::XeTeX,
            },
            false,
        );
    }
    if lua_only {
        return (
            CompileMode::UnicodeFonts {
                engine: Engine::LuaTeX,
            },
            false,
        );
    }
    if unicode {
        // fontspec/unicode-math documents compile with either engine. XeTeX is
        // the reference: its HarfBuzz shaping is closest to what FlashTeX
        // implements and has no Lua callbacks.
        return (
            CompileMode::UnicodeFonts {
                engine: Engine::XeTeX,
            },
            false,
        );
    }
    (CompileMode::Legacy8Bit, false)
}

fn lua_diagnostic(offset: usize, line: usize, construct: &str, snippet: &str) -> Diagnostic {
    let mut s: String = snippet.trim().chars().take(80).collect();
    if snippet.trim().chars().count() > 80 {
        s.push('…');
    }
    Diagnostic {
        severity: Severity::Error,
        byte_offset: offset,
        line,
        code: "lua-code-unsupported",
        message: format!(
            "`{construct}` runs Lua code, which FlashTeX does not execute; its output (\"{s}\") will be missing. \
             Replace it with TeX macros or precompute the result"
        ),
    }
}

fn inside_conditional(code: &str, offset: usize) -> bool {
    // Nearest preceding engine conditional without its \fi before `offset`.
    let before = &code[..offset];
    let last_if = CONDITIONALS
        .iter()
        .filter_map(|c| before.rfind(&format!("\\{c}")))
        .max();
    match last_if {
        Some(p) => !before[p..].contains("\\fi"),
        None => false,
    }
}

struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    fn new(src: &str) -> LineIndex {
        let mut starts = vec![0];
        starts.extend(src.match_indices('\n').map(|(i, _)| i + 1));
        LineIndex { starts }
    }
    /// 1-based line of `offset`.
    fn line(&self, offset: usize) -> usize {
        match self.starts.binary_search(&offset) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    }
}

fn line_starts(src: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut off = 0;
    src.split_inclusive('\n').map(move |l| {
        let o = off;
        off += l.len();
        (o, l)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_article_is_legacy() {
        let d = detect(
            "\\documentclass{article}\n\\usepackage[utf8]{inputenc}\n\\begin{document}é\\end{document}",
        );
        assert_eq!(d.mode, CompileMode::Legacy8Bit);
        assert!(d.has(SignalKind::PdfTeXPackage));
    }

    #[test]
    fn fontspec_selects_unicode_xetex() {
        let d = detect(
            "\\documentclass{article}\n\\usepackage{amsmath,fontspec}\n\\setmainfont{TeX Gyre Termes}\n",
        );
        assert_eq!(
            d.mode,
            CompileMode::UnicodeFonts {
                engine: Engine::XeTeX
            }
        );
        assert!(!d.explicit);
        assert_eq!(
            d.signals
                .iter()
                .filter(|s| s.kind == SignalKind::FontspecCommand)
                .count(),
            1
        );
        assert_eq!(d.signals[0].line, 2);
    }

    #[test]
    fn magic_comment_wins() {
        let d = detect("% !TEX program = lualatex\n\\documentclass{article}\\usepackage{fontspec}");
        assert_eq!(
            d.mode,
            CompileMode::UnicodeFonts {
                engine: Engine::LuaTeX
            }
        );
        assert!(d.explicit);
        assert_eq!(d.conditional_value("\\ifluatex"), Some(true));
        assert_eq!(d.conditional_value("\\ifxetex"), Some(false));
        assert_eq!(d.conditional_value("\\iftutex"), Some(true));
    }

    #[test]
    fn lua_code_is_diagnosed_with_location() {
        let src = "\\documentclass{article}\n\\begin{document}\n\\directlua{tex.print(1+1)}\n\\begin{luacode}\nx = 1\n\\end{luacode}\n\\end{document}";
        let d = detect(src);
        assert_eq!(
            d.mode,
            CompileMode::UnicodeFonts {
                engine: Engine::LuaTeX
            }
        );
        let errs: Vec<_> = d
            .diagnostics
            .iter()
            .filter(|x| x.code == "lua-code-unsupported")
            .collect();
        assert_eq!(errs.len(), 2);
        assert_eq!(errs[0].line, 3);
        assert!(errs[0].message.contains("tex.print(1+1)"));
        assert_eq!(errs[1].line, 4);
        assert!(errs[1].message.contains("x = 1"));
    }

    #[test]
    fn guarded_lua_is_a_warning_and_comments_ignored() {
        let src =
            "% \\directlua{nope}\n\\usepackage{iftex}\\ifluatex\\directlua{a()}\\else\\relax\\fi";
        let d = detect(src);
        assert_eq!(d.diagnostics.len(), 1);
        assert_eq!(d.diagnostics[0].severity, Severity::Warning);
        assert_eq!(d.diagnostics[0].line, 2);
    }

    #[test]
    fn pdflatex_with_fontspec_is_an_error() {
        let d = detect("% !TEX program = pdflatex\n\\usepackage{fontspec}");
        assert_eq!(d.mode, CompileMode::Legacy8Bit);
        assert!(d.diagnostics.iter().any(
            |x| x.code == "pdftex-with-unicode-engine-package" && x.severity == Severity::Error
        ));
    }

    #[test]
    fn xetex_only_package() {
        let d = detect("\\usepackage{xeCJK}\\usepackage{fontspec}");
        assert_eq!(
            d.mode,
            CompileMode::UnicodeFonts {
                engine: Engine::XeTeX
            }
        );
    }
}
