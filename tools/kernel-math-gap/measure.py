#!/usr/bin/env python3
"""pdflatex oracle for the LaTeX kernel math commands FlashTeX does not support.

Method, as used by #200/#201: the width of `\\hbox{$\\sym$}` pins the family
and slot a command is declared with; the width of `\\hbox{$a\\sym b$}` against
the `$ab$` control pins its *atom class*, because the inter-atom glue TeX
inserts is a function of the class alone.

pdflatex is an **oracle only** and is never in the product path. Nothing here
is a parity claim. Python 3 standard library only; needs `pdflatex` on PATH.

    python3 tools/kernel-math-gap/measure.py            # the whole table
    python3 tools/kernel-math-gap/measure.py --tex-only # skip the classes
"""
import argparse
import os
import re
import subprocess
import sys
import tempfile

# Inter-atom glue at 10pt in text style. `mu` is 1/18 of the family-2 quad,
# which is 10pt for cmsy10 at 10pt, so 1mu = 0.55556pt. The gap below is the
# TOTAL added by `a<sym>b` over `ab` — one glue on each side where TeX puts
# one on each side.
MU = 10.0 / 18.0
CLASS_GAP = {
    "Ord": 0.0,
    "Op": 2 * 3 * MU,      # thinmuskip both sides
    "Bin": 2 * 4 * MU,     # medmuskip both sides
    "Rel": 2 * 5 * MU,     # thickmuskip both sides
    "Open": 0.0,
    "Close": 0.0,
    "Punct": 3 * MU,       # thinmuskip after only
    "Inner": 2 * 3 * MU,
}

# --------------------------------------------------------------------------
# What the TeX sources declare. Every row here is quoted from the file named,
# in TeX Live 2025; `measure.py` checks the declaration against pdflatex.
# --------------------------------------------------------------------------

# latexsym.sty (LaTeX base, v2.2e). All eleven; `\sqsubset`/`\sqsupset` are
# also declared by amsfonts.sty, which is why they are listed separately.
LATEXSYM = [
    # command      class   lasy10 slot
    ("mho",        "Ord",  0x30),
    ("Join",       "Rel",  0x31),
    ("Box",        "Ord",  0x32),
    ("Diamond",    "Ord",  0x33),
    ("leadsto",    "Rel",  0x3B),
    ("lhd",        "Bin",  0x01),
    ("unlhd",      "Bin",  0x02),
    ("rhd",        "Bin",  0x03),
    ("unrhd",      "Bin",  0x04),
]
LATEXSYM_ALSO = [("sqsubset", "Rel", 0x3C), ("sqsupset", "Rel", 0x3D)]

# The amssymb/amsfonts command usually recommended in place of each latexsym
# one, with the class *that* command carries.
AMS_STANDIN = {
    "mho":     ("mho", "Ord", "amsfonts.sty 101, AMSb \"66"),
    "Join":    (None,  None,  "no amssymb equivalent exists"),
    "Box":     ("square", "Ord", "amssymb.sty 47, AMSa \"03"),
    "Diamond": ("lozenge", "Ord", "amssymb.sty 50, AMSa \"06"),
    "leadsto": ("rightsquigarrow", "Rel", "amssymb.sty 78, AMSa \"20"),
    "lhd":     ("vartriangleleft", "Rel", "amssymb.sty 114, AMSa \"43"),
    "unlhd":   ("trianglelefteq", "Rel", "amssymb.sty 116, AMSa \"45"),
    "rhd":     ("vartriangleright", "Rel", "amssymb.sty 113, AMSa \"42"),
    "unrhd":   ("trianglerighteq", "Rel", "amssymb.sty 115, AMSa \"44"),
}

# fontmath.ltx 340-341, 374-377, 447-450: the seven pieces that exist only to
# build another command. Each is a plain \DeclareMathSymbol, NOT an extensible
# recipe part -- see the README.
PIECES = [
    # command       class  family         slot  cm glyph name    builds
    ("lhook",       "Rel", "letters",     0x2C, "arrowhookleft",      r"\hookrightarrow"),
    ("rhook",       "Rel", "letters",     0x2D, "arrowhookright",     r"\hookleftarrow"),
    ("mapstochar",  "Rel", "symbols",     0x37, "mapsto",             r"\mapsto, \longmapsto"),
    ("braceld",     "Ord", "largesymbols", 0x7A, "bracehtipdownleft",  r"\down/\upbracefill"),
    ("bracerd",     "Ord", "largesymbols", 0x7B, "bracehtipdownright", r"\down/\upbracefill"),
    ("bracelu",     "Ord", "largesymbols", 0x7C, "bracehtipupleft",    r"\down/\upbracefill"),
    ("braceru",     "Ord", "largesymbols", 0x7D, "bracehtipupright",   r"\down/\upbracefill"),
]

