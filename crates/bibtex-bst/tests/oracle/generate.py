#!/usr/bin/env python3
"""Oracle generator for flashtex-bibtex-bst.

Writes every case to tests/data/cases/<name>/ (job.aux plus any extra
.aux/.bib files), runs TeX Live's `bibtex` on it in a scratch directory and
stores the resulting .bbl/.blg (and exit status) in tests/data/expected/.

This is the only place `bibtex` is used; `cargo test` never runs it. Re-run
after changing a case:  python3 tests/oracle/generate.py
Search paths are restricted to the case directory and tests/data/{bst,bib}
(no trailing ':' so system styles are never picked up).
"""

import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

DATA = pathlib.Path(__file__).resolve().parent.parent / "data"
CASES_DIR = DATA / "cases"
EXPECTED_DIR = DATA / "expected"

STYLES = ["plain", "unsrt", "alpha", "abbrv", "acm", "ieeetr", "siam",
          "apalike", "plainnat", "abbrvnat", "unsrtnat"]
DATABASES = ["people", "strings", "crossref", "accents", "large"]


def aux(cites, bibdata, style):
    """A minimal .aux: one \\citation line per group, then data and style."""
    lines = ["\\relax"]
    for group in cites:
        lines.append("\\citation{%s}" % group)
    if bibdata is not None:
        lines.append("\\bibdata{%s}" % bibdata)
    if style is not None:
        lines.append("\\bibstyle{%s}" % style)
    return "\n".join(lines) + "\n"


CASES = []


def case(name, aux_text, **files):
    CASES.append((name, aux_text, files))


# 11 styles x 5 databases, whole database.
for st in STYLES:
    for db in DATABASES:
        case("%s-%s" % (st, db), aux(["*"], db, st))

# The error database under several styles.
for st in ["plain", "alpha", "unsrt", "plainnat"]:
    case("%s-errors" % st, aux(["*"], "errors", st))

# Explicit citations, order, missing keys.
PEOPLE_CITES = ["knuth1984,beethoven", "zz-missing", "poussin,knuth1984"]
case("explicit-plain", aux(PEOPLE_CITES, "people", "plain"))
case("explicit-unsrt", aux(PEOPLE_CITES, "people", "unsrt"))
case("explicit-abbrvnat", aux(PEOPLE_CITES, "people", "abbrvnat"))
case("case-mismatch", aux(["Knuth1984", "knuth1984,beethoven"], "people", "plain"))
case("db-case-diff", aux(["KNUTH1984,BeeThoven"], "people", "alpha"))
case("multi-bib", aux(["*"], "people,strings", "plain"))
case("dup-bibdata-file", aux(["*"], "people,people", "plain"))
case("nocite-twice", aux(["*,*"], "people", "plain"))
case("star-after-cites", aux(["str4,str1", "*", "str2,nosuch"], "strings", "unsrt"))
case("star-after-cites-plain", aux(["str9,str1", "*", "Str8,str3"], "strings", "plain"))
case("duplicates-explicit", aux(["dup,unknowntype,extrafield,DUP"], "errors", "plain"))
case("alpha-labels", aux(["l01,l02,l03", "l21,l22,l23,l24", "l06,l07,l08"], "large", "alpha"))

# Cross references.
case("crossref-min", aux(["child1,child2,child3,nestedchild,badchild"], "crossref", "plain"))
case("crossref-explicit-parent", aux(["child3,singleparent"], "crossref", "unsrt"))
case("crossref-case", aux(["child2,child1"], "crossref", "unsrt"))
case("crossref-parent-cited-case", aux(["child1,PARENTPROC"], "crossref", "plain"))
case("crossref-nested", aux(["nestedchild,middlebook"], "crossref", "abbrv"))

# .aux-level problems.
case("missing-style", aux(["*"], "people", "nosuchstyle"))
case("missing-bib", aux(["*"], "people,nosuchbib", "plain"))
case("no-citations", aux([], "people", "plain"))
case("no-bibdata", aux(["knuth1984"], None, "plain"))
case("no-bibstyle", aux(["knuth1984"], "people", None))
case("repeated-commands",
     "\\citation{knuth1984}\n\\bibdata{people}\n\\bibdata{strings}\n"
     "\\bibstyle{plain}\n\\bibstyle{alpha}\n")
case("aux-errors",
     "\\citation{a b}\n\\citation{knuth1984}x\n\\citation{beethoven\n"
     "\\bibdata{people} \n\\bibstyle{plain}trailing\n\\bibstyle{plain}\n"
     "\\@input{sub.tex}\n\\@input{nosuch.aux}\n\\citation{*,*}\n"
     "\\citation{}\n\\citation{poussin,}\n\\bogus{x}\nno brace here\n")
case("input-aux",
     "\\relax\n\\@input{sub.aux}\n\\citation{knuth1984}\n\\bibdata{people}\n"
     "\\bibstyle{plain}\n\\@input{sub.aux}\n",
     **{"sub.aux": "\\citation{beethoven}\n\\@input{sub2.aux}\n\\citation{poussin}\n",
        "sub2.aux": "\\citation{brinch}\n\\@input{job.aux}\n"})

