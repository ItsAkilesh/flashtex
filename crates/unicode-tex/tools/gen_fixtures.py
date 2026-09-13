#!/usr/bin/env python3
"""Generate the KC-104 XeLaTeX/LuaLaTeX oracle fixtures.

Writes fixtures/specs.json (the machine-readable fixture list read by
`cargo test`) and fixtures/tex/<id>.tex (the documents compiled by
tools/run_oracle.py). Every body line is its own `\\noindent ...\\par`
paragraph, shorter than \\textwidth, so its interword glue stays at natural
width (the paragraph's last line ends with \\parfillskip); line breaking is
therefore not exercised here, only widths, glue and positions inside a line.

TeX is an oracle only: cargo tests read the committed JSON and never run TeX.
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

TERMES = ("\\setmainfont{texgyretermes}[Extension=.otf,UprightFont=*-regular,"
          "BoldFont=*-bold,ItalicFont=*-italic,BoldItalicFont=*-bolditalic%s]")
PAGELLA = ("\\setmainfont{texgyrepagella}[Extension=.otf,UprightFont=*-regular,"
           "BoldFont=*-bold%s]")

PANGRAM = [
    "The quick brown fox jumps over the lazy dog.",
    "AVAST Wavy Toffee office, affine fjord flow.",
    "Typography: kerning Yo To LT WA pairs; fine.",
]
TEXLIG = [
    "``Quoted'' text --- and -- dashes, `single' too.",
    "Guillemets <<ici>> and ,,low'' plus !`hola ?`que.",
    "Straight \"double\" and it's done.",
]
DIGITS = ["Numbers 0123456789 and 2026, 1.5 or 42.", "Office 3141 affluent 7 fjords."]
SPACEFACTOR = [
    "Mr. Smith went home. Then U.S. Army came!",
    "Yes? No: maybe; ok, fine. (Really.) End.",
]
UNICODE_ACCENTS = ["Café naïve résumé Zürich façade.", "Ærøskøbing Øresund Łódź Šťastný ĳ."]
UNICODE_COMBINING = ["Café naïve résumé done.", "Zürich Å Å."]
UNICODE_SCRIPTS = ["Αλφα βήτα γάμμα δέλτα.", "Кириллица текст здесь."]
UNICODE_PUNCT = ["“Unicode” quotes ‘single’ – en — em … end."]

MATH_ALPHA_LINES = [
    "$\\mathbb{RZN}$",
    "$\\mathcal{ABL}$",
    "$\\mathscr{AB}$",
    "$\\mathfrak{gA}$",
    "$\\symbf{xA}$",
    "$\\symbfit{vA}$",
    "$\\symsf{Ab}$",
    "$\\symtt{ab}$",
    "$\\symbfsf{Ab}$",
    "$\\mathbf{xA}$",
    "$\\mathit{hA}$",
    "$\\mathrm{dA}$",
    "$h x A$",
    "$\\alpha\\beta\\Gamma\\Delta$",
    "$\\symbf{\\alpha\\Gamma}$",
    "$\\partial\\nabla$",
    "$\\mathbb{1}\\symbf{2}$",
]

fixtures = []


def text(fid, preamble, lines, body_prefix="", doc_opts="", notes=""):
    fixtures.append({
        "id": fid, "kind": "text", "class_options": doc_opts,
        "preamble": preamble, "body_prefix": body_prefix, "lines": lines,
        "notes": notes,
    })


def math(fid, preamble, lines=MATH_ALPHA_LINES, notes=""):
    fixtures.append({
        "id": fid, "kind": "math", "class_options": "",
        "preamble": preamble, "body_prefix": "", "lines": lines, "notes": notes,
    })


# --- system fonts (macOS) ---------------------------------------------------
text("tnr_main", ["\\setmainfont{Times New Roman}"], PANGRAM + TEXLIG)
text("tnr_newfamily_notex", ["\\newfontfamily\\tnr{Times New Roman}"], PANGRAM + TEXLIG,
     body_prefix="\\tnr ", notes="\\newfontfamily gets no Ligatures=TeX default")
text("tnr_bold", ["\\setmainfont{Times New Roman}"], PANGRAM, body_prefix="\\bfseries ")
text("tnr_italic", ["\\setmainfont{Times New Roman}"], PANGRAM, body_prefix="\\itshape ")
text("tnr_bolditalic", ["\\setmainfont{Times New Roman}"], PANGRAM,
     body_prefix="\\bfseries\\itshape ")
text("tnr_color", ["\\setmainfont{Times New Roman}[Color=FF0000]"], PANGRAM[:2])
text("tnr_spacefactor", ["\\setmainfont{Times New Roman}"], SPACEFACTOR)
text("tnr_frenchspacing", ["\\setmainfont{Times New Roman}"], SPACEFACTOR,
     body_prefix="\\frenchspacing ")
text("tnr_accents", ["\\setmainfont{Times New Roman}"], UNICODE_ACCENTS)
text("tnr_combining", ["\\setmainfont{Times New Roman}"], UNICODE_COMBINING)
text("tnr_greek_cyrillic", ["\\setmainfont{Times New Roman}"], UNICODE_SCRIPTS)
text("tnr_12pt", ["\\setmainfont{Times New Roman}"], PANGRAM[:2], doc_opts="12pt")
text("helvneue_main", ["\\setmainfont{Helvetica Neue}"], PANGRAM)
text("helvneue_bold", ["\\setmainfont{Helvetica Neue}"], PANGRAM[:2], body_prefix="\\bfseries ")
text("helvneue_matchlowercase", ["\\setsansfont{Helvetica Neue}[Scale=MatchLowercase]"],
     PANGRAM[:2], body_prefix="\\sffamily ")
text("menlo_mono", ["\\setmonofont{Menlo}"], PANGRAM[:2] + SPACEFACTOR[:1],
     body_prefix="\\ttfamily ")
text("menlo_fontspec", ["\\newfontfamily\\menlo{Menlo}"], PANGRAM[:2], body_prefix="\\menlo ")
text("georgia_main", ["\\setmainfont{Georgia}"], DIGITS + PANGRAM[:1])
text("georgia_lining", ["\\setmainfont{Georgia}[Numbers=Lining]"], DIGITS)
text("arial_sans", ["\\setsansfont{Arial}"], PANGRAM, body_prefix="\\sffamily ")
text("couriernew_mono", ["\\setmonofont{Courier New}"], PANGRAM[:2], body_prefix="\\ttfamily ")

# --- TeX Gyre / Latin Modern OpenType by file name --------------------------
text("termes_main", [TERMES % ""], PANGRAM + TEXLIG)
text("termes_bold", [TERMES % ""], PANGRAM, body_prefix="\\bfseries ")
text("termes_italic", [TERMES % ""], PANGRAM, body_prefix="\\itshape ")
text("termes_smallcaps", [TERMES % ",Letters=SmallCaps"], PANGRAM[:2])
text("termes_rawfeature_smcp", [TERMES % ",RawFeature=+smcp"], PANGRAM[:2])
text("termes_nocommon", [TERMES % ",Ligatures=NoCommon"], PANGRAM[:2])
text("termes_rare", [TERMES % ",Ligatures=Rare"], PANGRAM[:2])
text("termes_letterspace", [TERMES % ",LetterSpace=5"], PANGRAM[:2])
text("termes_wordspace", [TERMES % ",WordSpace=1.5"], PANGRAM[:2])
text("termes_wordspace3", [TERMES % ",WordSpace={2,0,0}"], PANGRAM[:2])
text("termes_scale", [TERMES % ",Scale=0.9"], PANGRAM[:2])
text("termes_oldstyle", [TERMES % ",Numbers=OldStyle"], DIGITS)
text("termes_unicode_punct", [TERMES % ""], UNICODE_PUNCT + UNICODE_ACCENTS)
text("termes_spacefactor", [TERMES % ""], SPACEFACTOR)
text("pagella_oldstyle", [PAGELLA % ",Numbers=OldStyle"], DIGITS)
text("pagella_proportional", [PAGELLA % ",Numbers={Proportional,OldStyle}"], DIGITS)
text("heros_sans", ["\\setsansfont{texgyreheros}[Extension=.otf,UprightFont=*-regular]"],
     PANGRAM, body_prefix="\\sffamily ")
text("cursor_mono", ["\\setmonofont{texgyrecursor}[Extension=.otf,UprightFont=*-regular]"],
     PANGRAM[:2], body_prefix="\\ttfamily ")
text("lm_default", [], PANGRAM + TEXLIG, notes="fontspec loaded, no \\setmainfont: Latin Modern Roman OTF")
text("lm_accents", [], UNICODE_ACCENTS + UNICODE_COMBINING)
text("lm_12pt", [], PANGRAM[:2], doc_opts="12pt", notes="optical size lmroman12")
text("lm_file_newfamily", ["\\newfontfamily\\lmr{lmroman10-regular.otf}"], PANGRAM[:1] + TEXLIG[:1],
     body_prefix="\\lmr ")

# --- unicode-math -----------------------------------------------------------
math("math_lm", ["\\usepackage{unicode-math}", "\\setmathfont{latinmodern-math.otf}"])
math("math_lm_iso", ["\\usepackage{unicode-math}", "\\setmathfont{latinmodern-math.otf}[math-style=ISO]"])
math("math_lm_french", ["\\usepackage{unicode-math}", "\\setmathfont{latinmodern-math.otf}[math-style=french]"])
math("math_lm_upright", ["\\usepackage{unicode-math}", "\\setmathfont{latinmodern-math.otf}[math-style=upright]"])
math("math_lm_boldiso", ["\\usepackage{unicode-math}", "\\setmathfont{latinmodern-math.otf}[bold-style=ISO]"])
math("math_stix2", ["\\usepackage{unicode-math}", "\\setmathfont{STIXTwoMath-Regular.otf}"])
math("math_newcm", ["\\usepackage{unicode-math}", "\\setmathfont{NewCMMath-Regular.otf}"])
math("math_lm_mathbf_sym", ["\\usepackage[mathbf=sym]{unicode-math}", "\\setmathfont{latinmodern-math.otf}"])

TEMPLATE = r"""\documentclass%(opts)s{article}
\usepackage{fontspec}
%(preamble)s
\pagestyle{empty}
\setlength{\parindent}{0pt}
\makeatletter
\newcommand\kcprobe{%%
  \typeout{KCFONT \fontname\font}%%
  \typeout{KCDIMS size=\the\fontdimen6\font\space space=\the\fontdimen2\font\space stretch=\the\fontdimen3\font\space shrink=\the\fontdimen4\font\space extra=\the\fontdimen7\font\space xheight=\the\fontdimen5\font}}
