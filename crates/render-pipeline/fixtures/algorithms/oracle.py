#!/usr/bin/env python3
"""Pseudocode (algorithm / algorithmic / algpseudocode) oracle. TEST ONLY:
pdflatex is never in the product path, and `cargo test` never runs TeX.

For every `NN-*.tex` fixture next to this script, runs MacTeX `pdflatex`
twice on a copy whose preamble only adds `\\pdfcompresslevel=0
\\pdfobjcompresslevel=0` (output encoding; layout is unchanged) and writes
`reference/NN-*.json`:

  pages   page count
  words   every word `pdftotext -bbox` reports: page, text, `x` (the pen
          position of its first glyph, bp from the left edge) and `baseline`
          (bp from the top edge). poppler's box of a Latin Modern text or
          math-italic glyph run spans baseline-0.8847*size+0.19409*size ..
          baseline+0.19409*size (calibrated against `\\pdfsavepos` for the
          roman, bold, italic and math italic faces at 8, 9 and 10pt), so the
          baseline is recovered exactly when the box height is 0.8847 times a
          LaTeX size; for other faces (symbols, small caps) it is null and
          only x is compared.
  rules   every stroked rule pdfTeX wrote (`x y cm ... w 0 0 m l 0 l S`):
          page, x, top, width, height in bp (top-left origin).

`tests/algorithms_oracle.rs` compares the pipeline's display list with these
files: word x and baseline within 0.5 bp, rules within 0.1 bp.

usage: python3 oracle.py [--pdflatex PATH] [--pdftotext PATH] [names...]
"""
import argparse
import html
import json
import os
import re
import shutil
import subprocess
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
WORD = re.compile(r'<word xMin="([-\d.]+)" yMin="([-\d.]+)" xMax="([-\d.]+)" yMax="([-\d.]+)">(.*?)</word>')
RULE = re.compile(rb"1 0 0 1 ([-\d.]+) ([-\d.]+) cm\s*\[\]0 d 0 J ([-\d.]+) w 0 0 m ([-\d.]+) 0 l S")
SIZES = [5, 6, 7, 8, 9, 10, 10.95, 12, 14.4, 17.28, 20.74, 24.88]
HEIGHT_RATIO = 0.8847
DEPTH_RATIO = 0.19409


def baseline_of(y_min, y_max):
    h = y_max - y_min
    for size_pt in SIZES:
        # The ratios are per TeX point of the font size (8.847 bp for 10pt).
        if abs(h - HEIGHT_RATIO * size_pt) < 0.004:
            return round(y_max - DEPTH_RATIO * size_pt, 3)
    return None


def page_contents(pdf):
    """The content stream of every page, in page-tree order."""
    objects = {int(m.group(1)): m.group(2) for m in re.finditer(rb"(?s)(\d+) 0 obj\s*(.*?)endobj", pdf)}

    def stream(num):
        m = re.search(rb"(?s)stream\r?\n(.*?)endstream", objects[num])
        return m.group(1) if m else b""

    def kids(num):
        body = objects[num]
        if re.search(rb"/Type\s*/Pages\b", body):
            arr = re.search(rb"(?s)/Kids\s*\[(.*?)\]", body).group(1)
            out = []
            for k in re.findall(rb"(\d+) 0 R", arr):
                out.extend(kids(int(k)))
            return out
        contents = re.search(rb"/Contents\s+(\d+) 0 R", body)
        return [stream(int(contents.group(1)))] if contents else [b""]

    root = re.search(rb"/Type\s*/Catalog.*?/Pages\s+(\d+) 0 R", pdf, re.S)
    return kids(int(root.group(1)))


def run(name, pdflatex, pdftotext):
    src = open(os.path.join(HERE, name + ".tex")).read()
    tex = src.replace("\\begin{document}", "\\pdfcompresslevel=0 \\pdfobjcompresslevel=0\n\\begin{document}", 1)
    with tempfile.TemporaryDirectory() as tmp:
        open(os.path.join(tmp, "doc.tex"), "w").write(tex)
        env = dict(os.environ, SOURCE_DATE_EPOCH="0", FORCE_SOURCE_DATE="1")
        for _ in range(2):
            r = subprocess.run([pdflatex, "-interaction=batchmode", "-halt-on-error", "doc.tex"], cwd=tmp, capture_output=True, env=env)
            if r.returncode != 0:
                raise SystemExit(f"{name}: pdflatex failed\n" + open(os.path.join(tmp, "doc.log")).read()[-3000:])
        pdf_path = os.path.join(tmp, "doc.pdf")
        xml = subprocess.run([pdftotext, "-bbox", pdf_path, "-"], capture_output=True, text=True, check=True).stdout
        pdf = open(pdf_path, "rb").read()
        log = open(os.path.join(tmp, "doc.log")).read()
    pages = int(re.search(r"Output written on doc.pdf \((\d+) page", log).group(1))
    words = []
    for page_no, chunk in enumerate(xml.split("<page ")[1:], start=1):
        for m in WORD.finditer(chunk):
            x_min, y_min, _x_max, y_max = (float(v) for v in m.groups()[:4])
            words.append({"page": page_no, "text": html.unescape(m.group(5)), "x": round(x_min, 3), "baseline": baseline_of(y_min, y_max)})
    rules = []
    contents = page_contents(pdf)
    if len(contents) != pages:
        raise SystemExit(f"{name}: {len(contents)} page content streams for {pages} pages")
    for page_no, s in enumerate(contents, start=1):
        for m in RULE.finditer(s):
            x, y, w, length = (float(v) for v in m.groups())
            rules.append({"page": page_no, "x": round(x, 3), "top": round(792 - y - w / 2, 3), "width": round(length, 3), "height": round(w, 3)})
    version = subprocess.run([pdflatex, "--version"], capture_output=True, text=True).stdout.splitlines()[0]
    out = {"fixture": name + ".tex", "engine": version, "pages": pages, "words": words, "rules": rules}
    os.makedirs(os.path.join(HERE, "reference"), exist_ok=True)
    with open(os.path.join(HERE, "reference", name + ".json"), "w") as f:
        json.dump(out, f, indent=1, ensure_ascii=False)
        f.write("\n")
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pdflatex", default=shutil.which("pdflatex") or "/Library/TeX/texbin/pdflatex")
    ap.add_argument("--pdftotext", default=shutil.which("pdftotext") or "pdftotext")
    ap.add_argument("names", nargs="*")
    args = ap.parse_args()
    names = args.names or sorted(n[:-4] for n in os.listdir(HERE) if re.match(r"\d\d-.*\.tex$", n))
    for n in names:
        o = run(n, args.pdflatex, args.pdftotext)
        unknown = sum(1 for w in o["words"] if w["baseline"] is None)
        print(f"{n}: pages {o['pages']} words {len(o['words'])} (x only {unknown}) rules {len(o['rules'])}")


if __name__ == "__main__":
    main()