# The whole each group of pieces composes, and the composition fontmath gives.
COMPOSED = [
    ("hookrightarrow", r"\lhook\joinrel\rightarrow"),
    ("hookleftarrow",  r"\leftarrow\joinrel\rhook"),
    ("mapsto",         r"\mapstochar\rightarrow"),
    ("longmapsto",     r"\mapstochar\longrightarrow"),
]


def pdflatex(items, preamble="", size="10pt"):
    """items: [(label, tex)]. -> {label: (wd, ht, dp)} in pt."""
    body = "\n".join(
        r"\setbox0=\hbox{$%s$}\message{^^JFTMEAS|%s|\the\wd0|\the\ht0|\the\dp0|^^J}"
        % (tex, label) for label, tex in items)
    src = ("\\documentclass[%s]{article}\n%s\n\\begin{document}\n%s\n"
           "\\end{document}\n" % (size, preamble, body))
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "m.tex")
        with open(p, "w") as fh:
            fh.write(src)
        r = subprocess.run(
            ["pdflatex", "-interaction=nonstopmode", "-output-directory", d, p],
            capture_output=True, text=True)
        out = r.stdout + r.stderr
    res = {}
    for m in re.finditer(
            r"FTMEAS\|(\S+?)\|([-\d.]+)pt\|([-\d.]+)pt\|([-\d.]+)pt\|", out):
        res[m.group(1)] = tuple(float(m.group(i)) for i in (2, 3, 4))
    for label, _ in items:
        if label not in res:
            sys.stderr.write("  !! no measurement for %r\n" % label)
    return res


def classify(width, delta):
    """delta = wd($a SYM b$) - wd($ab$). -> (class, residual pt)."""
    gap = delta - width
    name, ideal = min(CLASS_GAP.items(), key=lambda kv: abs(kv[1] - gap))
    return name, gap - ideal