\newcommand\kcmathprobe{%%
  \ifcsname Umathaxis\endcsname
  \typeout{KCUMATH axis=\the\Umathaxis\textstyle\space fractionrule=\the\Umathfractionrule\textstyle\space supshiftup=\the\Umathsupshiftup\textstyle\space subshiftdown=\the\Umathsubshiftdown\textstyle\space radicalrule=\the\Umathradicalrule\textstyle\space radicalvgap=\the\Umathradicalvgap\textstyle\space radicalvgapdisplay=\the\Umathradicalvgap\displaystyle\space overbarrule=\the\Umathoverbarrule\textstyle\space stackvgap=\the\Umathstackvgap\textstyle\space stackvgapdisplay=\the\Umathstackvgap\displaystyle\space subsupvgap=\the\Umathsubsupvgap\textstyle\space supbottommin=\the\Umathsupbottommin\textstyle\space subtopmax=\the\Umathsubtopmax\textstyle\space spaceafterscript=\the\Umathspaceafterscript\textstyle\space limitabovevgap=\the\Umathlimitabovevgap\textstyle\space limitbelowvgap=\the\Umathlimitbelowvgap\textstyle\space fractionnumup=\the\Umathfractionnumup\textstyle\space fractionnumupdisplay=\the\Umathfractionnumup\displaystyle\space fractiondenomdown=\the\Umathfractiondenomdown\textstyle\space fractiondenomdowndisplay=\the\Umathfractiondenomdown\displaystyle}%%
  \fi}
