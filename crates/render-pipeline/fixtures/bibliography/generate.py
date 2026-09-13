#!/usr/bin/env python3
"""Regenerate the pdflatex expectations for tests/bibliography_oracle.rs.

pdflatex (TeX Live 2026, pdfTeX 1.40.29) and bibtex are ORACLES only;
`cargo test` never runs TeX. For every `*.tex` next to this script it builds
the PDF in a scratch directory — pdflatex, bibtex when the document has
`\\bibliography` (the resulting `<name>.bbl` is kept next to the `.tex` as
fixture input, since the compiler reads the project's `.bbl`, never a
`.bib`), then pdflatex twice more — reads each word's box with poppler's
`pdftotext -bbox` and writes `<name>.expected`:

    # pdflatex <version>
    line <page> <yMax bp>
    word <xMin bp> <text>
    ...

Words are grouped into lines by their `yMax` (within 3bp).

usage: python3 generate.py [--pdflatex PATH] [--bibtex PATH] [--pdftotext PATH]
"""

import argparse
import glob
import html
import os
import re
import shutil
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
WORD = re.compile(r'<word xMin="([-\d.]+)" yMin="([-\d.]+)" xMax="([-\d.]+)" yMax="([-\d.]+)">(.*?)</word>')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pdflatex", default=shutil.which("pdflatex") or "/Library/TeX/texbin/pdflatex")
    ap.add_argument("--bibtex", default=shutil.which("bibtex") or "/Library/TeX/texbin/bibtex")
    ap.add_argument("--pdftotext", default=shutil.which("pdftotext") or "pdftotext")
    args = ap.parse_args()
    version = subprocess.run([args.pdflatex, "--version"], capture_output=True, text=True).stdout.splitlines()[0]
    env = dict(os.environ, SOURCE_DATE_EPOCH="0", FORCE_SOURCE_DATE="1")
    for tex in sorted(glob.glob(os.path.join(HERE, "*.tex"))):
        name = os.path.splitext(os.path.basename(tex))[0]
        source = open(tex, encoding="utf-8").read()
        with tempfile.TemporaryDirectory() as tmp:
            shutil.copy(tex, tmp)
            for bib in glob.glob(os.path.join(HERE, "*.bib")):
                shutil.copy(bib, tmp)
            latex = lambda: subprocess.run([args.pdflatex, "-interaction=nonstopmode", name + ".tex"],
                                           cwd=tmp, env=env, capture_output=True)
            latex()
            if "\\bibliography{" in source:
                subprocess.run([args.bibtex, name], cwd=tmp, env=env, check=True, capture_output=True)
                shutil.copy(os.path.join(tmp, name + ".bbl"), os.path.join(HERE, name + ".bbl"))
            latex()
            latex()
            xml = subprocess.run([args.pdftotext, "-bbox", os.path.join(tmp, name + ".pdf"), "-"],
                                 capture_output=True, text=True, check=True).stdout
        out = [f"# {version}"]
        for page_no, chunk in enumerate(xml.split("<page ")[1:], start=1):
            words = sorted((float(m.group(4)), float(m.group(1)), html.unescape(m.group(5))) for m in WORD.finditer(chunk))
            lines = []
            for y, x, t in words:
                if lines and abs(lines[-1][0] - y) <= 3.0:
                    lines[-1][1].append((x, t))
                else:
                    lines.append((y, [(x, t)]))
            for y, ws in lines:
                out.append(f"line {page_no} {y:.3f}")
                out.extend(f"word {x:.3f} {t}" for x, t in sorted(ws))
        with open(os.path.join(HERE, name + ".expected"), "w") as f:
            f.write("\n".join(out) + "\n")
        print(name, sum(1 for l in out if l.startswith("line")), "lines")


if __name__ == "__main__":
    main()