# Tricky database syntax.
TRICKY_BIB = (
    "@article{same1, author={A One}, title={T1}, journal={J}, year=2001} "
    "@article{same2, author={B Two}, title={T2}, journal={J}, year=2002}\n"
    "@string(paren = \"Paren\" # { String })\n"
    "@string{bad = }\n"
    "@string{ = \"noname\"}\n"
    "@string{selfish = selfish # \"x\"}\n"
    "@preamble( \"paren preamble\" )\n"
    "@preamble{ undefinedmac # \"x\" }\n"
    "@article(parenkey, author = \"P K\", title = \"Parens\", journal = paren, year = 2003)\n"
    "@article{nofields}\n"
    "@article{onlycomma,}\n"
    "@article{weird key, title={Space in key}}\n"
    "@misc{numfield, year = 2004, note = 12 # 34}\n"
    "@misc{quoted, title = \"unbalanced } brace\"}\n"
    "@misc{bracequote, title = {has \"quote\" inside}, note = \"has {\"} inside\"}\n"
    "@misc{lastone, title = {Last}, year = 2005} trailing text\n"
    "@ misc { spaced , title = { Spaced   Out } , year = 2006 }\n"
    "@misc{multi,\n  title = {Line one\n\n   line two},\n  note = \"quoted\n  over lines\"}\n"
    "@misc{final1, title={F1}} @misc{final2, title={F2}}"
)
EOF_BIB = "@misc{early, title = {Fine}}\n@misc{unterminated, title = {Never closed\n  still going\n"
case("tricky-bib", aux(["*"], "tricky,eof", "plain"), **{"tricky.bib": TRICKY_BIB, "eof.bib": EOF_BIB})
case("tricky-bib-explicit", aux(["same2,final2,final1,nosuch,parenkey,spaced"], "tricky", "unsrt"),
     **{"tricky.bib": TRICKY_BIB})
case("empty-bib", aux(["*"], "empty", "plain"), **{"empty.bib": ""})
CRLF_BIB = ("@article{a,\r\n  author = {A B},\r\n  title = {T},\r\n  journal = {J},\r\n"
            "  year = 2000}\r\n\r\n@article{b,\r\n  author = {A B}\r\n  title = {T}}\r\n")
case("crlf", aux(["*"], "crlf", "plain"), **{"crlf.bib": CRLF_BIB})
CR_BIB = "@misc{cr1, title = {CR only}}\r@misc{cr2, title = {Second\rline}}\r"
case("cr-only", aux(["*"], "cronly", "unsrt"), **{"cronly.bib": CR_BIB})

# Original test styles.
for db in ["people", "crossref", "accents", "strings"]:
    case("builtins-%s" % db, aux(["*"], db, "builtins"))
case("builtins-explicit", aux(["child2,brinch", "missingkey"], "crossref,people", "builtins"))
case("badstyle", aux(["*"], "people", "badstyle"))


def write_cases():
    if CASES_DIR.exists():
        shutil.rmtree(CASES_DIR)
    CASES_DIR.mkdir(parents=True)
    for name, aux_text, files in CASES:
        d = CASES_DIR / name
        d.mkdir()
        (d / "job.aux").write_bytes(aux_text.encode("utf-8"))
        for fname, content in files.items():
            (d / fname).write_bytes(content.encode("utf-8"))


def run_oracle():
    if EXPECTED_DIR.exists():
        shutil.rmtree(EXPECTED_DIR)
    EXPECTED_DIR.mkdir(parents=True)
    version = subprocess.run(["bibtex", "--version"], capture_output=True, text=True).stdout
    for name, _, _ in CASES:
        src = CASES_DIR / name
        with tempfile.TemporaryDirectory() as tmp:
            for f in src.iterdir():
                shutil.copy(f, tmp)
            env = dict(os.environ)
            env["BSTINPUTS"] = "%s:%s" % (tmp, DATA / "bst")
            env["BIBINPUTS"] = "%s:%s" % (tmp, DATA / "bib")
            for var in ["TEXMFOUTPUT", "TEXMF_OUTPUT_DIRECTORY", "min_crossrefs"]:
                env.pop(var, None)
            proc = subprocess.run(["bibtex", "job"], cwd=tmp, env=env,
                                  capture_output=True)
            for ext in ["bbl", "blg"]:
                p = pathlib.Path(tmp) / ("job." + ext)
                if p.exists():
                    shutil.copy(p, EXPECTED_DIR / ("%s.%s" % (name, ext)))
            (EXPECTED_DIR / ("%s.exit" % name)).write_text("%d\n" % proc.returncode)
    (EXPECTED_DIR / "ORACLE_VERSION").write_text(version.splitlines()[0] + "\n")
    print("generated %d cases with %s" % (len(CASES), version.splitlines()[0]))


if __name__ == "__main__":
    write_cases()
    if "--cases-only" not in sys.argv:
        run_oracle()