\makeatother
\begin{document}
%(body)s
\end{document}
"""


def render(fx):
    body = []
    for i, line in enumerate(fx["lines"]):
        probe = "\\kcprobe" if fx["kind"] == "text" else "\\kcmathprobe"
        extra = probe + "{}" if i == 0 else ""
        body.append("\\noindent %s%s%s\\par" % (fx["body_prefix"], extra, line) if fx["kind"] == "text"
                    else "\\noindent %s%s\\par" % (line, "\\(\\kcmathprobe\\)" if i == 0 else ""))
    opts = "[%s]" % fx["class_options"] if fx["class_options"] else ""
    return TEMPLATE % {"opts": opts, "preamble": "\n".join(fx["preamble"]), "body": "\n".join(body)}


def main():
    os.makedirs(os.path.join(ROOT, "fixtures", "tex"), exist_ok=True)
    for fx in fixtures:
        with open(os.path.join(ROOT, "fixtures", "tex", fx["id"] + ".tex"), "w", encoding="utf-8") as f:
            f.write(render(fx))
    with open(os.path.join(ROOT, "fixtures", "specs.json"), "w", encoding="utf-8") as f:
        json.dump({"schema": 1, "fixtures": fixtures}, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("wrote %d fixtures" % len(fixtures))


if __name__ == "__main__":
    main()