def grid(headers, rows):
    w = [max(len(str(r[i])) for r in [headers] + rows) for i in range(len(headers))]
    def fmt(r):
        return "  " + "  ".join(str(r[i]).ljust(w[i]) for i in range(len(headers)))
    print(fmt(headers))
    print("  " + "  ".join("-" * x for x in w))
    for r in rows:
        print(fmt(r))
    print()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tex-only", action="store_true",
                    help="widths only; skip the atom-class probe")
    a = ap.parse_args()

    ver = subprocess.run(["pdflatex", "--version"], capture_output=True,
                         text=True).stdout.splitlines()[0]
    print("oracle: %s\n" % ver)

    ctrl = pdflatex([("ab", "ab"), ("a", "a")], r"\usepackage{latexsym}")
    CTRL = ctrl["ab"][0]
    print("control  wd($ab$) = %.5f pt    wd($a$) = %.5f pt   (10pt)\n"
          % (CTRL, ctrl["a"][0]))

    # ---- latexsym -------------------------------------------------------
    print("== latexsym.sty, set from the real lasy10 ==\n")
    items = []
    for n, _, _ in LATEXSYM + LATEXSYM_ALSO:
        items.append((n, "\\" + n))
        if not a.tex_only:
            items.append((n + "X", "a\\%s b" % n))
    got = pdflatex(items, r"\usepackage{latexsym}")
    rows = []
    for n, cls, slot in LATEXSYM + LATEXSYM_ALSO:
        if n not in got:
            continue
        w = got[n][0]
        if a.tex_only:
            rows.append(("\\" + n, '"%02X' % slot, cls, "%.5f" % w, "", ""))
            continue
        m, res = classify(w, got[n + "X"][0] - CTRL)
        rows.append(("\\" + n, '"%02X' % slot, cls, "%.5f" % w, m,
                     "ok" if m == cls and abs(res) < 1e-3 else "MISMATCH"))
    grid(("command", "slot", "declared", "wd($sym$)", "measured", ""), rows)

    # ---- latexsym vs the amssymb stand-in -------------------------------
    print("== substituting the amssymb command usually recommended ==\n")
    subs = [(v[0], "\\" + v[0]) for v in AMS_STANDIN.values() if v[0]]
    amsw = pdflatex(subs, r"\usepackage{amssymb}")
    rows = []
    for n, cls, _ in LATEXSYM:
        eq, eqcls, where = AMS_STANDIN[n]
        if eq is None:
            rows.append(("\\" + n, "(none)", "%.5f" % got[n][0], "-", "-",
                         where))
            continue
        d = amsw[eq][0] - got[n][0]
        note = []
        if abs(d) > 0.005:
            note.append("advance %+.5f pt" % d)
        if eqcls != cls:
            note.append("class %s->%s = %+.5f pt of glue"
                        % (cls, eqcls, CLASS_GAP[eqcls] - CLASS_GAP[cls]))
        rows.append(("\\" + n, "\\" + eq, "%.5f" % got[n][0],
                     "%.5f" % amsw[eq][0], "%+.5f" % d,
                     "; ".join(note) if note else "exact"))
    grid(("latexsym", "stand-in", "lasy wd", "ams wd", "delta", "what breaks"),
         rows)

    # ---- the seven pieces ----------------------------------------------
    print("== the seven assembly pieces (plain \\DeclareMathSymbol rows) ==\n")
    items = []
    for n, _, _, _, _, _ in PIECES:
        items.append((n, "\\" + n))
        if not a.tex_only:
            items.append((n + "X", "a\\%s b" % n))
    for n, built in COMPOSED:
        items.append((n, "\\" + n))
        items.append((n + "B", built))
    items += [("lmoustache", r"\lmoustache"), ("rmoustache", r"\rmoustache")]
    got = pdflatex(items)
    rows = []
    for n, cls, fam, slot, gname, builds in PIECES:
        if n not in got:
            continue
        w, ht, dp = got[n]
        m = classify(w, got[n + "X"][0] - CTRL)[0] if not a.tex_only else ""
        rows.append(("\\" + n, "%s \"%02X" % (fam, slot), gname, cls,
                     "%.5f" % w, "%.5f" % ht, m))
    grid(("command", "family/slot", "cm glyph", "class", "wd", "ht", "measured"),
         rows)

    print("  each piece's whole, and the same whole built from the pieces:\n")
    rows = [("\\" + n, "%.5f" % got[n][0], "%.5f" % got[n + "B"][0],
             "%+.5f" % (got[n + "B"][0] - got[n][0]), built)
            for n, built in COMPOSED if n in got]
    grid(("whole", "wd", "built wd", "delta", "fontmath.ltx definition"), rows)

    print("  cmex \"7A/\"7B serve twice -- as a brace tip and as the small\n"
          "  variant of the moustache delimiters, so these are one glyph each:\n")
    rows = [("\\" + n, "%.5f" % got[n][0], "%.5f" % got[n][1], "%.5f" % got[n][2])
            for n in ("braceld", "lmoustache", "bracerd", "rmoustache",
                      "bracelu", "braceru") if n in got]
    grid(("command", "wd", "ht", "dp"), rows)

    # ---- dddot / ddddot -------------------------------------------------
    print("== amsmath \\dddot and \\ddddot: not accents ==\n")
    dd = pdflatex([("a", "a"), ("dot", r"\dot{a}"), ("ddot", r"\ddot{a}"),
                   ("dddot", r"\dddot{a}"), ("ddddot", r"\ddddot{a}")],
                  r"\usepackage{amsmath}")
    rows = [(k, "%.5f" % dd[k][0], "%.5f" % dd[k][1], "%.5f" % dd[k][2],
             "%+.5f" % (dd[k][0] - dd["a"][0]))
            for k in ("a", "dot", "ddot", "dddot", "ddddot") if k in dd]
    grid(("expr", "wd", "ht", "dp", "wd - wd(a)"), rows)
    print("  \\dot and \\ddot keep the nucleus width; \\dddot and \\ddddot do\n"
          "  not, because amsmath 744-750 builds them as a \\vbox of text-size\n"
          "  roman periods over a \\mathop, not as a math accent.\n")

    # ---- sqsubset / sqsupset -------------------------------------------
    print("== \\sqsubset/\\sqsupset: two different designs, by package ==\n")
    rows = []
    for pkg, tag in ((r"\usepackage{latexsym}", "latexsym (lasy \"3C/\"3D)"),
                     (r"\usepackage{amssymb}", "amssymb (AMSa \"40/\"41)")):
        r = pdflatex([(n, "\\" + n) for n in
                      ("sqsubset", "sqsupset", "sqsubseteq", "sqsupseteq")], pkg)
        for n in ("sqsubset", "sqsupset"):
            if n in r:
                rows.append((tag, "\\" + n, "%.5f" % r[n][0],
                             "%.5f" % r[n][1], "%.5f" % r[n][2]))
    grid(("package", "command", "wd", "ht", "dp"), rows)
    print("  Same advance and the same Rel class either way, but a different\n"
          "  height and depth: which package is loaded changes the ink.\n")

    # ---- mathscr --------------------------------------------------------
    print("== \\mathscr: three different alphabets, none of them \\mathcal ==\n")
    rows = []
    for pkg, tag in ((r"\usepackage{mathrsfs}", "mathrsfs (rsfs10)"),
                     (r"\usepackage[mathscr]{euscript}", "euscript (eusm10)")):
        r = pdflatex([("A", r"\mathscr{A}"), ("F", r"\mathscr{F}"),
                      ("cA", r"\mathcal{A}"), ("cF", r"\mathcal{F}")], pkg)
        for k, lbl in (("A", r"\mathscr{A}"), ("F", r"\mathscr{F}")):
            if k in r:
                rows.append((tag, lbl, "%.5f" % r[k][0],
                             "%.5f" % r["c" + k][0],
                             "%+.5f" % (r[k][0] - r["c" + k][0])))
    grid(("package", "expr", "wd", "\\mathcal wd", "delta"), rows)


if __name__ == "__main__":
    main()
